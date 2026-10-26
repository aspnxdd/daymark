# Development

## Toolchain and native libraries

Use a current stable Rust/Cargo with edition 2024 support. The implementation was built with **rustc 1.99.1** on Linux. `Cargo.toml` declares Rust 1.93; that lower bound has not been independently exercised.

Freya renders through Skia and Winit. A compiler toolchain and desktop development libraries are required. On Debian/Ubuntu, a typical starting set is:

```sh
sudo apt install build-essential pkg-config clang cmake \
  libfontconfig1-dev libfreetype6-dev libgl1-mesa-dev libegl1-mesa-dev \
  libx11-dev libx11-xcb-dev libxcursor-dev libxi-dev libxrandr-dev \
  libxkbcommon-dev libwayland-dev
```

Review these package operations yourself; they are not run automatically by a project script. Package names vary by distribution. macOS/Windows builds need their platform Rust/C++ SDKs and have not been tested here.

Skia's upstream build may download prebuilt native artifacts. Building is not promised to be offline; **running Daymark does not perform network requests**. Do not enable Freya's `all`, `remote-asset`, webview or Markdown remote-content features.

## Commands

```sh
cargo run --locked
cargo run --locked -- --data-dir /absolute/path/to/a/dedicated-daymark-folder
cargo run --locked -- --help
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

Choose a dedicated directory: Daymark deliberately restricts its Unix permissions. Do not point `--data-dir` at a shared folder, your home directory, or the checkout. Keep real journals outside version control and cloud-synced folders if local-only storage matters to you.

## Structure and conventions

Read [ARCHITECTURE.md](ARCHITECTURE.md), [AGENTS.md](../AGENTS.md) and the relevant local skill. UI callbacks call the model; SQL belongs in `storage.rs`. No database access at compile time. No generated schema state, local `.env` database URL or SQLx prepare step is needed.

Use `Result` with context at application boundaries; preserve error handling during refactors. The one startup `expect` checks the app's one-time state invariant, not user data. Avoid new dependencies for behavior the standard library or Freya already supplies.

## Debug without leaking writing

Reproduce against a new temporary dedicated directory and fictional entries. Do not paste a real journal, query or database into logs, bug reports, screenshots or agents. Inspect schemas/permissions using test data. Font license files must stay with redistributed binaries/assets.

## Release status

There is no installer, code signing, auto-update, bundle metadata or release pipeline yet. Native launch was smoke-tested on the available Linux display; packaging and other operating systems remain follow-up work.
