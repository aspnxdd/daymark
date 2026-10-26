# Architecture

```text
main.rs                 CLI, OS data path, store startup, fonts, native close veto
   │
ui/mod.rs               DaymarkApp state initialization and global shortcuts
ui/sidebar.rs           search, month-grouped timeline and navigation
ui/workspace.rs         writing layout, confirmation/error/privacy UI and prompt
ui/editor.rs            title Input + multiline Freya editable paragraph
ui/components.rs        reusable Action, icons, captions, prose, dividers
ui/theme.rs             one dark-green palette and Freya Theme
   │ events
model.rs / Journal      open draft, search, selection, saving, errors, editor generation
   │ typed operations
storage.rs / Store      one rusqlite connection, schema, SQL, transactions, permissions
   │
journal.sqlite3         authoritative local data
```

## State flow

`DaymarkApp` initializes one `State<Journal>`. Child components receive that handle rather than their own copies of database state. An edit updates `Journal::draft`, calls `Store::save`, and refreshes the filtered timeline. Search updates only `query` and summaries; it never swaps the draft.

The title uses Freya's single-line `Input`. The body retains the tested `use_editable`/`RopeEditor` engine with one wrapped paragraph and a bounded `ScrollView` on pinned Freya 0.5.0-rc.13. It is plain text, not HTML or remotely rendered Markdown.

The body mirrors `Input`'s local pointer-release cancellation so a previously focused input cannot steal its newly requested focus via global outside-click handling. Global release still ends a selection when the pointer is released outside the body.

`editor_generation` keys `WritingArea`. New/select/delete remount its editable state so text, selection and undo history cannot leak between entries, including entries with identical text.

## Save invariants

- Blank unsaved drafts are not inserted. Existing entries can be edited back to empty.
- `Store::save` only mutates draft identity/revision after transaction commit.
- Every update/delete includes both ID and revision. IDs use `AUTOINCREMENT` to prevent delete/recreate identity reuse defeating optimistic concurrency.
- Failed saves keep text in the editor and set a shared atomic unsaved flag. Navigation and deletion are blocked while unresolved.
- `with_on_close` reads that flag and returns `KeepOpen` on failure. Successful edits have already committed synchronously; there is no pending debounce queue to flush.
- A process kill or OS shutdown cannot be vetoed. The unsaved buffer is not crash-recovery storage.

## Deliberate simplifications

One synchronous connection, one model, no ORM/repository trait, async runtime, event bus or service layer. Rendering itself does not perform database writes. I/O occurs in event/model methods. This keeps commit/error ordering obvious but can cause input latency on slow storage; see `docs/PRODUCT.md` before scaling it up.

Freya's programmatic `Platform::close_window` and `exit` bypass native close interception. There are no such calls in Daymark; future Quit actions must preserve the save gate.

## Trust boundaries

SQL parameters, filesystem paths, existing database ownership/version, external skills and upstream snippets are separate boundaries. Do not execute journal content or automatically load remote content. `Store::open` refuses obvious symlink journal files and foreign/newer databases; it is not a hostile-filesystem sandbox or a substitute for user ACLs.
