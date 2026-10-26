# Persistence rules

- One authoritative SQLite store; bind user values; commit before acknowledging success.
- Preserve failed drafts and close/navigation/delete gates. No ignored write failures or queue-and-forget autosave.
- IDs must not be reused; updates/deletes must check revisions.
- Migrate transactionally, reject foreign/newer schemas, never reset a real database to fix an error.
- No SQLx compile-time database macros. No raw copying of an open database or manual deletion of SQLite sidecars.
- Test nontrivial storage/model changes against temporary real SQLite databases.

Details: [STORAGE.md](../../docs/STORAGE.md), [BACKUP.md](../../docs/BACKUP.md).
