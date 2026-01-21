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
                Direction::TOP => atlas_pos_to_coordinates([0.0, 0.0], texture_corner, tile_size, atlas_size),
                Direction::BOTTOM => atlas_pos_to_coordinates([2.0, 0.0], texture_corner, tile_size, atlas_size),
                Direction::RIGHT => atlas_pos_to_coordinates([3.0, 0.0], texture_corner, tile_size, atlas_size),
                Direction::LEFT => atlas_pos_to_coordinates([3.0, 0.0], texture_corner, tile_size, atlas_size),
                Direction::FRONT => atlas_pos_to_coordinates([3.0, 0.0], texture_corner, tile_size, atlas_size),
                Direction::BACK => atlas_pos_to_coordinates([3.0, 0.0], texture_corner, tile_size, atlas_size),
            },
            MaterialType::DIRT => atlas_pos_to_coordinates([2.0, 0.0], texture_corner, tile_size, atlas_size),
            MaterialType::ROCK => atlas_pos_to_coordinates([0.0, 1.0], texture_corner, tile_size, atlas_size),
            MaterialType::WATER => atlas_pos_to_coordinates([13.0, 0.0], texture_corner, tile_size, atlas_size),
            MaterialType::AIR => [0.0, 0.0],
            MaterialType::DEBUG => atlas_pos_to_coordinates([5.0, 0.0], texture_corner, tile_size, atlas_size),
            // match quad_side {
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
