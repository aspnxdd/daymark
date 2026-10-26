# ADR 0002: pin Freya 0.5.0-rc.13

**Status:** accepted for MVP · **Date:** 2026-10-25

## Context

Current upstream skill/main and published packages expose different controlled-input APIs. The MVP originally pinned stable 0.4.3; the requested upgrade moves it to 0.5.0-rc.13. A reliable version boundary matters more than following latest snippets.

## Decision

Pin both `freya` and `freya-testing` to `=0.5.0-rc.13`, commit `Cargo.lock`, retain single-line Inputs for title/search and the tested editable-paragraph engine for multiline body text. Keep that wiring in `src/ui/editor.rs` and test actual text events/persistence. Align the key, padding, cursor and editable-event APIs with the published rc.13 sources; retain global pointer-up handling for selection release outside the body.

## Consequences

Reproducible prerelease baseline and no Git dependency; more manual body-editor wiring, with long-document caret-follow and native IME/accessibility checks still needed. Keep Freya's existing default features plus `icons-lucide`, without `all` or `remote-asset`. The lockfile refresh also aligns `prettyplease` with the new `bindgen`/`syn` versions to avoid a transitive build mismatch. A future editor simplification must preserve keyboard, selection, entry-history isolation and save/error behavior. Do not mix APIs from Dioxus-era Freya or floating main.
