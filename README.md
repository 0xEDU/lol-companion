# League Companion

A Windows prototype written in Rust and GPUI. It shows a small status window and a click-through overlay while the League game window is focused. The overlay text and click-through were verified in a live Borderless-mode game. This version does not connect to a game-data provider.

## Build on Windows

Install the Rust toolchain listed in `rust-toolchain.toml`, Visual Studio 2022 Build Tools with the C++ x64/x86 compiler, and the Windows 11 SDK 10.0.26100. Open an x64 Native Tools Command Prompt for VS 2022 and run:

```bat
cargo build --locked
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
```

Run `target\debug\lol-companion.exe` on Windows. Test the overlay with League in borderless mode. A live fullscreen test did not show the overlay.

## Current limitations

The current prototype uses the League game window's title to detect it; changes to the title can break detection. Focus changes, resizing, and DPI behavior still need live-game validation. Provider selection and OP.GG data are planned, not implemented. Do not include account credentials or personal game data in logs or fixtures.

A Windows GNU cross-build from WSL previously compiled but failed at runtime: GPUI 0.2.2 used a Linux build path to locate debug shaders. Native Windows builds avoid that particular path mismatch. This does not establish that WSL cross-builds or optimized builds work.
