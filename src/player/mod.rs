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
