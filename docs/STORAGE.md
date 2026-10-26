# Local storage assessment

## Decision: bundled SQLite + rusqlite

The journal needs independent entry updates, reliable saves, filtering and a timeline. SQLite provides these in one mature, inspectable, portable local database without a service. `rusqlite` matches a small synchronous desktop MVP and avoids adding an async runtime merely for storage.

| Option | Strengths | Costs for this MVP | Decision |
|---|---|---|---|
| SQLite + rusqlite | Transactions, bound SQL, simple connection, bundled engine | Blocking calls; explicit migrations | **Chosen** |
| SQLite + SQLx | Useful async API/pools when already needed | Runtime/pool machinery and more save coordination | Valid later; use runtime `query`/`query_as`, never `query!`/`query_as!` |
| One JSON file | Human-readable, trivial first prototype | Whole-file rewrites, atomic replacement/recovery, conflicts and searching are our responsibility | Not the authoritative store |
| Markdown per entry | Portable/file-first editing | Metadata conventions, indexing, rename/atomic-write conflicts and synchronization | Consider as explicit export later |
| Embedded key/value store | Local, efficient lookup | Entry metadata indexes/search/migrations still custom; less inspection tooling | No benefit here |

There is exactly one authoritative store. Do not mirror independently writable Markdown files alongside SQLite.

## File and schema

`directories::ProjectDirs::from("dev", "daymark", "Daymark")` chooses the OS local application-data directory; `--data-dir PATH` overrides it. The UI privacy panel shows the resolved database path. Unix permissions are `0700` for the dedicated directory, `0600` for the database. Windows relies on the user's application-data ACLs; custom Windows ACL hardening has not been implemented.

Existing journals remain compatible. The default directory now uses Daymark's name; no existing files are moved or deleted automatically. Use `--data-dir PATH` with the existing dedicated journal directory to continue using a previous installation's database.

Schema version is `PRAGMA user_version = 1`. SQLite's `application_id` is a stable file-type tag in the database header: decimal `1297044307`. The code expresses it as `const APPLICATION_ID: i32 = 1_297_044_307` and uses that same constant to set and check the header. The tag stays unchanged across app renames so existing journals remain readable. It is **not** authentication, encryption or complete schema validation.

New schema creation is transactional. An unrelated populated database or newer schema is refused rather than reset. Empty-schema detection excludes only SQLite's literal `sqlite_` internal prefix using `GLOB`, not SQL `LIKE` (where `_` means any character).

```sql
CREATE TABLE entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    search_text TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX entries_chronological ON entries(created_at DESC, id DESC);
```

Creation timestamps are RFC 3339 with the original local offset. Dates in the journal reflect that recorded local day; there is no date editing or calendar feature. Ordering uses stored timestamp text and ID, not a cross-time-zone event-log guarantee.

## Filesystem helpers

Keep `std::fs` plus the existing `directories` dependency for OS-specific application-data paths. A wrapper such as `fs-err` could add path context to errors, but would not remove exclusive creation, Unix permission setup/repair or symlink-error handling. Atomic-file-write crates are for replacing files; SQLite already owns transactional writes. Do not add a dependency or generic file layer just to hide these few operations.

## Durability and privacy settings

- `journal_mode = DELETE`: a single serialized connection does not need WAL. Rollback journals may exist while writing; the containing private directory matters.
- `synchronous = FULL`: prefer committed writing over maximum throughput.
- `busy_timeout = 250 ms`: report a lock rather than silently dropping the update.
- `foreign_keys = ON`, `temp_store = MEMORY`, `secure_delete = ON`.
- Transactions and revision guards for updates; revision guards for deletes.
- User text is always bound through parameters. The search query is not interpreted as SQL, LIKE syntax or FTS syntax.

`secure_delete` is best-effort page clearing, **not** a forensic-erasure guarantee. Standard SQLite/bundled rusqlite is **not encrypted**.

## Search and scale

`search_text` is the Rust Unicode-lowercased title + newline + body. Searches trim and lowercase the query, then use SQLite `instr`. This is literal substring search, not stemming, accent folding, language-aware collation or token ranking. Full results are loaded at each refresh.

FTS5, pagination and background/debounced persistence are deferred until measured journal sizes or input latency justify them. Any future FTS index is equally sensitive and must participate in deletion, migrations and backup.

## Evolutions

Schema changes must migrate existing journals transactionally, preserve IDs and revisions, test old fixtures, and never delete the source on migration failure. Keep database backup/restore separate from entry editing. For backups now, close **all** Daymark instances before copying; see [BACKUP.md](BACKUP.md).
