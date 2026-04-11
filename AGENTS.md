# AGENTS.md

## Build Rules

- This repository is a Tauri application. Release builds that are expected to produce the final desktop executable must use `pnpm tauri build`.
- Do not use `cargo build` or `cargo build --release` as a substitute for the final app build in this repo. Those commands may still be used for Rust-only checks such as `cargo check` or `cargo test`, but not for producing the distributable executable.
- If the user asks for the final `.exe`, build it from the project root with `pnpm tauri build` and then use the generated Tauri artifact, instead of copying binaries directly from a plain Cargo release build.

## Toolchain

- No repo-local `rust-toolchain.toml` is pinned right now. Before changing Rust toolchain behavior, check whether the user wants a repo-pinned version or only a local machine upgrade.
