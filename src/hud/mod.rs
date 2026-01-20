use icons_atlas::IconType;

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
    palette: Vec<IconType>,
    selected_index: usize,
    debug_overlay: Option<DebugOverlay>,
    aspect_correction: f32,
}

struct HUDElement {
    texture: Texture,
    bind_group: wgpu::BindGroup,
    model: Model<HUDVertex>,
}

impl HUD {
    pub fn new(
        renderer: &Renderer,
        global_layout: &GlobalsLayouts,
        shader: wgpu::ShaderModule,
        show_debug_overlay: bool,
    ) -> Self {
        let aspect_correction = renderer.size.height as f32 / renderer.size.width as f32;
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

        let selected_icon = IconType::ROCK; // Значок по умолчанию
        // Геометрия элементов HUD
        let (crosshair_verts, crosshair_indices) =
            create_hud_quad(0.0, 0.0, 0.06, 0.06, aspect_correction); // Размер прицела
        let (widget_verts, widget_indices) =
            create_hud_quad(0.0, -0.85, 0.7, 0.18, aspect_correction); // Окно хотбара

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
        );

        // Создаём bind group для каждого элемента
        let crosshair = HUDElement {
            texture: crosshair_tex,
            bind_group: crosshair_bind_group,
            model: crosshair_model,
        };

        let widget = HUDElement {
            texture: widget_tex,
            bind_group: widget_bind_group,
            model: widget_model,
        };

        let toolbar = HUDElement {
            texture: icons_atlas_tex,
            bind_group: icons_bind_group,
            model: toolbar_model,
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
            palette,
            selected_index,
            debug_overlay,
            aspect_correction,
        }
    }

    pub fn update(&mut self, renderer: &Renderer) {
        self.toolbar.model = build_toolbar_model(
            &renderer.device,
            &self.palette,
            self.selected_index,
            self.aspect_correction,
        );
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

        let (widget_verts, widget_indices) =
            create_hud_quad(0.85, -0.85, 0.2, 0.2, self.aspect_correction);
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
        let mut elements: Vec<&HUDElement> = vec![&self.crosshair, &self.widget, &self.toolbar];
        if let Some(overlay) = &self.debug_overlay {
            if overlay.visible {
                elements.push(&overlay.element);
            }
        }

        for element in elements {
            render_pass.set_bind_group(0, &element.bind_group, &[]);
            render_pass.set_vertex_buffer(0, element.model.vbuf().slice(..));
            render_pass.set_index_buffer(element.model.ibuf().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..element.model.num_indices, 0, 0..1);
        }

        Ok(())
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
            position: [center_x - half_w, center_y - half_h],
            uv: [0.0, 0.0],
        },
        // Верхний правый угол
        HUDVertex {
            position: [center_x + half_w, center_y - half_h],
            uv: [1.0, 0.0],
        },
        // Нижний правый угол
        HUDVertex {
            position: [center_x + half_w, center_y + half_h],
            uv: [1.0, 1.0],
        },
        // Нижний левый угол
        HUDVertex {
            position: [center_x - half_w, center_y + half_h],
            uv: [0.0, 1.0],
        },
    ];

    let indices = vec![0u32, 1, 2, 0, 2, 3];

    (vertices, indices)
}

fn build_toolbar_model(
    device: &wgpu::Device,
    palette: &[IconType],
    selected_index: usize,
    aspect_correction: f32,
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
        let (quad_verts, quad_indices) =
            icon.get_vertex_quad(center_x, center_y, width, height, aspect_correction);
        let base_index = verts.len() as u32;
        verts.extend_from_slice(&quad_verts);
        indices.extend(quad_indices.iter().map(|idx| idx + base_index));
    }

    Model::new(
        device,
        &Mesh {
            verts,
            indices,
        },
    )
    .expect("Failed to build toolbar model")
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
        let (verts, indices) = create_hud_quad(-0.75, 0.85, 0.6, 0.22, aspect_correction);
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
        let primary = [240, 240, 240, 255];
        let accent = [120, 200, 255, 255];

        self.draw_text_line(&format!("FPS   {:>5.1}", stats.fps), 6, 8, primary);
        self.draw_text_line(&format!("MS    {:>5.2}", stats.frame_ms), 6, 22, accent);
        self.draw_text_line(
            &format!("CHUNKS {:>4}", stats.chunks_loaded),
            6,
            36,
            primary,
        );
        self.draw_text_line(&format!("DRAWS  {:>4}", stats.draw_calls), 6, 50, primary);

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

    fn draw_text_line(&mut self, text: &str, start_x: usize, start_y: usize, color: [u8; 4]) {
        let mut cursor_x = start_x;
        for ch in text.chars() {
            self.draw_char(ch, cursor_x, start_y, color);
            cursor_x += FONT_WIDTH + 1;
        }
    }

    fn draw_char(&mut self, ch: char, start_x: usize, start_y: usize, color: [u8; 4]) {
        let glyph = glyph_bitmap(ch);
        for (row, bits) in glyph.iter().enumerate() {
            let y = start_y + (FONT_HEIGHT - 1 - row);
            for col in 0..FONT_WIDTH {
                if bits & (1 << (FONT_WIDTH - 1 - col)) != 0 {
                    self.pixel(start_x + col, y, color);
                }
            }
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

const FONT_WIDTH: usize = 5;
const FONT_HEIGHT: usize = 7;

fn glyph_bitmap(ch: char) -> [u8; FONT_HEIGHT] {
    match ch {
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111,
        ],
        '3' => [
            0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b01110, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b11110, 0b10000, 0b11110, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b10000,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'W' => [
            0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'G' => [
            0b01110, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        ' ' => [0; FONT_HEIGHT],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100,
        ],
        _ => [0; FONT_HEIGHT],
    }
}
