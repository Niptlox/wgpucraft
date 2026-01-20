use std::{
    collections::VecDeque,
    collections::HashSet,
    sync::{Arc, RwLock},
};

use crate::render::pipelines::GlobalsLayouts;
use crate::terrain_gen::chunk::{CHUNK_AREA, Chunk, ChunkManager};
use crate::{
    render::{
        atlas::{Atlas, MaterialType},
        model::{DynamicModel, Model},
        mesh::Mesh,
        pipelines::terrain::{BlockVertex, create_terrain_pipeline},
        renderer::{Draw, Renderer},
        Vertex,
    },
    terrain_gen::biomes::PRAIRIE_PARAMS,
};

use cgmath::{EuclideanSpace, Point3, Vector3};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};
#[cfg(feature = "tracy")]
use tracy_client::span;
use wgpu::Queue;

pub const LAND_LEVEL: usize = 9;
const MAX_JOBS_IN_FLIGHT: usize = 8;
const MIN_VERTEX_CAP: usize = 4 * 1024;
const MIN_INDEX_CAP: usize = 8 * 1024;

use super::noise::NoiseGenerator;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OutlineVertex {
    pub pos: [f32; 3],
}

impl Vertex for OutlineVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<OutlineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        }
    }
}

pub struct TerrainGen {
    pipeline: wgpu::RenderPipeline,
    highlight_pipeline: wgpu::RenderPipeline,
    atlas: Atlas,
    pub chunks: ChunkManager,
    chunks_view_size: usize,
    chunk_indices: Arc<RwLock<Vec<Option<usize>>>>,
    free_chunk_indices: Arc<RwLock<VecDeque<usize>>>,
    center_offset: Vector3<i32>,
    chunks_origin: Vector3<i32>,
    pub chunk_models: Vec<Arc<RwLock<DynamicModel<BlockVertex>>>>,
    job_tx: Sender<ChunkJob>,
    ready_rx: Receiver<usize>,
    pending_jobs: HashSet<usize>,
    save_dir: PathBuf,
    dirty_queue: VecDeque<usize>,
    dirty_set: HashSet<usize>,
    save_tx: Sender<(PathBuf, Vec<u8>)>,
    highlight_model: Option<Model<OutlineVertex>>,
    highlight_pos: Option<Vector3<i32>>,
}

struct ChunkJob {
    chunk_index: usize,
    offset: Vector3<i32>,
    chunk: Arc<RwLock<Chunk>>,
    save_dir: PathBuf,
}

impl TerrainGen {
    pub fn new(
        renderer: &Renderer,
        render_distance_chunks: usize,
        seed: u32,
        world_name: &str,
    ) -> Self {
        let save_dir = PathBuf::from(format!("saves/{world_name}"));
        let _ = std::fs::create_dir_all(&save_dir);
        let global_layouts = GlobalsLayouts::new(&renderer.device);
        let atlas = Atlas::new(&renderer.device, &renderer.queue, &global_layouts).unwrap();
        let mut chunk_models = vec![];
        let mut chunks = ChunkManager::new();
        let chunks_view_size = render_distance_chunks.max(2);
        let chunk_capacity = chunks_view_size * chunks_view_size;
        let chunk_indices: Vec<Option<usize>> = vec![None; chunk_capacity];
        let mut free_chunk_indices = VecDeque::new();

        let noise_gen = NoiseGenerator::new(seed);

        for x in 0..chunk_capacity {
            chunks.add_chunk(Chunk::new([0, 0, 0]));
            // Начинаем с небольшого GPU-буфера и при необходимости растим его.
            let vertex_capacity = MIN_VERTEX_CAP; // увеличится, если чанку нужно больше
            let index_capacity = MIN_INDEX_CAP;
            let mut chunk_model =
                DynamicModel::new(&renderer.device, vertex_capacity, index_capacity);

            chunk_model.update(
                &renderer.device,
                &renderer.queue,
                &chunks.get_chunk(x).unwrap().read().unwrap().mesh,
            );
            chunk_models.push(Arc::new(RwLock::new(chunk_model)));
            free_chunk_indices.push_back(x);
        }

        let shader = renderer
            .device
            .create_shader_module(wgpu::include_wgsl!("../../assets/shaders/shader.wgsl"));

        let world_pipeline =
            create_terrain_pipeline(&renderer.device, &global_layouts, shader.clone(), &renderer.config);
        let highlight_shader = create_highlight_shader(&renderer.device);
        let highlight_pipeline =
            create_highlight_pipeline(&renderer.device, &global_layouts, &highlight_shader, &renderer.config);

        let center_offset = Vector3::new(0, 0, 0);
        let chunks_origin = center_offset
            - Vector3::new(chunks_view_size as i32 / 2, 0, chunks_view_size as i32 / 2);

        let (job_tx, job_rx) = mpsc::channel::<ChunkJob>();
        let (ready_tx, ready_rx) = mpsc::channel::<usize>();
        let (save_tx, save_rx) = mpsc::channel::<(PathBuf, Vec<u8>)>();
        let noise_for_worker = noise_gen.clone();

        std::thread::spawn(move || {
            while let Ok(job) = job_rx.recv() {
                if let Ok(mut chunk) = job.chunk.write() {
                    let path = job.save_dir.join(format!(
                        "chunk_{}_{}_{}.bin",
                        job.offset.x, job.offset.y, job.offset.z
                    ));
                    let loaded = path.exists() && chunk.load_from(&path, job.offset.into()).is_ok();
                    if !loaded {
                        chunk.update_blocks(job.offset.into(), &noise_for_worker, &PRAIRIE_PARAMS);
                    }
                    chunk.update_mesh(PRAIRIE_PARAMS, None);
                }
                let _ = ready_tx.send(job.chunk_index);
            }
        });

        thread::spawn(move || {
            while let Ok((path, materials)) = save_rx.recv() {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(path, materials);
            }
        });

        let mut world = Self {
            pipeline: world_pipeline,
            highlight_pipeline,
            atlas,
            chunks,
            chunk_models,
            chunks_view_size,
            center_offset,
            chunks_origin,
            chunk_indices: Arc::new(RwLock::new(chunk_indices)),
            free_chunk_indices: Arc::new(RwLock::new(free_chunk_indices)),
            job_tx,
            ready_rx,
            pending_jobs: HashSet::new(),
            save_dir,
            dirty_queue: VecDeque::new(),
            dirty_set: HashSet::new(),
            save_tx,
            highlight_model: None,
            highlight_pos: None,
        };

        println!("about to load first chunks");
        world.load_empty_chunks(center_offset);

        world
    }

    pub fn update_highlight_model(
        &mut self,
        device: &wgpu::Device,
        block: Option<Vector3<i32>>,
    ) {
        if self.highlight_pos == block {
            return;
        }
        self.highlight_pos = block;
        if let Some(pos) = block {
            let inflate = 0.01;
            let base = Vector3::new(
                pos.x as f32 - inflate,
                pos.y as f32 - inflate,
                pos.z as f32 - inflate,
            );
            let max = Vector3::new(
                pos.x as f32 + 1.0 + inflate,
                pos.y as f32 + 1.0 + inflate,
                pos.z as f32 + 1.0 + inflate,
            );
            let verts = [
                OutlineVertex { pos: [base.x, base.y, base.z] }, // 0
                OutlineVertex { pos: [max.x, base.y, base.z] },  // 1
                OutlineVertex { pos: [max.x, max.y, base.z] },   // 2
                OutlineVertex { pos: [base.x, max.y, base.z] },  // 3
                OutlineVertex { pos: [base.x, base.y, max.z] },  // 4
                OutlineVertex { pos: [max.x, base.y, max.z] },   // 5
                OutlineVertex { pos: [max.x, max.y, max.z] },    // 6
                OutlineVertex { pos: [base.x, max.y, max.z] },   // 7
            ];
            let mut mesh = Mesh {
                verts: verts.into(),
                indices: vec![
                    0, 1, 1, 2, 2, 3, 3, 0, // front square
                    4, 5, 5, 6, 6, 7, 7, 4, // back square
                    0, 4, 1, 5, 2, 6, 3, 7, // vertical edges
                ],
            };
            self.highlight_model = Model::new(device, &mesh);
        } else {
            self.highlight_model = None;
        }
    }

    // вызывается каждый кадр
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &Queue,
        player_position: &Point3<f32>,
    ) {
        #[cfg(feature = "tracy")]
        let _span = span!("update_world"); // <- Отметка начала блока

        let new_center_offset = Self::world_pos_to_chunk_offset(player_position.to_vec());
        let new_chunk_origin = new_center_offset
            - Vector3::new(
                self.chunks_view_size as i32 / 2,
                0,
                self.chunks_view_size as i32 / 2,
            );

        let moved_to_new_chunk = new_chunk_origin != self.chunks_origin;
        if moved_to_new_chunk {
            let old_origin = self.chunks_origin;
            self.center_offset = new_center_offset;
            self.chunks_origin = new_chunk_origin;
            let chunk_indices_copy = self.chunk_indices.read().unwrap().clone();
            let mut new_indices = vec![None; self.chunk_indices.read().unwrap().len()];

            for i in 0..chunk_indices_copy.len() {
                if let Some(chunk_index) = chunk_indices_copy[i] {
                    let chunk_offset = self.get_chunk_offset_from_origin(old_origin, i);
                    if self.chunk_in_bounds(chunk_offset.into()) {
                        let new_chunk_world_index = self.get_chunk_world_index(chunk_offset.into());
                        new_indices[new_chunk_world_index] = Some(chunk_index);
                    } else {
                        if !self.pending_jobs.contains(&chunk_index) {
                            self.shrink_and_free_chunk(device, chunk_index);
                        }
                    }
                }
            }

            *self.chunk_indices.write().unwrap() = new_indices;
        }

        self.process_ready_chunks(device, queue);
        self.process_dirty_chunks(device, queue, 2);

        if moved_to_new_chunk || self.has_missing_chunks() {
            self.load_empty_chunks(new_center_offset);
        }
    }

    fn process_ready_chunks(&mut self, device: &wgpu::Device, queue: &Queue) {
        while let Ok(chunk_index) = self.ready_rx.try_recv() {
            self.pending_jobs.remove(&chunk_index);
            let mapped = self
                .chunk_indices
                .read()
                .unwrap()
                .iter()
                .any(|entry| entry.map_or(false, |idx| idx == chunk_index));

            if let Some(chunk_model) = self.chunk_models.get(chunk_index) {
                let offset = if let Ok(chunk) = self.chunks.get_chunk(chunk_index).unwrap().read() {
                    let mut chunk_model = chunk_model.write().unwrap();
                    chunk_model.update(device, queue, &chunk.mesh);
                    Some(chunk.offset)
                } else {
                    None
                };
                if let Some(offset) = offset {
                    self.chunks.update_chunk_offset(chunk_index, offset);
                }
            }

            if !mapped {
                self.shrink_and_free_chunk(device, chunk_index);
            }
        }
    }

    fn process_dirty_chunks(&mut self, device: &wgpu::Device, queue: &Queue, max_per_frame: usize) {
        let mut processed = 0;
        while processed < max_per_frame {
            let Some(idx) = self.dirty_queue.pop_front() else { break };
            self.dirty_set.remove(&idx);
            if let Some(chunk_arc) = self.chunks.get_chunk(idx) {
                let mut chunk = chunk_arc.write().unwrap();
                if chunk.dirty {
                    let y_range = chunk.dirty_y_range();
                    chunk.update_mesh(PRAIRIE_PARAMS, y_range);
                    chunk.dirty = false;
                    let mesh = chunk.mesh.clone();
                    drop(chunk);
                    if let Some(chunk_model) = self.chunk_models.get(idx) {
                        let mut model = chunk_model.write().unwrap();
                        model.update(device, queue, &mesh);
                    }
                    processed += 1;
                }
            }
        }
    }

    pub fn mark_chunks_dirty(&mut self, indices: &[usize]) {
        for &idx in indices {
            if self.dirty_set.insert(idx) {
                self.dirty_queue.push_back(idx);
            }
        }
    }

    pub fn load_empty_chunks(&mut self, player_chunk: Vector3<i32>) {
        #[cfg(feature = "tracy")]
        let _span = span!("load empty chunks"); // <- Отметка начала блока

        let mut missing: Vec<(usize, Vector3<i32>)> = self
            .chunk_indices
            .read()
            .unwrap()
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .map(|(i, _)| (i, self.get_chunk_offset(i)))
            .collect();

        // Сортируем по расстоянию до чанка игрока.
        missing.sort_by_key(|(_, offset)| {
            let d = *offset - player_chunk;
            d.x * d.x + d.z * d.z
        });

        for (world_index, chunk_offset) in missing.into_iter() {
            if self.pending_jobs.len() >= MAX_JOBS_IN_FLIGHT {
                break;
            }
            if let Some(new_index) = self.free_chunk_indices.write().unwrap().pop_front() {
                self.chunk_indices.write().unwrap()[world_index] = Some(new_index);
                self.pending_jobs.insert(new_index);
                if let Some(chunk_arc) = self.chunks.get_chunk(new_index) {
                    let job = ChunkJob {
                        chunk_index: new_index,
                        offset: chunk_offset,
                        chunk: chunk_arc,
                        save_dir: self.save_dir.clone(),
                    };
                    let _ = self.job_tx.send(job);
                }
            } else {
                break;
            }
        }
    }

    fn has_missing_chunks(&self) -> bool {
        self.chunk_indices
            .read()
            .unwrap()
            .iter()
            .any(|entry| entry.is_none())
    }

    fn shrink_and_free_chunk(&mut self, device: &wgpu::Device, chunk_index: usize) {
        if let Some(chunk_arc) = self.chunks.get_chunk(chunk_index) {
            if let Ok(chunk) = chunk_arc.read() {
                if chunk.dirty {
                    let path = self.save_dir.join(format!(
                        "chunk_{}_{}_{}.bin",
                        chunk.offset[0], chunk.offset[1], chunk.offset[2]
                    ));
                    let materials: Vec<u8> = chunk
                        .blocks
                        .iter()
                        .map(|b| match b.material_type {
                            MaterialType::DIRT => 0,
                            MaterialType::GRASS => 1,
                            MaterialType::ROCK => 2,
                            MaterialType::WATER => 3,
                            MaterialType::AIR => 4,
                            MaterialType::DEBUG => 5,
                        })
                        .collect();
                    let _ = self.save_tx.send((path, materials));
                }
            }
        }

        if let Some(chunk_model) = self.chunk_models.get(chunk_index) {
            if let Ok(mut model) = chunk_model.write() {
                model.shrink_to(device, MIN_VERTEX_CAP, MIN_INDEX_CAP);
            }
        }
        self.chunks.remove_chunk_from_map(chunk_index);
        self.free_chunk_indices
            .write()
            .unwrap()
            .push_back(chunk_index);
    }

    pub fn world_pos_in_bounds(&self, world_pos: Vector3<f32>) -> bool {
        let chunk_offset = Self::world_pos_to_chunk_offset(world_pos);
        self.chunk_in_bounds(chunk_offset)
    }

    pub fn loaded_chunks(&self) -> usize {
        self.chunk_indices
            .read()
            .unwrap()
            .iter()
            .filter(|entry| entry.is_some())
            .count()
    }

    // индекс массива мировых чанков -> смещение чанка
    fn get_chunk_offset(&self, i: usize) -> Vector3<i32> {
        return self.chunks_origin
            + Vector3::new(
                i as i32 % self.chunks_view_size as i32,
                0,
                i as i32 / self.chunks_view_size as i32,
            );
    }

    fn get_chunk_offset_from_origin(
        &self,
        origin: Vector3<i32>,
        i: usize,
    ) -> Vector3<i32> {
        origin
            + Vector3::new(
                i as i32 % self.chunks_view_size as i32,
                0,
                i as i32 / self.chunks_view_size as i32,
            )
    }

    fn chunk_in_bounds(&self, chunk_offset: Vector3<i32>) -> bool {
        let p = chunk_offset - self.chunks_origin;
        if p.x >= 0
            && p.z >= 0
            && p.x < self.chunks_view_size as i32
            && p.z < self.chunks_view_size as i32
        {
            return true;
        }
        return false;
    }

    fn world_pos_to_chunk_offset(world_pos: Vector3<f32>) -> Vector3<i32> {
        Vector3::new(
            (world_pos.x / CHUNK_AREA as f32).floor() as i32,
            0,
            (world_pos.z / CHUNK_AREA as f32).floor() as i32,
        )
    }

    fn get_chunk_world_index(&self, chunk_offset: Vector3<i32>) -> usize {
        let p = chunk_offset - self.chunks_origin;
        (p.z as usize * self.chunks_view_size) + p.x as usize
    }
}

impl Draw for TerrainGen {
    fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        globals: &'a wgpu::BindGroup,
    ) -> Result<(), wgpu::Error> {
        #[cfg(feature = "tracy")]
        let _span = span!("drawing world"); // <- Отметка начала блока

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.atlas.bind_group, &[]);
        render_pass.set_bind_group(1, globals, &[]);

        for chunk_model in &self.chunk_models {
            let chunk_model = chunk_model.read().unwrap();

            let vertex_buffer = chunk_model.vbuf().slice(..);
            let index_buffer = chunk_model.ibuf().slice(..);
            let num_indices = chunk_model.num_indices;

            render_pass.set_vertex_buffer(0, vertex_buffer);
            render_pass.set_index_buffer(index_buffer, wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..num_indices, 0, 0..1 as _);
        }

        if let Some(model) = &self.highlight_model {
            render_pass.set_pipeline(&self.highlight_pipeline);
            render_pass.set_bind_group(0, globals, &[]);
            render_pass.set_vertex_buffer(0, model.vbuf().slice(..));
            render_pass.set_index_buffer(model.ibuf().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..model.num_indices, 0, 0..1);
        }

        Ok(())
    }
}

fn create_highlight_pipeline(
    device: &wgpu::Device,
    global_layout: &GlobalsLayouts,
    shader: &wgpu::ShaderModule,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Highlight Pipeline Layout"),
        bind_group_layouts: &[&global_layout.globals],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Highlight Pipeline"),
        layout: Some(&pipeline_layout),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[OutlineVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: crate::render::texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

fn create_highlight_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    let source = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    fog: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VSIn {
    @location(0) pos: vec3<f32>,
};

struct VSOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(input: VSIn) -> VSOut {
    var out: VSOut;
    out.pos = globals.view_proj * vec4<f32>(input.pos, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.6, 0.0, 1.0);
}
"#;
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Highlight Shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}
