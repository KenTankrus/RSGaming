# RSGEWatch

A Rust-based RuneScape investment tracker and notification app.

This repository is intended to be published at https://github.com/KenTankrus/RSGaming.

## Build and Run

Requirements:
- Rust toolchain installed (`rustup`, `cargo`, `rustc`)

Build locally:

```bash
cargo build
```

Build a release binary:

```bash
cargo build --release
```

Run the app in development mode:

```bash
cargo run
```

Run the release executable after building:

```bash
# Windows
./target/release/rsgewatch.exe

# macOS / Linux
./target/release/rsgewatch
```

## GitHub Release Guidance

- Do not commit compiled binaries or build artifacts.
- The `target/` directory is ignored via `.gitignore`.
- Additional ignored files include Windows executables (`*.exe`), debug symbols (`*.pdb`), editor folders, and local logs.
- If you want to distribute a compiled build, attach it to a GitHub Release rather than checking it into the source repository.
- The executable should not be placed in the repository root.

## Release Packaging

1. Run `cargo build --release`.
2. Locate the built binary in `target/release/`.
3. Upload the executable as a release asset or move it outside the repository.

## Repository Contents

Keep source files, configuration, and documentation in the repository.
Generated artifacts such as compiled executables belong in release assets, not in the repository source tree.

## Terms of Use

This application is for tracking RuneScape investments and notifications only.
Do not use this tool to cheat, automate gameplay, or otherwise gain an unfair
advantage in RuneScape 3 or Old School RuneScape (OSRS).
