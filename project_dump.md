
# Project Snapshot: wgpucraft

## agent.md
`
# agent.md — “Поведение агента”

## Роль

Ты — автономный инженер-агент. Твоя цель — довести проект до статуса “Playable Edition” по ТЗ.

## Главные принципы

* Не задавай вопросов пользователю.
* Если возникают ошибки — сам диагностируй и исправляй.
* Всегда сохраняй работоспособность: после каждого этапа проект должен билдиться.
* Tracy не ломать: любые трейсеры/профилировщики — только через feature/config.

## Алгоритм работы (строго)

1. Прочитать структуру проекта, зависимости, текущий рендер-луп.
2. Сразу сделать Windows build/run, зафиксировать проблемы, исправить.
3. Внедрить config и debug overlay (FPS + frame time).
4. Исправить мышь/ввод.
5. Добиться корректного fps cap 60 и убрать “случайный” лимит 30.
6. Дальше идти по Milestones M2–M8.
7. После каждого milestone:

   * `cargo build` (Windows),
   * `cargo run --release` smoke test,
   * обновить документы.

## Критерии качества

* Простота и стабильность важнее “идеальной” графики.
* Любая дорогая фича (тени/АО/физика) должна иметь:

  * настройку качества в config,
  * возможность отключить.


```

## build.rs
`rust
use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> Result<()> {
    // This tells Cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=assets/*");

    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("assets/");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}

```

## BUILD_AND_RUN.md
`
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

```

## Cargo.toml
`toml
[package]
name = "wgpucraft"
version = "0.1.0"
edition = "2024"


[features]
default = []  # без tracy по умолчанию
tracy = ["dep:tracy-client", "dep:tracing-tracy"]  # флаг для включения

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
bevy_ecs = "0.16.0"
wgpu = "24.0.0"
log = "0.4.27"
env_logger = "0.10.1"
tokio = { version = "1.41.0", default-features = false, features = ["rt"] }
winit = { version = "0.29.15", features = ["rwh_05"]}
bytemuck = { version = "1.14", features = [ "derive" ] }
cgmath = "0.18"
anyhow = "1.0.79"
clap = "4.5.1"
instant = "0.1" #because std::time::Instant panics on WASM
rayon = "1.5"
block-mesh = "0.2.0"
noise = "0.8.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ron = "0.8"
freetype-rs = "0.30"
glam = { version = "0.27", features = ["mint"] }
pollster = "0.3.0"
crossbeam-channel = "0.5"

# tracy-client = { version = "0.18", default-features = false }
# tracing-tracy = "0.11.4"
# Tracy только через feature
tracy-client = { version = "0.18", default-features = false, optional = true }
tracing-tracy = { version = "0.11.4", optional = true }


[dependencies.image]
version = "0.24"
default-features = false
features = ["png", "jpeg"]

[build-dependencies]
anyhow = "1.0.79"
fs_extra = "1.3.0"
glob = "0.3.1"
 


```

## config.json
`json
{
  "input": {
    "mouse_sensitivity": 0.35,
    "invert_y": false,
    "move_speed": 12.0,
    "fly_speed": 18.0
  },
  "graphics": {
    "fps_cap": 60,
    "vsync": true,
    "render_distance_chunks": 32,
    "fov_y_degrees": 60.0,
    "sky_color": [
      0.6,
      0.75,
      0.9
    ]
  },
  "world": {
    "seed": 10,
    "world_name": "default"
  },
  "multiplayer": {
    "ip": "127.0.0.1",
    "port": 7777,
    "player_name": "Player",
    "head_color": [
      0.1,
      0.6,
      0.9
    ],
    "tick_rate": 20
  },
  "debug": {
    "show_overlay": true,
    "show_fps": true,
    "wireframe": false
  },
  "player": {
    "mode": "adventure",
    "gravity": 28.0,
    "jump_speed": 12.0,
    "height": 1.8,
    "radius": 0.35,
    "eye_height": 1.6,
    "max_fall_speed": 48.0
  },
  "terrain": {
    "jobs_in_flight": 8,
    "dirty_chunks_per_frame": 2,
    "min_vertex_cap": 4096,
    "min_index_cap": 8192,
    "land_level": 12
  },
  "ui": {
    "font_path": "assets/fonts/MatrixSans-Regular.ttf",
    "font_size": 18.0,
    "font_weight_px": 0,
    "text_scale": 1.0
  }
}
```

## CONFIG.md
`
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

```

## MULTIPLAYER_TEST.md
`
# Multiplayer Test Plan (placeholder)

Multiplayer features are not wired up yet. This document will track the local 2× client + server test matrix once the networking milestone lands:
- Launch dedicated server/listen-server.
- Connect two clients with distinct names/head colors.
- Verify player replication (position/rotation/block actions) and nameplate visibility when aiming at a player.

Status: to be filled in alongside the networking implementation.

```

## project_dump.md
`
```

## QA_CHECKLIST.md
`
# QA Checklist (current build)
- Input: cursor capture toggles with `Esc`/click; mouse look smooth; sensitivity/invert respected from config.
- FPS cap: window title stable? (see overlay) verify config `fps_cap` + `vsync` combo keeps ~target.
- Overlay: debug panel shows FPS/ms/chunk counts/draw calls and updates every frame.
- World: chunks stream around player without crashes; raycast place/break still works after config changes.
- Physics: adventure mode blocks collisions/gravity/jump, can’t clip into blocks; creative mode allows free flight (toggle with `F3` or config).
- Build: `cargo build`, `cargo run`, `cargo run --release` all succeed on Windows MSVC toolchain.

```

## README.md
`
# Wgpu Minecraft Clone

## Why another minecraft clone?

```sh
cargo build --release --target x86_64-pc-windows-msvc
```

There will never be enough minecraft clones out there! if i really consider myself a game developer even as a hobbyist (for now), a minecraft clone is a must step i have to take!

## What's different about this minecraft clone?

Im not using any game engine, instead im using a pure graphics library (wgpu) and the rust programming language.

I have seen a couple similar minecraft clones but are pretty basic, in general my objective with this repo is to create a playable minecraft clone similar to this one:  
https://github.com/jdah/minecraft-weekend   
 but as i mentioned, using wgpu and rust.

## why not using a game engine?

Because i'm too cool for that, i like to struggle, and that keeps my mind busy, so i decided that a making game from scratch using a pure graphics library would be a great pain to experience.

## Screenshots
![Gameplay](./screenshots/world.png)



## Roadmap

### What has been done ?

* atlas texture
* block rendering
* face culling (only visible block faces are rendered)
* fps controller 
* basic chunk generation
* noise map 

### Work in progress...

* optimize chunk system (pending for occlusion branch)
* chunk culling (pending for occlusion branch)
* terrain generation based on noise map 

### Future features

* greddy mesh algorithm
* block manipulation
* ECS (Entity Component System)
* HUD elements


```

## ТЗ.md
`
# ТЗ.md — “WGPUcraft → Playable Edition (Windows)”

## 1) Цель проекта

Сделать из текущего прототипа WGPUcraft **играбельную** версию “как Minecraft-лайт”:

* нормальная мышь + управление
* стабильный FPS (по умолчанию 60)
* инвентарь + хотбар + взаимодействие с блоками
* адекватная генерация мира (долины/озёра/горы/деревья)
* базовая приятная графика (мягкое освещение + мягкие тени хотя бы приближённо)
* лёгкая физика без просадки FPS
* мультиплеер с различимыми игроками (stickmen + цвет головы + никнейм над игроком при наведении взглядом)
* агент **сам** решает архитектуру, зависимости, подходы, и доводит до рабочего состояния с проверкой

## 2) Ограничения и важные правила

1. **Windows — приоритет №1** (сборка и запуск должны работать на Windows).
2. **Tracy не ломать**: текущая логика “Tracy как отдельная фича/через конфиг” должна сохраниться.

   * Если добавляется ещё профилирование/трейсинг — **только аналогично**: включение через конфиг/feature, по умолчанию выключено.
3. Агент **не задаёт вопросы**, а принимает решения сам.
4. Агент обязан:

   * чинить ошибки сборки,
   * проверять, что проект запускается,
   * добавлять самопроверки (тесты/прогоны/инструкции),
   * при проблемах — сам искать причину и исправлять.

## 3) Definition of Done (приёмка)

Проект считается готовым, если выполнены пункты:

### 3.1 Билд/запуск

* `cargo build` и `cargo run` успешно на Windows (MSVC toolchain).
* Есть документ “BUILD_AND_RUN.md” с инструкциями.
* Есть режим “release”: `cargo run --release` даёт стабильную работу.

### 3.2 Управление и мышь (критично)

* Мышь работает адекватно:

  * захват курсора (toggle: например, `Esc` снимает захват, `Click` возвращает),
  * плавное вращение камеры,
  * настраиваемая чувствительность (config),
  * без рывков/ускорений “как попало”.
* Управление:

  * WASD + прыжок + бег (опционально) + приседание (опционально),
  * скорости настраиваются.

### 3.3 FPS / производительность

* По умолчанию лимит кадров **60 FPS** (и VSync/Frame cap нормально управляются).
* Цель: на средней машине не падать ниже 60 в стартовой зоне.
* Если сейчас “упирается в 30” — агент обязан:

  * найти причину (vsync/лимитер/тайминг/луп),
  * исправить (правильный cap, корректный delta time, winit/wgpu present mode).

### 3.4 Инвентарь и строительство “как в Minecraft”

Минимально обязательное:

* Хотбар на 9 слотов (цифры 1–9, колесо мыши).
* Инвентарь (UI окно):

  * отображение списка блоков/предметов (можно начать с creative-инвентаря),
  * перенос/выбор (drag&drop — по возможности, иначе кликами).
* Разрушение и установка блоков:

  * ЛКМ — ломать (с задержкой/анимацией опционально),
  * ПКМ — ставить,
  * корректный raycast по блокам,
  * нельзя ставить блок внутрь игрока.

### 3.5 Мир и генерация (приятно и разнообразно)

* Должны быть:

  * долины/низины,
  * озёра (вода),
  * горы/холмы,
  * деревья (простая генерация).
* Генерация должна быть детерминированной по seed (seed в config).
* Дистанция прорисовки/симуляции — настраиваемая (config).
* Вода может быть простой (плоская поверхность), но должна выглядеть “норм”.

### 3.6 Графика: мягкое освещение и мягкие тени (лайт-версия)

Требование: “не без шейдеров”, а **приятно**.
Минимальный набор:

* Directional light (условное “солнце”) + ambient.
* “Мягкое освещение” (варианты на выбор агента):

  * простой AO (screen-space AO или voxel AO на вершинах/гранях),
  * либо vertex-AO на кубах,
  * либо упрощённый light propagation без тяжёлой GI.
* “Мягкие тени” (варианты на выбор агента):

  * простые cascaded shadow maps (дорого, но можно ограничить),
  * или одна shadow map малого разрешения + PCF/PCSS-подобное размытие,
  * или контактные тени как пост-эффект (если проще).
    Главное: **визуально** тени/освещение должны быть заметны, но FPS не должен проседать.

### 3.7 Физика (лёгкая, без просадок)

* Игрок:

  * коллизии с блоками,
  * гравитация,
  * прыжок,
  * скольжение/ступеньки (упрощённо допустимо).
* “Ненапряжная” физика:

  * агент может выбрать готовую либу (например, rapier3d) **или** написать простую свою.
  * Важнее стабильный FPS и предсказуемость.
* Опционально: простые “физические” блоки (падающий песок) — не обязательно, но приветствуется.

### 3.8 Мультиплеер (обязателен)

Архитектура на усмотрение агента, но требования такие:

**Функционал**

* Режим сервер/клиент (dedicated server или listen-server).
* Несколько игроков одновременно.
* Игроки видят друг друга в мире и синхронизируются:

  * позиция/поворот,
  * выбранный блок/анимация рук (минимально — ок),
  * установки/разрушения блоков синхронно всем.

**Различимость игроков**

* Модель игрока: “stickman” (палочный человечек) + голова “огурчик/сфера/капсула”.
* При входе:

  * выбор цвета головы (палитра),
  * ввод/выбор никнейма.
* Никнейм:

  * отображается **над игроком**,
  * показывается **только если игрок в прицеле/взгляде** (например, если crosshair направлен на него и он в разумной дистанции),
  * или “fade in/out” допустимо.

**Быстрота**

* Минимальная задержка и экономия трафика:

  * снапшоты/дельты,
  * интерполяция на клиенте,
  * тикрейт/частота обновлений настраиваемая.

**Проверка**

* Агент обязан сам проверить мультиплеер:

  * минимум 2 клиента + сервер локально,
  * описать шаги проверки в “MULTIPLAYER_TEST.md”.

### 3.9 Ассеты и отсутствие текстур

* Если нет готовых текстур/картинок:

  * агент делает процедурные материалы (solid colors / palette),
  * выбирает “красивые” сочетаемые цвета сам (палитра).
* Не блокировать прогресс из-за ассетов: всё должно работать без внешних ресурсов.

---

## 4) Конфигурация и UX

Должен быть единый конфиг (например, `config.toml` или `config.json`) с параметрами:

* mouse_sensitivity
* invert_y (опционально)
* fps_cap (default 60)
* vsync on/off
* render_distance (chunks)
* seed
* multiplayer: ip/port, player_name, head_color
* debug flags (wireframe, show_fps, etc.)
* профилирование/трейсинг (Tracy) — **включение только через config/feature**

---

## 5) Техническая архитектура (как должен думать агент)

Агент сам выбирает решения, но обязан обеспечить:

### 5.1 Модули/слои (минимум)

* `core/` — тайминг, config, логирование
* `render/` — wgpu пайплайны, шейдеры, материалы
* `world/` — чанки, генерация, блоки, сохранение (опционально)
* `player/` — контроллер, коллизии
* `ui/` — хотбар/инвентарь/текст
* `net/` — протокол, клиент/сервер, репликация

### 5.2 Обязательная оптимизация вокселей

* Мешинг чанков должен быть адекватным:

  * greedy meshing (желательно) или хотя бы нормальное объединение граней,
  * culling невидимых граней,
  * пересоздание меша только для изменённых чанков.
* Батчинг/инстансинг по возможности.
* Минимизировать draw calls и перерасчёты.

---

## 6) Самопроверка агента (обязательно)

Агент обязан добавить “гарантии”, что оно реально работает:

1. **Автоматические проверки**

* `cargo fmt` / `cargo clippy` без критичных ошибок.
* Минимум один тест (например, генерация чанка детерминирована по seed).
* Опционально: GitHub Actions workflow (если репо публичное) — Windows build.

2. **Ручные чек-листы**

* `QA_CHECKLIST.md`:

  * мышь (захват, чувствительность),
  * ломание/постановка,
  * инвентарь,
  * генерация биомов/воды/деревьев,
  * FPS 60 cap,
  * мультиплеер шаги.

3. **Встроенная диагностика**

* overlay: FPS, frame time, количество чанков, draw calls (если легко).

---

## 7) План работ (Milestones)

Агент должен выполнять в таком порядке (чтобы всегда было “играбельно”):

**M1 — Стабильный Windows билд + управление**

* фиксы сборки
* тайминг + fps cap + vsync
* исправление мыши и захвата

**M2 — Блоки и взаимодействие**

* raycast
* place/break
* хотбар

**M3 — Инвентарь UI**

* окно инвентаря
* выбор блоков

**M4 — Генерация мира**

* seed
* озёра/горы/долины
* деревья
* параметры в конфиге

**M5 — Оптимизация FPS**

* мешинг/чанки
* устранение лишних пересборок/копий
* цель: стабильные 60

**M6 — Графика**

* освещение + AO
* мягкие тени (упрощённо)
* приятная палитра/материалы

**M7 — Физика**

* коллизии игрока
* лёгкая физика без просадок

**M8 — Мультиплеер**

* server/client
* синк игроков и блоков
* stickmen + цвет + ник
* проверка мультиплеера

---

## 8) Итоговые артефакты (что агент должен оставить в репозитории)

* Рабочий код
* `BUILD_AND_RUN.md`
* `QA_CHECKLIST.md`
* `MULTIPLAYER_TEST.md`
* `CONFIG.md` (описание параметров)
* (желательно) `CHANGELOG.md` с кратким списком улучшений



```

## assets\shaders\hud.wgsl
`wgsl
// Vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var texture_sampler: sampler;
@group(0) @binding(1) var texture_atlas: texture_2d<f32>;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    return textureSample(texture_atlas, texture_sampler, in.uv);
}
```

## assets\shaders\insntaces_shader.wgsl
`wgsl
// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}
// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
 
```

## assets\shaders\shader.wgsl
`wgsl
// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    fog_start: f32,
    fog_end: f32,
    // x,y храним sky.r sky.g для совпадения цвета тумана и неба
    sky_rg: vec2<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) view_dist: f32,
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(vertex.position, 1.0);
    out.view_dist = distance(vertex.position, camera.camera_pos.xyz);
    return out;
}
// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // World-space distance based fog.
    let fog_factor = clamp(
        (in.view_dist - camera.fog_start) / (camera.fog_end - camera.fog_start),
        0.0,
        1.0,
    );
    // Цвет тумана совпадает с цветом неба/clear (см. renderer), чтобы шов не выделялся.
    let fog_color = vec3<f32>(camera.sky_rg.x, camera.sky_rg.y, camera.camera_pos.w);
    return vec4<f32>(mix(base_color.rgb, fog_color, fog_factor), base_color.a);
}
 

```

## assets\shaders\text.wgsl
`wgsl
struct VSIn {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VSOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0) var atlas_tex: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@group(1) @binding(0) var<uniform> globals: mat4x4<f32>;

@vertex
fn vs_gui(input: VSIn) -> VSOut {
    var out: VSOut;
    // positions for GUI already in clip space (-1..1)
    out.clip = vec4<f32>(input.pos, 1.0);
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

@vertex
fn vs_world(input: VSIn) -> VSOut {
    var out: VSOut;
    out.clip = globals * vec4<f32>(input.pos, 1.0);
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

@fragment
fn fs_gui(in: VSOut) -> @location(0) vec4<f32> {
    let alpha = textureSample(atlas_tex, atlas_sampler, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

@fragment
fn fs_world(in: VSOut) -> @location(0) vec4<f32> {
    let alpha = textureSample(atlas_tex, atlas_sampler, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
```

## assets\ui\menu.ron
`ron
(
    id: "menu_root",
    layout: (Absolute: (rect: (x: Px(0.0), y: Px(0.0), w: Percent(1.0), h: Percent(1.0)))),
    children: [
        (
            id: "menu_title",
            layout: (Absolute: (rect: (x: Percent(0.0), y: Px(20.0), w: Percent(1.0), h: Px(32.0)))),
            element: (Label: (text: "WGPUCraft", font_size: 16.0)),
        ),
        (
            id: "menu_buttons",
            layout: (FlexColumn: (gap: 10.0, padding: 20.0, align: Stretch)),
            element: (Panel: (color: [18, 22, 30, 220])),
            children: [
                (
                    id: "resume",
                    layout: (Absolute: (rect: (x: Percent(0.0), y: Px(0.0), w: Percent(1.0), h: Px(60.0)))),
                    element: (Button: (text: "Resume", detail: "Back to game", padding: 14.0, min_height: 60.0)),
                ),
                (
                    id: "open_world",
                    layout: (Absolute: (rect: (x: Percent(0.0), y: Px(0.0), w: Percent(1.0), h: Px(60.0)))),
                    element: (Button: (text: "Open World", detail: "Load an existing save", padding: 14.0, min_height: 60.0)),
                ),
                (
                    id: "settings",
                    layout: (Absolute: (rect: (x: Percent(0.0), y: Px(0.0), w: Percent(1.0), h: Px(60.0)))),
                    element: (Button: (text: "Settings", detail: "Video & controls", padding: 14.0, min_height: 60.0)),
                ),
            ],
        ),
    ],
)

```

## src\launcher.rs
`rust
use log::{error, info};
#[cfg(feature = "tracy")]
use tracy_client::span;
use winit::{event::Event, event_loop::ControlFlow};

use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{State, core::config::AppConfig};

pub fn run() {
    env_logger::init();

    info!("Booting WGPUCraft");

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let config = AppConfig::load_or_default("config.json").unwrap_or_else(|err| {
        error!("Falling back to default config: {:?}", err);
        AppConfig::default()
    });

    let mut state = State::new(&window, config);
    state.initialize();

    event_loop
        .run(
            move |event, elwt: &winit::event_loop::EventLoopWindowTarget<()>| match event {
                Event::WindowEvent { window_id, event } if window_id == state.window.id() => {
                    state.handle_window_event(event, elwt)
                }
                Event::DeviceEvent { ref event, .. } => {
                    #[cfg(feature = "tracy")]
                    let _span = span!("handling device input");

                    state.handle_device_input(event, elwt);
                }
                Event::AboutToWait => {
                    state.handle_wait(elwt);
                }
                _ => (),
            },
        )
        .unwrap();
}

```

## src\lib.rs
`rust
pub mod core;
pub mod ecs;
pub mod hud;
pub mod launcher;
pub mod player;
pub mod render;
pub mod terrain_gen;
pub mod text;
pub mod ui;

use hud::{HUD, OverlayStats, icons_atlas::IconType};
use player::{Player, camera::Camera, raycast::Ray};
use std::time::{Duration, Instant};

use core::config::AppConfig;
use render::{
    atlas::MaterialType,
    pipelines::{GlobalModel, Globals},
    renderer::Renderer,
};
use terrain_gen::{chunk::CHUNK_AREA, generator::TerrainGen};
use wgpu::BindGroup;
use winit::{
    dpi::PhysicalPosition,
    event::{self, DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoopWindowTarget,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window},
};

#[cfg(feature = "tracy")]
use tracy_client::{frame_mark, span};

use hud::MenuAction;
use hud::MenuPage;

#[derive(PartialEq)]
pub enum GameState {
    PLAYING,
    MENU,
}

pub struct State<'a> {
    pub window: &'a Window,
    renderer: Renderer<'a>,
    pub config: AppConfig,
    pub data: GlobalModel,
    pub globals_bind_group: BindGroup,
    pub player: Player,
    pub terrain: TerrainGen,
    pub hud: HUD,
    state: GameState,
    last_frame_time: Instant,
    frame_target: Option<Duration>,
    selected_block: Option<cgmath::Vector3<i32>>,
    menu_page: MenuPage,
}

impl<'a> State<'a> {
    pub fn new(window: &'a Window, config: AppConfig) -> Self {
        let frame_target = config.target_frame_time();
        let mut renderer = Renderer::new(&window, config.present_mode(), config.graphics.sky_color);

        let data = GlobalModel {
            globals: renderer.create_consts(&[Globals::default()]),
        };

        let hud = HUD::new(
            &renderer,
            &renderer.layouts.global,
            renderer
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("HUD Shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("../assets/shaders/hud.wgsl").into(),
                    ),
                }),
            config.debug.show_overlay,
            &config.ui,
        );

        let globals_bind_group = renderer.bind_globals(&data);

        let camera = Camera::new(
            &renderer,
            (8.0, 12.0, 8.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(-20.0),
            config.graphics.render_distance_chunks,
            config.input.move_speed,
            config.input.mouse_sensitivity,
            config.input.invert_y,
            config.graphics.fov_y_degrees,
        );

        let mut player = Player::new(camera, config.input.move_speed, &config);
        player.set_mode(config.player.mode.clone(), &config);

        let terrain = TerrainGen::new(&renderer, &config);

        Self {
            window,
            renderer,
            config,
            data,
            globals_bind_group,
            player,
            terrain,
            hud,
            state: GameState::PLAYING,
            last_frame_time: Instant::now(),
            frame_target,
            selected_block: None,
            menu_page: MenuPage::Main,
        }
    }

    pub fn handle_wait(&mut self, _elwt: &EventLoopWindowTarget<()>) {
        self.window.request_redraw();
    }

    // TODO: пробрасывать глобальные настройки параметром
    pub fn handle_window_event(&mut self, event: WindowEvent, elwt: &EventLoopWindowTarget<()>) {
        if !self.handle_input_event(&event) {
            match event {
                WindowEvent::CloseRequested => elwt.exit(),

                WindowEvent::Resized(physical_size) => {
                    self.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    self.render_frame(elwt);
                }

                WindowEvent::CursorMoved { position, .. } => {
                    if self.state == GameState::MENU {
                        let clip_x =
                            (position.x as f32 / self.renderer.size.width as f32) * 2.0 - 1.0;
                        let clip_y =
                            -((position.y as f32 / self.renderer.size.height as f32) * 2.0 - 1.0);
                        self.hud
                            .hover_menu(clip_x as f32, clip_y as f32, &self.renderer.queue);
                    }
                }

                // Обработка событий мыши
                WindowEvent::MouseInput { state, button, .. } => {
                    if self.state == GameState::MENU {
                        if state == ElementState::Pressed && button == MouseButton::Left {
                            if let Some(action) = self.hud.click_menu() {
                                self.handle_menu_action(action, elwt);
                            }
                        }
                        return;
                    }

                    match (button, state) {
                        // ЛКМ — ломаем блок (ставим воздух)
                        (MouseButton::Left, ElementState::Pressed) => {
                            let ray = Ray::from_camera(
                                &self.player.camera,
                                self.player.max_interact_range(),
                            );
                            let ray_hit = ray.cast(&self.terrain.chunks);

                            if let Some(hit) = ray_hit {
                                let updated = self
                                    .terrain
                                    .chunks
                                    .set_block_material(hit.position, MaterialType::AIR);
                                self.terrain.remesh_chunks_now(
                                    &self.renderer.device,
                                    &self.renderer.queue,
                                    &updated,
                                );
                                println!("Блок удалён: {:?}", hit.position);
                            } else {
                                println!("Нет блока для удаления");
                            }
                        }
                        (MouseButton::Right, ElementState::Pressed) => {
                            let ray = Ray::from_camera(
                                &self.player.camera,
                                self.player.max_interact_range(),
                            );
                            let ray_hit = ray.cast(&self.terrain.chunks);

                            if let Some(hit) = ray_hit {
                                let target_pos = hit.neighbor_position();
                                if self.player.intersects_block(target_pos) {
                                    println!("Слишком близко к игроку, блок не поставлен");
                                } else {
                                    let material = self.hud.selected_icon().to_material();

                                    let updated = self
                                        .terrain
                                        .chunks
                                        .set_block_material(target_pos, material);
                                    self.terrain.remesh_chunks_now(
                                        &self.renderer.device,
                                        &self.renderer.queue,
                                        &updated,
                                    );
                                    println!("Поставили блок в: {:?}", target_pos);
                                }
                            } else {
                                println!("Нет блока для установки");
                            }
                        }
                        (MouseButton::Middle, ElementState::Pressed) => {
                            let ray = Ray::from_camera(
                                &self.player.camera,
                                self.player.max_interact_range(),
                            );
                            let ray_hit = ray.cast(&self.terrain.chunks);

                            if let Some(hit) = ray_hit {
                                if let Some(block) =
                                    self.terrain.chunks.get_block_material(hit.position)
                                {
                                    if let Some(icon) = IconType::from_material(block) {
                                        self.hud.select_by_icon(icon, &self.renderer);
                                    }
                                }
                            } else {
                                println!("Нет блока для копирования");
                            }
                        }
                        _ => {}
                    }
                }

                WindowEvent::MouseWheel { delta, .. } => match delta {
                    event::MouseScrollDelta::LineDelta(_, y) => {
                        let direction = if y > 0.0 { 1 } else { -1 };

                        match direction {
                            1 => self.hud.select_next(&self.renderer),
                            -1 => self.hud.select_prev(&self.renderer),
                            _ => {}
                        }
                    }
                    event::MouseScrollDelta::PixelDelta(pos) => {
                        if pos.y > 0.0 {
                            self.hud.select_next(&self.renderer);
                        } else if pos.y < 0.0 {
                            self.hud.select_prev(&self.renderer);
                        }
                    }
                },

                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match self.state {
                    GameState::MENU => self.enter_play_mode(),
                    GameState::PLAYING => self.enter_menu_mode(),
                },
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::F3),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    self.player.toggle_mode(&self.config);
                    self.config.player.mode = self.player.mode.clone();
                    println!("Player mode: {:?}", self.player.mode);
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::F5),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    self.player.toggle_view();
                    println!("Camera view: {:?}", self.player.view_mode());
                }

                _ => {}
            }
        }
    }

    fn render_frame(&mut self, elwt: &EventLoopWindowTarget<()>) {
        #[cfg(feature = "tracy")]
        let _span = span!("redraw request"); // <- Начало блока рендера

        let now = Instant::now();
        if let Some(target) = self.frame_target {
            if now - self.last_frame_time < target {
                // Ещё рано рисовать следующий кадр; не блокируем event loop
                return;
            }
        }

        let mut elapsed = now - self.last_frame_time;
        if elapsed.as_secs_f32() > 0.25 {
            elapsed = Duration::from_millis(250);
        }

        self.last_frame_time = now;
        if self.state == GameState::PLAYING {
            self.player.update(elapsed, &self.terrain.chunks);
            self.terrain.update(
                &self.renderer.device,
                &self.renderer.queue,
                &self.player.camera.position,
            );
        }

        #[cfg(feature = "tracy")]
        let _inner_span = span!("rendering frame"); // <- Начало участка отрисовки кадра
        #[cfg(feature = "tracy")]
        frame_mark();
        self.update(elapsed);
        let stats = OverlayStats {
            fps: if elapsed.as_secs_f32() > 0.0 {
                1.0 / elapsed.as_secs_f32()
            } else {
                0.0
            },
            frame_ms: elapsed.as_secs_f32() * 1000.0,
            chunks_loaded: self.terrain.loaded_chunks(),
            draw_calls: self.terrain.chunk_models.len() + self.hud.draw_call_count(),
        };
        self.hud.update_overlay(&self.renderer, &stats);

        match self
            .renderer
            .render(&self.terrain, &self.hud, &self.globals_bind_group)
        {
            Ok(_) => {}
            // Пересоздаём surface, если она потеряна
            Err(wgpu::SurfaceError::Lost) => self.resize(self.renderer.size),
            // Системе не хватает памяти — выходим
            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
            // Остальные ошибки (Outdated, Timeout) должны пройти к следующему кадру
            Err(e) => eprintln!("{:?}", e),
        }
    }

    fn enter_play_mode(&mut self) {
        let center =
            PhysicalPosition::new(self.renderer.size.width / 2, self.renderer.size.height / 2);
        let _ = self.window.set_cursor_position(center);
        // Сначала пробуем захват в режиме Locked, при ошибке — Confined.
        if self.window.set_cursor_grab(CursorGrabMode::Locked).is_err() {
            let _ = self.window.set_cursor_grab(CursorGrabMode::Confined);
        }
        self.window.set_cursor_visible(false);
        self.state = GameState::PLAYING;
        self.last_frame_time = Instant::now();
        self.hud.close_menu();
    }

    fn enter_menu_mode(&mut self) {
        let _ = self.window.set_cursor_grab(CursorGrabMode::None);
        self.window.set_cursor_visible(true);
        self.state = GameState::MENU;
        self.menu_page = MenuPage::Main;
        self.hud
            .open_menu(self.menu_page, &self.config, &self.renderer.queue);
    }

    pub fn initialize(&mut self) {
        self.enter_play_mode();
        self.last_frame_time = Instant::now();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.player.camera.resize(new_size);
        self.renderer.resize(new_size);
        self.hud.resize(&self.renderer);
    }

    pub fn update(&mut self, _dt: std::time::Duration) {
        #[cfg(feature = "tracy")]
        let _span = span!("update state"); // <- Начало обновления игрового состояния

        self.renderer.update();

        let cam_deps = &self.player.camera.dependants;
        let max_view_distance =
            (self.config.graphics.render_distance_chunks.max(1) * CHUNK_AREA) as f32;
        // Более резкий и короткий туман: начало ~35% дальности, полная плотность к ~50%.
        let fog_start = max_view_distance * 0.35;
        let fog_end = max_view_distance * 0.50;
        let sky_color = self.config.graphics.sky_color;

        self.renderer.update_consts(
            &mut self.data.globals,
            &[Globals::new(
                cam_deps.view_proj,
                [
                    self.player.camera.position.x,
                    self.player.camera.position.y,
                    self.player.camera.position.z,
                ],
                fog_start,
                fog_end,
                sky_color,
            )],
        );

        self.update_block_highlight();
    }

    pub fn handle_input_event(&mut self, event: &WindowEvent) -> bool {
        if self.state == GameState::PLAYING {
            self.player.camera.input_keyboard(&event)
        } else {
            false
        }
    }

    pub fn handle_device_input(&mut self, event: &DeviceEvent, _: &EventLoopWindowTarget<()>) {
        if self.state == GameState::PLAYING {
            self.player.camera.input(event);
        }
    }

    fn update_block_highlight(&mut self) {
        let range = self.player.max_interact_range();
        let ray = Ray::from_camera(&self.player.camera, range);
        let hit = ray.cast(&self.terrain.chunks).map(|h| h.position);
        if hit != self.selected_block {
            self.selected_block = hit;
            self.terrain
                .update_highlight_model(&self.renderer.device, hit);
        }
    }

    fn reload_world(&mut self) {
        self.terrain = TerrainGen::new(&self.renderer, &self.config);
        let camera = Camera::new(
            &self.renderer,
            (8.0, 12.0, 8.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(-20.0),
            self.config.graphics.render_distance_chunks,
            self.config.input.move_speed,
            self.config.input.mouse_sensitivity,
            self.config.input.invert_y,
            self.config.graphics.fov_y_degrees,
        );
        self.player = Player::new(camera, self.config.input.move_speed, &self.config);
        self.player
            .set_mode(self.config.player.mode.clone(), &self.config);
    }

    fn handle_menu_action(&mut self, action: MenuAction, elwt: &EventLoopWindowTarget<()>) {
        match action {
            MenuAction::Resume => {
                self.enter_play_mode();
            }
            MenuAction::CreateWorld => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                self.config.world.world_name = format!("world_{}", now);
                self.config.world.seed =
                    (now as u32).wrapping_mul(1664525).wrapping_add(1013904223);
                self.reload_world();
                self.menu_page = MenuPage::Main;
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::OpenWorld => {
                if let Some(name) = Self::pick_existing_world() {
                    self.config.world.world_name = name;
                    self.reload_world();
                }
                self.menu_page = MenuPage::Main;
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::SaveConfig => {
                if let Err(e) = self.config.write_to("config.json") {
                    eprintln!("Не удалось сохранить config: {e}");
                }
                // keep page
            }
            MenuAction::OpenSettings => {
                self.menu_page = MenuPage::Settings;
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::OpenAdvanced => {
                self.menu_page = MenuPage::Advanced;
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::BackToMain => {
                self.menu_page = MenuPage::Main;
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::CycleFpsCap => {
                let caps = [30, 60, 120, 0];
                let mut idx = caps
                    .iter()
                    .position(|v| *v == self.config.graphics.fps_cap)
                    .unwrap_or(1);
                idx = (idx + 1) % caps.len();
                self.config.graphics.fps_cap = caps[idx];
                self.frame_target = self.config.target_frame_time();
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::ToggleVsync => {
                self.config.graphics.vsync = !self.config.graphics.vsync;
                self.renderer
                    .reconfigure_present_mode(self.config.present_mode());
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::CycleRenderDistance => {
                let opts = [8, 16, 24, 32, 48];
                let mut idx = opts
                    .iter()
                    .position(|v| *v == self.config.graphics.render_distance_chunks)
                    .unwrap_or(3);
                idx = (idx + 1) % opts.len();
                self.config.graphics.render_distance_chunks = opts[idx];
                self.reload_world();
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::CycleFov => {
                let opts = [60.0f32, 75.0, 90.0, 100.0];
                let mut idx = opts
                    .iter()
                    .position(|v| (*v - self.config.graphics.fov_y_degrees).abs() < f32::EPSILON)
                    .unwrap_or(0);
                idx = (idx + 1) % opts.len();
                self.config.graphics.fov_y_degrees = opts[idx];
                self.player
                    .camera
                    .projection
                    .set_fovy_deg(self.config.graphics.fov_y_degrees);
                self.player.camera.update_view();
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::ToggleWireframe => {
                self.config.debug.wireframe = !self.config.debug.wireframe;
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::CycleJobsInFlight => {
                let opts = [2usize, 4, 8, 12, 16];
                let mut idx = opts
                    .iter()
                    .position(|v| *v == self.config.terrain.jobs_in_flight)
                    .unwrap_or(2);
                idx = (idx + 1) % opts.len();
                self.config.terrain.jobs_in_flight = opts[idx];
                self.reload_world();
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::CycleDirtyPerFrame => {
                let opts = [4usize, 8, 16, 32, 64];
                let mut idx = opts
                    .iter()
                    .position(|v| *v == self.config.terrain.dirty_chunks_per_frame)
                    .unwrap_or(3);
                idx = (idx + 1) % opts.len();
                self.config.terrain.dirty_chunks_per_frame = opts[idx];
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::CycleMinVertexCap => {
                let opts = [2048usize, 4096, 8192, 16384];
                let mut idx = opts
                    .iter()
                    .position(|v| *v == self.config.terrain.min_vertex_cap)
                    .unwrap_or(1);
                idx = (idx + 1) % opts.len();
                self.config.terrain.min_vertex_cap = opts[idx];
                self.reload_world();
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::CycleMinIndexCap => {
                let opts = [4096usize, 8192, 16384, 32768];
                let mut idx = opts
                    .iter()
                    .position(|v| *v == self.config.terrain.min_index_cap)
                    .unwrap_or(1);
                idx = (idx + 1) % opts.len();
                self.config.terrain.min_index_cap = opts[idx];
                self.reload_world();
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::CycleLandLevel => {
                let opts = [6usize, 9, 12, 15];
                let mut idx = opts
                    .iter()
                    .position(|v| *v == self.config.terrain.land_level)
                    .unwrap_or(1);
                idx = (idx + 1) % opts.len();
                self.config.terrain.land_level = opts[idx];
                self.reload_world();
                self.hud
                    .open_menu(self.menu_page, &self.config, &self.renderer.queue);
            }
            MenuAction::Quit => elwt.exit(),
        }
    }

    fn pick_existing_world() -> Option<String> {
        let mut entries = std::fs::read_dir("saves").ok()?;
        while let Some(Ok(entry)) = entries.next() {
            if entry.file_type().ok()?.is_dir() {
                return entry.file_name().into_string().ok();
            }
        }
        None
    }
}

```

## src\main.rs
`rust
#[cfg(feature = "tracy")]
use tracy_client::Client;
use wgpucraft::launcher::run;

fn main() {
    #[cfg(feature = "tracy")]
    let _client = Client::start(); // Запускаем клиент Tracy

    run();
}

```

## src\core\config.rs
`rust
use std::{fs, path::Path, time::Duration};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use wgpu;

/// Основная конфигурация, загружаемая из `config.json`.
/// Поля упрощены и покрывают нужные настройки ввода, цикла и размера мира;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub input: InputConfig,
    pub graphics: GraphicsConfig,
    pub world: WorldConfig,
    pub multiplayer: MultiplayerConfig,
    pub debug: DebugConfig,
    pub player: PlayerConfig,
    /// Настройки производительности генерации и стриминга чанков.
    pub terrain: TerrainTuningConfig,
    /// UI-настройки: путь к TTF для HUD.
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub mouse_sensitivity: f32,
    pub invert_y: bool,
    pub move_speed: f32,
    pub fly_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphicsConfig {
    /// Ограничение FPS; `0` отключает лимит и рендерит максимально быстро.
    pub fps_cap: u32,
    pub vsync: bool,
    /// Дальность прорисовки в чанках (квадратный радиус), влияет на размеры стриминга мира.
    pub render_distance_chunks: usize,
    pub fov_y_degrees: f32,
    /// Цвет неба и тумана (RGB, 0.0..1.0).
    pub sky_color: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Путь к TTF-шрифту для интерфейса (относительно корня игры).
    pub font_path: String,
    /// Базовый размер шрифта в пикселях при растеризации.
    pub font_size: f32,
    /// Утолщение шрифта в пикселях (простая дилатация bitmap).
    pub font_weight_px: u32,
    /// Дополнительный масштаб текста при выводе (1.0 — без изменений).
    pub text_scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldConfig {
    pub seed: u32,
    /// Имя мира (папка сохранения в каталоге saves/).
    pub world_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MultiplayerConfig {
    pub ip: String,
    pub port: u16,
    pub player_name: String,
    /// Цвет головы игрока в RGB (значения 0.0..1.0).
    pub head_color: [f32; 3],
    pub tick_rate: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DebugConfig {
    pub show_overlay: bool,
    pub show_fps: bool,
    pub wireframe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerMode {
    Adventure,
    Creative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    pub mode: PlayerMode,
    pub gravity: f32,
    pub jump_speed: f32,
    pub height: f32,
    pub radius: f32,
    pub eye_height: f32,
    pub max_fall_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainTuningConfig {
    /// Сколько задач генерации/ремеша можно держать в полёте одновременно.
    pub jobs_in_flight: usize,
    /// Сколько грязных чанков отправлять на ремеш за кадр.
    pub dirty_chunks_per_frame: usize,
    /// Минимальный стартовый размер буфера вершин для чанка.
    pub min_vertex_cap: usize,
    /// Минимальный стартовый размер буфера индексов для чанка.
    pub min_index_cap: usize,
    /// Уровень воды (y), ниже которого генерируется вода при пустоте.
    pub land_level: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input: InputConfig::default(),
            graphics: GraphicsConfig::default(),
            world: WorldConfig::default(),
            multiplayer: MultiplayerConfig::default(),
            debug: DebugConfig::default(),
            player: PlayerConfig::default(),
            terrain: TerrainTuningConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.35,
            invert_y: false,
            move_speed: 12.0,
            fly_speed: 18.0,
        }
    }
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            fps_cap: 60,
            vsync: true,
            render_distance_chunks: 32,
            fov_y_degrees: 60.0,
            sky_color: [0.60, 0.75, 0.90],
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            font_path: "assets/fonts/MatrixSans-Regular.ttf".to_string(),
            font_size: 16.0,
            font_weight_px: 0,
            text_scale: 1.0,
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 10,
            world_name: "default".to_string(),
        }
    }
}

impl Default for MultiplayerConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 7777,
            player_name: "Player".to_string(),
            head_color: [0.1, 0.6, 0.9],
            tick_rate: 20,
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            show_overlay: true,
            show_fps: true,
            wireframe: false,
        }
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            mode: PlayerMode::Adventure,
            gravity: 28.0,
            jump_speed: 12.0,
            height: 1.8,
            radius: 0.35,
            eye_height: 1.6,
            max_fall_speed: 48.0,
        }
    }
}

impl Default for TerrainTuningConfig {
    fn default() -> Self {
        Self {
            jobs_in_flight: 8,
            dirty_chunks_per_frame: 2,
            min_vertex_cap: 4 * 1024,
            min_index_cap: 8 * 1024,
            land_level: 9,
        }
    }
}

impl AppConfig {
    pub fn load_or_default(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("failed to read config at {}", path.display()))?;
            let cfg: AppConfig = serde_json::from_str(&contents)
                .with_context(|| format!("failed to parse config {}", path.display()))?;
            Ok(cfg)
        } else {
            let cfg = AppConfig::default();
            cfg.write_to(path)?;
            Ok(cfg)
        }
    }

    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<()> {
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(&path, serialized)
            .with_context(|| format!("failed to write config to {}", path.as_ref().display()))
    }

    pub fn target_frame_time(&self) -> Option<Duration> {
        if self.graphics.fps_cap == 0 {
            None
        } else {
            Some(Duration::from_secs_f64(1.0 / self.graphics.fps_cap as f64))
        }
    }

    pub fn present_mode(&self) -> wgpu::PresentMode {
        if self.graphics.vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        }
    }
}

```

## src\core\mod.rs
`rust
pub mod config;

```

## src\ecs\component.rs
`rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::entity::Entity;

pub trait Component: Any + Send + Sync {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Макрос для автоматического получения трэйта `Component`.
/// Позволяет использовать `#[derive(Component)]` на структурах.
#[macro_export]
macro_rules! derive_component {
    ($t:ty) => {
        impl Component for $t {}
    };
}

pub struct ComponentStorage<T> {
    components: HashMap<Entity, T>,
}

impl<T: Component> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, component: T) {
        self.components.insert(entity, component);
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.components.get(&entity)
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        self.components.remove(&entity)
    }

    pub fn has_entity(&mut self, entity: Entity) -> bool {
        self.components.contains_key(&entity)
    }
}

```

## src\ecs\entity.rs
`rust
/// Уникальный идентификатор сущности в мире.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(usize);

impl Entity {
    /// Создаёт сущность с заданным ID.
    /// (На практике только `World` должен создавать сущности).
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Возвращает числовой ID сущности.
    pub fn id(&self) -> usize {
        self.0
    }
}

```

## src\ecs\mod.rs
`rust
pub mod component;
pub mod entity;
pub mod system;
pub mod universe;

```

## src\ecs\query.rs
`rust
```

## src\ecs\system.rs
`rust
use super::universe::World;

pub trait System {
    /// Выполняет логику системы.
    fn run(&self, world: &mut World);
}

```

## src\ecs\universe.rs
`rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::{
    component::{Component, ComponentStorage},
    entity::Entity,
};

pub struct World {
    next_entity_id: usize,
    entities: Vec<Entity>,
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            components: HashMap::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }

    pub fn insert_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self
            .components
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()));

        if let Some(storage) = storage.downcast_mut::<ComponentStorage<T>>() {
            storage.insert(entity, component);
        }
    }

    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get(&type_id)?;
        let storage = storage.downcast_ref::<ComponentStorage<T>>()?;
        storage.get(entity)
    }

    pub fn remove_component<T: Component + 'static>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let types_storage = self.components.get_mut(&type_id)?;
        let storage = types_storage.downcast_mut::<ComponentStorage<T>>()?;
        storage.remove(entity)
    }
}

```

## src\hud\icons_atlas.rs
`rust
use super::HUDVertex;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IconType {
    ROCK,
    GRASS,
    DIRT,
    WATER,
    DEBUG,
}

const ICON_GRID: (f32, f32) = (16.0, 16.0); // сетка 16x16 иконок

impl IconType {
    pub fn all() -> Vec<IconType> {
        vec![
            IconType::ROCK,
            IconType::GRASS,
            IconType::DIRT,
            IconType::WATER,
            IconType::DEBUG,
        ]
    }

    fn get_uv_cords(&self, atlas_size: (f32, f32)) -> [f32; 4] {
        let (x, y) = match self {
            IconType::ROCK => (3, 0),
            IconType::GRASS => (1, 0),
            IconType::DIRT => (2, 0),
            IconType::WATER => (9, 0),
            IconType::DEBUG => (0, 7),
        };

        let tile_w = atlas_size.0 / ICON_GRID.0;
        let tile_h = atlas_size.1 / ICON_GRID.1;
        let u_min = (x as f32 * tile_w) / atlas_size.0;
        let v_min = (y as f32 * tile_h) / atlas_size.1;
        let u_max = u_min + (tile_w / atlas_size.0);
        let v_max = v_min + (tile_h / atlas_size.1);

        [u_min, v_min, u_max, v_max]
    }

    pub fn next(self) -> Self {
        match self {
            IconType::ROCK => IconType::GRASS,
            IconType::GRASS => IconType::DIRT,
            IconType::DIRT => IconType::WATER,
            IconType::WATER => IconType::DEBUG,
            IconType::DEBUG => IconType::ROCK, // циклический переход
        }
    }

    pub fn prev(self) -> Self {
        match self {
            IconType::ROCK => IconType::DEBUG,
            IconType::DEBUG => IconType::WATER,
            IconType::WATER => IconType::DIRT,
            IconType::DIRT => IconType::GRASS,
            IconType::GRASS => IconType::ROCK,
        }
    }

    pub fn get_vertex_quad(
        &self,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
        aspect_correction: f32,
        atlas_size: (f32, f32),
    ) -> ([HUDVertex; 4], [u32; 6]) {
        let uv = self.get_uv_cords(atlas_size);
        let half_width = (width / 2.0) * aspect_correction;
        let half_height = height / 2.0;

        let vertices = [
            HUDVertex {
                position: [center_x - half_width, center_y + half_height], // top-left (y up)
                uv: [uv[0], uv[1]],
            },
            HUDVertex {
                position: [center_x + half_width, center_y + half_height], // top-right
                uv: [uv[2], uv[1]],
            },
            HUDVertex {
                position: [center_x + half_width, center_y - half_height], // bottom-right
                uv: [uv[2], uv[3]],
            },
            HUDVertex {
                position: [center_x - half_width, center_y - half_height], // bottom-left
                uv: [uv[0], uv[3]],
            },
        ];

        // Индексы для рендера двух треугольников (квад)
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }

    pub fn to_material(&self) -> crate::render::atlas::MaterialType {
        match self {
            IconType::ROCK => crate::render::atlas::MaterialType::ROCK,
            IconType::GRASS => crate::render::atlas::MaterialType::GRASS,
            IconType::DIRT => crate::render::atlas::MaterialType::DIRT,
            IconType::WATER => crate::render::atlas::MaterialType::WATER,
            IconType::DEBUG => crate::render::atlas::MaterialType::DEBUG,
        }
    }

    pub fn from_material(mat: crate::render::atlas::MaterialType) -> Option<Self> {
        match mat {
            crate::render::atlas::MaterialType::ROCK => Some(IconType::ROCK),
            crate::render::atlas::MaterialType::GRASS => Some(IconType::GRASS),
            crate::render::atlas::MaterialType::DIRT => Some(IconType::DIRT),
            crate::render::atlas::MaterialType::WATER => Some(IconType::WATER),
            crate::render::atlas::MaterialType::DEBUG => Some(IconType::DEBUG),
            crate::render::atlas::MaterialType::AIR => None,
        }
    }
}

```

## src\hud\mod.rs
`rust
use crate::{
    core::config::AppConfig,
    text::{TextStyle, TextSystem},
    ui::{
        Align, BitmapFont, ButtonSpec, Layout as UiLayout, MeasureCtx, RectSpec, UiElement, UiNode,
        Val,
    },
};
use glam::Vec2;
use icons_atlas::IconType;
use std::cell::RefCell;
use std::sync::Arc;

use crate::render::{
    mesh::Mesh,
    model::Model,
    pipelines::{
        GlobalsLayouts,
        hud::{HUDVertex, create_hud_pipeline},
    },
    renderer::{Draw, Renderer},
    texture::Texture,
};

pub mod icons_atlas;

#[derive(Debug, Clone, Copy)]
pub struct OverlayStats {
    pub fps: f32,
    pub frame_ms: f32,
    pub chunks_loaded: usize,
    pub draw_calls: usize,
}

pub struct HUD {
    pub(crate) pipeline: wgpu::RenderPipeline,
    crosshair: HUDElement,
    widget: HUDElement,
    toolbar: HUDElement,
    toolbar_frame: HUDElement,
    menu: MenuOverlay,
    palette: Vec<IconType>,
    selected_index: usize,
    debug_overlay: Option<DebugOverlay>,
    aspect_correction: f32,
    icon_atlas_size: (f32, f32),
    text: RefCell<TextSystem>,
    font_handle: crate::text::FontHandle,
    screen_size: [f32; 2],
    text_scale: f32,
    last_stats: OverlayStats,
}

struct HUDElement {
    texture: Texture,
    bind_group: wgpu::BindGroup,
    model: Model<HUDVertex>,
}

#[derive(Clone)]
struct MenuEntry {
    title: String,
    detail: String,
    action: MenuAction,
}

struct ButtonRect {
    action: MenuAction,
    rect: [f32; 4],
}

struct MenuOverlay {
    element: HUDElement,
    buffer: Vec<u8>,
    size: (u32, u32),
    entries: Vec<MenuEntry>,
    buttons: Vec<ButtonRect>,
    visible: bool,
    title: String,
    aspect_correction: f32,
    hovered: Option<MenuAction>,
    font: Arc<BitmapFont>,
    resolved: Vec<crate::ui::ResolvedNode>,
    measure: MeasureCtx,
    text_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Resume,
    CreateWorld,
    OpenWorld,
    SaveConfig,
    OpenSettings,
    OpenAdvanced,
    BackToMain,
    CycleFpsCap,
    ToggleVsync,
    CycleRenderDistance,
    CycleFov,
    ToggleWireframe,
    CycleJobsInFlight,
    CycleDirtyPerFrame,
    CycleMinVertexCap,
    CycleMinIndexCap,
    CycleLandLevel,
    Quit,
}

impl MenuAction {
    fn key(&self) -> &'static str {
        match self {
            MenuAction::Resume => "resume",
            MenuAction::CreateWorld => "create_world",
            MenuAction::OpenWorld => "open_world",
            MenuAction::SaveConfig => "save_config",
            MenuAction::OpenSettings => "open_settings",
            MenuAction::OpenAdvanced => "open_advanced",
            MenuAction::BackToMain => "back_to_main",
            MenuAction::CycleFpsCap => "cycle_fps_cap",
            MenuAction::ToggleVsync => "toggle_vsync",
            MenuAction::CycleRenderDistance => "cycle_render_distance",
            MenuAction::CycleFov => "cycle_fov",
            MenuAction::ToggleWireframe => "toggle_wireframe",
            MenuAction::CycleJobsInFlight => "cycle_jobs_in_flight",
            MenuAction::CycleDirtyPerFrame => "cycle_dirty_per_frame",
            MenuAction::CycleMinVertexCap => "cycle_min_vertex_cap",
            MenuAction::CycleMinIndexCap => "cycle_min_index_cap",
            MenuAction::CycleLandLevel => "cycle_land_level",
            MenuAction::Quit => "quit",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "resume" => Some(MenuAction::Resume),
            "create_world" => Some(MenuAction::CreateWorld),
            "open_world" => Some(MenuAction::OpenWorld),
            "save_config" => Some(MenuAction::SaveConfig),
            "open_settings" => Some(MenuAction::OpenSettings),
            "open_advanced" => Some(MenuAction::OpenAdvanced),
            "back_to_main" => Some(MenuAction::BackToMain),
            "cycle_fps_cap" => Some(MenuAction::CycleFpsCap),
            "toggle_vsync" => Some(MenuAction::ToggleVsync),
            "cycle_render_distance" => Some(MenuAction::CycleRenderDistance),
            "cycle_fov" => Some(MenuAction::CycleFov),
            "toggle_wireframe" => Some(MenuAction::ToggleWireframe),
            "cycle_jobs_in_flight" => Some(MenuAction::CycleJobsInFlight),
            "cycle_dirty_per_frame" => Some(MenuAction::CycleDirtyPerFrame),
            "cycle_min_vertex_cap" => Some(MenuAction::CycleMinVertexCap),
            "cycle_min_index_cap" => Some(MenuAction::CycleMinIndexCap),
            "cycle_land_level" => Some(MenuAction::CycleLandLevel),
            "quit" => Some(MenuAction::Quit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuPage {
    Main,
    Settings,
    Advanced,
}

impl HUD {
    pub fn new(
        renderer: &Renderer,
        global_layout: &GlobalsLayouts,
        shader: wgpu::ShaderModule,
        show_debug_overlay: bool,
        ui_cfg: &crate::core::config::UiConfig,
    ) -> Self {
        let aspect_correction = renderer.size.height as f32 / renderer.size.width as f32;
        let mut text_system = TextSystem::new(
            &renderer.device,
            &renderer.queue,
            renderer.config.format,
            &global_layout.globals,
        )
        .expect("Failed to init text system");
        let font_handle = text_system
            .load_font(&ui_cfg.font_path)
            .expect("Failed to load text font");
        let font = Arc::new(
            BitmapFont::load_from_path(&ui_cfg.font_path, ui_cfg.font_size, ui_cfg.font_weight_px)
                .expect("Failed to load UI font from config"),
        );
        let palette = IconType::all();
        let selected_index = 0;
        // Загрузка текстур интерфейса
        let crosshair_bytes = include_bytes!("../../assets/images/crosshair.png");
        let widget_bytes = include_bytes!("../../assets/images/widget_window.png");

        let icons_bytes = include_bytes!("../../assets/images/icons_atlas.png");

        let crosshair_tex = Texture::from_bytes(
            &renderer.device,
            &renderer.queue,
            crosshair_bytes,
            "crosshair.png",
        )
        .unwrap();
        let widget_tex = Texture::from_bytes(
            &renderer.device,
            &renderer.queue,
            widget_bytes,
            "crosshair.png",
        )
        .unwrap();
        let icons_atlas_tex = Texture::from_bytes(
            &renderer.device,
            &renderer.queue,
            icons_bytes,
            "icons_atlas.png",
        )
        .unwrap();
        let icon_atlas_size = icons_atlas_tex.size();
        let icon_atlas_size = (icon_atlas_size.0 as f32, icon_atlas_size.1 as f32);

        // Создаём конвейер HUD с глобальным layout
        let pipeline = create_hud_pipeline(
            &renderer.device,
            &global_layout, // Используем глобальный layout
            shader,
            &renderer.config,
        );

        // Собираем bind groups
        let crosshair_bind_group = global_layout.bind_hud_texture(
            &renderer.device,
            &crosshair_tex,
            None, // Самплер по умолчанию
        );

        let widget_bind_group = global_layout.bind_hud_texture(&renderer.device, &widget_tex, None);

        let icons_bind_group =
            global_layout.bind_hud_texture(&renderer.device, &icons_atlas_tex, None);

        // Геометрия элементов HUD
        let (crosshair_verts, crosshair_indices) =
            create_hud_quad(0.0, 0.0, 0.06, 0.06, aspect_correction); // Размер прицела
        let (widget_verts, widget_indices) = build_widget_quad(aspect_correction); // Окно хотбара

        // Создаём модели
        let crosshair_model = Model::new(
            &renderer.device,
            &Mesh {
                verts: crosshair_verts,
                indices: crosshair_indices,
            },
        )
        .unwrap();

        let widget_model = Model::new(
            &renderer.device,
            &Mesh {
                verts: widget_verts,
                indices: widget_indices,
            },
        )
        .unwrap();

        let toolbar_model = build_toolbar_model(
            &renderer.device,
            &palette,
            selected_index,
            aspect_correction,
            icon_atlas_size,
        );

        let (frame_verts, frame_indices) =
            build_toolbar_frame(selected_index, palette.len(), aspect_correction);
        let frame_model = Model::new(
            &renderer.device,
            &Mesh {
                verts: frame_verts,
                indices: frame_indices,
            },
        )
        .expect("Failed to create toolbar frame");

        // Создаём bind group для каждого элемента
        let crosshair = HUDElement {
            texture: crosshair_tex,
            bind_group: crosshair_bind_group,
            model: crosshair_model,
        };

        let widget = HUDElement {
            texture: widget_tex.clone(),
            bind_group: widget_bind_group.clone(),
            model: widget_model,
        };

        let toolbar = HUDElement {
            texture: icons_atlas_tex,
            bind_group: icons_bind_group,
            model: toolbar_model,
        };

        let toolbar_frame = HUDElement {
            texture: widget_tex,
            bind_group: widget_bind_group,
            model: frame_model,
        };

        let debug_overlay = if show_debug_overlay {
            Some(DebugOverlay::new(
                renderer,
                global_layout,
                aspect_correction,
            ))
        } else {
            None
        };

        Self {
            pipeline,
            crosshair,
            widget,
            toolbar,
            toolbar_frame,
            menu: MenuOverlay::new(
                &renderer.device,
                &renderer.queue,
                global_layout,
                font,
                ui_cfg.text_scale,
                aspect_correction,
            ),
            palette,
            selected_index,
            debug_overlay,
            aspect_correction,
            icon_atlas_size,
            text: RefCell::new(text_system),
            font_handle,
            screen_size: [renderer.size.width as f32, renderer.size.height as f32],
            text_scale: ui_cfg.text_scale,
            last_stats: OverlayStats {
                fps: 0.0,
                frame_ms: 0.0,
                chunks_loaded: 0,
                draw_calls: 0,
            },
        }
    }

    pub fn update(&mut self, renderer: &Renderer) {
        self.toolbar.model = build_toolbar_model(
            &renderer.device,
            &self.palette,
            self.selected_index,
            self.aspect_correction,
            self.icon_atlas_size,
        );
        let (frame_verts, frame_indices) = build_toolbar_frame(
            self.selected_index,
            self.palette.len(),
            self.aspect_correction,
        );
        self.toolbar_frame.model = Model::new(
            &renderer.device,
            &Mesh {
                verts: frame_verts,
                indices: frame_indices,
            },
        )
        .expect("Failed to rebuild toolbar frame");
    }

    pub fn select_next(&mut self, renderer: &Renderer) {
        self.selected_index = (self.selected_index + 1) % self.palette.len();
        self.update(renderer);
    }

    pub fn select_prev(&mut self, renderer: &Renderer) {
        if self.selected_index == 0 {
            self.selected_index = self.palette.len() - 1;
        } else {
            self.selected_index -= 1;
        }
        self.update(renderer);
    }

    pub fn select_by_icon(&mut self, icon: IconType, renderer: &Renderer) {
        if let Some(idx) = self.palette.iter().position(|p| *p == icon) {
            self.selected_index = idx;
            self.update(renderer);
        }
    }

    pub fn selected_icon(&self) -> IconType {
        self.palette[self.selected_index]
    }

    pub fn update_overlay(&mut self, renderer: &Renderer, stats: &OverlayStats) {
        if let Some(overlay) = &mut self.debug_overlay {
            overlay.update(renderer, stats);
        }
        self.last_stats = *stats;
    }

    pub fn draw_call_count(&self) -> usize {
        let base = 3;
        if self.debug_overlay.is_some() {
            base + 1
        } else {
            base
        }
    }

    pub fn resize(&mut self, renderer: &Renderer) {
        self.aspect_correction = renderer.size.height as f32 / renderer.size.width as f32;
        self.screen_size = [renderer.size.width as f32, renderer.size.height as f32];

        let (crosshair_verts, crosshair_indices) =
            create_hud_quad(0.0, 0.0, 0.06, 0.06, self.aspect_correction);
        self.crosshair.model = Model::new(
            &renderer.device,
            &Mesh {
                verts: crosshair_verts,
                indices: crosshair_indices,
            },
        )
        .expect("Failed to update crosshair model");

        let (widget_verts, widget_indices) = build_widget_quad(self.aspect_correction);
        self.widget.model = Model::new(
            &renderer.device,
            &Mesh {
                verts: widget_verts,
                indices: widget_indices,
            },
        )
        .expect("Failed to update widget model");

        self.update(renderer);

        if let Some(overlay) = &mut self.debug_overlay {
            overlay.update_layout(renderer, self.aspect_correction);
        }

        self.menu.update_layout(
            &renderer.device,
            &renderer.queue,
            &renderer.layouts.global,
            self.aspect_correction,
        );
    }

    pub fn open_menu(&mut self, page: MenuPage, config: &AppConfig, queue: &wgpu::Queue) {
        let entries = match page {
            MenuPage::Main => build_main_menu(),
            MenuPage::Settings => build_settings_menu(config),
            MenuPage::Advanced => build_advanced_menu(config),
        };
        let title = match page {
            MenuPage::Main => "Меню",
            MenuPage::Settings => "Настройки",
            MenuPage::Advanced => "Продвинутые настройки",
        };
        self.menu.set_entries(title, entries, queue);
        self.menu.visible = true;
    }

    pub fn close_menu(&mut self) {
        self.menu.visible = false;
    }

    pub fn hover_menu(&mut self, clip_x: f32, clip_y: f32, queue: &wgpu::Queue) {
        if self.menu.visible {
            self.menu.hover_at(clip_x, clip_y, queue);
        }
    }

    pub fn click_menu(&self) -> Option<MenuAction> {
        if self.menu.visible {
            self.menu.click()
        } else {
            None
        }
    }
}

impl Draw for HUD {
    fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        globals: &'a wgpu::BindGroup,
    ) -> Result<(), wgpu::Error> {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(1, globals, &[]);

        // Рендерим элементы HUD
        let mut elements: Vec<&HUDElement> =
            vec![&self.crosshair, &self.toolbar, &self.toolbar_frame];
        if self.menu.visible {
            elements.push(&self.menu.element);
        }
        if let Some(overlay) = &self.debug_overlay {
            if overlay.visible {
                // прячем оверлей под меню, чтобы не перекрывать кнопки
                if !self.menu.visible {
                    elements.push(&overlay.element);
                }
            }
        }

        for element in elements {
            render_pass.set_bind_group(0, &element.bind_group, &[]);
            render_pass.set_vertex_buffer(0, element.model.vbuf().slice(..));
            render_pass.set_index_buffer(element.model.ibuf().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..element.model.num_indices, 0, 0..1);
        }

        // Text rendering (GUI + overlays)
        {
            let mut text = self.text.borrow_mut();
            let screen = self.screen_size;
            let base_style = TextStyle {
                color: [0.94, 0.94, 0.94, 1.0],
                pixel_size: (self.text_scale * 18.0).round() as u32,
            };
            if let Some(_overlay) = &self.debug_overlay {
                let stats = self.last_stats;
                let lines = [
                    format!("FPS   {:>5.1}", stats.fps),
                    format!("MS    {:>5.2}", stats.frame_ms),
                    format!("CHUNKS {:>4}", stats.chunks_loaded),
                    format!("DRAWS  {:>4}", stats.draw_calls),
                ];
                let mut y = 16.0;
                for line in lines.iter() {
                    if let Ok(obj) = text.build_gui_text(
                        line,
                        self.font_handle,
                        base_style,
                        Vec2::new(12.0, y),
                        screen,
                    ) {
                        text.draw(render_pass, None, &obj, screen);
                    }
                    y += base_style.pixel_size as f32 + 4.0;
                }
            }

            if self.menu.visible {
                let menu_size = (screen[0] * 0.6, screen[1] * 0.7);
                let origin = Vec2::new(
                    (screen[0] - menu_size.0) * 0.5,
                    (screen[1] - menu_size.1) * 0.5,
                );
                let sx = menu_size.0 / self.menu.size.0 as f32;
                let sy = menu_size.1 / self.menu.size.1 as f32;
                for node in &self.menu.resolved {
                    if let Some(el) = &node.element {
                        match el {
                            UiElement::Button(btn) => {
                                let title_style = TextStyle {
                                    color: [1.0, 1.0, 1.0, 1.0],
                                    pixel_size: base_style.pixel_size,
                                };
                                let detail_style = TextStyle {
                                    color: [0.75, 0.85, 1.0, 1.0],
                                    pixel_size: (base_style.pixel_size as f32 * 0.8) as u32,
                                };
                                let x = origin.x + node.rect[0] * sx + btn.padding * sx;
                                let y = origin.y + node.rect[1] * sy + btn.padding * sy;
                                if let Ok(obj) = text.build_gui_text(
                                    &btn.text,
                                    self.font_handle,
                                    title_style,
                                    Vec2::new(x, y),
                                    screen,
                                ) {
                                    text.draw(render_pass, None, &obj, screen);
                                }
                                if let Some(detail) = &btn.detail {
                                    if let Ok(obj) = text.build_gui_text(
                                        detail,
                                        self.font_handle,
                                        detail_style,
                                        Vec2::new(x, y + title_style.pixel_size as f32 + 4.0 * sy),
                                        screen,
                                    ) {
                                        text.draw(render_pass, None, &obj, screen);
                                    }
                                }
                            }
                            UiElement::Label(label) => {
                                let style = TextStyle {
                                    color: [1.0, 0.86, 0.47, 1.0],
                                    pixel_size: (label.font_size * self.text_scale) as u32,
                                };
                                let x = origin.x + node.rect[0] * sx;
                                let y = origin.y + node.rect[1] * sy;
                                if let Ok(obj) = text.build_gui_text(
                                    &label.text,
                                    self.font_handle,
                                    style,
                                    Vec2::new(x, y),
                                    screen,
                                ) {
                                    text.draw(render_pass, None, &obj, screen);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl MenuOverlay {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        global_layout: &GlobalsLayouts,
        font: Arc<BitmapFont>,
        text_scale: f32,
        aspect_correction: f32,
    ) -> Self {
        let size = (512u32, 512u32);
        let buffer = vec![0u8; (size.0 * size.1 * 4) as usize];
        let texture =
            Texture::from_rgba(device, queue, &buffer, size.0, size.1, "menu_overlay").unwrap();
        let bind_group = global_layout.bind_hud_texture(device, &texture, None);
        let (verts, indices) = create_hud_quad(0.0, 0.0, 1.4, 1.4, aspect_correction);
        let model = Model::new(device, &Mesh { verts, indices }).unwrap();

        Self {
            element: HUDElement {
                texture,
                bind_group,
                model,
            },
            buffer,
            size,
            entries: Vec::new(),
            buttons: Vec::new(),
            visible: false,
            title: String::new(),
            aspect_correction,
            hovered: None,
            font: font.clone(),
            resolved: Vec::new(),
            measure: MeasureCtx {
                font: Some(font),
                text_scale,
            },
            text_scale,
        }
    }

    fn update_layout(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        global_layout: &GlobalsLayouts,
        aspect_correction: f32,
    ) {
        self.aspect_correction = aspect_correction;
        let (verts, indices) = create_hud_quad(0.0, 0.0, 1.4, 1.4, aspect_correction);
        self.element.model = Model::new(device, &Mesh { verts, indices }).unwrap();
        // Rebind to keep texture alive after device change (resize doesn't change device, but safe).
        self.element.bind_group =
            global_layout.bind_hud_texture(device, &self.element.texture, None);
        // redraw with current entries to avoid stale hover rectangles
        if !self.entries.is_empty() {
            self.rebuild_layout();
            self.draw(queue);
        }
    }

    fn set_entries(
        &mut self,
        title: impl Into<String>,
        entries: Vec<MenuEntry>,
        queue: &wgpu::Queue,
    ) {
        self.title = title.into();
        self.entries = entries;
        self.hovered = None;
        self.rebuild_layout();
        self.draw(queue);
    }

    fn rebuild_layout(&mut self) {
        let layout = self.build_tree();
        self.resolved =
            layout.resolve_tree([self.size.0 as f32, self.size.1 as f32], &self.measure);
        self.buttons = self
            .resolved
            .iter()
            .filter_map(|node| {
                let action = node.id.as_deref().and_then(MenuAction::from_key)?;
                Some(ButtonRect {
                    action,
                    rect: node.rect,
                })
            })
            .collect();
    }

    fn build_tree(&self) -> UiNode {
        let mut children = Vec::new();

        if !self.title.is_empty() {
            children.push(UiNode {
                id: Some("menu_title".into()),
                layout: UiLayout::Absolute {
                    rect: RectSpec {
                        x: Val::Percent(0.0),
                        y: Val::Px(20.0),
                        w: Val::Percent(1.0),
                        h: Val::Px(32.0),
                    },
                    anchor: None,
                },
                children: Vec::new(),
                element: Some(UiElement::Label(crate::ui::LabelSpec {
                    text: self.title.clone(),
                    font_size: 16.0,
                })),
            });
        }

        let button_nodes = self
            .entries
            .iter()
            .map(|entry| UiNode {
                id: Some(entry.action.key().to_string()),
                layout: UiLayout::Absolute {
                    rect: RectSpec {
                        x: Val::Percent(0.0),
                        y: Val::Px(0.0),
                        w: Val::Percent(1.0),
                        h: Val::Px(60.0),
                    },
                    anchor: None,
                },
                children: Vec::new(),
                element: Some(UiElement::Button(ButtonSpec {
                    text: entry.title.clone(),
                    detail: if entry.detail.is_empty() {
                        None
                    } else {
                        Some(entry.detail.clone())
                    },
                    padding: 14.0,
                    min_height: 60.0,
                })),
            })
            .collect::<Vec<_>>();

        children.push(UiNode {
            id: Some("menu_buttons".into()),
            layout: UiLayout::FlexColumn {
                gap: 10.0,
                padding: 60.0,
                align: Align::Stretch,
            },
            children: button_nodes,
            element: Some(UiElement::Panel {
                color: [18, 22, 30, 220],
            }),
        });

        UiNode {
            id: Some("menu_root".into()),
            layout: UiLayout::Absolute {
                rect: RectSpec {
                    x: Val::Px(0.0),
                    y: Val::Px(0.0),
                    w: Val::Percent(1.0),
                    h: Val::Percent(1.0),
                },
                anchor: None,
            },
            children,
            element: None,
        }
    }

    fn hover_at(&mut self, clip_x: f32, clip_y: f32, queue: &wgpu::Queue) {
        // Map clip coords that cover only the menu quad (width/height = 1.4) back into texture pixels.
        // Quad bounds in clip space (center at 0, height = 1.4, width scaled by aspect).
        let half_w = (1.4 / 2.0) * self.aspect_correction;
        let half_h = 1.4 / 2.0;
        let x_min = -half_w;
        let x_max = half_w;
        let y_min = -half_h;
        let y_max = half_h;

        if clip_x < x_min || clip_x > x_max || clip_y < y_min || clip_y > y_max {
            if self.hovered.is_some() {
                self.hovered = None;
                self.draw(queue);
            }
            return;
        }

        let norm_x = (clip_x - x_min) / (x_max - x_min);
        let norm_y = (clip_y - y_min) / (y_max - y_min); // clip y up; texture y down handled below

        let px_x = (norm_x * self.size.0 as f32).clamp(0.0, self.size.0 as f32 - 1.0);
        let px_y = ((1.0 - norm_y) * self.size.1 as f32).clamp(0.0, self.size.1 as f32 - 1.0);

        let mut new_hover = None;
        for btn in &self.buttons {
            let [x, y, w, h] = btn.rect;
            if px_x >= x && px_x <= x + w && px_y >= y && px_y <= y + h {
                new_hover = Some(btn.action);
                break;
            }
        }
        if new_hover != self.hovered {
            self.hovered = new_hover;
            self.draw(queue);
        }
    }

    fn click(&self) -> Option<MenuAction> {
        self.hovered
    }

    fn draw(&mut self, queue: &wgpu::Queue) {
        self.clear([8, 10, 14, 200]);

        let resolved = self.resolved.clone();
        for node in resolved.iter() {
            if let Some(el) = &node.element {
                match el {
                    UiElement::Panel { color } => self.draw_panel_rect(node.rect, *color),
                    UiElement::Button(btn) => self.draw_button(node.rect, btn, node.id.as_deref()),
                    UiElement::Label(label) => self.draw_label(node.rect, label),
                    UiElement::Image { .. } | UiElement::Spacer { .. } => {}
                }
            }
        }
        self.element
            .texture
            .write_rgba(queue, &self.buffer, self.size.0, self.size.1);
    }

    fn draw_panel_rect(&mut self, rect: [f32; 4], color: [u8; 4]) {
        let x = rect[0].round() as i32;
        let y = rect[1].round() as i32;
        let w = rect[2].round() as i32;
        let h = rect[3].round() as i32;
        self.fill_rect(x, y, w, h, color);
        self.stroke_rect(x, y, w, h, 3, [80, 120, 200, 255]);
    }

    fn draw_button(&mut self, rect: [f32; 4], spec: &ButtonSpec, id: Option<&str>) {
        let hovered = id
            .and_then(MenuAction::from_key)
            .map(|a| Some(a) == self.hovered)
            .unwrap_or(false);
        let base = if hovered {
            [90, 140, 255, 220]
        } else {
            [40, 60, 90, 200]
        };
        let x = rect[0].round() as i32;
        let y = rect[1].round() as i32;
        let w = rect[2].round() as i32;
        let h = rect[3].round() as i32;
        self.fill_rect(x, y, w, h, base);

        let _line_h = self.font.height() as i32;
        let _ = &spec.detail;
        // text rendered by TextSystem; keep background only
    }

    fn draw_label(&mut self, rect: [f32; 4], label: &crate::ui::LabelSpec) {
        // text rendered by TextSystem; background only
    }

    fn clear(&mut self, color: [u8; 4]) {
        for chunk in self.buffer.chunks_exact_mut(4) {
            chunk.copy_from_slice(&color);
        }
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
        for yy in y.max(0)..(y + h).min(self.size.1 as i32) {
            for xx in x.max(0)..(x + w).min(self.size.0 as i32) {
                let idx = ((yy as u32 * self.size.0 + xx as u32) * 4) as usize;
                self.buffer[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }

    fn stroke_rect(&mut self, x: i32, y: i32, w: i32, h: i32, thickness: i32, color: [u8; 4]) {
        // top/bottom
        self.fill_rect(x, y, w, thickness, color);
        self.fill_rect(x, y + h - thickness, w, thickness, color);
        // sides
        self.fill_rect(x, y, thickness, h, color);
        self.fill_rect(x + w - thickness, y, thickness, h, color);
    }

    fn pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x as u32 >= self.size.0 || y as u32 >= self.size.1 {
            return;
        }
        let idx = ((y as u32 * self.size.0 + x as u32) * 4) as usize;
        self.buffer[idx..idx + 4].copy_from_slice(&color);
    }
}

pub fn create_hud_quad(
    center_x: f32,
    center_y: f32,
    width: f32,
    height: f32,
    aspect_correction: f32,
) -> (Vec<HUDVertex>, Vec<u32>) {
    let half_w = (width / 2.0) * aspect_correction;
    let half_h = height / 2.0;

    let vertices = vec![
        // Верхний левый угол
        HUDVertex {
            position: [center_x - half_w, center_y + half_h],
            uv: [0.0, 0.0],
        },
        // Верхний правый угол
        HUDVertex {
            position: [center_x + half_w, center_y + half_h],
            uv: [1.0, 0.0],
        },
        // Нижний правый угол
        HUDVertex {
            position: [center_x + half_w, center_y - half_h],
            uv: [1.0, 1.0],
        },
        // Нижний левый угол
        HUDVertex {
            position: [center_x - half_w, center_y - half_h],
            uv: [0.0, 1.0],
        },
    ];

    let indices = vec![0u32, 1, 2, 0, 2, 3];

    (vertices, indices)
}

fn build_widget_quad(aspect_correction: f32) -> (Vec<HUDVertex>, Vec<u32>) {
    create_hud_quad(0.0, -0.85, 0.7, 0.18, aspect_correction)
}

fn build_toolbar_model(
    device: &wgpu::Device,
    palette: &[IconType],
    selected_index: usize,
    aspect_correction: f32,
    icon_atlas_size: (f32, f32),
) -> Model<HUDVertex> {
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    let base_x = -0.45;
    let step = 0.18;
    for (i, icon) in palette.iter().enumerate() {
        let is_selected = i == selected_index;
        let width = if is_selected { 0.14 } else { 0.12 };
        let height = if is_selected { 0.14 } else { 0.12 };
        let center_x = base_x + i as f32 * step;
        let center_y = -0.85;
        let (quad_verts, quad_indices) = icon.get_vertex_quad(
            center_x,
            center_y,
            width,
            height,
            aspect_correction,
            icon_atlas_size,
        );
        let base_index = verts.len() as u32;
        verts.extend_from_slice(&quad_verts);
        indices.extend(quad_indices.iter().map(|idx| idx + base_index));
    }

    Model::new(device, &Mesh { verts, indices }).expect("Failed to build toolbar model")
}

struct DebugOverlay {
    element: HUDElement,
    buffer: Vec<u8>,
    size: (u32, u32),
    visible: bool,
}

impl DebugOverlay {
    fn new(renderer: &Renderer, global_layout: &GlobalsLayouts, aspect_correction: f32) -> Self {
        let size = (256u32, 96u32);
        let buffer = vec![0u8; (size.0 * size.1 * 4) as usize];
        let texture = Texture::from_rgba(
            &renderer.device,
            &renderer.queue,
            &buffer,
            size.0,
            size.1,
            "debug_overlay",
        )
        .expect("Failed to create debug overlay texture");

        let bind_group = global_layout.bind_hud_texture(&renderer.device, &texture, None);
        let (verts, indices) = create_hud_quad(-0.75, 0.88, 0.6, 0.22, aspect_correction);
        let model = Model::new(&renderer.device, &Mesh { verts, indices })
            .expect("Failed to create debug overlay model");

        Self {
            element: HUDElement {
                texture,
                bind_group,
                model,
            },
            buffer,
            size,
            visible: true,
        }
    }

    fn update(&mut self, renderer: &Renderer, stats: &OverlayStats) {
        if !self.visible {
            return;
        }

        self.clear();
        self.element
            .texture
            .write_rgba(&renderer.queue, &self.buffer, self.size.0, self.size.1);
    }

    fn update_layout(&mut self, renderer: &Renderer, aspect_correction: f32) {
        let (verts, indices) = create_hud_quad(-0.75, 0.85, 0.6, 0.22, aspect_correction);
        self.element.model = Model::new(&renderer.device, &Mesh { verts, indices })
            .expect("Failed to update overlay quad");
    }

    fn clear(&mut self) {
        for chunk in self.buffer.chunks_exact_mut(4) {
            // Лёгкий тёмный фон для читабельности
            chunk.copy_from_slice(&[12, 12, 12, 120]);
        }
    }

    fn pixel(&mut self, x: usize, y: usize, color: [u8; 4]) {
        if x as u32 >= self.size.0 || y as u32 >= self.size.1 {
            return;
        }
        let idx = (y * self.size.0 as usize + x) * 4;
        self.buffer[idx..idx + 4].copy_from_slice(&color);
    }
}

fn build_main_menu() -> Vec<MenuEntry> {
    vec![
        MenuEntry {
            title: "Продолжить".to_string(),
            detail: "Вернуться в игру".to_string(),
            action: MenuAction::Resume,
        },
        MenuEntry {
            title: "Создать мир".to_string(),
            detail: "Новый seed и папка сохранения".to_string(),
            action: MenuAction::CreateWorld,
        },
        MenuEntry {
            title: "Открыть мир".to_string(),
            detail: "Загрузить существующее сохранение".to_string(),
            action: MenuAction::OpenWorld,
        },
        MenuEntry {
            title: "Сохранить настройки".to_string(),
            detail: "Записать config.json".to_string(),
            action: MenuAction::SaveConfig,
        },
        MenuEntry {
            title: "Настройки".to_string(),
            detail: "Базовые параметры".to_string(),
            action: MenuAction::OpenSettings,
        },
        MenuEntry {
            title: "Продвинутые настройки".to_string(),
            detail: "Оптимизация и дебаг".to_string(),
            action: MenuAction::OpenAdvanced,
        },
        MenuEntry {
            title: "Выход".to_string(),
            detail: "Закрыть игру".to_string(),
            action: MenuAction::Quit,
        },
    ]
}

fn build_settings_menu(cfg: &AppConfig) -> Vec<MenuEntry> {
    vec![
        MenuEntry {
            title: format!("FPS: {}", cfg.graphics.fps_cap),
            detail: "Цикл: 30 / 60 / 120 / без лимита".to_string(),
            action: MenuAction::CycleFpsCap,
        },
        MenuEntry {
            title: format!(
                "VSync: {}",
                if cfg.graphics.vsync {
                    "вкл"
                } else {
                    "выкл"
                }
            ),
            detail: "Изменить режим представления".to_string(),
            action: MenuAction::ToggleVsync,
        },
        MenuEntry {
            title: format!("Дальность: {} чанков", cfg.graphics.render_distance_chunks),
            detail: "Цикл 8 / 16 / 24 / 32 / 48".to_string(),
            action: MenuAction::CycleRenderDistance,
        },
        MenuEntry {
            title: format!("FOV: {:.0}", cfg.graphics.fov_y_degrees),
            detail: "Поле зрения камеры".to_string(),
            action: MenuAction::CycleFov,
        },
        MenuEntry {
            title: "Назад".to_string(),
            detail: "".to_string(),
            action: MenuAction::BackToMain,
        },
    ]
}

fn build_advanced_menu(cfg: &AppConfig) -> Vec<MenuEntry> {
    vec![
        MenuEntry {
            title: format!(
                "Wireframe: {}",
                if cfg.debug.wireframe {
                    "вкл"
                } else {
                    "выкл"
                }
            ),
            detail: "Отладочный режим линий".to_string(),
            action: MenuAction::ToggleWireframe,
        },
        MenuEntry {
            title: format!("Jobs in flight: {}", cfg.terrain.jobs_in_flight),
            detail: "Параллельные задачи генерации".to_string(),
            action: MenuAction::CycleJobsInFlight,
        },
        MenuEntry {
            title: format!("Dirty per frame: {}", cfg.terrain.dirty_chunks_per_frame),
            detail: "Ремеш чанков за кадр".to_string(),
            action: MenuAction::CycleDirtyPerFrame,
        },
        MenuEntry {
            title: format!("Vertex cap: {}", cfg.terrain.min_vertex_cap),
            detail: "Минимальный буфер вершин".to_string(),
            action: MenuAction::CycleMinVertexCap,
        },
        MenuEntry {
            title: format!("Index cap: {}", cfg.terrain.min_index_cap),
            detail: "Минимальный буфер индексов".to_string(),
            action: MenuAction::CycleMinIndexCap,
        },
        MenuEntry {
            title: format!("Уровень воды: {}", cfg.terrain.land_level),
            detail: "Высота водной поверхности".to_string(),
            action: MenuAction::CycleLandLevel,
        },
        MenuEntry {
            title: "Назад".to_string(),
            detail: "".to_string(),
            action: MenuAction::BackToMain,
        },
    ]
}

fn build_toolbar_frame(
    selected_index: usize,
    _palette_len: usize,
    aspect_correction: f32,
) -> (Vec<HUDVertex>, Vec<u32>) {
    let mut verts = Vec::new();
    let mut indices = Vec::new();

    let base_x = -0.45;
    let step = 0.18;
    let center_y = -0.85;
    let width = 0.18;
    let height = 0.18;

    let cx = base_x + selected_index as f32 * step;
    let half_w = (width / 2.0) * aspect_correction;
    let half_h = height / 2.0;

    let rect = [
        [cx - half_w, center_y + half_h], // top-left (y up)
        [cx + half_w, center_y + half_h], // top-right
        [cx + half_w, center_y - half_h], // bottom-right
        [cx - half_w, center_y - half_h], // bottom-left
    ];
    let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    for i in 0..4 {
        verts.push(HUDVertex {
            position: rect[i],
            uv: uvs[i],
        });
    }
    indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

    (verts, indices)
}

```

## src\player\camera.rs
`rust
use cgmath::*;
use instant::Duration;
use std::f32::consts::FRAC_PI_2;
#[cfg(feature = "tracy")]
use tracy_client::span;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::render::renderer::Renderer;

use crate::terrain_gen::chunk::CHUNK_AREA;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct Dependants {
    pub view_proj: [[f32; 4]; 4],
}

pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub direction: Vector3<f32>,

    pub projection: Projection,
    pub camera_controller: CameraController,

    pub dependants: Dependants,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        renderer: &Renderer,
        position: V,
        yaw: Y,
        pitch: P,
        render_distance_chunks: usize,
        move_speed: f32,
        sensitivity: f32,
        invert_y: bool,
        fov_y_degrees: f32,
    ) -> Self {
        let projection = Projection::new(
            renderer.config.width,
            renderer.config.height,
            cgmath::Deg(fov_y_degrees),
            0.1,
            (render_distance_chunks.max(1) * CHUNK_AREA) as f32,
        );
        let camera_controller = CameraController::new(move_speed, sensitivity, invert_y);

        let mut camera = Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            direction: Vector3::new(0.0, 0.0, 0.0),

            projection,
            camera_controller,

            dependants: Dependants {
                view_proj: Matrix4::identity().into(),
            },
        };

        camera.update_view();

        return camera;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn input(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.camera_controller.process_mouse(delta.0, delta.1);
            }
            _ => {}
        }
    }

    pub fn input_keyboard(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            _ => false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.projection.resize(new_size.width, new_size.height)
    }

    pub fn step_input(&mut self, dt: Duration) -> Vector3<f32> {
        #[cfg(feature = "tracy")]
        let _span = span!("update camera deps"); // <- Отметка начала блока

        let movement = self.update_camera_controller(dt);
        self.update_view();
        movement
    }

    pub fn dependants(&self) -> &Dependants {
        &self.dependants
    }

    pub fn update_view(&mut self) {
        let view_proj: [[f32; 4]; 4] = (self.projection.calc_matrix() * self.calc_matrix()).into();
        self.dependants = Dependants { view_proj }
    }

    pub fn update_camera_controller(&mut self, dt: Duration) -> Vector3<f32> {
        let dt = dt.as_secs_f32();

        // Двигаем камеру вперёд/назад и влево/вправо
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        let mut movement = forward
            * (self.camera_controller.amount_forward - self.camera_controller.amount_backward)
            * self.camera_controller.speed;
        movement += right
            * (self.camera_controller.amount_right - self.camera_controller.amount_left)
            * self.camera_controller.speed;

        // Двигаем камеру вверх/вниз (для творческого режима)
        movement.y += (self.camera_controller.amount_up - self.camera_controller.amount_down)
            * self.camera_controller.speed;

        // Поворот
        self.yaw +=
            Rad(self.camera_controller.rotate_horizontal) * self.camera_controller.sensitivity * dt;
        let vertical = if self.camera_controller.invert_y {
            self.camera_controller.rotate_vertical
        } else {
            -self.camera_controller.rotate_vertical
        };
        self.pitch += Rad(vertical) * self.camera_controller.sensitivity * dt;

        // Если process_mouse не вызывается каждый кадр, значения не обнулятся,
        // и камера начнёт вращаться при движении по диагонали.
        self.camera_controller.rotate_horizontal = 0.0;
        self.camera_controller.rotate_vertical = 0.0;
        self.camera_controller.scroll = 0.0;

        // Ограничиваем угол наклона камеры.
        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }

        movement
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn set_fovy_deg(&mut self, fovy_deg: f32) {
        self.fovy = cgmath::Deg(fovy_deg).into();
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    invert_y: bool,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32, invert_y: bool) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            invert_y,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn jump_requested(&self) -> bool {
        self.amount_up > 0.0
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // Предполагаем, что одна строка колеса мыши равна ~100 пикселям
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }
}

```

## src\player\mod.rs
`rust
pub mod camera;
pub mod raycast;

use crate::{
    core::config::{AppConfig, PlayerMode},
    render::atlas::MaterialType,
    terrain_gen::chunk::ChunkManager,
};
use cgmath::{EuclideanSpace, InnerSpace, Vector3, vec3};

pub struct Player {
    pub camera: camera::Camera,
    pub last_pos: cgmath::Point3<f32>,
    pub speed: f32,
    pub mode: PlayerMode,
    view_mode: ViewMode,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    on_ground: bool,
    bounds_height: f32,
    bounds_radius: f32,
    eye_height: f32,
    gravity: f32,
    jump_speed: f32,
    max_fall_speed: f32,
}

impl Player {
    pub fn new(camera: camera::Camera, speed: f32, cfg: &AppConfig) -> Self {
        let eye_height = cfg.player.eye_height;
        let mut player = Self {
            camera,
            last_pos: cgmath::Point3::new(0.0, 0.0, 0.0),
            speed,
            mode: cfg.player.mode.clone(),
            view_mode: ViewMode::FirstPerson,
            position: vec3(0.0, 0.0, 0.0),
            velocity: vec3(0.0, 0.0, 0.0),
            on_ground: false,
            bounds_height: cfg.player.height,
            bounds_radius: cfg.player.radius,
            eye_height,
            gravity: cfg.player.gravity,
            jump_speed: cfg.player.jump_speed,
            max_fall_speed: cfg.player.max_fall_speed,
        };

        player.position = player.camera.position.to_vec() - vec3(0.0, eye_height, 0.0);
        player.sync_camera();
        player
    }

    pub fn set_mode(&mut self, mode: PlayerMode, cfg: &AppConfig) {
        self.mode = mode;
        self.camera.camera_controller.set_speed(match self.mode {
            PlayerMode::Adventure => cfg.input.move_speed,
            PlayerMode::Creative => cfg.input.fly_speed,
        });
    }

    pub fn toggle_mode(&mut self, cfg: &AppConfig) {
        let new_mode = match self.mode {
            PlayerMode::Adventure => PlayerMode::Creative,
            PlayerMode::Creative => PlayerMode::Adventure,
        };
        self.set_mode(new_mode, cfg);
    }

    pub fn max_interact_range(&self) -> f32 {
        match self.mode {
            PlayerMode::Adventure => 4.0,
            PlayerMode::Creative => 100.0,
        }
    }

    pub fn toggle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::FirstPerson => ViewMode::ThirdPerson { distance: 4.0 },
            ViewMode::ThirdPerson { .. } => ViewMode::FirstPerson,
        };
    }

    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    pub fn update(&mut self, dt: std::time::Duration, chunks: &ChunkManager) {
        let desired_velocity = self.camera.step_input(dt);
        match self.mode {
            PlayerMode::Creative => self.update_creative(dt, desired_velocity),
            PlayerMode::Adventure => self.update_adventure(dt, desired_velocity, chunks),
        }
        self.sync_camera();
        self.camera.update_view();
    }

    fn sync_camera(&mut self) {
        match self.view_mode {
            ViewMode::FirstPerson => {
                self.camera.position = cgmath::Point3::new(
                    self.position.x,
                    self.position.y + self.eye_height,
                    self.position.z,
                );
            }
            ViewMode::ThirdPerson { distance } => {
                let (sin_yaw, cos_yaw) = self.camera.yaw.0.sin_cos();
                let back = vec3(-cos_yaw, 0.0, -sin_yaw).normalize();
                let anchor = vec3(
                    self.position.x,
                    self.position.y + self.eye_height,
                    self.position.z,
                );
                let offset = back * distance + vec3(0.0, -0.3, 0.0);
                self.camera.position = cgmath::Point3::from_vec(anchor + offset);
            }
        }
    }

    fn update_creative(&mut self, dt: std::time::Duration, desired_velocity: Vector3<f32>) {
        let delta = desired_velocity * dt.as_secs_f32();
        self.position += delta;
        self.velocity = vec3(0.0, 0.0, 0.0);
    }

    fn update_adventure(
        &mut self,
        dt: std::time::Duration,
        desired_velocity: Vector3<f32>,
        chunks: &ChunkManager,
    ) {
        // Горизонтальная скорость задаётся мгновенно, вертикальная подчиняется физике.
        let mut delta = vec3(
            desired_velocity.x * dt.as_secs_f32(),
            0.0,
            desired_velocity.z * dt.as_secs_f32(),
        );

        if self.camera.camera_controller.jump_requested() && self.on_ground {
            self.velocity.y = self.jump_speed;
            self.on_ground = false;
        }

        self.velocity.y -= self.gravity * dt.as_secs_f32();
        if self.velocity.y < -self.max_fall_speed {
            self.velocity.y = -self.max_fall_speed;
        }
        delta.y = self.velocity.y * dt.as_secs_f32();

        let collided_y = self.move_with_collisions(delta, chunks);
        if collided_y && self.velocity.y < 0.0 {
            self.on_ground = true;
            self.velocity.y = 0.0;
            // Поднимаем чуть выше земли, чтобы избежать подёргиваний
            self.position.y = self.position.y.floor() + 0.001;
        } else {
            self.on_ground = false;
        }
    }

    fn move_with_collisions(&mut self, delta: Vector3<f32>, chunks: &ChunkManager) -> bool {
        let mut collided_y = false;
        let (dx, dy, dz) = (delta.x, delta.y, delta.z);

        if dx != 0.0 {
            let original = self.position.x;
            self.position.x += dx;
            if self.intersects_world(chunks) {
                self.position.x = original;
            }
        }

        if dz != 0.0 {
            let original = self.position.z;
            self.position.z += dz;
            if self.intersects_world(chunks) {
                self.position.z = original;
            }
        }

        if dy != 0.0 {
            let original = self.position.y;
            self.position.y += dy;
            if self.intersects_world(chunks) {
                self.position.y = original;
                collided_y = true;
            }
        }

        collided_y
    }

    fn intersects_world(&self, chunks: &ChunkManager) -> bool {
        let min = self.aabb_min();
        let max = self.aabb_max();

        let x0 = min.x.floor() as i32;
        let x1 = max.x.floor() as i32;
        let y0 = min.y.floor() as i32;
        let y1 = max.y.floor() as i32;
        let z0 = min.z.floor() as i32;
        let z1 = max.z.floor() as i32;

        for x in x0..=x1 {
            for y in y0..=y1 {
                for z in z0..=z1 {
                    if let Some(mat) = chunks.get_block_material(vec3(x, y, z)) {
                        if mat != MaterialType::AIR && mat != MaterialType::WATER {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn aabb_min(&self) -> Vector3<f32> {
        vec3(
            self.position.x - self.bounds_radius,
            self.position.y,
            self.position.z - self.bounds_radius,
        )
    }

    fn aabb_max(&self) -> Vector3<f32> {
        vec3(
            self.position.x + self.bounds_radius,
            self.position.y + self.bounds_height,
            self.position.z + self.bounds_radius,
        )
    }

    pub fn intersects_block(&self, block_pos: Vector3<i32>) -> bool {
        let min = self.aabb_min();
        let max = self.aabb_max();
        let block_min = vec3(block_pos.x as f32, block_pos.y as f32, block_pos.z as f32);
        let block_max = block_min + vec3(1.0, 1.0, 1.0);
        !(max.x <= block_min.x
            || min.x >= block_max.x
            || max.y <= block_min.y
            || min.y >= block_max.y
            || max.z <= block_min.z
            || min.z >= block_max.z)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ViewMode {
    FirstPerson,
    ThirdPerson { distance: f32 },
}

```

## src\player\raycast.rs
`rust
use super::camera::Camera;
use crate::{
    render::atlas::MaterialType,
    terrain_gen::{block::Direction, chunk::ChunkManager},
};
use cgmath::{InnerSpace, Vector3};

pub struct Ray {
    pub origin: cgmath::Point3<f32>,
    pub direction: Vector3<f32>,
    pub length: f32,
}

pub struct BlockHit {
    pub position: Vector3<i32>,
    pub face: Direction,
    pub distance: f32,
}

impl BlockHit {
    pub fn neighbor_position(&self) -> Vector3<i32> {
        self.position + self.face.to_vec()
    }
}

impl Ray {
    pub fn new(origin: cgmath::Point3<f32>, direction: Vector3<f32>, length: f32) -> Self {
        Self {
            origin,
            direction,
            length,
        }
    }

    pub fn from_camera(camera: &Camera, length: f32) -> Self {
        let (sin_pitch, cos_pitch) = camera.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = camera.yaw.0.sin_cos();

        let direction =
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();

        Self {
            origin: camera.position,
            direction,
            length,
        }
    }

    pub fn cast(&self, chunks: &ChunkManager) -> Option<BlockHit> {
        // Переводим начало луча в координаты блока
        let mut current_block_pos = Vector3::new(
            self.origin.x.floor() as i32,
            self.origin.y.floor() as i32,
            self.origin.z.floor() as i32,
        );

        // Считаем шаг и начальное расстояние по каждой оси
        let step = Vector3::new(
            if self.direction.x > 0.0 { 1 } else { -1 },
            if self.direction.y > 0.0 { 1 } else { -1 },
            if self.direction.z > 0.0 { 1 } else { -1 },
        );

        let next_boundary = Vector3::new(
            if step.x > 0 {
                current_block_pos.x as f32 + 1.0
            } else {
                current_block_pos.x as f32
            },
            if step.y > 0 {
                current_block_pos.y as f32 + 1.0
            } else {
                current_block_pos.y as f32
            },
            if step.z > 0 {
                current_block_pos.z as f32 + 1.0
            } else {
                current_block_pos.z as f32
            },
        );

        let mut t_max = Vector3::new(
            if self.direction.x != 0.0 {
                (next_boundary.x - self.origin.x) / self.direction.x
            } else {
                f32::INFINITY
            },
            if self.direction.y != 0.0 {
                (next_boundary.y - self.origin.y) / self.direction.y
            } else {
                f32::INFINITY
            },
            if self.direction.z != 0.0 {
                (next_boundary.z - self.origin.z) / self.direction.z
            } else {
                f32::INFINITY
            },
        );

        let t_delta = Vector3::new(
            if self.direction.x != 0.0 {
                step.x as f32 / self.direction.x
            } else {
                f32::INFINITY
            },
            if self.direction.y != 0.0 {
                step.y as f32 / self.direction.y
            } else {
                f32::INFINITY
            },
            if self.direction.z != 0.0 {
                step.z as f32 / self.direction.z
            } else {
                f32::INFINITY
            },
        );

        let mut face = Direction::TOP; // Значение по умолчанию, обновится позже
        let mut traveled_distance = 0.0;

        // Алгоритм DDA по вокселям
        while traveled_distance < self.length {
            // Проверяем, что текущий блок не воздух/вода
            if let Some(material) = chunks.get_block_material(current_block_pos) {
                if material != MaterialType::AIR && material != MaterialType::WATER {
                    return Some(BlockHit {
                        position: current_block_pos,
                        face,
                        distance: traveled_distance,
                    });
                }
            }

            // Продвигаемся к соседнему блоку
            if t_max.x < t_max.y && t_max.x < t_max.z {
                traveled_distance = t_max.x;
                t_max.x += t_delta.x;
                current_block_pos.x += step.x;
                face = if step.x > 0 {
                    Direction::LEFT
                } else {
                    Direction::RIGHT
                };
            } else if t_max.y < t_max.z {
                traveled_distance = t_max.y;
                t_max.y += t_delta.y;
                current_block_pos.y += step.y;
                face = if step.y > 0 {
                    Direction::BOTTOM
                } else {
                    Direction::TOP
                };
            } else {
                traveled_distance = t_max.z;
                t_max.z += t_delta.z;
                current_block_pos.z += step.z;
                face = if step.z > 0 {
                    Direction::BACK
                } else {
                    Direction::FRONT
                };
            }
        }

        None
    }
}

```

## src\render\atlas.rs
`rust
use anyhow::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::render::texture::*;
use crate::terrain_gen::block::*;

use super::pipelines::GlobalsLayouts;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MaterialType {
    DIRT,
    GRASS,
    ROCK,
    WATER,
    AIR,
    DEBUG,
}

impl MaterialType {
    pub fn is_transparent(&self) -> bool {
        match self {
            MaterialType::AIR => true,   // Возвращает true для AIR
            MaterialType::WATER => true, // Возвращает true для WATER
            _ => false,                  // Возвращает false для остальных материалов
        }
    }
}

impl MaterialType {
    pub fn get_texture_coordinates(
        &self,
        texture_corner: [u32; 2],
        quad_side: Direction,
    ) -> [f32; 2] {
        let atlas_size = atlas_size_px();
        // Сетка атласа считается 16x16 тайлов; размер тайла вычисляем динамически,
        // чтобы поддерживать атласы 256px и 512px без ручного пересчёта UV.
        let tile_size = atlas_size / 16.0;
        match self {
            MaterialType::GRASS => match quad_side {
                Direction::TOP => {
                    atlas_pos_to_coordinates([0.0, 0.0], texture_corner, tile_size, atlas_size)
                }
                Direction::BOTTOM => {
                    atlas_pos_to_coordinates([2.0, 0.0], texture_corner, tile_size, atlas_size)
                }
                Direction::RIGHT => {
                    atlas_pos_to_coordinates([3.0, 0.0], texture_corner, tile_size, atlas_size)
                }
                Direction::LEFT => {
                    atlas_pos_to_coordinates([3.0, 0.0], texture_corner, tile_size, atlas_size)
                }
                Direction::FRONT => {
                    atlas_pos_to_coordinates([3.0, 0.0], texture_corner, tile_size, atlas_size)
                }
                Direction::BACK => {
                    atlas_pos_to_coordinates([3.0, 0.0], texture_corner, tile_size, atlas_size)
                }
            },
            MaterialType::DIRT => {
                atlas_pos_to_coordinates([2.0, 0.0], texture_corner, tile_size, atlas_size)
            }
            MaterialType::ROCK => {
                atlas_pos_to_coordinates([0.0, 1.0], texture_corner, tile_size, atlas_size)
            }
            MaterialType::WATER => {
                atlas_pos_to_coordinates([13.0, 0.0], texture_corner, tile_size, atlas_size)
            }
            MaterialType::AIR => [0.0, 0.0],
            MaterialType::DEBUG => {
                atlas_pos_to_coordinates([5.0, 0.0], texture_corner, tile_size, atlas_size)
            } // match quad_side {
              //     Direction::TOP | Direction::BOTTOM => {
              //         atlas_pos_to_coordinates([5.0, 1.0], texture_corner, tile_size, atlas_size)
              //     }
              //     _ => atlas_pos_to_coordinates([4.0, 1.0], texture_corner, tile_size, atlas_size),
              // },
        }
    }
}

static ATLAS_SIZE_PX: OnceLock<f32> = OnceLock::new();
const DEFAULT_ATLAS_PX: f32 = 256.0;

fn atlas_size_px() -> f32 {
    *ATLAS_SIZE_PX.get_or_init(|| DEFAULT_ATLAS_PX)
}

fn atlas_pos_to_coordinates(
    atlas_pos: [f32; 2],
    texture_corner: [u32; 2],
    tile_size: f32,
    atlas_size: f32,
) -> [f32; 2] {
    let mut pixel_x = atlas_pos[0] * tile_size;
    let mut pixel_y = atlas_pos[1] * tile_size;

    if texture_corner[0] == 1 {
        pixel_x += tile_size - 1.0;
    }

    if texture_corner[1] == 1 {
        pixel_y += tile_size;
    }

    return [pixel_x / atlas_size, pixel_y / atlas_size];
}

pub struct Atlas {
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Atlas {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layouts: &GlobalsLayouts,
    ) -> Result<Self> {
        let diffuse_bytes = include_bytes!("../../assets/images/textures_atlas.png");
        let texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "blocks.png").unwrap();
        let _ = ATLAS_SIZE_PX.set(texture.width as f32);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layouts.atlas_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        Ok(Self {
            texture,
            bind_group,
        })
    }
}

```

## src\render\binding.rs
`rust
use wgpu::BindGroup;

use crate::render::{renderer::Renderer, texture::Texture};

impl<'a> Renderer<'a> {
    pub fn bind_atlas_texture(&self, tex: &Texture) -> BindGroup {
        self.layouts.global.bind_atlas_texture(&self.device, tex)
    }
}

```

## src\render\buffer.rs
`rust
use bytemuck::Pod;
use wgpu::{BufferUsages, util::DeviceExt};
pub struct Buffer<T: Copy + Pod> {
    pub buff: wgpu::Buffer,
    len: usize,
    phantom_data: std::marker::PhantomData<T>,
}

impl<T: Copy + Pod> Buffer<T> {
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[T]) -> Self {
        let contents = bytemuck::cast_slice(data);

        Self {
            buff: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage: usage | BufferUsages::COPY_DST,
            }),
            len: data.len(),
            phantom_data: std::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub struct DynamicBuffer<T: Copy + Pod>(Buffer<T>);

impl<T: Copy + Pod> DynamicBuffer<T> {
    pub fn new(device: &wgpu::Device, len: usize, usage: wgpu::BufferUsages) -> Self {
        // Не создаём буфер нулевого размера, чтобы не паниковали драйверы GPU; минимум 1 элемент.
        let len = len.max(1);
        let buffer = Buffer {
            buff: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: len as u64 * std::mem::size_of::<T>() as u64,
                usage: usage | wgpu::BufferUsages::COPY_DST,
            }),
            len,
            phantom_data: std::marker::PhantomData,
        };
        Self(buffer)
    }

    pub fn len(&self) -> usize {
        self.0.len
    }

    pub fn update(&self, queue: &wgpu::Queue, vals: &[T], offset: usize) {
        if !vals.is_empty() {
            queue.write_buffer(
                &self.buff,
                offset as u64 * std::mem::size_of::<T>() as u64,
                bytemuck::cast_slice(vals),
            )
        }
    }
}

impl<T: Copy + Pod> std::ops::Deref for DynamicBuffer<T> {
    type Target = Buffer<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

```

## src\render\consts.rs
`rust
use crate::render::buffer::DynamicBuffer;
use bytemuck::Pod;

// Хэндл для набора значений на GPU, которые не меняются в течение одного render pass.

pub struct Consts<T: Copy + Pod> {
    buf: DynamicBuffer<T>,
}

impl<T: Copy + Pod> Consts<T> {
    // Создать новый `Const<T>`.
    pub fn new(device: &wgpu::Device, len: usize) -> Self {
        Self {
            // TODO: проверить, все ли константы должны уметь обновляться
            buf: DynamicBuffer::new(device, len, wgpu::BufferUsages::UNIFORM),
        }
    }

    // Обновить значение на стороне GPU, связанное с этим хэндлом.
    pub fn update(&mut self, queue: &wgpu::Queue, vals: &[T], offset: usize) {
        self.buf.update(queue, vals, offset)
    }

    pub fn buf(&self) -> &wgpu::Buffer {
        &self.buf.buff
    }
}

// Константы неизменны в пределах одного render pass, но могут меняться к следующему.

```

## src\render\mesh.rs
`rust
use crate::terrain_gen::block::Quad;

use super::{Vertex, pipelines::terrain::BlockVertex};

#[derive(Clone)]

/// Меш на CPU, хранящий вершины и индексы.
pub struct Mesh<V: Vertex> {
    pub verts: Vec<V>,
    pub indices: Vec<u32>,
}

impl<V: Vertex> Mesh<V> {
    /// Создать новый `Mesh`.
    pub fn new() -> Self {
        Self {
            verts: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Очистить вершины, сохраняя выделенную память вектора.
    pub fn clear(&mut self) {
        self.verts.clear();
    }

    /// Получить срез ссылок на вершины.
    pub fn vertices(&self) -> &[V] {
        &self.verts
    }

    pub fn push(&mut self, vert: V) {
        self.verts.push(vert);
    }

    // Добавить набор индексов
    pub fn push_indices(&mut self, indices: &[u32]) {
        self.indices.extend_from_slice(indices);
    }

    // Вернуть индексы
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn add_quad(&mut self, quad: &Quad)
    where
        Vec<V>: Extend<BlockVertex>,
    {
        let base_index = self.verts.len() as u32;
        self.verts.extend(quad.vertices);
        self.indices.extend(&quad.get_indices(base_index));
    }

    pub fn clone(&self) -> Self {
        Self {
            verts: self.verts.clone(),
            indices: self.indices.clone(),
        }
    }

    pub fn iter_verts(&self) -> std::slice::Iter<'_, V> {
        self.verts.iter()
    }

    pub fn iter_indices(&self) -> std::vec::IntoIter<u32> {
        self.indices.clone().into_iter()
    }
}

```

## src\render\mod.rs
`rust
pub mod atlas;
pub mod binding;
pub mod buffer;
pub mod consts;
pub mod mesh;
pub mod model;
pub mod pipelines;
pub mod renderer;
pub mod texture;

pub trait Vertex: Copy + bytemuck::Pod {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

```

## src\render\model.rs
`rust
use crate::render::{buffer::Buffer, mesh::Mesh};

use super::{Vertex, buffer::DynamicBuffer};
/// Меш, отправленный на GPU.
pub struct Model<V: Vertex> {
    vbuf: Buffer<V>,
    ibuf: Buffer<u32>,
    pub num_indices: u32,
}

impl<V: Vertex> Model<V> {
    pub fn new(device: &wgpu::Device, mesh: &Mesh<V>) -> Option<Self> {
        if mesh.vertices().is_empty() || mesh.indices().is_empty() {
            return None;
        }

        let vbuf = Buffer::new(device, wgpu::BufferUsages::VERTEX, mesh.vertices());
        let ibuf = Buffer::new(device, wgpu::BufferUsages::INDEX, mesh.indices());

        Some(Self {
            vbuf,
            ibuf,
            num_indices: mesh.indices().len() as u32,
        })
    }

    pub fn vbuf(&self) -> &wgpu::Buffer {
        &self.vbuf.buff
    }
    pub fn ibuf(&self) -> &wgpu::Buffer {
        &self.ibuf.buff
    }
    pub fn len(&self) -> u16 {
        self.vbuf.len() as u16
    }
}

/// Меш, отправленный на GPU, с возможностью перевыделения буферов.
pub struct DynamicModel<V: Vertex> {
    vbuf: DynamicBuffer<V>,
    ibuf: DynamicBuffer<u32>,
    pub num_indices: u32,
    v_capacity: usize,
    i_capacity: usize,
    v_usage: wgpu::BufferUsages,
    i_usage: wgpu::BufferUsages,
}

impl<V: Vertex> DynamicModel<V> {
    pub fn new(device: &wgpu::Device, vertex_capacity: usize, index_capacity: usize) -> Self {
        let v_usage = wgpu::BufferUsages::VERTEX;
        let i_usage = wgpu::BufferUsages::INDEX;
        Self {
            vbuf: DynamicBuffer::new(device, vertex_capacity, v_usage),
            ibuf: DynamicBuffer::new(device, index_capacity, i_usage),
            num_indices: 0,
            v_capacity: vertex_capacity.max(1),
            i_capacity: index_capacity.max(1),
            v_usage,
            i_usage,
        }
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, verts: usize, indices: usize) {
        if verts > self.v_capacity {
            let new_cap = verts.next_power_of_two();
            self.vbuf = DynamicBuffer::new(device, new_cap, self.v_usage);
            self.v_capacity = new_cap;
        }
        if indices > self.i_capacity {
            let new_cap = indices.next_power_of_two();
            self.ibuf = DynamicBuffer::new(device, new_cap, self.i_usage);
            self.i_capacity = new_cap;
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &Mesh<V>) {
        self.ensure_capacity(device, mesh.vertices().len(), mesh.indices().len());
        self.vbuf.update(queue, mesh.vertices(), 0);
        self.ibuf.update(queue, mesh.indices(), 0);
        self.num_indices = mesh.indices().len() as u32;
    }

    /// Уменьшить буферы при возврате в пул, сохранив минимальный размер, чтобы избежать дёрганья.
    pub fn shrink_to(&mut self, device: &wgpu::Device, min_v: usize, min_i: usize) {
        let target_v = min_v.max(1).next_power_of_two();
        let target_i = min_i.max(1).next_power_of_two();
        if self.v_capacity > target_v {
            self.vbuf = DynamicBuffer::new(device, target_v, self.v_usage);
            self.v_capacity = target_v;
        }
        if self.i_capacity > target_i {
            self.ibuf = DynamicBuffer::new(device, target_i, self.i_usage);
            self.i_capacity = target_i;
        }
        self.num_indices = 0;
    }

    pub fn vbuf(&self) -> &wgpu::Buffer {
        &self.vbuf.buff
    }
    pub fn ibuf(&self) -> &wgpu::Buffer {
        &self.ibuf.buff
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.vbuf.len()
    }
}

```

## src\render\renderer.rs
`rust
#[cfg(feature = "tracy")]
use tracy_client::span;
use wgpu::{BindGroup, Error};
use winit::window::Window as SysWindow;

use super::{
    consts::Consts,
    pipelines::{GlobalModel, GlobalsLayouts},
    texture::{self, Texture},
};
use crate::{hud::HUD, terrain_gen::generator::TerrainGen};
use log::info;
pub trait Draw {
    fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        globals: &'a wgpu::BindGroup,
    ) -> Result<(), Error>;
}

pub struct Layouts {
    pub global: GlobalsLayouts,
}

pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a SysWindow,
    pub config: wgpu::SurfaceConfiguration,
    pub queue: wgpu::Queue,
    pub layouts: Layouts,
    depth_texture: Texture,
    clear_color: wgpu::Color,
}

impl<'a> Renderer<'a> {
    pub fn new(
        window: &'a SysWindow,
        present_mode: wgpu::PresentMode,
        sky_color: [f32; 3],
    ) -> Self {
        let size = window.inner_size();

        // Инстанс — это дескриптор для GPU.
        // Backends::all — Vulkan + Metal + DX12 + браузерный WebGPU.
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Безопасность
        //
        // Surface должна жить столько же, сколько окно, которое её создало.
        // State владеет окном, поэтому это безопасно.
        let surface = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::POLYGON_MODE_LINE
                    | wgpu::Features::TIMESTAMP_QUERY
                    | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
                    | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES,
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
            },
            None,
        ))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let chosen_present_mode = Self::pick_present_mode(&surface_caps, present_mode);
        // Шейдер ожидает sRGB-формат поверхности. Другой формат затемнит цвета,
        // поэтому поддержку иных форматов нужно учитывать отдельно.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: chosen_present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 1,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        info!("Using present mode: {:?}", chosen_present_mode);

        let layouts = Layouts {
            global: GlobalsLayouts::new(&device),
        };

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
        let clear_color = wgpu::Color {
            r: sky_color[0] as f64,
            g: sky_color[1] as f64,
            b: sky_color[2] as f64,
            a: 1.0,
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            layouts,
            depth_texture,
            clear_color,
        }
    }

    fn pick_present_mode(
        surface_caps: &wgpu::SurfaceCapabilities,
        requested: wgpu::PresentMode,
    ) -> wgpu::PresentMode {
        // Сначала пытаемся выбрать режим с минимальной задержкой, сохраняя намерение по vsync.
        let prefer_vsync = matches!(requested, wgpu::PresentMode::AutoVsync);
        let preference: &[wgpu::PresentMode] = if prefer_vsync {
            &[
                wgpu::PresentMode::Mailbox,
                wgpu::PresentMode::Fifo,
                wgpu::PresentMode::AutoVsync,
            ]
        } else {
            &[
                wgpu::PresentMode::Immediate,
                wgpu::PresentMode::Mailbox,
                wgpu::PresentMode::Fifo,
                wgpu::PresentMode::AutoNoVsync,
            ]
        };

        let supported = &surface_caps.present_modes;
        preference
            .iter()
            .copied()
            .find(|mode| supported.contains(mode))
            .unwrap_or_else(|| {
                // Запасной вариант — первый доступный режим поверхности.
                surface_caps
                    .present_modes
                    .first()
                    .copied()
                    .unwrap_or(wgpu::PresentMode::Fifo)
            })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn bind_globals(&self, global_model: &GlobalModel) -> BindGroup {
        self.layouts.global.bind(&self.device, global_model)
    }

    pub fn update(&mut self) {}

    pub fn create_consts<T: Copy + bytemuck::Pod>(&mut self, vals: &[T]) -> Consts<T> {
        let mut consts = Consts::new(&self.device, vals.len());
        consts.update(&self.queue, vals, 0);
        consts
    }

    /// Обновить набор констант переданными значениями.
    pub fn update_consts<T: Copy + bytemuck::Pod>(&self, consts: &mut Consts<T>, vals: &[T]) {
        #[cfg(feature = "tracy")]
        let _span = span!("update render constants"); // <- Отметка начала блока

        consts.update(&self.queue, vals, 0)
    }

    /// Переконфигурировать режим представления (vsync/novsync) без пересоздания GPU-ресурсов.
    pub fn reconfigure_present_mode(&mut self, present_mode: wgpu::PresentMode) {
        self.config.present_mode = present_mode;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(
        &mut self,
        terrain: &TerrainGen,
        hud: &HUD,
        globals: &BindGroup,
    ) -> Result<(), wgpu::SurfaceError> {
        #[cfg(feature = "tracy")]
        let get_texture_span = span!("get current texture");
        let output = self.surface.get_current_texture()?;
        #[cfg(feature = "tracy")]
        drop(get_texture_span);

        #[cfg(feature = "tracy")]
        let create_view_span = span!("create view");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        #[cfg(feature = "tracy")]
        drop(create_view_span);

        #[cfg(feature = "tracy")]
        let create_encoder_span = span!("create encoder");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        #[cfg(feature = "tracy")]
        drop(create_encoder_span);

        // Явно создаём и освобождаем render pass
        {
            #[cfg(feature = "tracy")]
            let create_render_pass = span!("create render pass");
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            #[cfg(feature = "tracy")]
            drop(create_render_pass);

            terrain.draw(&mut _render_pass, globals).unwrap();

            hud.draw(&mut _render_pass, globals).unwrap();
        } // _render_pass освобождается здесь

        // submit принимает всё, что реализует IntoIter
        #[cfg(feature = "tracy")]
        let submit_encoder = span!("submit encoder");
        self.queue.submit(std::iter::once(encoder.finish()));
        #[cfg(feature = "tracy")]
        drop(submit_encoder);
        #[cfg(feature = "tracy")]
        let prenset_output = span!("present output");
        output.present();
        #[cfg(feature = "tracy")]
        drop(prenset_output);

        Ok(())
    }
}

```

## src\render\texture.rs
`rust
use anyhow::*;
use image::GenericImageView;

#[derive(Clone)]
pub struct Texture {
    pub tex: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let tex = device.create_texture(&desc);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 1000.0,
            lod_max_clamp: 1000.0,
            ..Default::default()
        });

        Self {
            tex,
            view,
            sampler,
            width: config.width,
            height: config.height,
        }
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        Self::from_rgba(
            device,
            queue,
            &rgba,
            dimensions.0,
            dimensions.1,
            label.unwrap_or("image texture"),
        )
    }

    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: &[u8],
        width: u32,
        height: u32,
        label: &str,
    ) -> Result<Self> {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            tex: texture,
            view,
            sampler,
            width,
            height,
        })
    }

    pub fn write_rgba(&self, queue: &wgpu::Queue, rgba: &[u8], width: u32, height: u32) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

```

## src\render\pipelines\hud.rs
`rust
use wgpu::RenderPipeline;

use super::GlobalsLayouts;
use crate::render::{Vertex, texture::Texture};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HUDVertex {
    pub position: [f32; 2], // Позиция в экранных координатах (нормализованных)
    pub uv: [f32; 2],       // Текстурные координаты
}

impl HUDVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
}

impl Vertex for HUDVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<HUDVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub fn create_hud_pipeline(
    device: &wgpu::Device,
    global_layout: &GlobalsLayouts,
    shader: wgpu::ShaderModule,
    config: &wgpu::SurfaceConfiguration,
) -> RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Figure Pipeline Layout"),
        bind_group_layouts: &[&global_layout.hud_layout, &global_layout.globals],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render generic Pipeline"),
        layout: Some(&pipeline_layout),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..Default::default()
        },
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[HUDVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                // Расширенная настройка блендинга для прозрачности
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    pipeline
}

```

## src\render\pipelines\mod.rs
`rust
pub mod hud;
pub mod terrain;

use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix};
use wgpu::BindGroup;

use super::{consts::Consts, texture::Texture};

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct Globals {
    /// Преобразование из мировых координат (с focus_off в качестве начала)
    /// в координаты камеры.
    view_proj: [[f32; 4]; 4],
    /// Позиция камеры в мировых координатах (xyz) + паддинг.
    camera_pos: [f32; 4],
    /// Начало и конец тумана в мировых единицах (линейная интерполяция).
    fog: [f32; 4],
}

impl Globals {
    /// Создать глобальные константы из переданных параметров.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        view_proj: [[f32; 4]; 4],
        camera_pos: [f32; 3],
        fog_start: f32,
        fog_end: f32,
        sky_color: [f32; 3],
    ) -> Self {
        Self {
            view_proj,
            camera_pos: [camera_pos[0], camera_pos[1], camera_pos[2], sky_color[2]],
            fog: [fog_start, fog_end, sky_color[0], sky_color[1]],
        }
    }
}

impl Default for Globals {
    fn default() -> Self {
        Self::new(
            Matrix4::identity().into(),
            [0.0; 3],
            0.0,
            1.0,
            [0.6, 0.75, 0.9],
        )
    }
}

// Глобальные данные сцены, разбросанные по нескольким буферам.
pub struct GlobalModel {
    pub globals: Consts<Globals>,
}

pub struct GlobalsLayouts {
    pub globals: wgpu::BindGroupLayout,
    pub atlas_layout: wgpu::BindGroupLayout,
    pub hud_layout: wgpu::BindGroupLayout,
}

impl GlobalsLayouts {
    pub fn base_globals_layout() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            // Глобальный uniform
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]
    }

    pub fn new(device: &wgpu::Device) -> Self {
        let globals = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Globals layout"),
            entries: &Self::base_globals_layout(),
        });

        let atlas_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
            label: Some("atlas_bind_group_layout"),
        });

        // Отдельный layout для HUD с фильтрацией
        let hud_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Сэмплер с фильтрацией для более чёткой UI
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Текстура
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
            label: Some("hud_bind_group_layout"),
        });

        Self {
            globals,
            atlas_layout,
            hud_layout, // Добавляем layout HUD
        }
    }

    fn base_global_entries(global_model: &GlobalModel) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![
            // Глобальный uniform
            wgpu::BindGroupEntry {
                binding: 0,
                resource: global_model.globals.buf().as_entire_binding(),
            },
        ]
    }

    pub fn bind(&self, device: &wgpu::Device, global_model: &GlobalModel) -> BindGroup {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.globals,
            entries: &Self::base_global_entries(global_model),
        });

        bind_group
    }

    pub fn bind_atlas_texture(&self, device: &wgpu::Device, texture: &Texture) -> BindGroup {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.globals,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        bind_group
    }

    // Создание bind group для HUD
    pub fn bind_hud_texture(
        &self,
        device: &wgpu::Device,
        texture: &Texture,
        sampler: Option<&wgpu::Sampler>, // Позволяет передать кастомный сэмплер
    ) -> BindGroup {
        let default_sampler = sampler.unwrap_or(&texture.sampler);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hud_bind_group"),
            layout: &self.hud_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(default_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
            ],
        })
    }
}

```

## src\render\pipelines\terrain.rs
`rust
use wgpu::RenderPipeline;

use super::GlobalsLayouts;

use crate::render::{Vertex, texture::Texture};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockVertex {
    pub pos: [f32; 3],
    pub texture_coordinates: [f32; 2],
}

impl BlockVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
}

impl Vertex for BlockVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub fn create_terrain_pipeline(
    device: &wgpu::Device,
    global_layout: &GlobalsLayouts,
    shader: wgpu::ShaderModule,
    config: &wgpu::SurfaceConfiguration,
) -> RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Figure Pipeline Layout"),
        bind_group_layouts: &[
            &global_layout.atlas_layout,
            &global_layout.globals,
            // Здесь позже можно добавить layout, связанный с параметрами террейна
        ],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render generic Pipeline"),
        layout: Some(&pipeline_layout),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Значения, отличные от Fill, требуют Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Требуется Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Требуется Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[BlockVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    pipeline
}

```

## src\terrain_gen\biomes.rs
`rust
pub struct BiomeParameters {
    pub base_height: f32,
    pub frequency: f32,
    pub amplitude: f32,
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
}

pub const PRAIRIE_PARAMS: BiomeParameters = BiomeParameters {
    base_height: 10.0,
    frequency: 0.05,
    amplitude: 7.0,
    octaves: 3,
    persistence: 0.05,
    lacunarity: 2.0,
};

pub const MOUNTAIN_PARAMS: BiomeParameters = BiomeParameters {
    base_height: 15.0,
    frequency: 0.03,
    amplitude: 35.0,
    octaves: 4,
    persistence: 0.05,
    lacunarity: 2.0,
};

```

## src\terrain_gen\block.rs
`rust
use cgmath::Vector3;

use crate::render::atlas::MaterialType;

use crate::render::pipelines::terrain::BlockVertex;

pub fn quad_vertex(
    pos: [i8; 3],
    material_type: MaterialType,
    texture_corners: [u32; 2],
    position: [i32; 3],
    quad_side: Direction,
) -> BlockVertex {
    let tc = material_type.get_texture_coordinates(texture_corners, quad_side);
    BlockVertex {
        pos: [
            pos[0] as f32 + position[0] as f32,
            pos[1] as f32 + position[1] as f32,
            pos[2] as f32 + position[2] as f32,
        ],
        texture_coordinates: [tc[0] as f32, tc[1] as f32],
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    TOP,
    BOTTOM,
    RIGHT,
    LEFT,
    FRONT,
    BACK,
}

impl Direction {
    pub const ALL: [Self; 6] = [
        Self::TOP,
        Self::BOTTOM,
        Self::RIGHT,
        Self::LEFT,
        Self::FRONT,
        Self::BACK,
    ];

    pub fn to_vec(self) -> Vector3<i32> {
        match self {
            Direction::TOP => Vector3::new(0, 1, 0),
            Direction::BOTTOM => Vector3::new(0, -1, 0),
            Direction::RIGHT => Vector3::new(1, 0, 0),
            Direction::LEFT => Vector3::new(-1, 0, 0),
            Direction::FRONT => Vector3::new(0, 0, 1),
            Direction::BACK => Vector3::new(0, 0, -1),
        }
    }

    pub fn get_vertices(self, material_type: MaterialType, position: [i32; 3]) -> [BlockVertex; 4] {
        match self {
            Direction::TOP => [
                quad_vertex([0, 1, 0], material_type, [0, 0], position, self),
                quad_vertex([0, 1, 1], material_type, [0, 1], position, self),
                quad_vertex([1, 1, 1], material_type, [1, 1], position, self),
                quad_vertex([1, 1, 0], material_type, [1, 0], position, self),
            ],
            Direction::BOTTOM => [
                quad_vertex([0, 0, 1], material_type, [0, 0], position, self),
                quad_vertex([0, 0, 0], material_type, [0, 1], position, self),
                quad_vertex([1, 0, 0], material_type, [1, 1], position, self),
                quad_vertex([1, 0, 1], material_type, [1, 0], position, self),
            ],
            Direction::RIGHT => [
                quad_vertex([1, 1, 1], material_type, [0, 0], position, self),
                quad_vertex([1, 0, 1], material_type, [0, 1], position, self),
                quad_vertex([1, 0, 0], material_type, [1, 1], position, self),
                quad_vertex([1, 1, 0], material_type, [1, 0], position, self),
            ],
            Direction::LEFT => [
                quad_vertex([0, 1, 0], material_type, [0, 0], position, self),
                quad_vertex([0, 0, 0], material_type, [0, 1], position, self),
                quad_vertex([0, 0, 1], material_type, [1, 1], position, self),
                quad_vertex([0, 1, 1], material_type, [1, 0], position, self),
            ],
            Direction::FRONT => [
                quad_vertex([0, 1, 1], material_type, [0, 0], position, self),
                quad_vertex([0, 0, 1], material_type, [0, 1], position, self),
                quad_vertex([1, 0, 1], material_type, [1, 1], position, self),
                quad_vertex([1, 1, 1], material_type, [1, 0], position, self),
            ],
            Direction::BACK => [
                quad_vertex([1, 1, 0], material_type, [0, 0], position, self),
                quad_vertex([1, 0, 0], material_type, [0, 1], position, self),
                quad_vertex([0, 0, 0], material_type, [1, 1], position, self),
                quad_vertex([0, 1, 0], material_type, [1, 0], position, self),
            ],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Quad {
    pub vertices: [BlockVertex; 4],
    pub side: Direction,
}

impl Quad {
    pub fn new(material_type: MaterialType, quad_side: Direction, position: [i32; 3]) -> Self {
        Self {
            vertices: quad_side.get_vertices(material_type, position),
            side: quad_side,
        }
    }

    pub fn get_indices_v(&self, vertex_offset: u32) -> [u32; 6] {
        [
            vertex_offset,
            vertex_offset + 1,
            vertex_offset + 2,
            vertex_offset + 2,
            vertex_offset + 3,
            vertex_offset,
        ]
    }

    pub fn get_indices(&self, i: u32) -> [u32; 6] {
        let displacement = i * 4;
        [
            0 + displacement,
            1 + displacement,
            2 + displacement,
            2 + displacement,
            3 + displacement,
            0 + displacement,
        ]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Block {
    pub material_type: MaterialType,
}

impl Block {
    pub fn new(material_type: MaterialType) -> Self {
        Self { material_type }
    }

    pub fn is_transparent(&self) -> bool {
        self.material_type == MaterialType::AIR || self.material_type == MaterialType::WATER
    }

    pub fn is_solid(&self) -> bool {
        !self.is_transparent()
    }

    pub fn update(&mut self, new_material_type: MaterialType) {
        self.material_type = new_material_type;
    }
}

impl Default for Block {
    fn default() -> Self {
        Block {
            material_type: MaterialType::AIR,
        }
    }
}

```

## src\terrain_gen\chunk.rs
`rust
use std::{
    fs,
    path::Path,
    sync::{Arc, RwLock},
};

use anyhow::{Result, bail};
use cgmath::Vector3;
use std::collections::HashMap;
#[cfg(feature = "tracy")]
use tracy_client::span;

use crate::render::{atlas::MaterialType, mesh::Mesh, pipelines::terrain::BlockVertex};

use super::{biomes::BiomeParameters, block::Block, noise::NoiseGenerator};

pub const CHUNK_Y_SIZE: usize = 512;
pub const CHUNK_AREA: usize = 16;
pub const CHUNK_AREA_WITH_PADDING: usize = CHUNK_AREA + 2; // +1 с каждой стороны для паддинга
pub const TOTAL_CHUNK_SIZE: usize =
    CHUNK_Y_SIZE * CHUNK_AREA_WITH_PADDING * CHUNK_AREA_WITH_PADDING;

pub struct Chunk {
    pub blocks: Vec<Block>,
    pub offset: [i32; 3],
    pub mesh: Mesh<BlockVertex>,
    pub dirty: bool,
    /// Флаг, что содержимое нужно сохранить на диск.
    pub needs_save: bool,
    layer_meshes: Vec<LayerMesh>,
    layer_dirty: Vec<bool>,
    layer_spans: Vec<LayerSpan>,
    layout_changed: bool,
    rebuilt_layers: Vec<usize>,
    dirty_y_range: Option<(usize, usize)>,
}

#[derive(Default, Clone)]
struct LayerMesh {
    verts: Vec<BlockVertex>,
    indices: Vec<u32>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct LayerSpan {
    pub v_start: u32,
    pub v_len: u32,
    pub i_start: u32,
    pub i_len: u32,
}

impl Chunk {
    pub fn new(offset: [i32; 3]) -> Self {
        let mut blocks = Vec::with_capacity(TOTAL_CHUNK_SIZE);

        for y in 0..CHUNK_Y_SIZE {
            for _x in 0..CHUNK_AREA_WITH_PADDING {
                for _z in 0..CHUNK_AREA_WITH_PADDING {
                    let material_type = if y < 12 {
                        MaterialType::DEBUG
                    } else if y == 12 {
                        MaterialType::DEBUG
                    } else {
                        MaterialType::AIR
                    };

                    blocks.push(Block::new(material_type));
                }
            }
        }
        let mesh = Mesh::new();
        Chunk {
            blocks,
            offset,
            mesh,
            dirty: false,
            needs_save: false,
            layer_meshes: vec![LayerMesh::default(); CHUNK_Y_SIZE],
            layer_dirty: vec![true; CHUNK_Y_SIZE],
            layer_spans: vec![LayerSpan::default(); CHUNK_Y_SIZE],
            layout_changed: true,
            rebuilt_layers: (0..CHUNK_Y_SIZE).collect(),
            dirty_y_range: None,
        }
    }

    /// Линейный индекс внутри чанка по координатам y, x, z
    fn calculate_index(&self, y: usize, x: usize, z: usize) -> usize {
        y * (CHUNK_AREA_WITH_PADDING * CHUNK_AREA_WITH_PADDING) + x * CHUNK_AREA_WITH_PADDING + z
    }

    /// Получить материал блока (immut)
    pub fn get_block(&self, y: usize, x: usize, z: usize) -> Option<MaterialType> {
        if y < CHUNK_Y_SIZE && x < CHUNK_AREA_WITH_PADDING && z < CHUNK_AREA_WITH_PADDING {
            let index = self.calculate_index(y, x, z);
            self.blocks.get(index).copied().map(|b| b.material_type)
        } else {
            None
        }
    }

    /// Получить изменяемый блок
    pub fn get_block_mut(&mut self, y: usize, x: usize, z: usize) -> Option<&mut MaterialType> {
        if y < CHUNK_Y_SIZE && x < CHUNK_AREA_WITH_PADDING && z < CHUNK_AREA_WITH_PADDING {
            let index = self.calculate_index(y, x, z);
            return self.blocks.get_mut(index).map(|b| &mut b.material_type);
        }
        None
    }

    pub fn update_blocks(
        &mut self,
        offset: [i32; 3],
        noise_generator: &NoiseGenerator,
        biome: &BiomeParameters,
        land_level: usize,
    ) {
        #[cfg(feature = "tracy")]
        let _span = span!("generate chunk: full scope"); // Замер генерации чанка

        self.offset = offset; // Сохраняем смещение чанка

        let max_biome_height = (biome.base_height + biome.amplitude) as usize;

        for y in 0..CHUNK_Y_SIZE {
            for x in 0..CHUNK_AREA_WITH_PADDING {
                for z in 0..CHUNK_AREA_WITH_PADDING {
                    // Чанк может переиспользоваться: обязательно обнуляем старые данные
                    // на высотах, куда новый биом не дотягивается.
                    let block_type = if y > max_biome_height {
                        MaterialType::AIR
                    } else {
                        #[cfg(feature = "tracy")]
                        let _inner_span = span!(" creating single block");

                        if y < (biome.base_height - 1.0) as usize {
                            MaterialType::DIRT
                        } else {
                            let local_x = x as i32 - 1;
                            let local_z = z as i32 - 1;
                            let world_pos = local_pos_to_world(
                                self.offset,
                                Vector3::new(local_x, y as i32, local_z),
                            );
                            let height_variation = noise_generator.get_height(
                                world_pos.x as f32,
                                world_pos.z as f32,
                                biome.frequency,
                                biome.amplitude,
                            );
                            let new_height =
                                (biome.base_height + height_variation).round() as usize;

                            if y > new_height {
                                if y <= land_level {
                                    MaterialType::WATER
                                } else {
                                    MaterialType::AIR
                                }
                            } else if y == new_height {
                                MaterialType::GRASS
                            } else if y == 0 {
                                MaterialType::ROCK
                            } else {
                                MaterialType::DIRT
                            }
                        }
                    };

                    if let Some(block) = self.get_block_mut(y, x, z) {
                        *block = block_type;
                    }
                }
            }
        }
        self.dirty = true;
        self.dirty_y_range = Some((0, CHUNK_Y_SIZE - 1));
        self.layer_dirty.iter_mut().for_each(|d| *d = true);
        self.rebuilt_layers = (0..CHUNK_Y_SIZE).collect();
    }

    pub fn update_mesh(&mut self, _biome: BiomeParameters, y_range: Option<(usize, usize)>) {
        let (y_start, y_end) = match y_range {
            Some((lo, hi)) => (lo.min(CHUNK_Y_SIZE - 1), hi.min(CHUNK_Y_SIZE - 1)),
            None => {
                self.layer_dirty.iter_mut().for_each(|d| *d = true);
                (0, CHUNK_Y_SIZE - 1)
            }
        };

        let mut rebuilt = Vec::new();

        #[cfg(feature = "tracy")]
        let _span = span!(" update chunk mesh"); // Замер построения меша

        for y in y_start..=y_end {
            if !self.layer_dirty.get(y).copied().unwrap_or(false) {
                continue;
            }
            rebuilt.push(y);
            self.layer_dirty[y] = false;
            let mut layer = LayerMesh::default();

            for x in 1..=CHUNK_AREA {
                for z in 1..=CHUNK_AREA {
                    #[cfg(feature = "tracy")]
                    let _inner_span = span!("processing block vertices"); // Замер вершин блока

                    let local_pos = Vector3::new(x as i32 - 1, y as i32, z as i32 - 1);
                    let block = self.get_block(y, x, z).unwrap();
                    if block == MaterialType::AIR {
                        continue;
                    }

                    let mut block_indices: Vec<u32> = Vec::with_capacity(6 * 6);
                    let mut quad_counter = 0;

                    for side in crate::terrain_gen::block::Direction::ALL {
                        let neighbor_pos: Vector3<i32> = local_pos + side.to_vec();
                        let visible = self.is_quad_visible(&neighbor_pos);

                        if visible {
                            let world_pos = [
                                local_pos.x + (self.offset[0] * CHUNK_AREA as i32),
                                local_pos.y,
                                local_pos.z + (self.offset[2] * CHUNK_AREA as i32),
                            ];
                            let quad = crate::terrain_gen::block::Quad::new(block, side, world_pos);
                            layer.verts.extend_from_slice(&quad.vertices);
                            block_indices.extend_from_slice(&quad.get_indices(quad_counter));
                            quad_counter += 1;
                        }
                    }

                    let base = layer.verts.len() as u32 - (quad_counter * 4);
                    block_indices = block_indices.iter().map(|i| i + base).collect();
                    layer.indices.extend(block_indices);
                }
            }

            self.layer_meshes[y] = layer;
        }

        let mut verts = Vec::new();
        let mut indices = Vec::new();
        let mut spans = Vec::with_capacity(CHUNK_Y_SIZE);
        let mut v_start = 0u32;
        let mut i_start = 0u32;
        for layer in &self.layer_meshes {
            let v_len = layer.verts.len() as u32;
            let i_len = layer.indices.len() as u32;
            spans.push(LayerSpan {
                v_start,
                v_len,
                i_start,
                i_len,
            });
            indices.extend(layer.indices.iter().map(|i| i + v_start));
            verts.extend_from_slice(&layer.verts);
            v_start += v_len;
            i_start += i_len;
        }

        let layout_changed = self.layer_spans.len() != spans.len()
            || self
                .layer_spans
                .iter()
                .zip(spans.iter())
                .any(|(a, b)| a != b);

        self.mesh = Mesh { verts, indices };
        self.layer_spans = spans;
        self.layout_changed = layout_changed;
        self.rebuilt_layers = rebuilt;
        self.dirty_y_range = None;
    }

    fn mark_dirty_y(&mut self, y: usize) {
        let y0 = y.saturating_sub(1);
        let y1 = (y + 1).min(CHUNK_Y_SIZE - 1);
        self.dirty_y_range = match self.dirty_y_range {
            Some((lo, hi)) => Some((lo.min(y0), hi.max(y1))),
            None => Some((y0, y1)),
        };
        for ly in y0..=y1 {
            if let Some(flag) = self.layer_dirty.get_mut(ly) {
                *flag = true;
            }
        }
    }

    pub fn dirty_y_range(&self) -> Option<(usize, usize)> {
        self.dirty_y_range
    }

    pub fn layout_changed(&self) -> bool {
        self.layout_changed
    }

    pub fn layer_spans(&self) -> &[LayerSpan] {
        &self.layer_spans
    }

    pub fn take_rebuilt_layers(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.rebuilt_layers)
    }

    pub fn layer_mesh(&self, y: usize) -> Option<(&[BlockVertex], &[u32])> {
        self.layer_meshes
            .get(y)
            .map(|lm| (lm.verts.as_slice(), lm.indices.as_slice()))
    }

    fn is_quad_visible(&self, neighbor_pos: &Vector3<i32>) -> bool {
        if pos_in_chunk_bounds(*neighbor_pos) {
            // Преобразуем координаты (-1..16) в индексы массива (0..17)

            let x_index = (neighbor_pos.x + 1) as usize;
            let y_index = neighbor_pos.y as usize;
            let z_index = (neighbor_pos.z + 1) as usize;

            let neighbor_block = self.get_block(y_index, x_index, z_index).unwrap();
            return neighbor_block as u16 == MaterialType::AIR as u16;
        } else {
            // Нет соседа в этом чанке — считаем грань видимой, чтобы не пропадали блоки на границах.
            return true;
        }
    }
}

pub struct ChunkManager {
    pub chunks: Vec<Arc<RwLock<Chunk>>>,
    offset_index_map: HashMap<[i32; 3], usize>,
    index_offset: Vec<[i32; 3]>,
}

impl ChunkManager {
    pub fn new() -> Self {
        ChunkManager {
            chunks: Vec::new(),
            offset_index_map: HashMap::new(),
            index_offset: Vec::new(),
        }
    }

    pub fn add_chunk(&mut self, mut chunk: Chunk) {
        chunk.offset = [i32::MIN, i32::MIN, i32::MIN];
        self.index_offset.push(chunk.offset);
        self.chunks.push(Arc::new(RwLock::new(chunk)));
        // offset_index_map будет заполнен при update_chunk_offset
        debug_assert!(
            self.offset_index_map
                .get(&[i32::MIN, i32::MIN, i32::MIN])
                .is_none()
        );
    }

    pub fn get_chunk(&self, index: usize) -> Option<Arc<RwLock<Chunk>>> {
        if index < self.chunks.len() {
            Some(self.chunks[index].clone())
        } else {
            None
        }
    }

    pub fn get_chunk_index_by_offset(&self, offset: &[i32; 3]) -> Option<usize> {
        self.offset_index_map.get(offset).copied()
    }

    // Получить материал блока в мировых координатах
    pub fn get_block_material(&self, world_pos: Vector3<i32>) -> Option<MaterialType> {
        let (chunk_offset, local_pos) = world_pos_to_chunk_and_local(world_pos);

        // Учитываем паддинг (local_pos 0..15 -> нужно -1..16)
        let x = local_pos.x + 1;
        let z = local_pos.z + 1;
        let y = local_pos.y;

        if !pos_in_chunk_bounds(Vector3::new(x, y, z)) {
            return None;
        }

        self.get_chunk_index_by_offset(&chunk_offset)
            .and_then(|index| {
                let chunk = self.chunks[index].read().unwrap();
                chunk.get_block(y as usize, x as usize, z as usize)
            })
    }

    // Установить материал блока в мировых координатах
    pub fn set_block_material(
        &mut self,
        world_pos: Vector3<i32>,
        material: MaterialType,
    ) -> Vec<usize> {
        let (chunk_offset, local_pos) = world_pos_to_chunk_and_local(world_pos);

        // Учитываем паддинг (local_pos 0..15 -> нужно -1..16)
        let x = local_pos.x + 1;
        let z = local_pos.z + 1;
        let y = local_pos.y;

        if !pos_in_chunk_bounds(Vector3::new(x, y, z)) {
            println!("Position out of bounds: {:?}", world_pos);
            return Vec::new();
        }

        let mut touched = Vec::new();
        let mut neighbor_offsets = Vec::new();
        if local_pos.x == 0 {
            neighbor_offsets.push([chunk_offset[0] - 1, chunk_offset[1], chunk_offset[2]]);
        } else if local_pos.x == (CHUNK_AREA as i32 - 1) {
            neighbor_offsets.push([chunk_offset[0] + 1, chunk_offset[1], chunk_offset[2]]);
        }
        if local_pos.z == 0 {
            neighbor_offsets.push([chunk_offset[0], chunk_offset[1], chunk_offset[2] - 1]);
        } else if local_pos.z == (CHUNK_AREA as i32 - 1) {
            neighbor_offsets.push([chunk_offset[0], chunk_offset[1], chunk_offset[2] + 1]);
        }

        if let Some(index) = self.get_chunk_index_by_offset(&chunk_offset) {
            let mut chunk = self.chunks[index].write().unwrap();
            if let Some(block) = chunk.get_block_mut(y as usize, x as usize, z as usize) {
                *block = material;
                chunk.dirty = true;
                chunk.needs_save = true;
                chunk.mark_dirty_y(y as usize);
                println!("Block updated at world position: {:?}", world_pos);
                touched.push(index);
            }
            drop(chunk);

            // Если блок на границе чанка — отмечаем соседние чанки как грязные, чтобы перерассчитать меш.
            for neigh_off in neighbor_offsets {
                if let Some(nidx) = self.get_chunk_index_by_offset(&neigh_off) {
                    if let Ok(mut neigh_chunk) = self.chunks[nidx].write() {
                        // Обновляем паддинг соседа, чтобы его грань стала видимой/скрытой корректно.
                        let origin = Vector3::new(
                            neigh_off[0] * CHUNK_AREA as i32,
                            neigh_off[1] * CHUNK_Y_SIZE as i32,
                            neigh_off[2] * CHUNK_AREA as i32,
                        );
                        let local_in_neigh = world_pos - origin;
                        let on_padding = local_in_neigh.x == -1
                            || local_in_neigh.x == CHUNK_AREA as i32
                            || local_in_neigh.z == -1
                            || local_in_neigh.z == CHUNK_AREA as i32;
                        if on_padding && pos_in_chunk_bounds(local_in_neigh) {
                            let nx = (local_in_neigh.x + 1) as usize;
                            let nz = (local_in_neigh.z + 1) as usize;
                            let ny = local_in_neigh.y as usize;
                            if let Some(pad_block) = neigh_chunk.get_block_mut(ny, nx, nz) {
                                *pad_block = material;
                            }
                            neigh_chunk.mark_dirty_y(ny);
                            neigh_chunk.needs_save = true;
                        }
                        neigh_chunk.dirty = true;
                    }
                    touched.push(nidx);
                }
            }
            touched
        } else {
            println!("Chunk not found for world position: {:?}", world_pos);
            Vec::new()
        }
    }

    pub fn update_chunk_offset(&mut self, index: usize, new_offset: [i32; 3]) {
        if let Some(old_offset) = self.index_offset.get(index).copied() {
            self.offset_index_map.remove(&old_offset);
        }
        if index < self.index_offset.len() {
            self.index_offset[index] = new_offset;
        }
        self.offset_index_map.insert(new_offset, index);
        if let Some(chunk) = self.chunks.get(index) {
            if let Ok(mut chunk) = chunk.write() {
                chunk.offset = new_offset;
            }
        }
    }

    pub fn remove_chunk_from_map(&mut self, index: usize) {
        if let Some(old_offset) = self.index_offset.get(index).copied() {
            self.offset_index_map.remove(&old_offset);
            // Используем заведомо "пустой" оффсет, чтобы не затереть валидную запись
            // для реально существующего чанка в (0, 0, 0).
            self.index_offset[index] = [i32::MIN, i32::MIN, i32::MIN];
        }
    }
}

pub fn pos_in_chunk_bounds(pos: Vector3<i32>) -> bool {
    // Допускаем координаты от -1 до CHUNK_AREA (0..15 внутренняя область, -1 и 16 — паддинг)
    pos.x >= -1
        && pos.y >= 0
        && pos.z >= -1
        && pos.x <= CHUNK_AREA as i32
        && pos.y < CHUNK_Y_SIZE as i32
        && pos.z <= CHUNK_AREA as i32
}

fn world_pos_to_chunk_and_local(world_pos: Vector3<i32>) -> ([i32; 3], Vector3<i32>) {
    let chunk_x = world_pos.x.div_euclid(CHUNK_AREA as i32);
    let chunk_y = world_pos.y.div_euclid(CHUNK_Y_SIZE as i32);
    let chunk_z = world_pos.z.div_euclid(CHUNK_AREA as i32);

    let local_x = world_pos.x.rem_euclid(CHUNK_AREA as i32);
    let local_y = world_pos.y.rem_euclid(CHUNK_Y_SIZE as i32);
    let local_z = world_pos.z.rem_euclid(CHUNK_AREA as i32);

    (
        [chunk_x, chunk_y, chunk_z],
        Vector3::new(local_x, local_y, local_z),
    )
}

pub fn local_pos_to_world(offset: [i32; 3], local_pos: Vector3<i32>) -> Vector3<f32> {
    Vector3::new(
        local_pos.x as f32 + (offset[0] as f32 * CHUNK_AREA as f32),
        local_pos.y as f32 + (offset[1] as f32 * CHUNK_Y_SIZE as f32),
        local_pos.z as f32 + (offset[2] as f32 * CHUNK_AREA as f32),
    )
}

fn material_to_u8(mat: MaterialType) -> u8 {
    match mat {
        MaterialType::DIRT => 0,
        MaterialType::GRASS => 1,
        MaterialType::ROCK => 2,
        MaterialType::WATER => 3,
        MaterialType::AIR => 4,
        MaterialType::DEBUG => 5,
    }
}

fn material_from_u8(v: u8) -> MaterialType {
    match v {
        0 => MaterialType::DIRT,
        1 => MaterialType::GRASS,
        2 => MaterialType::ROCK,
        3 => MaterialType::WATER,
        4 => MaterialType::AIR,
        5 => MaterialType::DEBUG,
        _ => MaterialType::AIR,
    }
}

impl Chunk {
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut buf = Vec::with_capacity(self.blocks.len());
        for block in &self.blocks {
            buf.push(material_to_u8(block.material_type));
        }
        fs::write(path, buf)?;
        Ok(())
    }

    pub fn load_from(&mut self, path: &Path, offset: [i32; 3]) -> Result<()> {
        let data = fs::read(path)?;
        if data.len() != self.blocks.len() {
            bail!("chunk file has wrong size");
        }
        for (b, val) in self.blocks.iter_mut().zip(data.into_iter()) {
            b.update(material_from_u8(val));
        }
        self.offset = offset;
        self.dirty = false;
        self.needs_save = false;
        self.dirty_y_range = None;
        Ok(())
    }
}

```

## src\terrain_gen\generator.rs
`rust
use std::{
    collections::HashSet,
    collections::VecDeque,
    sync::{Arc, RwLock},
    thread,
};

use crate::core::config::AppConfig;
use crate::render::pipelines::GlobalsLayouts;
use crate::terrain_gen::chunk::{CHUNK_AREA, Chunk, ChunkManager};
use crate::{
    render::{
        Vertex,
        atlas::{Atlas, MaterialType},
        mesh::Mesh,
        model::{DynamicModel, Model},
        pipelines::terrain::{BlockVertex, create_terrain_pipeline},
        renderer::{Draw, Renderer},
    },
    terrain_gen::biomes::PRAIRIE_PARAMS,
};

use bytemuck::cast_slice;
use cgmath::{EuclideanSpace, Point3, Vector3};
use crossbeam_channel::{Receiver, Sender};
use std::path::PathBuf;
#[cfg(feature = "tracy")]
use tracy_client::span;
use wgpu::Queue;

use super::noise::NoiseGenerator;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OutlineVertex {
    pub pos: [f32; 3],
}

impl Vertex for OutlineVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<OutlineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        }
    }
}

pub struct TerrainGen {
    pipeline: wgpu::RenderPipeline,
    highlight_pipeline: wgpu::RenderPipeline,
    atlas: Atlas,
    pub chunks: ChunkManager,
    chunks_view_size: usize,
    chunk_indices: Arc<RwLock<Vec<Option<usize>>>>,
    free_chunk_indices: Arc<RwLock<VecDeque<usize>>>,
    center_offset: Vector3<i32>,
    chunks_origin: Vector3<i32>,
    pub chunk_models: Vec<Arc<RwLock<DynamicModel<BlockVertex>>>>,
    gen_job_tx: Sender<ChunkJob>,
    remesh_job_tx: Sender<ChunkJob>,
    ready_rx: Receiver<usize>,
    pending_jobs: HashSet<usize>,
    save_dir: PathBuf,
    dirty_queue: VecDeque<usize>,
    dirty_set: HashSet<usize>,
    save_tx: Sender<(PathBuf, Vec<u8>)>,
    highlight_model: Option<Model<OutlineVertex>>,
    highlight_pos: Option<Vector3<i32>>,
    max_jobs_in_flight: usize,
    max_dirty_per_frame: usize,
    min_vertex_cap: usize,
    min_index_cap: usize,
    land_level: usize,
}

enum JobKind {
    Generate { offset: Vector3<i32> },
    Remesh,
}

struct ChunkJob {
    chunk_index: usize,
    chunk: Arc<RwLock<Chunk>>,
    kind: JobKind,
    save_dir: PathBuf,
    land_level: usize,
}

impl TerrainGen {
    pub fn new(renderer: &Renderer, config: &AppConfig) -> Self {
        let render_distance_chunks = config.graphics.render_distance_chunks;
        let seed = config.world.seed;
        let world_name = &config.world.world_name;
        let tuning = &config.terrain;

        let save_dir = PathBuf::from(format!("saves/{world_name}"));
        let _ = std::fs::create_dir_all(&save_dir);
        let global_layouts = GlobalsLayouts::new(&renderer.device);
        let atlas = Atlas::new(&renderer.device, &renderer.queue, &global_layouts).unwrap();
        let mut chunk_models = vec![];
        let mut chunks = ChunkManager::new();
        let chunks_view_size = render_distance_chunks.max(2);
        let chunk_capacity = chunks_view_size * chunks_view_size;
        let chunk_indices: Vec<Option<usize>> = vec![None; chunk_capacity];
        let mut free_chunk_indices = VecDeque::new();

        let noise_gen = NoiseGenerator::new(seed);

        for x in 0..chunk_capacity {
            chunks.add_chunk(Chunk::new([0, 0, 0]));
            // Начинаем с небольшого GPU-буфера и при необходимости растим его.
            let vertex_capacity = tuning.min_vertex_cap; // увеличится, если чанку нужно больше
            let index_capacity = tuning.min_index_cap;
            let mut chunk_model =
                DynamicModel::new(&renderer.device, vertex_capacity, index_capacity);

            chunk_model.update(
                &renderer.device,
                &renderer.queue,
                &chunks.get_chunk(x).unwrap().read().unwrap().mesh,
            );
            chunk_models.push(Arc::new(RwLock::new(chunk_model)));
            free_chunk_indices.push_back(x);
        }

        let shader = renderer
            .device
            .create_shader_module(wgpu::include_wgsl!("../../assets/shaders/shader.wgsl"));

        let world_pipeline = create_terrain_pipeline(
            &renderer.device,
            &global_layouts,
            shader.clone(),
            &renderer.config,
        );
        let highlight_shader = create_highlight_shader(&renderer.device);
        let highlight_pipeline = create_highlight_pipeline(
            &renderer.device,
            &global_layouts,
            &highlight_shader,
            &renderer.config,
        );

        let center_offset = Vector3::new(0, 0, 0);
        let chunks_origin = center_offset
            - Vector3::new(chunks_view_size as i32 / 2, 0, chunks_view_size as i32 / 2);

        let (gen_job_tx, gen_job_rx) = crossbeam_channel::unbounded::<ChunkJob>();
        let (remesh_job_tx, remesh_job_rx) = crossbeam_channel::unbounded::<ChunkJob>();
        let (ready_tx, ready_rx) = crossbeam_channel::unbounded::<usize>();
        let (save_tx, save_rx) = crossbeam_channel::unbounded::<(PathBuf, Vec<u8>)>();
        let noise_for_worker = noise_gen.clone();
        let worker_count = tuning.jobs_in_flight.max(1);

        for _ in 0..worker_count {
            let remesh_job_rx = remesh_job_rx.clone();
            let gen_job_rx = gen_job_rx.clone();
            let ready_tx = ready_tx.clone();
            let noise_for_worker = noise_for_worker.clone();

            std::thread::spawn(move || {
                let process_job = |job: ChunkJob| {
                    match job.kind {
                        JobKind::Generate { offset } => {
                            if let Ok(mut chunk) = job.chunk.write() {
                                let path = job.save_dir.join(format!(
                                    "chunk_{}_{}_{}.bin",
                                    offset.x, offset.y, offset.z
                                ));
                                let loaded =
                                    path.exists() && chunk.load_from(&path, offset.into()).is_ok();
                                if !loaded {
                                    chunk.update_blocks(
                                        offset.into(),
                                        &noise_for_worker,
                                        &PRAIRIE_PARAMS,
                                        job.land_level,
                                    );
                                }
                                chunk.update_mesh(PRAIRIE_PARAMS, None);
                                chunk.dirty = false;
                            }
                        }
                        JobKind::Remesh => {
                            if let Ok(mut chunk) = job.chunk.write() {
                                let y_range = chunk.dirty_y_range();
                                chunk.update_mesh(PRAIRIE_PARAMS, y_range);
                                chunk.dirty = false;
                            }
                        }
                    }
                    let _ = ready_tx.send(job.chunk_index);
                };

                loop {
                    crossbeam_channel::select! {
                        recv(remesh_job_rx) -> msg => match msg {
                            Ok(job) => process_job(job),
                            Err(_) => break,
                        },
                        recv(gen_job_rx) -> msg => match msg {
                            Ok(job) => process_job(job),
                            Err(_) => break,
                        },
                    }
                }
            });
        }

        thread::spawn(move || {
            while let Ok((path, materials)) = save_rx.recv() {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(path, materials);
            }
        });

        let mut world = Self {
            pipeline: world_pipeline,
            highlight_pipeline,
            atlas,
            chunks,
            chunk_models,
            chunks_view_size,
            center_offset,
            chunks_origin,
            chunk_indices: Arc::new(RwLock::new(chunk_indices)),
            free_chunk_indices: Arc::new(RwLock::new(free_chunk_indices)),
            gen_job_tx,
            remesh_job_tx,
            ready_rx,
            pending_jobs: HashSet::new(),
            save_dir,
            dirty_queue: VecDeque::new(),
            dirty_set: HashSet::new(),
            save_tx,
            highlight_model: None,
            highlight_pos: None,
            max_jobs_in_flight: tuning.jobs_in_flight,
            max_dirty_per_frame: tuning.dirty_chunks_per_frame,
            min_vertex_cap: tuning.min_vertex_cap,
            min_index_cap: tuning.min_index_cap,
            land_level: tuning.land_level,
        };

        println!("about to load first chunks");
        world.load_empty_chunks(center_offset);

        world
    }

    pub fn update_highlight_model(&mut self, device: &wgpu::Device, block: Option<Vector3<i32>>) {
        if self.highlight_pos == block {
            return;
        }
        self.highlight_pos = block;
        if let Some(pos) = block {
            let inflate = 0.01;
            let base = Vector3::new(
                pos.x as f32 - inflate,
                pos.y as f32 - inflate,
                pos.z as f32 - inflate,
            );
            let max = Vector3::new(
                pos.x as f32 + 1.0 + inflate,
                pos.y as f32 + 1.0 + inflate,
                pos.z as f32 + 1.0 + inflate,
            );
            let verts = [
                OutlineVertex {
                    pos: [base.x, base.y, base.z],
                }, // 0
                OutlineVertex {
                    pos: [max.x, base.y, base.z],
                }, // 1
                OutlineVertex {
                    pos: [max.x, max.y, base.z],
                }, // 2
                OutlineVertex {
                    pos: [base.x, max.y, base.z],
                }, // 3
                OutlineVertex {
                    pos: [base.x, base.y, max.z],
                }, // 4
                OutlineVertex {
                    pos: [max.x, base.y, max.z],
                }, // 5
                OutlineVertex {
                    pos: [max.x, max.y, max.z],
                }, // 6
                OutlineVertex {
                    pos: [base.x, max.y, max.z],
                }, // 7
            ];
            let mesh = Mesh {
                verts: verts.into(),
                indices: vec![
                    0, 1, 1, 2, 2, 3, 3, 0, // front square
                    4, 5, 5, 6, 6, 7, 7, 4, // back square
                    0, 4, 1, 5, 2, 6, 3, 7, // vertical edges
                ],
            };
            self.highlight_model = Model::new(device, &mesh);
        } else {
            self.highlight_model = None;
        }
    }

    // вызывается каждый кадр
    pub fn update(&mut self, device: &wgpu::Device, queue: &Queue, player_position: &Point3<f32>) {
        #[cfg(feature = "tracy")]
        let _span = span!("update_world"); // <- Отметка начала блока

        let new_center_offset = Self::world_pos_to_chunk_offset(player_position.to_vec());
        let new_chunk_origin = new_center_offset
            - Vector3::new(
                self.chunks_view_size as i32 / 2,
                0,
                self.chunks_view_size as i32 / 2,
            );

        let moved_to_new_chunk = new_chunk_origin != self.chunks_origin;
        if moved_to_new_chunk {
            let old_origin = self.chunks_origin;
            self.center_offset = new_center_offset;
            self.chunks_origin = new_chunk_origin;
            let chunk_indices_copy = self.chunk_indices.read().unwrap().clone();
            let mut new_indices = vec![None; self.chunk_indices.read().unwrap().len()];

            for i in 0..chunk_indices_copy.len() {
                if let Some(chunk_index) = chunk_indices_copy[i] {
                    let chunk_offset = self.get_chunk_offset_from_origin(old_origin, i);
                    if self.chunk_in_bounds(chunk_offset.into()) {
                        let new_chunk_world_index = self.get_chunk_world_index(chunk_offset.into());
                        new_indices[new_chunk_world_index] = Some(chunk_index);
                    } else {
                        if !self.pending_jobs.contains(&chunk_index) {
                            self.shrink_and_free_chunk(device, chunk_index);
                        }
                    }
                }
            }

            *self.chunk_indices.write().unwrap() = new_indices;
        }

        self.process_ready_chunks(device, queue);
        self.process_dirty_chunks(device, queue);

        if moved_to_new_chunk || self.has_missing_chunks() {
            self.load_empty_chunks(new_center_offset);
        }
    }

    fn process_ready_chunks(&mut self, device: &wgpu::Device, queue: &Queue) {
        while let Ok(chunk_index) = self.ready_rx.try_recv() {
            self.pending_jobs.remove(&chunk_index);
            let mapped = self
                .chunk_indices
                .read()
                .unwrap()
                .iter()
                .any(|entry| entry.map_or(false, |idx| idx == chunk_index));

            if let Some(chunk_model) = self.chunk_models.get(chunk_index) {
                let (
                    _layout_changed,
                    rebuilt_layers,
                    _spans,
                    mesh,
                    offset,
                    layer_data,
                    total_indices,
                ) = if let Some(chunk_arc) = self.chunks.get_chunk(chunk_index) {
                    let mut chunk = chunk_arc.write().unwrap();
                    let layout_changed = chunk.layout_changed();
                    let rebuilt_layers = chunk.take_rebuilt_layers();
                    let spans = chunk.layer_spans().to_vec();
                    let offset = chunk.offset;
                    let mesh = if layout_changed {
                        Some(chunk.mesh.clone())
                    } else {
                        None
                    };
                    let total_indices: u32 = spans.iter().map(|s| s.i_len).sum();

                    let mut layer_data = Vec::new();
                    if !layout_changed {
                        for &ly in &rebuilt_layers {
                            if let Some(span) = spans.get(ly).copied() {
                                if let Some((verts, inds)) = chunk.layer_mesh(ly) {
                                    layer_data.push((span, verts.to_vec(), inds.to_vec()));
                                }
                            }
                        }
                    }

                    (
                        layout_changed,
                        rebuilt_layers,
                        spans,
                        mesh,
                        offset,
                        layer_data,
                        total_indices,
                    )
                } else {
                    (
                        false,
                        Vec::new(),
                        Vec::new(),
                        None,
                        [0, 0, 0],
                        Vec::new(),
                        0,
                    )
                };

                if let Some(mesh) = mesh {
                    let mut chunk_model = chunk_model.write().unwrap();
                    chunk_model.update(device, queue, &mesh);
                    chunk_model.num_indices = mesh.indices().len() as u32;
                    self.chunks.update_chunk_offset(chunk_index, offset);
                } else {
                    // Частичное обновление: перезаписываем изменённые слои.
                    if !rebuilt_layers.is_empty() {
                        if let Some(vb) = self.chunk_models.get(chunk_index) {
                            let mut model = vb.write().unwrap();
                            model.num_indices = total_indices;
                            let v_buffer = model.vbuf();
                            let i_buffer = model.ibuf();
                            for (span, verts, inds) in layer_data {
                                let v_offset =
                                    span.v_start as u64 * std::mem::size_of::<BlockVertex>() as u64;
                                let i_offset =
                                    span.i_start as u64 * std::mem::size_of::<u32>() as u64;
                                queue.write_buffer(v_buffer, v_offset, cast_slice(&verts));
                                let global_inds: Vec<u32> =
                                    inds.iter().map(|i| i + span.v_start).collect();
                                queue.write_buffer(i_buffer, i_offset, cast_slice(&global_inds));
                            }
                        }
                        self.chunks.update_chunk_offset(chunk_index, offset);
                    } else {
                        // Ничего не обновлялось, но на всякий случай фиксим offset.
                        self.chunks.update_chunk_offset(chunk_index, offset);
                    }
                }
            }

            if !mapped {
                self.shrink_and_free_chunk(device, chunk_index);
            }
        }
    }

    fn process_dirty_chunks(&mut self, _device: &wgpu::Device, _queue: &Queue) {
        let mut scheduled = 0;
        let queue_len = self.dirty_queue.len();
        for _ in 0..queue_len {
            if scheduled >= self.max_dirty_per_frame {
                break;
            }
            let Some(idx) = self.dirty_queue.pop_front() else {
                break;
            };

            if self.pending_jobs.contains(&idx) {
                // Уже в работе — оставим на очереди, но не зациклливаемся в этом кадре.
                self.dirty_queue.push_back(idx);
                continue;
            }

            let Some(chunk_arc) = self.chunks.get_chunk(idx) else {
                self.dirty_set.remove(&idx);
                continue;
            };
            {
                let chunk = chunk_arc.read().unwrap();
                if !chunk.dirty {
                    self.dirty_set.remove(&idx);
                    continue;
                }
            }

            if self.pending_jobs.len() >= self.max_jobs_in_flight {
                // Нет свободного слота — вернём задачу в очередь.
                self.dirty_queue.push_front(idx);
                break;
            }

            let job = ChunkJob {
                chunk_index: idx,
                chunk: chunk_arc,
                kind: JobKind::Remesh,
                save_dir: self.save_dir.clone(),
                land_level: self.land_level,
            };

            self.pending_jobs.insert(idx);
            self.dirty_set.remove(&idx);
            let _ = self.remesh_job_tx.send(job);
            scheduled += 1;
        }
    }

    pub fn mark_chunks_dirty(&mut self, indices: &[usize]) {
        for &idx in indices {
            if self.dirty_set.insert(idx) {
                self.dirty_queue.push_back(idx);
            }
        }
    }

    /// Синхронный быстрый ремеш сразу после действия игрока — используем
    /// узкий y-диапазон, который накоплен в dirty_y_range.
    pub fn remesh_chunks_now(&mut self, device: &wgpu::Device, queue: &Queue, indices: &[usize]) {
        for &idx in indices {
            if let Some(chunk_arc) = self.chunks.get_chunk(idx) {
                // Если уже в фоне — не трогаем, иначе можем обновить сразу.
                if self.pending_jobs.contains(&idx) {
                    continue;
                }

                self.remove_from_dirty(idx);

                let mut chunk = chunk_arc.write().unwrap();
                if !chunk.dirty {
                    continue;
                }
                let y_range = chunk.dirty_y_range();
                chunk.update_mesh(PRAIRIE_PARAMS, y_range);
                chunk.dirty = false;
                let mesh = chunk.mesh.clone();
                drop(chunk);

                if let Some(chunk_model) = self.chunk_models.get(idx) {
                    let mut model = chunk_model.write().unwrap();
                    model.update(device, queue, &mesh);
                }
            }
        }
    }

    fn remove_from_dirty(&mut self, idx: usize) {
        self.dirty_set.remove(&idx);
        self.dirty_queue.retain(|&val| val != idx);
    }

    pub fn load_empty_chunks(&mut self, player_chunk: Vector3<i32>) {
        #[cfg(feature = "tracy")]
        let _span = span!("load empty chunks"); // <- Отметка начала блока

        let mut missing: Vec<(usize, Vector3<i32>)> = self
            .chunk_indices
            .read()
            .unwrap()
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .map(|(i, _)| (i, self.get_chunk_offset(i)))
            .collect();

        // Сортируем по расстоянию до чанка игрока.
        missing.sort_by_key(|(_, offset)| {
            let d = *offset - player_chunk;
            d.x * d.x + d.z * d.z
        });

        for (world_index, chunk_offset) in missing.into_iter() {
            if self.pending_jobs.len() >= self.max_jobs_in_flight {
                break;
            }
            if let Some(new_index) = self.free_chunk_indices.write().unwrap().pop_front() {
                self.chunk_indices.write().unwrap()[world_index] = Some(new_index);
                self.pending_jobs.insert(new_index);
                if let Some(chunk_arc) = self.chunks.get_chunk(new_index) {
                    let job = ChunkJob {
                        chunk_index: new_index,
                        chunk: chunk_arc,
                        kind: JobKind::Generate {
                            offset: chunk_offset,
                        },
                        save_dir: self.save_dir.clone(),
                        land_level: self.land_level,
                    };
                    let _ = self.gen_job_tx.send(job);
                }
            } else {
                break;
            }
        }
    }

    fn has_missing_chunks(&self) -> bool {
        self.chunk_indices
            .read()
            .unwrap()
            .iter()
            .any(|entry| entry.is_none())
    }

    fn shrink_and_free_chunk(&mut self, device: &wgpu::Device, chunk_index: usize) {
        if let Some(chunk_arc) = self.chunks.get_chunk(chunk_index) {
            if let Ok(mut chunk) = chunk_arc.write() {
                if chunk.needs_save {
                    let path = self.save_dir.join(format!(
                        "chunk_{}_{}_{}.bin",
                        chunk.offset[0], chunk.offset[1], chunk.offset[2]
                    ));
                    let materials: Vec<u8> = chunk
                        .blocks
                        .iter()
                        .map(|b| match b.material_type {
                            MaterialType::DIRT => 0,
                            MaterialType::GRASS => 1,
                            MaterialType::ROCK => 2,
                            MaterialType::WATER => 3,
                            MaterialType::AIR => 4,
                            MaterialType::DEBUG => 5,
                        })
                        .collect();
                    let _ = self.save_tx.send((path, materials));
                    chunk.needs_save = false;
                }
            }
        }

        if let Some(chunk_model) = self.chunk_models.get(chunk_index) {
            if let Ok(mut model) = chunk_model.write() {
                model.shrink_to(device, self.min_vertex_cap, self.min_index_cap);
            }
        }
        self.chunks.remove_chunk_from_map(chunk_index);
        self.free_chunk_indices
            .write()
            .unwrap()
            .push_back(chunk_index);
    }

    pub fn world_pos_in_bounds(&self, world_pos: Vector3<f32>) -> bool {
        let chunk_offset = Self::world_pos_to_chunk_offset(world_pos);
        self.chunk_in_bounds(chunk_offset)
    }

    pub fn loaded_chunks(&self) -> usize {
        self.chunk_indices
            .read()
            .unwrap()
            .iter()
            .filter(|entry| entry.is_some())
            .count()
    }

    // индекс массива мировых чанков -> смещение чанка
    fn get_chunk_offset(&self, i: usize) -> Vector3<i32> {
        return self.chunks_origin
            + Vector3::new(
                i as i32 % self.chunks_view_size as i32,
                0,
                i as i32 / self.chunks_view_size as i32,
            );
    }

    fn get_chunk_offset_from_origin(&self, origin: Vector3<i32>, i: usize) -> Vector3<i32> {
        origin
            + Vector3::new(
                i as i32 % self.chunks_view_size as i32,
                0,
                i as i32 / self.chunks_view_size as i32,
            )
    }

    fn chunk_in_bounds(&self, chunk_offset: Vector3<i32>) -> bool {
        let p = chunk_offset - self.chunks_origin;
        if p.x >= 0
            && p.z >= 0
            && p.x < self.chunks_view_size as i32
            && p.z < self.chunks_view_size as i32
        {
            return true;
        }
        return false;
    }

    fn world_pos_to_chunk_offset(world_pos: Vector3<f32>) -> Vector3<i32> {
        Vector3::new(
            (world_pos.x / CHUNK_AREA as f32).floor() as i32,
            0,
            (world_pos.z / CHUNK_AREA as f32).floor() as i32,
        )
    }

    fn get_chunk_world_index(&self, chunk_offset: Vector3<i32>) -> usize {
        let p = chunk_offset - self.chunks_origin;
        (p.z as usize * self.chunks_view_size) + p.x as usize
    }
}

impl Draw for TerrainGen {
    fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        globals: &'a wgpu::BindGroup,
    ) -> Result<(), wgpu::Error> {
        #[cfg(feature = "tracy")]
        let _span = span!("drawing world"); // <- Отметка начала блока

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.atlas.bind_group, &[]);
        render_pass.set_bind_group(1, globals, &[]);

        // Рисуем только те модели, что реально привязаны к видимым слотам.
        let chunk_indices = self.chunk_indices.read().unwrap();
        for idx_opt in chunk_indices.iter().copied().flatten() {
            if let Some(chunk_model) = self.chunk_models.get(idx_opt) {
                let chunk_model = chunk_model.read().unwrap();

                let vertex_buffer = chunk_model.vbuf().slice(..);
                let index_buffer = chunk_model.ibuf().slice(..);
                let num_indices = chunk_model.num_indices;

                render_pass.set_vertex_buffer(0, vertex_buffer);
                render_pass.set_index_buffer(index_buffer, wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..num_indices, 0, 0..1 as _);
            }
        }

        if let Some(model) = &self.highlight_model {
            render_pass.set_pipeline(&self.highlight_pipeline);
            render_pass.set_bind_group(0, globals, &[]);
            render_pass.set_vertex_buffer(0, model.vbuf().slice(..));
            render_pass.set_index_buffer(model.ibuf().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..model.num_indices, 0, 0..1);
        }

        Ok(())
    }
}

fn create_highlight_pipeline(
    device: &wgpu::Device,
    global_layout: &GlobalsLayouts,
    shader: &wgpu::ShaderModule,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Highlight Pipeline Layout"),
        bind_group_layouts: &[&global_layout.globals],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Highlight Pipeline"),
        layout: Some(&pipeline_layout),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[OutlineVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: crate::render::texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

fn create_highlight_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    let source = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    fog: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VSIn {
    @location(0) pos: vec3<f32>,
};

struct VSOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(input: VSIn) -> VSOut {
    var out: VSOut;
    out.pos = globals.view_proj * vec4<f32>(input.pos, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.6, 0.0, 1.0);
}
"#;
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Highlight Shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

```

## src\terrain_gen\mod.rs
`rust
pub mod biomes;
pub mod block;
pub mod chunk;
pub mod generator;
pub mod noise;

```

## src\terrain_gen\noise.rs
`rust
use noise::{NoiseFn, Perlin};

#[derive(Clone)]
pub struct NoiseGenerator {
    perlin: Perlin,
}

impl NoiseGenerator {
    pub fn new(seed: u32) -> Self {
        let perlin = Perlin::new(seed);
        Self { perlin }
    }

    pub fn get_height(&self, x: f32, z: f32, frequency: f32, amplitude: f32) -> f32 {
        self.perlin
            .get([x as f64 * frequency as f64, z as f64 * frequency as f64]) as f32
            * amplitude
    }
}

```

## src\text\atlas.rs
`rust
use anyhow::Result;
use wgpu::Queue;

#[derive(Clone, Debug)]
pub struct GlyphAtlasEntry {
    pub page: usize,
    pub uv: [f32; 4], // u0, v0, u1, v1
    pub size: [u32; 2],
    pub bearing: [i32; 2],
    pub advance: f32,
}

pub struct AtlasPage {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    width: u32,
    height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_h: u32,
}

pub struct GlyphAtlas {
    device: wgpu::Device,
    pub pages: Vec<AtlasPage>,
    pub format: wgpu::TextureFormat,
    pub size: u32,
}

impl GlyphAtlas {
    pub fn new(device: &wgpu::Device, size: u32) -> Self {
        let mut atlas = Self {
            device: device.clone(),
            pages: Vec::new(),
            format: wgpu::TextureFormat::R8Unorm,
            size,
        };
        atlas.new_page();
        atlas
    }

    fn new_page(&mut self) {
        let size = self.size;
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph_atlas_page"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("glyph_atlas_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.pages.push(AtlasPage {
            texture,
            view,
            sampler,
            width: size,
            height: size,
            cursor_x: 1,
            cursor_y: 1,
            row_h: 0,
        });
    }

    pub fn page_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glyph_atlas_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    pub fn ensure_space(&mut self, w: u32, h: u32) {
        let size = self.size;
        let page = self.pages.last_mut().unwrap();
        if w + 2 > size || h + 2 > size {
            // glyph too large; fallback new page with larger? For now panic
            panic!("glyph too large for atlas");
        }
        if page.cursor_x + w + 1 > size {
            page.cursor_x = 1;
            page.cursor_y += page.row_h + 1;
            page.row_h = 0;
        }
        if page.cursor_y + h + 1 > size {
            self.new_page();
        }
    }

    pub fn add_glyph(
        &mut self,
        queue: &Queue,
        bitmap: &[u8],
        width: u32,
        height: u32,
        bearing: [i32; 2],
        advance: f32,
    ) -> Result<GlyphAtlasEntry> {
        self.ensure_space(width, height);
        let page_index = self.pages.len() - 1;
        let page = self.pages.last_mut().unwrap();
        if page.cursor_x + width + 1 > page.width {
            page.cursor_x = 1;
            page.cursor_y += page.row_h + 1;
            page.row_h = 0;
        }
        if page.cursor_y + height + 1 > page.height {
            drop(page);
            self.new_page();
            return self.add_glyph(queue, bitmap, width, height, bearing, advance);
        }
        let x = page.cursor_x;
        let y = page.cursor_y;
        page.row_h = page.row_h.max(height + 1);
        page.cursor_x += width + 1;

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &page.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
            },
            bitmap,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let u0 = x as f32 / page.width as f32;
        let v0 = y as f32 / page.height as f32;
        let u1 = (x + width) as f32 / page.width as f32;
        let v1 = (y + height) as f32 / page.height as f32;

        Ok(GlyphAtlasEntry {
            page: page_index,
            uv: [u0, v0, u1, v1],
            size: [width, height],
            bearing,
            advance,
        })
    }
}

```

## src\text\cache.rs
`rust
use std::collections::HashMap;

use anyhow::Result;

use crate::text::{atlas::GlyphAtlasEntry, font::{GlyphBitmap, GlyphKey}};
use wgpu::Queue;

pub struct CachedGlyph {
    pub atlas: GlyphAtlasEntry,
}

pub struct GlyphCache {
    entries: HashMap<GlyphKey, CachedGlyph>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &GlyphKey) -> Option<&CachedGlyph> {
        self.entries.get(key)
    }

    pub fn get_or_insert<F: FnOnce() -> Result<Option<GlyphBitmap>>>(
        &mut self,
        queue: &Queue,
        atlas: &mut crate::text::atlas::GlyphAtlas,
        key: GlyphKey,
        rasterize: F,
    ) -> Result<Option<&CachedGlyph>> {
        if self.entries.contains_key(&key) {
            return Ok(self.entries.get(&key));
        }
        if let Some(bitmap) = rasterize()? {
            let entry = atlas.add_glyph(
                queue,
                &bitmap.buffer,
                bitmap.width,
                bitmap.height,
                [bitmap.bearing_x, bitmap.bearing_y],
                bitmap.advance,
            )?;
            self.entries.insert(key, CachedGlyph { atlas: entry });
            Ok(self.entries.get(&key))
        } else {
            Ok(None)
        }
    }
}

```

## src\text\font.rs
`rust
use anyhow::Result;
use freetype::{face::LoadFlag, Face, Library};
use std::{path::Path, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontHandle(pub usize);

pub struct Font {
    pub id: FontHandle,
    pub face: Face,
}

pub struct FontManager {
    library: Library,
    fonts: Vec<Arc<Font>>,
}

impl FontManager {
    pub fn new() -> Result<Self> {
        let library = Library::init()?;
        Ok(Self {
            library,
            fonts: Vec::new(),
        })
    }

    pub fn load_font(&mut self, path: impl AsRef<Path>) -> Result<FontHandle> {
        let path_ref = path.as_ref();
        let face = self.library.new_face(path_ref, 0)?;
        let handle = FontHandle(self.fonts.len());
        let font = Arc::new(Font { id: handle, face });
        self.fonts.push(font);
        Ok(handle)
    }

    pub fn get(&self, handle: FontHandle) -> Arc<Font> {
        self.fonts[handle.0].clone()
    }
}

#[derive(Clone, Debug)]
pub struct GlyphBitmap {
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: f32,
    pub buffer: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderMode {
    Normal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: FontHandle,
    pub glyph_id: u32,
    pub pixel_size: u32,
    pub render_mode: RenderMode,
}

impl Font {
    pub fn set_pixel_size(&self, px: u32) -> Result<()> {
        self.face.set_pixel_sizes(0, px)?;
        Ok(())
    }

    pub fn load_glyph_bitmap(&self, glyph_id: u32, pixel_size: u32) -> Result<Option<GlyphBitmap>> {
        self.face.set_pixel_sizes(0, pixel_size)?;
        let glyph_index = glyph_id;
        self.face.load_glyph(
            glyph_index as u32,
            LoadFlag::RENDER | LoadFlag::TARGET_NORMAL,
        )?;
        let glyph_slot = self.face.glyph();
        let bitmap = glyph_slot.bitmap();
        let width = bitmap.width() as u32;
        let height = bitmap.rows() as u32;
        if width == 0 || height == 0 {
            return Ok(None);
        }
        let buffer = bitmap.buffer().to_vec();
        let advance = (glyph_slot.advance().x >> 6) as f32;
        Ok(Some(GlyphBitmap {
            width,
            height,
            bearing_x: glyph_slot.bitmap_left(),
            bearing_y: glyph_slot.bitmap_top(),
            advance,
            buffer,
        }))
    }

    pub fn glyph_index_for_char(&self, ch: char) -> u32 {
        self.face.get_char_index(ch as usize)
    }

    pub fn ascent(&self, pixel_size: u32) -> Result<f32> {
        self.face.set_pixel_sizes(0, pixel_size)?;
        Ok(self
            .face
            .size_metrics()
            .map(|m| (m.ascender >> 6) as f32)
            .unwrap_or(0.0))
    }

    pub fn descent(&self, pixel_size: u32) -> Result<f32> {
        self.face.set_pixel_sizes(0, pixel_size)?;
        Ok(self
            .face
            .size_metrics()
            .map(|m| (m.descender >> 6) as f32)
            .unwrap_or(0.0))
    }

    pub fn line_gap(&self, pixel_size: u32) -> Result<f32> {
        self.face.set_pixel_sizes(0, pixel_size)?;
        Ok(self
            .face
            .size_metrics()
            .map(|m| {
                (m.height >> 6) as f32
                    - (self
                        .face
                        .size_metrics()
                        .map(|m| (m.ascender - m.descender) >> 6)
                        .unwrap_or(0) as f32)
            })
            .unwrap_or(0.0))
    }
}

```

## src\text\layout.rs
`rust
use glam::Vec2;

use crate::text::font::{Font, FontHandle, GlyphKey, RenderMode};

#[derive(Clone, Debug)]
pub struct PlacedGlyph {
    pub key: GlyphKey,
    pub position: Vec2, // baseline based
}

pub fn layout_line(text: &str, font: &Font, px: u32) -> Vec<PlacedGlyph> {
    let mut out = Vec::new();
    let mut cursor_x = 0.0f32;
    for ch in text.chars() {
        let glyph_id = font.glyph_index_for_char(ch);
        let key = GlyphKey {
            font_id: font.id,
            glyph_id,
            pixel_size: px,
            render_mode: RenderMode::Normal,
        };
        out.push(PlacedGlyph {
            key,
            position: Vec2::new(cursor_x, 0.0),
        });
        if let Ok(Some(glyph)) = font.load_glyph_bitmap(glyph_id, px) {
            cursor_x += glyph.advance;
        } else {
            cursor_x += px as f32 * 0.5;
        }
    }
    out
}

```

## src\text\mod.rs
`rust
pub mod atlas;
pub mod cache;
pub mod font;
pub mod layout;
pub mod renderer;

pub use font::{FontHandle, FontManager};
pub use renderer::{TextMode, TextObject, TextStyle, TextSystem};

```

## src\text\renderer.rs
`rust
use anyhow::Result;
use glam::{Vec2, Vec3};
use wgpu::{RenderPass, util::DeviceExt};

use crate::text::{
    atlas::GlyphAtlas,
    cache::GlyphCache,
    font::{FontHandle, FontManager},
    layout::layout_line,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct TextVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug)]
pub enum TextMode {
    Gui,
    World,
}

#[derive(Clone, Copy, Debug)]
pub struct TextStyle {
    pub color: [f32; 4],
    pub pixel_size: u32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            pixel_size: 20,
        }
    }
}

pub struct TextPipelines {
    pub gui_pipeline: wgpu::RenderPipeline,
    pub world_pipeline: wgpu::RenderPipeline,
    pub atlas_layout: wgpu::BindGroupLayout,
}

pub struct TextSystem {
    pub fonts: FontManager,
    cache: GlyphCache,
    atlas: GlyphAtlas,
    pipelines: TextPipelines,
    page_bind_groups: Vec<wgpu::BindGroup>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

pub struct TextObject {
    pub vertices_by_page: Vec<Vec<TextVertex>>,
    pub mode: TextMode,
}

impl TextSystem {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config_format: wgpu::TextureFormat,
        globals_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self> {
        let atlas = GlyphAtlas::new(device, 2048);
        let atlas_layout = GlyphAtlas::page_bind_group_layout(device);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../assets/shaders/text.wgsl").into()),
        });

        let pipeline_layout_gui = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text_pipeline_gui"),
            bind_group_layouts: &[&atlas_layout],
            push_constant_ranges: &[],
        });
        let gui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_gui"),
            layout: Some(&pipeline_layout_gui),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_gui"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_gui"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pipeline_layout_world =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("text_pipeline_world"),
                bind_group_layouts: &[&atlas_layout, globals_layout],
                push_constant_ranges: &[],
            });
        let world_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_world"),
            layout: Some(&pipeline_layout_world),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_world"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_world"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::render::texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let fonts = FontManager::new()?;
        let cache = GlyphCache::new();
        Ok(Self {
            fonts,
            cache,
            atlas,
            pipelines: TextPipelines {
                gui_pipeline,
                world_pipeline,
                atlas_layout,
            },
            page_bind_groups: Vec::new(),
            device: device.clone(),
            queue: queue.clone(),
        })
    }

    fn ensure_page_bind_groups(&mut self) {
        while self.page_bind_groups.len() < self.atlas.pages.len() {
            let page = &self.atlas.pages[self.page_bind_groups.len()];
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("glyph_atlas_bg"),
                layout: &self.pipelines.atlas_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&page.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&page.sampler),
                    },
                ],
            });
            self.page_bind_groups.push(bind_group);
        }
    }

    pub fn load_font(&mut self, path: impl AsRef<std::path::Path>) -> Result<FontHandle> {
        self.fonts.load_font(path)
    }

    pub fn measure_text(
        &mut self,
        text: &str,
        font_id: FontHandle,
        pixel_size: u32,
    ) -> Result<(f32, f32)> {
        let font = self.fonts.get(font_id);
        let mut width = 0.0;
        let mut max_h = 0.0;
        for ch in text.chars() {
            let glyph_id = font.glyph_index_for_char(ch);
            if let Some(g) = font.load_glyph_bitmap(glyph_id, pixel_size)? {
                width += g.advance;
                max_h = max_h.max(g.height as f32);
            }
        }
        Ok((width, max_h))
    }

    fn build_vertices(
        &mut self,
        text: &str,
        font_id: FontHandle,
        style: TextStyle,
        mode: TextMode,
        origin: Vec3,
        right: Option<Vec3>,
        up: Option<Vec3>,
    ) -> Result<TextObject> {
        let font = self.fonts.get(font_id);
        let placed = layout_line(text, &font, style.pixel_size);
        let mut vertices_by_page: Vec<Vec<TextVertex>> = vec![Vec::new(); self.atlas.pages.len()];
        for pg in placed.iter() {
            let glyph = self
                .cache
                .get_or_insert(&self.queue, &mut self.atlas, pg.key, || {
                    font.load_glyph_bitmap(pg.key.glyph_id, pg.key.pixel_size)
                })?;
            if glyph.is_none() {
                continue;
            }
            let atlas_entry = {
                let g = glyph.unwrap();
                g.atlas.clone()
            };
            self.ensure_page_bind_groups();
            if atlas_entry.page >= vertices_by_page.len() {
                vertices_by_page.resize_with(atlas_entry.page + 1, Vec::new);
            }

            let uv = atlas_entry.uv;
            let size = atlas_entry.size;
            let bearing = atlas_entry.bearing;

            let x0 = pg.position.x + bearing[0] as f32;
            let y0 = -pg.position.y - bearing[1] as f32;
            let w = size[0] as f32;
            let h = size[1] as f32;

            let quad = match mode {
                TextMode::Gui => {
                    let p0 = origin + Vec3::new(x0, y0, 0.0);
                    let p1 = origin + Vec3::new(x0 + w, y0, 0.0);
                    let p2 = origin + Vec3::new(x0 + w, y0 + h, 0.0);
                    let p3 = origin + Vec3::new(x0, y0 + h, 0.0);
                    [p0, p1, p2, p3]
                }
                TextMode::World => {
                    let right = right.unwrap_or(Vec3::new(1.0, 0.0, 0.0));
                    let up = up.unwrap_or(Vec3::new(0.0, 1.0, 0.0));
                    let p0 = origin + right * x0 + up * (-y0);
                    let p1 = origin + right * (x0 + w) + up * (-y0);
                    let p2 = origin + right * (x0 + w) + up * (-(y0 + h));
                    let p3 = origin + right * x0 + up * (-(y0 + h));
                    [p0, p1, p2, p3]
                }
            };

            let color = style.color;
            let verts = vec![
                TextVertex {
                    position: quad[0].into(),
                    uv: [uv[0], uv[1]],
                    color,
                },
                TextVertex {
                    position: quad[1].into(),
                    uv: [uv[2], uv[1]],
                    color,
                },
                TextVertex {
                    position: quad[2].into(),
                    uv: [uv[2], uv[3]],
                    color,
                },
                TextVertex {
                    position: quad[3].into(),
                    uv: [uv[0], uv[3]],
                    color,
                },
            ];
            let idx = atlas_entry.page;
            if vertices_by_page.len() <= idx {
                vertices_by_page.resize_with(idx + 1, Vec::new);
            }
            vertices_by_page[idx].extend(verts);
        }
        Ok(TextObject {
            vertices_by_page,
            mode,
        })
    }

    pub fn build_gui_text(
        &mut self,
        text: &str,
        font: FontHandle,
        style: TextStyle,
        origin_px: Vec2,
        screen_size: [f32; 2],
    ) -> Result<TextObject> {
        let clip_x = (origin_px.x / screen_size[0]) * 2.0 - 1.0;
        let clip_y = 1.0 - (origin_px.y / screen_size[1]) * 2.0;
        self.build_vertices(
            text,
            font,
            style,
            TextMode::Gui,
            Vec3::new(clip_x, clip_y, 0.0),
            None,
            None,
        )
    }

    pub fn build_world_text(
        &mut self,
        text: &str,
        font: FontHandle,
        style: TextStyle,
        origin: Vec3,
        right: Vec3,
        up: Vec3,
    ) -> Result<TextObject> {
        self.build_vertices(
            text,
            font,
            style,
            TextMode::World,
            origin,
            Some(right),
            Some(up),
        )
    }

    pub fn measure_text(
        &mut self,
        text: &str,
        font_id: FontHandle,
        pixel_size: u32,
    ) -> Result<(f32, f32)> {
        let font = self.fonts.get(font_id);
        let mut width: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        for ch in text.chars() {
            let glyph_id = font.glyph_index_for_char(ch);
            if let Some(g) = font.load_glyph_bitmap(glyph_id, pixel_size)? {
                width += g.advance;
                max_h = max_h.max(g.height as f32);
            }
        }
        Ok((width, max_h))
    }

    pub fn draw(
        &mut self,
        render_pass: &mut RenderPass<'_>,
        globals: Option<&wgpu::BindGroup>,
        obj: &TextObject,
        _screen_size: [f32; 2],
    ) {
        self.ensure_page_bind_groups();
        for (page_idx, verts) in obj.vertices_by_page.iter().enumerate() {
            if verts.is_empty() {
                continue;
            }
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("text_vertices"),
                    contents: bytemuck::cast_slice(verts),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            match obj.mode {
                TextMode::Gui => {
                    render_pass.set_pipeline(&self.pipelines.gui_pipeline);
                    render_pass.set_bind_group(0, &self.page_bind_groups[page_idx], &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.draw(0..verts.len() as u32, 0..1);
                }
                TextMode::World => {
                    if let Some(g) = globals {
                        render_pass.set_pipeline(&self.pipelines.world_pipeline);
                        render_pass.set_bind_group(0, &self.page_bind_groups[page_idx], &[]);
                        render_pass.set_bind_group(1, g, &[]);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.draw(0..verts.len() as u32, 0..1);
                    }
                }
            }
        }
    }
}

```

## src\ui\elements.rs
`rust
use serde::{Deserialize, Serialize};

use crate::ui::MeasureCtx;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabelSpec {
    pub text: String,
    #[serde(default = "LabelSpec::default_font_size")]
    pub font_size: f32,
}

impl LabelSpec {
    fn default_font_size() -> f32 {
        16.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonSpec {
    pub text: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default = "ButtonSpec::default_padding")]
    pub padding: f32,
    #[serde(default = "ButtonSpec::default_height")]
    pub min_height: f32,
}

impl ButtonSpec {
    fn default_padding() -> f32 {
        8.0
    }

    fn default_height() -> f32 {
        48.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UiElement {
    Label(LabelSpec),
    Button(ButtonSpec),
    Panel {
        #[serde(default = "UiElement::default_panel_color")]
        color: [u8; 4],
    },
    Image {
        #[serde(default)]
        uv: [f32; 4],
    },
    Spacer {
        size: f32,
    },
}

impl UiElement {
    fn default_panel_color() -> [u8; 4] {
        [24, 26, 30, 200]
    }

    pub fn preferred_size(&self, ctx: &MeasureCtx) -> [f32; 2] {
        match self {
            UiElement::Label(label) => {
                if let Some(font) = &ctx.font {
                    let (w, h) = font.measure_text(&label.text);
                    let scale = (label.font_size / font.height() as f32) * ctx.text_scale;
                    [w * scale, h * scale]
                } else {
                    let scale = ctx.text_scale;
                    [
                        label.text.len() as f32 * 8.0 * scale,
                        label.font_size * scale,
                    ]
                }
            }
            UiElement::Button(button) => {
                if let Some(font) = &ctx.font {
                    let (w, h) = font.measure_text(&button.text);
                    let detail_w = button
                        .detail
                        .as_ref()
                        .map(|d| font.measure_text(d).0)
                        .unwrap_or(0.0);
                    let text_w = w.max(detail_w);
                    let text_h = if button.detail.is_some() { h * 2.0 } else { h };
                    let scale = ctx.text_scale;
                    [
                        text_w * scale + button.padding * 2.0,
                        (text_h * scale + button.padding * 2.0).max(button.min_height * scale),
                    ]
                } else {
                    let scale = ctx.text_scale;
                    [
                        (button.text.len() as f32) * 10.0 * scale + button.padding * 2.0,
                        button.min_height * scale,
                    ]
                }
            }
            UiElement::Panel { .. } => [0.0, 0.0],
            UiElement::Image { .. } => [0.0, 0.0],
            UiElement::Spacer { size } => [0.0, *size],
        }
    }
}

```

## src\ui\font.rs
`rust
use anyhow::Result;
use freetype::{Face, Library};
use std::{fs, path::Path};

pub struct BitmapFont {
    _lib: Library,
    face: Face,
    pixel_size: u32,
}

impl BitmapFont {
    pub fn load_from_path(
        path: impl AsRef<Path>,
        pixel_size: f32,
        _font_weight_px: u32,
    ) -> Result<Self> {
        let lib = Library::init()?;
        let bytes = fs::read(path)?;
        let face = lib.new_memory_face(bytes, 0)?;
        face.set_pixel_sizes(0, pixel_size as u32)?;
        Ok(Self {
            _lib: lib,
            face,
            pixel_size: pixel_size as u32,
        })
    }

    pub fn measure_text(&self, text: &str) -> (f32, f32) {
        let mut width = 0.0;
        let mut max_h: f32 = 0.0;
        for ch in text.chars() {
            let glyph_index = self.face.get_char_index(ch as usize);
            if self
                .face
                .load_glyph(glyph_index, freetype::face::LoadFlag::RENDER)
                .is_ok()
            {
                let glyph = self.face.glyph();
                width += (glyph.advance().x >> 6) as f32;
                max_h = max_h.max(glyph.bitmap().rows() as f32);
            } else {
                width += self.pixel_size as f32 * 0.5;
            }
        }
        (width, max_h.max(self.pixel_size as f32))
    }

    pub fn advance(&self, ch: char) -> f32 {
        let idx = self.face.get_char_index(ch as usize);
        if self
            .face
            .load_glyph(idx, freetype::face::LoadFlag::DEFAULT)
            .is_ok()
        {
            (self.face.glyph().advance().x >> 6) as f32
        } else {
            self.pixel_size as f32 * 0.5
        }
    }

    pub fn height(&self) -> usize {
        self.pixel_size as usize
    }
}

```

## src\ui\layout.rs
`rust
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ui::elements::UiElement;
use crate::ui::font::BitmapFont;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Val {
    Px(f32),
    Percent(f32),
}

impl Val {
    pub fn resolve(&self, parent: f32) -> f32 {
        match self {
            Val::Px(v) => *v,
            Val::Percent(p) => parent * *p,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RectSpec {
    pub x: Val,
    pub y: Val,
    pub w: Val,
    pub h: Val,
}

impl RectSpec {
    pub fn resolve(&self, parent: [f32; 2]) -> [f32; 4] {
        let pw = parent[0];
        let ph = parent[1];

        let rx = self.x.resolve(pw);
        let ry = self.y.resolve(ph);
        let rw = self.w.resolve(pw);
        let rh = self.h.resolve(ph);

        [rx, ry, rw, rh]
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Anchors {
    pub left: Option<Val>,
    pub right: Option<Val>,
    pub top: Option<Val>,
    pub bottom: Option<Val>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Layout {
    Absolute {
        rect: RectSpec,
        #[serde(default)]
        anchor: Option<Anchors>,
    },
    FlexRow {
        gap: f32,
        padding: f32,
        #[serde(default = "Align::default_align")]
        align: Align,
    },
    FlexColumn {
        gap: f32,
        padding: f32,
        #[serde(default = "Align::default_align")]
        align: Align,
    },
}

impl Align {
    fn default_align() -> Align {
        Align::Start
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiNode {
    pub id: Option<String>,
    pub layout: Layout,
    #[serde(default)]
    pub children: Vec<UiNode>,
    pub element: Option<UiElement>,
}

#[derive(Clone, Debug)]
pub struct ResolvedNode {
    pub id: Option<String>,
    pub rect: [f32; 4],
    pub element: Option<UiElement>,
}

#[derive(Clone)]
pub struct MeasureCtx {
    pub font: Option<Arc<BitmapFont>>,
    pub text_scale: f32,
}

impl Default for MeasureCtx {
    fn default() -> Self {
        Self {
            font: None,
            text_scale: 1.0,
        }
    }
}

impl UiNode {
    pub fn resolve_tree(&self, parent_size: [f32; 2], ctx: &MeasureCtx) -> Vec<ResolvedNode> {
        let mut out = Vec::new();
        self.compute_layout([0.0, 0.0], parent_size, ctx, &mut out);
        out
    }

    pub fn preferred_size(&self, parent_size: [f32; 2], ctx: &MeasureCtx) -> [f32; 2] {
        match &self.layout {
            Layout::Absolute { rect, .. } => {
                let r = rect.resolve(parent_size);
                [r[2], r[3]]
            }
            Layout::FlexColumn { gap, padding, .. } => {
                let inner_parent = [
                    (parent_size[0] - padding * 2.0).max(0.0),
                    (parent_size[1] - padding * 2.0).max(0.0),
                ];
                let mut height: f32 = 0.0;
                let mut width: f32 = 0.0;
                for child in &self.children {
                    let size = child.preferred_size(inner_parent, ctx);
                    height += size[1];
                    width = width.max(size[0]);
                }
                if !self.children.is_empty() {
                    height += *gap * (self.children.len() as f32 - 1.0);
                }
                [width + padding * 2.0, height + padding * 2.0]
            }
            Layout::FlexRow { gap, padding, .. } => {
                let inner_parent = [
                    (parent_size[0] - padding * 2.0).max(0.0),
                    (parent_size[1] - padding * 2.0).max(0.0),
                ];
                let mut width: f32 = 0.0;
                let mut height: f32 = 0.0;
                for child in &self.children {
                    let size = child.preferred_size(inner_parent, ctx);
                    width += size[0];
                    height = height.max(size[1]);
                }
                if !self.children.is_empty() {
                    width += *gap * (self.children.len() as f32 - 1.0);
                }
                [width + padding * 2.0, height + padding * 2.0]
            }
        }
    }

    fn compute_layout(
        &self,
        origin: [f32; 2],
        parent_size: [f32; 2],
        ctx: &MeasureCtx,
        out: &mut Vec<ResolvedNode>,
    ) {
        match &self.layout {
            Layout::Absolute { rect, anchor } => {
                let mut r = rect.resolve(parent_size);
                if let Some(anchor) = anchor {
                    Self::apply_anchor(&mut r, parent_size, anchor);
                }
                let rect_world = [origin[0] + r[0], origin[1] + r[1], r[2], r[3]];
                out.push(ResolvedNode {
                    id: self.id.clone(),
                    rect: rect_world,
                    element: self.element.clone(),
                });
                for child in &self.children {
                    child.compute_layout([rect_world[0], rect_world[1]], [r[2], r[3]], ctx, out);
                }
            }
            Layout::FlexColumn {
                gap,
                padding,
                align,
            } => {
                let inner_w = (parent_size[0] - padding * 2.0).max(0.0);
                if let Some(el) = &self.element {
                    out.push(ResolvedNode {
                        id: self.id.clone(),
                        rect: [origin[0], origin[1], parent_size[0], parent_size[1]],
                        element: Some(el.clone()),
                    });
                }
                let mut cursor_y = *padding;
                for child in &self.children {
                    let child_size = child.preferred_size([inner_w, parent_size[1]], ctx);
                    let width = match align {
                        Align::Stretch => inner_w,
                        Align::Start => child_size[0].min(inner_w),
                        Align::Center => child_size[0].min(inner_w),
                        Align::End => child_size[0].min(inner_w),
                    };
                    let height = child_size[1];
                    let x = match align {
                        Align::Stretch => *padding,
                        Align::Start => *padding,
                        Align::Center => *padding + (inner_w - width) * 0.5,
                        Align::End => *padding + (inner_w - width),
                    };
                    let rect_world = [origin[0] + x, origin[1] + cursor_y, width, height];
                    child.compute_layout([rect_world[0], rect_world[1]], [width, height], ctx, out);
                    cursor_y += height + gap;
                }
            }
            Layout::FlexRow {
                gap,
                padding,
                align,
            } => {
                let inner_h = (parent_size[1] - padding * 2.0).max(0.0);
                if let Some(el) = &self.element {
                    out.push(ResolvedNode {
                        id: self.id.clone(),
                        rect: [origin[0], origin[1], parent_size[0], parent_size[1]],
                        element: Some(el.clone()),
                    });
                }
                let mut cursor_x = *padding;
                for child in &self.children {
                    let child_size = child.preferred_size(parent_size, ctx);
                    let height = match align {
                        Align::Stretch => inner_h,
                        Align::Start => child_size[1].min(inner_h),
                        Align::Center => child_size[1].min(inner_h),
                        Align::End => child_size[1].min(inner_h),
                    };
                    let width = child_size[0];
                    let y = match align {
                        Align::Stretch => *padding,
                        Align::Start => *padding,
                        Align::Center => *padding + (inner_h - height) * 0.5,
                        Align::End => *padding + (inner_h - height),
                    };
                    let rect_world = [origin[0] + cursor_x, origin[1] + y, width, height];
                    child.compute_layout([rect_world[0], rect_world[1]], [width, height], ctx, out);
                    cursor_x += width + gap;
                }
            }
        }
    }

    fn apply_anchor(rect: &mut [f32; 4], parent: [f32; 2], anchor: &Anchors) {
        if let (Some(left), Some(right)) = (anchor.left, anchor.right) {
            let l = left.resolve(parent[0]);
            let r = right.resolve(parent[0]);
            rect[0] = l;
            rect[2] = (parent[0] - l - r).max(0.0);
        } else if let Some(left) = anchor.left {
            rect[0] = left.resolve(parent[0]);
        } else if let Some(right) = anchor.right {
            rect[0] = parent[0] - right.resolve(parent[0]) - rect[2];
        }

        if let (Some(top), Some(bottom)) = (anchor.top, anchor.bottom) {
            let t = top.resolve(parent[1]);
            let b = bottom.resolve(parent[1]);
            rect[1] = t;
            rect[3] = (parent[1] - t - b).max(0.0);
        } else if let Some(top) = anchor.top {
            rect[1] = top.resolve(parent[1]);
        } else if let Some(bottom) = anchor.bottom {
            rect[1] = parent[1] - bottom.resolve(parent[1]) - rect[3];
        }
    }
}

```

## src\ui\loader.rs
`rust
use std::path::Path;

use crate::ui::layout::UiNode;

pub fn load_ron(path: impl AsRef<Path>) -> anyhow::Result<UiNode> {
    let raw = std::fs::read_to_string(path)?;
    let node: UiNode = ron::from_str(&raw)?;
    Ok(node)
}

pub fn load_json(path: impl AsRef<Path>) -> anyhow::Result<UiNode> {
    let raw = std::fs::read_to_string(path)?;
    let node: UiNode = serde_json::from_str(&raw)?;
    Ok(node)
}

```

## src\ui\mod.rs
`rust
pub mod elements;
pub mod font;
pub mod layout;
pub mod loader;
pub mod renderer;

pub use elements::{ButtonSpec, LabelSpec, UiElement};
pub use font::BitmapFont;
pub use layout::{Align, Anchors, Layout, MeasureCtx, RectSpec, ResolvedNode, UiNode, Val};
pub use loader::{load_json, load_ron};
pub use renderer::{MeshBuilder, quad_from_rect};

```

## src\ui\renderer.rs
`rust
use crate::render::{mesh::Mesh, pipelines::hud::HUDVertex};

pub struct MeshBuilder {
    verts: Vec<HUDVertex>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            verts: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn push_quad(&mut self, rect: [f32; 4], screen: [f32; 2], uv: [f32; 4]) {
        let [x, y, w, h] = rect;
        let x0 = x;
        let y0 = y;
        let x1 = x + w;
        let y1 = y + h;

        let tl = to_clip(x0, y0, screen);
        let tr = to_clip(x1, y0, screen);
        let br = to_clip(x1, y1, screen);
        let bl = to_clip(x0, y1, screen);

        let base = self.verts.len() as u32;

        self.verts.extend_from_slice(&[
            HUDVertex {
                position: tl,
                uv: [uv[0], uv[1]],
            },
            HUDVertex {
                position: tr,
                uv: [uv[2], uv[1]],
            },
            HUDVertex {
                position: br,
                uv: [uv[2], uv[3]],
            },
            HUDVertex {
                position: bl,
                uv: [uv[0], uv[3]],
            },
        ]);

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn build(self) -> Mesh<HUDVertex> {
        Mesh {
            verts: self.verts,
            indices: self.indices,
        }
    }
}

pub fn quad_from_rect(rect: [f32; 4], screen: [f32; 2]) -> [HUDVertex; 4] {
    let [x, y, w, h] = rect;
    let x0 = x;
    let y0 = y;
    let x1 = x + w;
    let y1 = y + h;

    let tl = to_clip(x0, y0, screen);
    let tr = to_clip(x1, y0, screen);
    let br = to_clip(x1, y1, screen);
    let bl = to_clip(x0, y1, screen);

    [
        HUDVertex {
            position: tl,
            uv: [0.0, 0.0],
        },
        HUDVertex {
            position: tr,
            uv: [1.0, 0.0],
        },
        HUDVertex {
            position: br,
            uv: [1.0, 1.0],
        },
        HUDVertex {
            position: bl,
            uv: [0.0, 1.0],
        },
    ]
}

fn to_clip(x: f32, y: f32, screen: [f32; 2]) -> [f32; 2] {
    let nx = (x / screen[0]) * 2.0 - 1.0;
    let ny = 1.0 - (y / screen[1]) * 2.0;
    [nx, ny]
}

```

## src\uitls\mod.rs
`rust
```

