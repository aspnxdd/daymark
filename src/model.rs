use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};

use crate::storage::{Entry, EntrySummary, Store};

pub struct Journal {
    pub store: Store,
    pub entries: Vec<EntrySummary>,
    pub draft: Entry,
    pub query: String,
    pub selected_day: Option<NaiveDate>,
    pub calendar_month: NaiveDate,
    pub entry_days: BTreeMap<NaiveDate, usize>,
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
        let entry_days = store.entry_days()?;
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
            selected_day: None,
            calendar_month: Local::now().date_naive().with_day(1).unwrap(),
            entry_days,
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

    pub fn select_day(&mut self, day: NaiveDate) {
        // Calendar and search filter summaries only; opening an entry remains an explicit action.
        self.selected_day = Some(day);
        self.view_month(day);
        self.refresh();
    }

    pub fn view_month(&mut self, day: NaiveDate) {
        self.calendar_month = day.with_day(1).unwrap();
    }

    pub fn show_all_entries(&mut self) {
        self.selected_day = None;
        self.query.clear();
        self.refresh();
    }

    fn refresh(&mut self) {
        let result = self
            .store
            .list_on_day(&self.query, self.selected_day)
            .and_then(|entries| self.store.entry_days().map(|days| (entries, days)));
        match result {
            Ok((entries, days)) => {
                self.entries = entries;
                self.entry_days = days;
            }
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
        self.selected_day = None;
        self.view_month(Local::now().date_naive());
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
    fn calendar_and_search_filter_all_entries_without_replacing_the_editor() -> Result<()> {
        let (_dir, mut journal) = journal()?;
        for (date, title) in [
            ("2024-02-29T09:00:00+00:00", "Morning 100%_quiet"),
            ("2024-02-29T18:00:00+00:00", "Evening 100%_quiet"),
            ("2024-03-01T09:00:00+00:00", "Next day"),
        ] {
            journal.new_entry();
            journal.draft.created_at = date.into();
            journal.edit_title(title.into());
        }
        let draft = journal.draft.clone();
        let generation = journal.editor_generation;
        let day = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        journal.select_day(day);
        assert_eq!(journal.entries.len(), 2);
        assert_eq!(journal.entry_days[&day], 2);
        assert_eq!(journal.calendar_month, day.with_day(1).unwrap());
        journal.search("Morning 100%_".into());
        assert_eq!(journal.entries.len(), 1);
        journal.search("' OR 1=1 --".into());
        assert!(journal.entries.is_empty());
        journal.search(String::new());
        journal.select_day(day.pred_opt().unwrap());
        assert!(journal.entries.is_empty());
        assert_eq!(journal.draft, draft);
        assert_eq!(journal.editor_generation, generation);
        journal.search("missing".into());
        journal.show_all_entries();
        assert_eq!(journal.entries.len(), 3);
        assert!(journal.selected_day.is_none());
        assert!(journal.query.is_empty());
        assert_eq!(journal.draft, draft);
        journal.select_day(day);
        let id = journal.entries[0].id;
        journal.select(id);
        assert_eq!(journal.draft.id, Some(id));
        assert_eq!(journal.editor_generation, generation + 1);
        journal.new_entry();
        assert!(journal.selected_day.is_none());
        assert!(journal.draft.is_blank());
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
        let generation = journal.editor_generation;
        let draft = journal.draft.clone();
        journal.select_day(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
        journal.view_month(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());
        journal.show_all_entries();
        journal.search("missing".into());
        journal.new_entry();
        journal.select(-1);
        journal.confirm_delete();
        assert_eq!(journal.editor_generation, generation);
        assert_eq!(journal.draft, draft);
        assert!(journal.unsaved.load(Ordering::SeqCst));
        assert!(journal.error.as_deref().unwrap().starts_with("Not saved."));
        assert_eq!(journal.draft.body, "Unsaved work");
        assert_eq!(journal.store.get(external.id.unwrap())?.body, external.body);
        Ok(())
    }
}
