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
