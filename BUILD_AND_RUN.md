# Build and Run (Windows-first)

1) Prereqs: Rust toolchain (MSVC), GPU drivers, `cargo` in `PATH`.
2) Build debug: `cargo build`
3) Run debug: `cargo run`
4) Release smoke (recommended): `cargo run --release`
5) Tracy profiling (optional): `cargo run --features tracy` (Tracy feature is opt-in only).

Notes:
- `config.json` is auto-created on first launch; tune FPS cap, vsync, sensitivity, and render distance there.
- Present mode is chosen from config (`AutoVsync`/`AutoNoVsync`); FPS pacing uses the same config cap.
- Cursor capture: `Esc` releases the mouse, any click re-captures.
- Player modes: set `player.mode` in `config.json` (adventure/creative) or toggle live with `F3`.
