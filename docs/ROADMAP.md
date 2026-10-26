# Follow-up roadmap

This is a priority list, not a promise or permission to expand the current MVP.

## First: confidence and recoverability

1. Native platform matrix: IME, screen readers, clipboard, high DPI, selection and window-close failures.
2. A safe conflict recovery workflow: copy/export draft, explicitly confirm discard/reload, never overwrite another revision.
3. Verified backup/restore UI using SQLite's supported snapshot API.
4. Dependency/advisory review and signed/package builds for chosen operating systems.

## Then: measured editing quality

- Caret-follow scrolling for long text; long-title and compact-state polish.
- Measure per-edit commit latency and timeline search with large journals. If needed, move persistence to a serialized worker with tested commit acknowledgments and a close gate.
- Pagination/virtualization and FTS only after representative measurements.
- Consider a deliberate upgrade to Freya's published multiline Input once the chosen release is stable and parity tests pass.

## Optional product decisions

Portable export/import, tags, attachments and encrypted storage require separate requirements, privacy review and recovery design. No custom cryptography. Calendar is explicitly out of the current scope; do not add it as incidental UI polish. Accounts, sync and AI/network features are not implicit next steps for a local-only journal.
