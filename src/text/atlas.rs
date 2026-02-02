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
            let _ = page;
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
