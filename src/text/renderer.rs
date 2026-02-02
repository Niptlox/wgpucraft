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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::render::texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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

    fn build_vertices(
        &mut self,
        text: &str,
        font_id: FontHandle,
        style: TextStyle,
        mode: TextMode,
        origin: Vec3,
        right: Option<Vec3>,
        up: Option<Vec3>,
        screen_size: Option<[f32; 2]>,
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
                    let screen = screen_size.expect("screen size required for GUI");
                    let to_clip = |pos: Vec3| -> Vec3 {
                        let nx = (pos.x / screen[0]) * 2.0 - 1.0;
                        let ny = 1.0 - (pos.y / screen[1]) * 2.0;
                        Vec3::new(nx, ny, 0.0)
                    };
                    let p0 = to_clip(origin + Vec3::new(x0, y0, 0.0));
                    let p1 = to_clip(origin + Vec3::new(x0 + w, y0, 0.0));
                    let p2 = to_clip(origin + Vec3::new(x0 + w, y0 + h, 0.0));
                    let p3 = to_clip(origin + Vec3::new(x0, y0 + h, 0.0));
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
                // first triangle
                TextVertex { position: quad[0].into(), uv: [uv[0], uv[1]], color },
                TextVertex { position: quad[1].into(), uv: [uv[2], uv[1]], color },
                TextVertex { position: quad[2].into(), uv: [uv[2], uv[3]], color },
                // second triangle
                TextVertex { position: quad[0].into(), uv: [uv[0], uv[1]], color },
                TextVertex { position: quad[2].into(), uv: [uv[2], uv[3]], color },
                TextVertex { position: quad[3].into(), uv: [uv[0], uv[3]], color },
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
        self.build_vertices(
            text,
            font,
            style,
            TextMode::Gui,
            Vec3::new(origin_px.x, origin_px.y, 0.0),
            None,
            None,
            Some(screen_size),
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
            None,
        )
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
