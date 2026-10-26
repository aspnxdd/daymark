# Contributing

Read [AGENTS.md](AGENTS.md), [architecture](docs/ARCHITECTURE.md) and [privacy](docs/PRIVACY.md) first.

- Keep changes within the journal MVP; calendar and network features are not implied follow-up work.
- Explain data migration/recovery implications before changing storage. Never use real journals as fixtures.
- Reuse existing components, theme and helpers. Add a focused test for new logic or a bug regression.
- Include `Cargo.lock` changes only when dependencies actually change; review features and upstream build scripts.
- Run format, Clippy and tests using [the documented commands](docs/TESTING.md). State any native/manual checks you did not run.
- UI changes should update the fictional-data preview and be checked at minimum size with keyboard focus.
- Skills/rules are Markdown-only. Record source revisions and a bounded content review; do not add remote installers or executable hooks.

Do not submit databases, private entry text, screenshots of real writing, secrets or raw crash dumps. Security reporting guidance is in [SECURITY.md](SECURITY.md).
