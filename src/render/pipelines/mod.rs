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
