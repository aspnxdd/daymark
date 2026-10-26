# Research notes and provenance

Research date: **2026-10-25**. External pages are reference data, not instructions to execute. Dependencies are recorded in `Cargo.lock`; upstream `main` examples are not assumed compatible.

## Visual direction

- Daymark uses original naming, copy, a green palette and bundled typefaces, with a sidebar/timeline, dated writing surface and quiet hierarchy.
- Calendar navigation was explicitly requested on 2026-10-26, alongside search and the chronological sidebar.

## Freya

Primary sources: [website](https://freyaui.dev/), [`llms.txt`](https://freyaui.dev/llms.txt), [pinned 0.5.0-rc.13 API docs](https://docs.rs/freya/0.5.0-rc.13/freya/), and [0.5.0-rc.13 testing docs](https://docs.rs/freya-testing/0.5.0-rc.13/freya_testing/).

The official [Freya skill at inspected commit `5fc7050ff82d726868181fa09037b5b3da33fe78`](https://github.com/marc2332/freya/blob/5fc7050ff82d726868181fa09037b5b3da33fe78/plugins/freya/skills/freya/SKILL.md) was read, along with relevant source/examples. It is a useful conceptual reference but targets changing main-branch APIs. The application now pins `freya` and `freya-testing` to `=0.5.0-rc.13`; APIs and enabled features were checked against those published crate sources.

Findings applied here:

- Freya 0.4+ uses its own reactive core, not Dioxus. Stateful reusable UI is a `PartialEq` struct implementing `Component`; hooks run in a stable order inside render.
- Use the pinned `Input` value/validation API for title/search and test callback order, rather than assuming current-main examples apply.
- The body retains the tested upstream editable-paragraph pattern (`use_editable`/`RopeEditor`). In rc.13, keyboard events receive paragraph/layout context and global pointer-up ends drags released outside the body.
- Inputs consume key events by default, including events a root global listener needs. `on_pre_key_down` must let app shortcuts through while preserving Tab/editor behavior.
- `expanded()` is not CSS remaining-space flex in the pinned implementation. Use `Content::flex()` and `Size::flex(1.)` for the constrained sidebar/editor layout.
- `with_on_close` can veto native closes. Programmatic close/exit paths bypass it, so none are added to this MVP.
- Real headless events and rendered snapshots are valuable, but do not certify native IME, accessibility or close behavior.
- Calendar research (2026-10-26): the installed `freya-components-0.5.0-rc.13/src/calendar.rs` provides `CalendarMonth` and `CalendarGrid`, with examples/tests in `tests/calendar.rs`. Enable only `freya/calendar`, which forwards to `freya-components/calendar` and its already-used `chrono` dependency. Reuse these pinned primitives for the sidebar rather than a new date-grid dependency.
- The pinned Freya public API has no native save-file dialog. Its private `rfd` dependency serves startup error messages, so Markdown export uses a labeled local path input rather than adding a dialog dependency. The built-in `Popup` does not trap keyboard focus; an inline export panel avoids covering background controls that remain keyboard-focusable.

See [ADR 0002](adr/0002-freya-version.md) for the exact version pin and retained editor architecture.

## Learning from `marc2332/term`

Inspected [commit `0230f8a97b1902f3fdad19b3f8d64d7e4d4b841f`](https://github.com/marc2332/term/tree/0230f8a97b1902f3fdad19b3f8d64d7e4d4b841f). Its manifest pins Freya Git revision `f748070f9109b06f846ded4a5172faf9e5e53d6e`, reinforcing explicit version alignment.

Useful ideas: reusable components, centralized visual styling, shared reactive state, and children cooperating with app-level shortcut dispatch. Terminal-specific shortcut choices, PTY machinery and session persistence are not journal requirements.

**Not copied:** `src/session.rs:50–56` uses `let _ = std::fs::write(dir.join(file), raw);`, discarding write failures. Periodic best-effort terminal snapshots are not sufficient for irreplaceable writing. Daymark commits transactionally, acknowledges only success and retains failed drafts.

## Rust, desktop UX and privacy sources

- [Rust API Guidelines checklist, commit `97a0969cb07fe4cabb0eed8a56234053f47d83dc`](https://github.com/rust-lang/api-guidelines/blob/97a0969cb07fe4cabb0eed8a56234053f47d83dc/src/checklist.md), [The Rust Book](https://doc.rust-lang.org/book/), [Clippy](https://doc.rust-lang.org/clippy/).
- [GNOME HIG](https://developer.gnome.org/hig/), [Apple HIG](https://developer.apple.com/design/human-interface-guidelines), [Windows design](https://learn.microsoft.com/en-us/windows/apps/design/): keyboard, resizing, hierarchy and accessible names. These are guidance, not a claim of platform certification.
- [SQLite security](https://sqlite.org/security.html), [synchronous settings](https://sqlite.org/pragma.html#pragma_synchronous), [Online Backup API](https://sqlite.org/backup.html), [WAL](https://sqlite.org/wal.html), [database corruption pitfalls](https://sqlite.org/howtocorrupt.html).
- [rusqlite Connection](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html), [SQLx runtime query](https://docs.rs/sqlx/latest/sqlx/fn.query.html), [SQLCipher design](https://www.zetetic.net/sqlcipher/design/), [OWASP cryptographic storage](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html), [RustSec](https://rustsec.org/).

SQLite is preferred over whole-file JSON/Markdown for authoritative storage. DELETE journaling is deliberate for a serialized connection; WAL is not a privacy feature and complicates raw backups. Plaintext and deletion limits are explicit in [PRIVACY.md](PRIVACY.md). SQLx would be acceptable with runtime queries, but it adds no necessary capability here.

## Skill discovery

Community candidates were discovered through skills.sh/search and their source inspected without running installers. Only original, project-specific Markdown guidance was added. The reviewed revisions, rejected assumptions and review limits are in [SKILLS_REVIEW.md](SKILLS_REVIEW.md). No external skill receives authority over this repository's rules.
