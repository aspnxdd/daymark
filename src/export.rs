use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::atomic::Ordering,
};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use anyhow::{Context, Result, ensure};
use chrono::{DateTime, NaiveDate};

use crate::{model::Journal, storage::Entry};

impl Journal {
    /// Export the whole displayed day, independent of search, without changing save state.
    pub fn export_day(&self, date: NaiveDate, path: &Path) -> Result<()> {
        let markdown = self.day_markdown(date)?;
        write_new_export(path, markdown.as_bytes())
    }

    fn day_markdown(&self, date: NaiveDate) -> Result<String> {
        let mut entries = Vec::new();
        for summary in self.store.list_on_day("", Some(date))? {
            entries.push(self.store.get(summary.id)?);
        }
        // A failed draft can be rescued by export, but export must not acknowledge a save.
        if self.unsaved.load(Ordering::SeqCst)
            || (self.draft.id.is_none() && !self.draft.is_blank())
        {
            let created = DateTime::parse_from_rfc3339(&self.draft.created_at)
                .context("The open draft has an invalid creation date; nothing was exported")?;
            if created.date_naive() == date {
                entries.retain(|entry| self.draft.id.is_none() || entry.id != self.draft.id);
                entries.push(self.draft.clone());
            }
        }
        ensure!(
            !entries.is_empty(),
            "There are no entries on this day to export."
        );
        markdown_for_day(date, entries)
    }
}

fn markdown_for_day(date: NaiveDate, entries: Vec<Entry>) -> Result<String> {
    let mut dated = entries
        .into_iter()
        .map(|entry| {
            DateTime::parse_from_rfc3339(&entry.created_at)
                .map(|created| (created, entry))
                .context("An entry has an invalid creation date; nothing was exported")
        })
        .collect::<Result<Vec<_>>>()?;
    dated.sort_by(|(left_date, left), (right_date, right)| {
        left_date.cmp(right_date).then(left.id.cmp(&right.id))
    });
    let mut markdown = format!("# {date}\n\n");
    for (_, entry) in dated {
        markdown.push_str("## ");
        markdown.push_str(&entry.title);
        markdown.push_str("\n\n");
        markdown.push_str(&entry.body);
        markdown.push_str("\n\n");
    }
    Ok(markdown)
}

fn write_new_export(path: &Path, contents: &[u8]) -> Result<()> {
    ensure!(
        path.file_name().is_some(),
        "Choose a filename for the export."
    );
    let absolute = std::path::absolute(path).context("Could not resolve the export path")?;
    let mut checked = std::path::PathBuf::new();
    // shortcut: ancestor checks race with renames; use descriptor-relative I/O for hostile folders.
    for component in absolute.components() {
        checked.push(component);
        match fs::symlink_metadata(&checked) {
            Ok(metadata) => ensure!(
                !metadata.file_type().is_symlink(),
                "Symbolic links are not allowed in the export path. Choose a local folder."
            ),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err).context("Could not inspect the export path"),
        }
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&absolute)
        .context("Could not create the export. Existing files are never replaced; choose a new filename and an existing writable folder")?;
    if let Err(err) = file.write_all(contents).and_then(|()| file.sync_all()) {
        drop(file);
        fs::remove_file(&absolute).context(
            "Export failed and the incomplete file could not be removed. Remove it before retrying",
        )?;
        return Err(err)
            .context("Could not finish writing the export; no complete export was created");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::AtomicBool};

    use super::*;
    use crate::storage::Store;

    fn journal() -> Result<(tempfile::TempDir, Journal)> {
        let dir = tempfile::tempdir()?;
        let store = Store::open(&dir.path().join("data"))?;
        Ok((dir, Journal::new(store, Arc::new(AtomicBool::new(false)))?))
    }

    fn add(journal: &mut Journal, created: &str, title: &str, body: &str) -> Result<Entry> {
        let mut entry = Entry::blank();
        entry.created_at = created.into();
        entry.title = title.into();
        entry.body = body.into();
        journal.store.save(&mut entry)?;
        Ok(entry)
    }

    #[test]
    fn whole_day_is_unfiltered_chronological_and_preserves_exact_text() -> Result<()> {
        let (dir, mut journal) = journal()?;
        add(
            &mut journal,
            "2026-10-25T00:01:00+00:00",
            "Other day",
            "Not exported",
        )?;
        add(&mut journal, "2026-10-24T10:00:00+00:00", "Later", "Last\n")?;
        add(
            &mut journal,
            "2026-10-24T09:00:00+01:00",
            "Café & 春 🌿\nTitle line",
            "\nFirst\r\n\nSecond 🌻\n",
        )?;
        add(
            &mut journal,
            "2026-10-24T08:00:00+00:00",
            "Same instant",
            "  whitespace  ",
        )?;
        add(
            &mut journal,
            "2026-10-24T23:30:00-10:00",
            "Date shown, not UTC",
            "Still the 24th",
        )?;
        journal.search("no matching entries".into());
        assert!(journal.entries.is_empty());
        let path = dir.path().join("2026-10-24.md");
        let day = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();
        journal.export_day(day, &path)?;
        assert_eq!(
            fs::read_to_string(&path)?,
            "# 2026-10-24\n\n## Café & 春 🌿\nTitle line\n\n\nFirst\r\n\nSecond 🌻\n\n\n## Same instant\n\n  whitespace  \n\n## Later\n\nLast\n\n\n## Date shown, not UTC\n\nStill the 24th\n\n"
        );
        assert_eq!(journal.day_markdown(day)?, fs::read_to_string(path)?);
        Ok(())
    }

    #[test]
    fn collision_and_failure_never_replace_files_or_change_draft() -> Result<()> {
        let (dir, mut journal) = journal()?;
        journal.edit_body("Fictional draft 🌿\nSecond line".into());
        let draft = journal.draft.clone();
        let day = DateTime::parse_from_rfc3339(&draft.created_at)?.date_naive();
        let path = dir.path().join("existing.md");
        fs::write(&path, b"Keep this file")?;
        assert!(journal.export_day(day, &path).is_err());
        assert_eq!(fs::read(&path)?, b"Keep this file");
        assert!(
            journal
                .export_day(day, &dir.path().join("missing/day.md"))
                .is_err()
        );
        assert!(journal.export_day(day, dir.path()).is_err());
        assert_eq!(journal.draft, draft);
        assert!(!journal.unsaved.load(Ordering::SeqCst));
        assert!(journal.error.is_none());
        Ok(())
    }

    #[test]
    fn failed_draft_is_exported_without_clearing_the_failed_save_gate() -> Result<()> {
        let (dir, mut journal) = journal()?;
        journal.edit_body("Original".into());
        let mut external = journal.draft.clone();
        external.body = "Changed in another window".into();
        journal.store.save(&mut external)?;
        journal.edit_title("Unsaved title 春".into());
        journal.edit_body("Unsaved words\n\nExact ending\n".into());
        let draft = journal.draft.clone();
        let error = journal.error.clone();
        let day = DateTime::parse_from_rfc3339(&draft.created_at)?.date_naive();
        let path = dir.path().join("rescue.md");
        journal.export_day(day, &path)?;
        assert_eq!(
            fs::read_to_string(path)?,
            format!("# {day}\n\n## Unsaved title 春\n\nUnsaved words\n\nExact ending\n\n\n")
        );
        assert_eq!(journal.draft, draft);
        assert_eq!(journal.error, error);
        assert!(journal.unsaved.load(Ordering::SeqCst));
        assert_eq!(journal.store.get(external.id.unwrap())?, external);
        journal.new_entry();
        assert_eq!(journal.draft, draft);
        Ok(())
    }

    #[test]
    fn nonempty_new_drafts_export_but_empty_days_do_not_create_files() -> Result<()> {
        let (dir, mut journal) = journal()?;
        let day = DateTime::parse_from_rfc3339(&journal.draft.created_at)?.date_naive();
        let path = dir.path().join("new.md");
        assert!(journal.export_day(day, &path).is_err());
        assert!(!path.exists());
        journal.draft.body = "Not committed yet 🌿".into();
        journal.export_day(day, &path)?;
        assert_eq!(
            fs::read_to_string(path)?,
            format!("# {day}\n\n## \n\nNot committed yet 🌿\n\n")
        );
        assert!(journal.draft.id.is_none());
        assert!(journal.store.list("")?.is_empty());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn private_permissions_and_leaf_or_ancestor_symlinks_are_refused() -> Result<()> {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let (dir, mut journal) = journal()?;
        journal.edit_body("Fictional private words".into());
        let day = DateTime::parse_from_rfc3339(&journal.draft.created_at)?.date_naive();
        let path = dir.path().join("private.md");
        journal.export_day(day, &path)?;
        assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);
        let link = dir.path().join("link.md");
        symlink(&path, &link)?;
        assert!(journal.export_day(day, &link).is_err());
        let dangling = dir.path().join("dangling.md");
        let missing = dir.path().join("must-not-be-created.md");
        symlink(&missing, &dangling)?;
        assert!(journal.export_day(day, &dangling).is_err());
        assert!(!missing.exists());
        let linked_directory = dir.path().join("linked-directory");
        symlink(dir.path(), &linked_directory)?;
        assert!(
            journal
                .export_day(day, &linked_directory.join("other.md"))
                .is_err()
        );
        assert!(!dir.path().join("other.md").exists());
        assert!(
            journal
                .export_day(day, &linked_directory.join("../escaped.md"))
                .is_err()
        );
        assert_eq!(
            fs::read_to_string(path)?,
            format!("# {day}\n\n## \n\nFictional private words\n\n")
        );
        Ok(())
    }
}
