# Testing

## Automated checks

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

The suite uses temporary directories, real bundled SQLite and Freya's real headless runner, not mocked persistence. It needs no compile-time database, account or server.

Coverage includes:

- Blank drafts, Unicode/multiline round trips and reopening the store.
- Literal search (`%`, `_`, quotes, SQL-looking text), updates and deletion.
- Revision conflicts and deletion/recreation without ID reuse.
- Foreign/newer database rejection, including tables such as `sqliteXnotes` that resemble an internal prefix; Unix permissions and obvious symlink refusal.
- Failed saves retaining drafts and blocking navigation/deletion.
- CLI argument validation and Daymark's default data-directory identity.
- The stable SQLite file-format tag across app renames.
- Daymark branding at both supported window sizes.
- Rendered UI typing/newlines, persisted output, Ctrl/Cmd search/new, entry switching, delete/cancel/confirm, title editing, focus mode, and minimum-size layout.
- Independent title/body undo histories, identical-text entry isolation, search-focus restoration from focus mode, and body drag-selection stopping on release both inside and outside the editor.

The screenshot generator is intentionally ignored in the normal suite:

```sh
cargo test --locked render_documentation_screenshot -- --ignored
```

It writes `docs/images/daymark.png` from fictional entries in a temporary database. No demo entries are seeded into the real application. Review the image when changing layout, typography or theme.

Freya-testing 0.4.3's `TestingNode::is_visible()` can panic for some text nodes with no effect-state entry. Tests use measured geometry and interaction/persistence assertions instead; native visibility and clipping still need visual checks.

## Native smoke check

Run against a disposable data directory, wait for a visible window, type a fictional entry, close and reopen. On the development Linux environment, the app launched without errors and created a mode-`0600` database. A timed startup smoke run is not a full interactive platform certification.

## Manual matrix before a release

- [ ] Linux X11 and Wayland, macOS and Windows: launch, resize and reopen.
- [ ] Ctrl/Cmd+N/F/S from body, title, search and a button; Tab and Shift+Tab order.
- [ ] Text selection, cut/copy/paste, undo/redo, Unicode, CJK IME, dead keys, emoji, long documents.
- [ ] Long-body scrolling and cursor position after edits; long titles/searches.
- [ ] Empty journal, many entries, no search results, focus mode, error and delete panels at minimum size.
- [ ] Disk-full/read-only/locked storage: visible error, exact draft retained, native close vetoed; successful retry clears it.
- [ ] Two processes: stale edit/delete cannot overwrite a newer entry; copy/recovery instructions are understandable.
- [ ] Screen reader and high-DPI/text scaling: accessible names, focus, caret, contrast.
- [ ] Backup and restore using fictional but nontrivial data.
- [ ] Dependency advisories via current RustSec/cargo-audit tooling.

The automated conflict test exercises the failed-save gate, but it does not simulate power loss, disk failure or verify native close behavior on every platform. Never turn those unchecked cases into guarantees.
