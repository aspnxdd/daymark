use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use anyhow::{Context, Result, ensure};
use chrono::{Local, NaiveDate};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

// Keep the file-format tag stable so existing journals remain compatible across app renames.
const APPLICATION_ID: i32 = 1_297_044_307;

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: Option<i64>,
    pub created_at: String,
    pub title: String,
    pub body: String,
    pub revision: i64,
}

impl Entry {
    pub fn blank() -> Self {
        Self {
            id: None,
            created_at: Local::now().to_rfc3339(),
            title: String::new(),
            body: String::new(),
            revision: 0,
        }
    }

    pub fn is_blank(&self) -> bool {
        self.title.trim().is_empty() && self.body.trim().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntrySummary {
    pub id: i64,
    pub created_at: String,
    pub title: String,
    pub preview: String,
}

pub struct Store {
    conn: Connection,
    pub path: PathBuf,
}

impl Store {
    pub fn open(directory: &Path) -> Result<Self> {
        ensure!(
            !is_symlink(directory)?,
            "The data directory must not be a symbolic link."
        );
        fs::create_dir_all(directory).context("Could not create the journal directory")?;
        #[cfg(unix)]
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
        let path = directory.join("journal.sqlite3");
        for suffix in ["", "-journal", "-wal", "-shm"] {
            ensure!(
                !is_symlink(&directory.join(format!("journal.sqlite3{suffix}")))?,
                "Refusing a symbolic link in the journal files."
            );
        }
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&path) {
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err).context("Could not create the journal database"),
        }
        #[cfg(unix)]
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        let mut conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .context("Could not open the journal database")?;
        conn.busy_timeout(Duration::from_millis(250))?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        let app_id: i32 = conn.pragma_query_value(None, "application_id", |row| row.get(0))?;
        ensure!(
            version <= 1,
            "This journal was created by a newer Daymark version. Upgrade Daymark; do not replace the database."
        );
        if version == 0 && app_id == 0 {
            let tables: i64 = conn.query_row(
                "SELECT count(*) FROM sqlite_master WHERE name NOT GLOB 'sqlite_*'",
                [],
                |r| r.get(0),
            )?;
            ensure!(
                tables == 0,
                "This is not an empty Daymark database. Refusing to modify it."
            );
            let tx = conn.transaction()?;
            tx.execute_batch(
                "CREATE TABLE entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                search_text TEXT NOT NULL,
                revision INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX entries_chronological ON entries(created_at DESC, id DESC);
            PRAGMA user_version = 1;",
            )?;
            tx.pragma_update(None, "application_id", APPLICATION_ID)?;
            tx.commit()?;
        } else {
            ensure!(
                app_id == APPLICATION_ID && version == 1,
                "This file is not a supported Daymark journal."
            );
        }
        conn.execute_batch("PRAGMA journal_mode = DELETE; PRAGMA synchronous = FULL; PRAGMA foreign_keys = ON; PRAGMA temp_store = MEMORY; PRAGMA secure_delete = ON;")?;
        Ok(Self { conn, path })
    }

    pub fn list(&self, query: &str) -> Result<Vec<EntrySummary>> {
        self.list_on_day(query, None)
    }

    pub fn list_on_day(&self, query: &str, day: Option<NaiveDate>) -> Result<Vec<EntrySummary>> {
        // Match the creation date in its recorded offset, just like the date beside the entry.
        let mut stmt = self.conn.prepare("SELECT id, created_at, title, substr(body, 1, 140) FROM entries WHERE instr(search_text, ?1) > 0 AND (?2 IS NULL OR substr(created_at, 1, 10) = ?2) ORDER BY created_at DESC, id DESC")?;
        let rows = stmt.query_map(
            params![query.trim().to_lowercase(), day.map(|day| day.to_string())],
            |r| {
                Ok(EntrySummary {
                    id: r.get(0)?,
                    created_at: r.get(1)?,
                    title: r.get(2)?,
                    preview: r.get(3)?,
                })
            },
        )?;
        rows.collect::<rusqlite::Result<_>>()
            .context("Could not read the timeline")
    }

    pub fn entry_days(&self) -> Result<BTreeMap<NaiveDate, usize>> {
        let mut stmt = self.conn.prepare(
            "SELECT substr(created_at, 1, 10), count(*) FROM entries GROUP BY substr(created_at, 1, 10)",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        rows.map(|row| {
            let (date, count) = row.context("Could not read calendar entry counts")?;
            let day = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .context("Could not read an entry's creation date")?;
            Ok((day, usize::try_from(count)?))
        })
        .collect()
    }

    pub fn get(&self, id: i64) -> Result<Entry> {
        self.conn
            .query_row(
                "SELECT id, created_at, title, body, revision FROM entries WHERE id = ?1",
                [id],
                |r| {
                    Ok(Entry {
                        id: Some(r.get(0)?),
                        created_at: r.get(1)?,
                        title: r.get(2)?,
                        body: r.get(3)?,
                        revision: r.get(4)?,
                    })
                },
            )
            .optional()?
            .context("This entry no longer exists. Your current draft has been kept.")
    }

    pub fn save(&mut self, entry: &mut Entry) -> Result<()> {
        if entry.id.is_none() && entry.is_blank() {
            return Ok(());
        }
        let search = format!("{}\n{}", entry.title, entry.body).to_lowercase();
        let tx = self.conn.transaction()?;
        let id = match entry.id {
            Some(id) => {
                let changed = tx.execute("UPDATE entries SET title = ?1, body = ?2, search_text = ?3, revision = revision + 1 WHERE id = ?4 AND revision = ?5", params![entry.title, entry.body, search, id, entry.revision])?;
                ensure!(
                    changed == 1,
                    "Another window changed this entry. Your draft is kept here; copy it somewhere safe before reloading."
                );
                id
            }
            None => {
                tx.execute("INSERT INTO entries (created_at, title, body, search_text) VALUES (?1, ?2, ?3, ?4)", params![entry.created_at, entry.title, entry.body, search])?;
                tx.last_insert_rowid()
            }
        };
        tx.commit().context("Could not commit the entry to disk")?;
        entry.id = Some(id);
        entry.revision += 1;
        Ok(())
    }

    pub fn delete(&mut self, entry: &Entry) -> Result<()> {
        if let Some(id) = entry.id {
            let changed = self.conn.execute(
                "DELETE FROM entries WHERE id = ?1 AND revision = ?2",
                params![id, entry.revision],
            )?;
            ensure!(
                changed == 1,
                "The entry changed in another window. It has not been deleted."
            );
        }
        Ok(())
    }
}

fn is_symlink(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(meta) => Ok(meta.file_type().is_symlink()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_reopen_search_update_delete() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let mut store = Store::open(dir.path())?;
        let mut entry = Entry::blank();
        store.save(&mut entry)?;
        assert!(store.list("")?.is_empty());
        entry.title = "Café & 春 🌿".into();
        entry.body = "First line\n100%_quiet\n' OR 1=1 --".into();
        store.save(&mut entry)?;
        assert_eq!(store.list("CAFÉ")?.len(), 1);
        assert_eq!(store.list("100%_")?.len(), 1);
        assert!(store.list("not present")?.is_empty());
        assert!(store.list("' OR 1=1 -- missing")?.is_empty());
        drop(store);
        let mut store = Store::open(dir.path())?;
        assert_eq!(
            store
                .conn
                .pragma_query_value(None, "application_id", |row| row.get::<_, i32>(0))?,
            1_297_044_307
        );
        assert_eq!(store.get(entry.id.unwrap())?, entry);
        entry.body = "Changed".into();
        store.save(&mut entry)?;
        assert!(store.list("First line")?.is_empty());
        store.delete(&entry)?;
        assert!(store.list("")?.is_empty());
        Ok(())
    }

    #[test]
    fn day_filter_counts_and_literal_search_preserve_recorded_dates() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let mut store = Store::open(dir.path())?;
        for (date, body) in [
            ("2024-02-29T23:30:00-08:00", "Café 100%_quiet"),
            ("2024-02-29T08:00:00+14:00", "Another note"),
            ("2024-03-01T01:00:00+00:00", "Café 100%_quiet"),
        ] {
            let mut entry = Entry::blank();
            entry.created_at = date.into();
            entry.body = body.into();
            store.save(&mut entry)?;
        }
        let leap_day = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        assert_eq!(store.list_on_day("", Some(leap_day))?.len(), 2);
        assert_eq!(store.list_on_day("CAFÉ 100%_", Some(leap_day))?.len(), 1);
        assert!(store.list_on_day("%_ missing", Some(leap_day))?.is_empty());
        assert!(store.list_on_day("' OR 1=1 --", Some(leap_day))?.is_empty());
        assert!(store.list_on_day("", leap_day.pred_opt())?.is_empty());
        let counts = store.entry_days()?;
        assert_eq!(counts[&leap_day], 2);
        assert_eq!(counts[&leap_day.succ_opt().unwrap()], 1);
        assert_eq!(store.list("")?.len(), 3);
        Ok(())
    }

    #[test]
    fn rejects_stale_updates_and_deletes() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let mut store = Store::open(dir.path())?;
        let mut entry = Entry::blank();
        entry.body = "Original".into();
        store.save(&mut entry)?;
        let mut stale = entry.clone();
        entry.body = "Newer".into();
        store.save(&mut entry)?;
        stale.body = "Stale".into();
        assert!(store.save(&mut stale).is_err());
        assert!(store.delete(&stale).is_err());
        assert_eq!(store.get(entry.id.unwrap())?.body, "Newer");
        Ok(())
    }

    #[test]
    fn deleted_entry_identity_is_never_reused() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let mut store = Store::open(dir.path())?;
        let mut original = Entry::blank();
        original.body = "Original".into();
        store.save(&mut original)?;
        store.delete(&original)?;
        let mut replacement = Entry::blank();
        replacement.body = "Replacement".into();
        store.save(&mut replacement)?;
        assert_ne!(original.id, replacement.id);
        original.body = "Stale write".into();
        assert!(store.save(&mut original).is_err());
        assert!(store.delete(&original).is_err());
        assert_eq!(store.get(replacement.id.unwrap())?.body, "Replacement");
        Ok(())
    }

    #[test]
    fn refuses_foreign_or_newer_databases() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let conn = Connection::open(dir.path().join("journal.sqlite3"))?;
        conn.execute_batch("CREATE TABLE important (value TEXT);")?;
        assert!(Store::open(dir.path()).is_err());
        conn.execute_batch("DROP TABLE important; PRAGMA user_version = 99;")?;
        assert!(Store::open(dir.path()).is_err());
        Ok(())
    }

    #[test]
    fn refuses_foreign_tables_with_sqlite_like_prefix() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let conn = Connection::open(dir.path().join("journal.sqlite3"))?;
        conn.execute_batch("CREATE TABLE sqliteXnotes (value TEXT);")?;
        assert!(Store::open(dir.path()).is_err());
        assert_eq!(
            conn.pragma_query_value(None, "application_id", |row| row.get::<_, i32>(0))?,
            0
        );
        let tables: i64 =
            conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get(0))?;
        assert_eq!(tables, 1);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn private_permissions_and_symlink_refusal() -> Result<()> {
        use std::os::unix::fs::symlink;
        let parent = tempfile::tempdir()?;
        let dir = parent.path().join("private");
        let store = Store::open(&dir)?;
        assert_eq!(fs::metadata(&dir)?.permissions().mode() & 0o777, 0o700);
        assert_eq!(
            fs::metadata(&store.path)?.permissions().mode() & 0o777,
            0o600
        );
        let link = parent.path().join("link");
        symlink(&dir, &link)?;
        assert!(Store::open(&link).is_err());
        let other = parent.path().join("other");
        fs::create_dir(&other)?;
        symlink(&store.path, other.join("journal.sqlite3"))?;
        assert!(Store::open(&other).is_err());
        Ok(())
    }
}
