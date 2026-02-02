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

    fn draw_label(&mut self, _rect: [f32; 4], _label: &crate::ui::LabelSpec) {
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

    #[allow(dead_code)]
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

    fn update(&mut self, renderer: &Renderer, _stats: &OverlayStats) {
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

    #[allow(dead_code)]
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
