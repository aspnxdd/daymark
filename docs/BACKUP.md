# Backup and recovery

The MVP has no automatic backup/export UI. Your journal is `journal.sqlite3`; click the sidebar privacy note for its exact path.

## Make a consistent manual backup

1. Check that the app says **Saved to this device**. Resolve any save error first.
2. Close **all** Daymark windows/processes using this data directory. If a close is vetoed, the draft is not saved: copy the text to a safe local place before doing anything destructive.
3. Copy `journal.sqlite3` to a protected backup directory. Keep the backup private and preferably on encrypted storage.
4. Test a copy of the backup by putting it in a fresh dedicated directory and opening Daymark with `--data-dir` pointing there. Compare fictional verification entries and search results.

Do not copy only the main file while it is live. Current Daymark uses DELETE journaling, but an active transaction/crash may have a `-journal` file. Never manually remove SQLite sidecars to “fix” a database. Future WAL mode would also require consistent snapshot handling.

For a future in-app live backup, use SQLite's Online Backup API or another supported consistent snapshot operation, not raw file copying.

## Restore without destroying the source

1. Close all Daymark processes.
2. Preserve the original database and any sidecars together. Do not overwrite your only copy.
3. Place a verified backup's `journal.sqlite3` into a **new** dedicated directory.
4. Run `cargo run --locked -- --data-dir /absolute/path/to/restored-daymark` and inspect the entries.
5. Keep the old files until you have verified the restore and made another safe backup.

A failed save caused by an external revision conflict cannot be resolved by repeatedly retrying the stale revision. Copy the draft first; no merge/reload UI exists yet. Never delete the database as a troubleshooting step. Newer/foreign schemas are refused deliberately, not silently reset.

Backups remain plaintext and may retain entries deleted later. Backup rotation, encryption and secure destruction are the user's responsibility in this MVP.
