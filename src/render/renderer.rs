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
