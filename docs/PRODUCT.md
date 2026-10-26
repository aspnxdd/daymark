# Product and scope

## The experience

A personal place to notice a day: choose an entry in the timeline, write without a formatting toolbar, and trust the save indicator. A quiet sidebar, dated writing surface and spacious typography keep the focus on writing.

The application is named **Daymark**. It is a native Freya/Skia window, not Electron, Tauri, a hosted site or a webview.

## Included

- Multiple entries per day, with title, multiline body and immutable creation date.
- Newest-first sidebar grouped by local creation month. Selecting an entry restores its text.
- Immediate per-edit local commits; title-only entries are valid. A completely blank new draft creates no database row.
- Literal substring search across title and body, ignoring surrounding query whitespace and using Rust Unicode lowercase. `%`, `_`, quotes and SQL-looking text are literals.
- Word count over body whitespace; focus mode; save status; inline errors; retry save.
- Delete confirmation and optimistic revision checks against concurrent windows.
- Empty and no-results states; searching never replaces the open draft.
- Keyboard-accessible controls, platform Ctrl/Cmd shortcuts, offline embedded fonts.

## Deliberately missing

No calendar view/date picker/backdating, rich text, Markdown preview, tags, mood tracking, attachments, account, cloud, sync, encryption, import/export, automatic backups, settings screen, multiple windows UI, installer or auto-update.

## Known MVP limits

- All timeline summaries load into memory. Search scans the database; no FTS or pagination.
- SQLite commits and timeline refreshes run synchronously after edits. Slow disks or a lock can pause input for up to the busy timeout (250 ms), or longer for an OS-level I/O stall. Measure before adding a worker/debounce pipeline.
- Long body text scrolls manually; there is no custom caret-follow scrolling yet. Native IME, assistive technology and platform text selection need broader testing.
- Entry-switching intentionally resets undo history. Undo is not a cross-entry history.
- Conflicting changes keep the draft and veto normal close. There is no merge or explicit discard/reload UI yet; copy your draft to a safe local location before restarting or force-quitting.
- Database contents and backups are plaintext. Permissions are not protection from software running as your user.
- The minimum window is 1060×760 logical pixels. The compact layout is tested, but this is desktop-first, not a mobile UI.

This is a working MVP foundation, not a fully packaged or security-certified product.
