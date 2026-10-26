# Privacy and threat model

## What Daymark does

Journal text is stored in a local SQLite file. The app has no accounts, analytics, telemetry, synchronization, remote images, remote fonts or application network requests. Fonts and icons are bundled. Journal text is rendered as text, not executed or interpreted as HTML.

On Unix, Daymark restricts the dedicated data directory to owner-only `0700` and the database to `0600`. It refuses obvious symlink journal paths and checks the database application ID/schema. SQLite temporary storage is kept in memory; secure deletion is enabled as a best-effort hygiene measure.

## What it does not protect

**SQLite is not encrypted.** Anyone able to read the database can read the writing and search index. There is no password, app lock, SQLCipher integration or key management. The UI lock symbol communicates local-only storage, not encryption.

Permissions do not protect against:

- Malware or other software running as your user, root/admin or an unlocked session.
- Screenshots, screen recording, swap, hibernation, crash dumps or clipboard history.
- OS/cloud backups, filesystem snapshots, indexing or synchronization you enable.
- A hostile filesystem or every symlink/hard-link race; path checks are defense in depth, not a sandbox.
- Physical theft without full-disk encryption.

Use OS disk encryption, a protected login and a trusted local disk. Avoid shared/synced data directories. Windows uses inherited user-directory ACLs; custom ACL hardening and platform certification are not part of the MVP.

## Saving and deletion

Successful save status follows a successful SQLite commit with `synchronous=FULL`; the OS/device still determines final hardware durability. On failure the draft stays in memory and normal close/navigation are blocked. A kill, crash or shutdown can still lose an unsaved buffer.

Delete requires confirmation and removes the current database row. `secure_delete=ON` does **not** guarantee removal from backups, SSD wear-leveling, caches or filesystem snapshots. Do not claim forensic erasure.

## Information leaving the app

Daymark makes no network requests, but your OS, backup tools and clipboard manager are outside its control. Clipboard contents leave the app when you copy text. Build tools/dependency downloads are also separate from runtime behavior. No software can promise absolute privacy solely because it uses a local file.

## Development policy

Never put real writing, queries or database dumps in logs, tests, screenshots, issue trackers or AI prompts. Use fictional fixtures. Reports should describe behavior and sanitized error types. See [SECURITY.md](../SECURITY.md) for disclosure guidance and [BACKUP.md](BACKUP.md) for safe copies.
