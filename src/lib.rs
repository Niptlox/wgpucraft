pub mod core;
pub mod ecs;
pub mod hud;
pub mod launcher;
pub mod player;
pub mod render;
pub mod terrain_gen;

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

#[derive(PartialEq)]
pub enum GameState {
    PLAYING,
    PAUSED,
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
}

impl<'a> State<'a> {
    pub fn new(window: &'a Window, config: AppConfig) -> Self {
        let frame_target = config.target_frame_time();
        let mut renderer = Renderer::new(&window, config.present_mode());

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

        let terrain = TerrainGen::new(
            &renderer,
            config.graphics.render_distance_chunks,
            config.world.seed,
            &config.world.world_name,
        );

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

                // Обработка событий мыши
                WindowEvent::MouseInput { state, button, .. } => {
                    if self.state == GameState::PAUSED && state == ElementState::Pressed {
                        self.enter_play_mode();
                        return;
                    }

                    match (button, state) {
                        // ЛКМ — ломаем блок (ставим воздух)
                        (MouseButton::Left, ElementState::Pressed) => {
                            let ray =
                                Ray::from_camera(&self.player.camera, self.player.max_interact_range());
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
                            let ray =
                                Ray::from_camera(&self.player.camera, self.player.max_interact_range());
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
                            let ray =
                                Ray::from_camera(&self.player.camera, self.player.max_interact_range());
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
                    GameState::PAUSED => self.enter_play_mode(),
                    GameState::PLAYING => self.enter_pause_mode(),
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
        self.player.update(elapsed, &self.terrain.chunks);
        self.terrain.update(
            &self.renderer.device,
            &self.renderer.queue,
            &self.player.camera.position,
        );

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
    }

    fn enter_pause_mode(&mut self) {
        let _ = self.window.set_cursor_grab(CursorGrabMode::None);
        self.window.set_cursor_visible(true);
        self.state = GameState::PAUSED;
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
        let max_view_distance = (self.config.graphics.render_distance_chunks.max(1) * CHUNK_AREA)
            as f32;
        // Приближаем туман: начало примерно с 35% дальности, полная плотность к ~65%.
        let fog_start = max_view_distance * 0.35;
        let fog_end = max_view_distance * 0.65;

        self.renderer
            .update_consts(
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
}
