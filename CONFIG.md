# Configuration

`config.json` is loaded on startup (created automatically if missing). All values are safe defaults for a Windows dev machine.

## Input
- `mouse_sensitivity` (f32): camera look sensitivity multiplier.
- `invert_y` (bool): invert vertical mouse movement.
- `move_speed` (f32): base WASD speed in world units/second.

## Graphics
- `fps_cap` (u32): `0` disables the cap; otherwise target FPS (also used for sleep pacing).
- `vsync` (bool): true uses `PresentMode::AutoVsync`, false uses `AutoNoVsync`.
- `render_distance_chunks` (usize): square radius of loaded chunks around the player.
- `fov_y_degrees` (f32): vertical field of view for the camera.

## World
- `seed` (u32): deterministic seed for terrain noise.

## Multiplayer
- `ip` / `port`: default endpoint.
- `player_name`: displayed name when networking lands.
- `head_color`: RGB floats (0.0–1.0) for the player's head tint.
- `tick_rate` (u16): network tick target (reserved for the net layer).

## Debug
- `show_overlay` (bool): toggles the in-game debug overlay (FPS, frame time, chunks, draw calls).
- `show_fps` (bool): reserved flag for future HUD/metrics toggles.
- `wireframe` (bool): placeholder toggle for render mode.

## Player
- `mode`: `"adventure"` (physics, gravity, collisions, jump) or `"creative"` (free-fly, no collisions). You can also toggle in-game with `F3`.
- `gravity`: downward acceleration in adventure mode.
- `jump_speed`: impulse velocity when jumping in adventure mode.
- `height` / `radius` / `eye_height`: player capsule dimensions and camera eye offset.
- `max_fall_speed`: terminal fall speed clamp.
