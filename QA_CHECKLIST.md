# QA Checklist (current build)
- Input: cursor capture toggles with `Esc`/click; mouse look smooth; sensitivity/invert respected from config.
- FPS cap: window title stable? (see overlay) verify config `fps_cap` + `vsync` combo keeps ~target.
- Overlay: debug panel shows FPS/ms/chunk counts/draw calls and updates every frame.
- World: chunks stream around player without crashes; raycast place/break still works after config changes.
- Physics: adventure mode blocks collisions/gravity/jump, can’t clip into blocks; creative mode allows free flight (toggle with `F3` or config).
- Build: `cargo build`, `cargo run`, `cargo run --release` all succeed on Windows MSVC toolchain.
