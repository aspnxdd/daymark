use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::Result;

use crate::storage::{Entry, EntrySummary, Store};

pub struct Journal {
    pub store: Store,
    pub entries: Vec<EntrySummary>,
    pub draft: Entry,
    pub query: String,
    pub error: Option<String>,
    pub delete_pending: bool,
    pub focus_mode: bool,
    pub privacy_open: bool,
    pub editor_generation: u64,
    pub unsaved: Arc<AtomicBool>,
}

impl Journal {
    pub fn new(store: Store, unsaved: Arc<AtomicBool>) -> Result<Self> {
        let entries = store.list("")?;
        let draft = entries
            .first()
            .map(|e| store.get(e.id))
            .transpose()?
            .unwrap_or_else(Entry::blank);
        Ok(Self {
            store,
            entries,
            draft,
            query: String::new(),
            error: None,
            delete_pending: false,
            focus_mode: false,
            privacy_open: false,
            editor_generation: 0,
            unsaved,
        })
    }

    pub fn save(&mut self) -> bool {
        match self.store.save(&mut self.draft) {
            Ok(()) => {
                self.unsaved.store(false, Ordering::SeqCst);
                self.error = None;
                self.refresh();
                true
            }
            Err(err) => {
                self.unsaved.store(true, Ordering::SeqCst);
                self.error = Some(format!(
                    "Not saved. {err:#} Close is blocked to protect your draft. Copy your text to a safe place, then retry."
                ));
                false
            }
        }
    }

    pub fn edit_title(&mut self, title: String) {
        if self.draft.title != title {
            self.draft.title = title;
            self.save();
        }
    }

    pub fn edit_body(&mut self, body: String) {
        if self.draft.body != body {
            self.draft.body = body;
            self.save();
        }
    }

    pub fn search(&mut self, query: String) {
        self.query = query;
        self.refresh();
    }

    fn refresh(&mut self) {
        match self.store.list(&self.query) {
            Ok(entries) => self.entries = entries,
            Err(err) => self.error = Some(format!("Could not refresh the timeline. {err:#}")),
        }
    }

    pub fn new_entry(&mut self) {
        if self.unsaved.load(Ordering::SeqCst) && !self.save() {
            return;
        }
        self.draft = Entry::blank();
        self.editor_generation += 1;
        self.delete_pending = false;
        self.query.clear();
        self.refresh();
    }

    pub fn select(&mut self, id: i64) {
        if self.draft.id == Some(id) {
            return;
        }
        if self.unsaved.load(Ordering::SeqCst) && !self.save() {
            return;
        }
        match self.store.get(id) {
            Ok(entry) => {
                self.draft = entry;
                self.editor_generation += 1;
                self.delete_pending = false;
                self.error = None;
            }
            Err(err) => self.error = Some(format!("Could not open the entry. {err:#}")),
        }
    }

    pub fn confirm_delete(&mut self) {
        // A failed save must never turn a delete action into an accidental draft discard.
        if self.unsaved.load(Ordering::SeqCst) {
            return;
        }
        match self.store.delete(&self.draft) {
            Ok(()) => {
                self.draft = Entry::blank();
                self.editor_generation += 1;
                self.delete_pending = false;
                self.error = None;
                self.refresh();
            }
            Err(err) => self.error = Some(format!("Could not delete the entry. {err:#}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn journal() -> Result<(tempfile::TempDir, Journal)> {
        let dir = tempfile::tempdir()?;
        let store = Store::open(dir.path())?;
        Ok((dir, Journal::new(store, Arc::new(AtomicBool::new(false)))?))
    }

    #[test]
    fn edits_persist_and_search_does_not_replace_draft() -> Result<()> {
        let (_dir, mut journal) = journal()?;
        journal.edit_body("A small good thing\nSecond line 🌿".into());
        let id = journal.draft.id.unwrap();
        journal.search("no match".into());
        assert!(journal.entries.is_empty());
        assert_eq!(journal.draft.id, Some(id));
        journal.new_entry();
        assert!(journal.draft.is_blank());
        journal.select(id);
        assert!(journal.draft.body.contains("Second line"));
        journal.delete_pending = true;
        journal.confirm_delete();
        assert!(journal.entries.is_empty());
        Ok(())
    }

    #[test]
    fn failed_save_keeps_draft_and_blocks_navigation_and_deletion() -> Result<()> {
        let (_dir, mut journal) = journal()?;
        journal.edit_body("Original".into());
        let mut external = journal.store.get(journal.draft.id.unwrap())?;
        external.body = "Written by another window".into();
        journal.store.save(&mut external)?;
        journal.edit_body("Unsaved work".into());
        assert!(journal.unsaved.load(Ordering::SeqCst));
        assert!(journal.error.is_some());
        journal.new_entry();
        journal.select(-1);
        journal.confirm_delete();
        assert_eq!(journal.draft.body, "Unsaved work");
        assert_eq!(journal.store.get(external.id.unwrap())?.body, external.body);
        Ok(())
    }
}
