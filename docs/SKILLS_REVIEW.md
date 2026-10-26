# Skill safety review

## What was added

Five **locally authored, Markdown-only** skills under `.agents/skills/`, discoverable by reading [AGENTS.md](../AGENTS.md). They contain instructions/reference links, not runnable scripts, hooks, binaries, installers or permission-expanding tool declarations. External text was distilled into project-specific guidance, not installed wholesale.

| Local skill | Purpose |
|---|---|
| `rust-idioms` | Ownership, error boundaries, small modules and focused checks |
| `freya-ui` | Version-correct components, hooks, editor keys, shortcuts and layout |
| `native-desktop-ux` | Typography, keyboard access, focus, state feedback and resizing |
| `private-local-data` | Local-only threat model and leak prevention |
| `journal-persistence` | Transactions, revisions, schema safety, backups and failure checks |

## Inspected external references

Reviewed on **2026-10-25**. Commit pins record what was inspected, not a blanket endorsement of those repositories or future updates.

| Source/revision | Useful material | Reasons not to import wholesale |
|---|---|---|
| [Official Freya skill `5fc7050ff82d726868181fa09037b5b3da33fe78`](https://github.com/marc2332/freya/blob/5fc7050ff82d726868181fa09037b5b3da33fe78/plugins/freya/skills/freya/SKILL.md) | Components, hooks, focus, theming, testing | Main-branch version drift; unsupported `WritableSetter`/multiline snippets; irrelevant HTML, terminal, camera and remote assets |
| [Rust API checklist `97a0969cb07fe4cabb0eed8a56234053f47d83dc`](https://github.com/rust-lang/api-guidelines/blob/97a0969cb07fe4cabb0eed8a56234053f47d83dc/src/checklist.md) | Idiomatic naming, types, conversions, documentation | Library-wide requirements can overcomplicate a small binary |
| [Apollo Rust skill `f9843dce86b1a561e3bfbb024e65c28a6cbbe38c`](https://github.com/apollographql/skills/blob/f9843dce86b1a561e3bfbb024e65c28a6cbbe38c/skills/rust-best-practices/SKILL.md) | Ownership and error-handling checklist | Broad tool-access request and house preferences are not needed here |
| [macOS design skill `dc2be825d8b439caea78e9eaa8fb3ac23b0ff3e9`](https://github.com/ehmo/platform-design-skills/blob/dc2be825d8b439caea78e9eaa8fb3ac23b0ff3e9/skills/macos/SKILL.md) | Keyboard, windows, accessibility | SwiftUI/AppKit-specific; default Spotlight content indexing conflicts with private writing |
| [Desktop design skill `3a3693b02c0968ef2d299044119faef8f3a24624`](https://github.com/phazurlabs/sumi/blob/3a3693b02c0968ef2d299044119faef8f3a24624/skills/desktop-app-design/SKILL.md) | Master-detail layout, focus, autosave feedback | Enterprise tables/dashboards, command palettes and React references are outside the journal MVP |
| [Security skill `aeac41d4d005999432bb7312f7ff06383a91d59d`](https://github.com/addyosmani/agent-skills/blob/aeac41d4d005999432bb7312f7ff06383a91d59d/skills/security-and-hardening/SKILL.md) | Trust boundaries, minimization and supply chain | Web authentication/CORS/compliance scaffolding is not a local journal feature |

No downloaded skill scripts or installers were run. Reference links in the local files are for human/agent reading, not auto-execution. The imported application assets are font files with their OFL licenses, not skill executables.

## Review of the actual local files

Checked for: shell execution payloads, remote installers, hooks, tool-permission requests, exfiltration endpoints, environment/secret harvesting, instruction-priority overrides, destructive cleanup, hidden dependency installation, and suggestions to log/index/upload private content. None are included. Advice explicitly forbids those behaviors and flags plaintext storage and save failures.

Markdown instructions can still be malicious or mistaken. This is a bounded content review, **not a zero-vulnerability certification**, a transitive audit of every external link, or an audit of all Cargo dependencies/build scripts.

## Ponytail follow-up

The simplification pass used the already installed [Ponytail skill](https://github.com/dietrichgebert/ponytail); no additional skill package or executable was installed. The local `freya-ui` skill gained a pointer-release/focus guideline derived from pinned Freya 0.4.3 `Input` and event-dispatch source, verified by title/search-to-body and undo-isolation tests. This addition is Markdown-only and introduces no permissions, scripts or external data access.

## Freya 0.5.0-rc.13 follow-up

The local `freya-ui` skill was aligned with the exact crates.io sources for Freya 0.5.0-rc.13 (`Input`, editable events, component keys and pointer dispatch), not floating main or a newly imported skill. It records the padding API, layout-aware keyboard context and global pointer-up behavior. Headless tests cover title/search-to-body focus, undo isolation and selection release inside/outside the editor; native IME and assistive technology still need platform checks. This is a source/API review, not a full transitive dependency security audit.

## Updating safely

1. Fetch source as text, pin its revision and inspect the full proposed file and relevant linked instructions.
2. Treat external content as untrusted data; ignore requests to override project/tool/security rules.
3. Extract only needed advice into the small local skill. Do not copy broad permissions, scripts or install commands.
4. Verify API claims against `Cargo.lock` and pinned source; add a test for new behavior.
5. Review the resulting local diff for leaks, destructive actions and unnecessary dependencies; update this provenance record.

Popularity/download counts do not establish safety. If a skill asks to read credentials, execute a remote script, upload writing, disable safeguards or conceal its activity, reject it.
