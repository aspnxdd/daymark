---
name: journal-persistence
description: Use when changing SQLite storage, autosave, search, concurrency, migrations, deletion, backup or draft recovery in Daymark.
---

# SQLite is the source of truth

1. Use existing `rusqlite`/`Store`, bound parameters and transactions. No server, independent file mirror or SQLx compile-check macros. If SQLx is introduced later, use runtime queries.
2. Never change draft ID/revision or acknowledge saved before commit. A failed draft remains available in memory; navigation/deletion/native close must not discard it.
3. Use non-reused identities and revision guards for updates/deletes. Test stale writes/deletes and delete/recreate ABA cases.
4. Blank new drafts do not create rows; existing entries may become empty. Creation dates stay stable. Search must treat SQL/wildcard-looking input literally.
5. Keep Unicode lowercase search behavior consistent with writes. Search scans are an MVP limit; add FTS/pagination only after measurements and migration/deletion/backup tests.
6. Preserve `application_id`/`user_version` checks. Migrate transactionally; reject unsupported/foreign schemas. Never reset a user's database to make startup pass.
7. DELETE journaling, FULL synchronous commits and short busy timeout are deliberate. WAL is optional and creates sensitive sidecars; do not change pragmas without durability/backup reasoning.
8. Copy a database only after every writer closes; a live backup must use SQLite's supported snapshot/backup API. Never delete journal/WAL files manually as recovery.
9. `secure_delete` is not encryption or a forensic guarantee. Protect plaintext backup/index data exactly like entries.
10. Test reopen, Unicode/newlines, literal search, conflicts, permissions, schema rejection and error preservation with real temporary databases. No real user data in fixtures.
11. Current per-edit synchronous I/O is a known limit. If a worker/debounce is added, serialize writes, acknowledge exact revisions, retain errors and flush/veto close; do not silently queue-and-forget.

References: [assessment](../../../docs/STORAGE.md), [recovery](../../../docs/BACKUP.md), [SQLite durability](https://sqlite.org/pragma.html#pragma_synchronous), [Online Backup API](https://sqlite.org/backup.html), [corruption pitfalls](https://sqlite.org/howtocorrupt.html).
