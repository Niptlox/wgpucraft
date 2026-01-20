use std::{
    fs,
    path::Path,
    sync::{Arc, RwLock},
};

use cgmath::Vector3;
use std::collections::HashMap;
use anyhow::{bail, Result};
#[cfg(feature = "tracy")]
use tracy_client::span;

use crate::render::{atlas::MaterialType, mesh::Mesh, pipelines::terrain::BlockVertex};

use super::{biomes::BiomeParameters, block::Block, generator::LAND_LEVEL, noise::NoiseGenerator};

pub const CHUNK_Y_SIZE: usize = 512;
pub const CHUNK_AREA: usize = 16;
pub const CHUNK_AREA_WITH_PADDING: usize = CHUNK_AREA + 2; // +1 с каждой стороны для паддинга
pub const TOTAL_CHUNK_SIZE: usize =
    CHUNK_Y_SIZE * CHUNK_AREA_WITH_PADDING * CHUNK_AREA_WITH_PADDING;

pub struct Chunk {
    pub blocks: Vec<Block>,
    pub offset: [i32; 3],
    pub mesh: Mesh<BlockVertex>,
    pub dirty: bool,
    dirty_y_range: Option<(usize, usize)>,
}

impl Chunk {
    pub fn new(offset: [i32; 3]) -> Self {
        let mut blocks = Vec::with_capacity(TOTAL_CHUNK_SIZE);

        for y in 0..CHUNK_Y_SIZE {
            for x in 0..CHUNK_AREA_WITH_PADDING {
                for z in 0..CHUNK_AREA_WITH_PADDING {
                    let position = Vector3 {
                        x: x as i32 - 1, // -1 для левого паддинга
                        y: y as i32,
                        z: z as i32 - 1, // -1 для фронтального паддинга
                    };

                    let material_type = if y < 12 {
                        MaterialType::DEBUG
                    } else if y == 12 {
                        MaterialType::DEBUG
                    } else {
                        MaterialType::AIR
                    };

                    blocks.push(Block::new(material_type, position.into(), offset));
                }
            }
        }
        let mesh = Mesh::new();
        Chunk {
            blocks,
            offset,
            mesh,
            dirty: false,
            dirty_y_range: None,
        }
    }

    /// Линейный индекс внутри чанка по координатам y, x, z
    fn calculate_index(&self, y: usize, x: usize, z: usize) -> usize {
        y * (CHUNK_AREA_WITH_PADDING * CHUNK_AREA_WITH_PADDING) + x * CHUNK_AREA_WITH_PADDING + z
    }

    /// Получить ссылку на блок (immut)
    pub fn get_block(&self, y: usize, x: usize, z: usize) -> Option<&Block> {
        if y < CHUNK_Y_SIZE && x < CHUNK_AREA_WITH_PADDING && z < CHUNK_AREA_WITH_PADDING {
            let index = self.calculate_index(y, x, z);
            self.blocks.get(index)
        } else {
            None
        }
    }

    /// Получить изменяемый блок
    pub fn get_block_mut(&mut self, y: usize, x: usize, z: usize) -> Option<&mut Block> {
        if y < CHUNK_Y_SIZE && x < CHUNK_AREA_WITH_PADDING && z < CHUNK_AREA_WITH_PADDING {
            let index = self.calculate_index(y, x, z);
            self.blocks.get_mut(index)
        } else {
            None
        }
    }

    pub fn update_blocks(
        &mut self,
        offset: [i32; 3],
        noise_generator: &NoiseGenerator,
        biome: &BiomeParameters,
    ) {
        #[cfg(feature = "tracy")]
        let _span = span!("generate chunk: full scope"); // Замер генерации чанка

        self.offset = offset; // Сохраняем смещение чанка

        let max_biome_height = (biome.base_height + biome.amplitude) as usize;

        for y in 0..CHUNK_Y_SIZE {
            for x in 0..CHUNK_AREA_WITH_PADDING {
                for z in 0..CHUNK_AREA_WITH_PADDING {
                    if y > max_biome_height {
                        continue; // Пропускаем высоту выше биома
                    }
                    #[cfg(feature = "tracy")]
                    let _inner_span = span!(" creating single block");

                    if y < (biome.base_height - 1.0) as usize {
                        self.get_block_mut(y, x, z)
                            .unwrap()
                            .update(MaterialType::DIRT, offset);
                        continue;
                    }

                    let local_x = x as i32 - 1;
                    let local_z = z as i32 - 1;
                    let world_pos =
                        local_pos_to_world(self.offset, Vector3::new(local_x, y as i32, local_z));
                    let height_variation = noise_generator.get_height(
                        world_pos.x as f32,
                        world_pos.z as f32,
                        biome.frequency,
                        biome.amplitude,
                    );
                    let new_height = (biome.base_height + height_variation).round() as usize;

                    let block_type = if y > new_height {
                        if y <= LAND_LEVEL {
                            MaterialType::WATER
                        } else {
                            MaterialType::AIR
                        }
                    } else if y == new_height {
                        MaterialType::GRASS
                    } else if y == 0 {
                        MaterialType::ROCK
                    } else {
                        MaterialType::DIRT
                    };
                    self.get_block_mut(y, x, z)
                        .unwrap()
                        .update(block_type, offset);
                }
            }
        }
        self.dirty = false;
    }

    pub fn update_mesh(&mut self, _biome: BiomeParameters, _y_range: Option<(usize, usize)>) {
        let mut verts = Vec::new();
        let mut indices = Vec::new();

        #[cfg(feature = "tracy")]
        let _span = span!(" update chunk mesh"); // Замер построения меша

        // Итерируемся только по внутренней области (1..CHUNK_AREA+1 исключает паддинг)
        for y in 0..CHUNK_Y_SIZE {
            for x in 1..=CHUNK_AREA {
                for z in 1..=CHUNK_AREA {
                    #[cfg(feature = "tracy")]
                    let _inner_span = span!("processing block vertices"); // Замер вершин блока

                    let block = self.get_block(y, x, z).unwrap();
                    let mut block_vertices = Vec::with_capacity(4 * 6);
                    let mut block_indices: Vec<u32> = Vec::with_capacity(6 * 6);

                    if block.material_type as i32 == MaterialType::AIR as i32 {
                        continue;
                    }

                    let mut quad_counter = 0;

                    for side in crate::terrain_gen::block::Direction::ALL {
                        let neighbor_pos: Vector3<i32> =
                            block.get_vec_position() + side.to_vec();
                        let visible = self.is_quad_visible(&neighbor_pos);

                        if visible {
                            let world_pos = block.get_world_position();
                            let quad = crate::terrain_gen::block::Quad::new(
                                block.material_type,
                                side,
                                world_pos,
                            );
                            block_vertices.extend_from_slice(&quad.vertices);
                            block_indices.extend_from_slice(&quad.get_indices(quad_counter));
                            quad_counter += 1;
                        }
                    }

                    block_indices = block_indices
                        .iter()
                        .map(|i| i + verts.len() as u32)
                        .collect();
                    verts.extend(block_vertices);
                    indices.extend(block_indices);
                }
            }
        }

        self.mesh = Mesh { verts, indices };
        self.dirty_y_range = None;
    }

    fn mark_dirty_y(&mut self, y: usize) {
        let y0 = y.saturating_sub(1);
        let y1 = (y + 1).min(CHUNK_Y_SIZE - 1);
        self.dirty_y_range = match self.dirty_y_range {
            Some((lo, hi)) => Some((lo.min(y0), hi.max(y1))),
            None => Some((y0, y1)),
        };
    }

    pub fn dirty_y_range(&self) -> Option<(usize, usize)> {
        self.dirty_y_range
    }

    fn is_quad_visible(&self, neighbor_pos: &Vector3<i32>) -> bool {
        if pos_in_chunk_bounds(*neighbor_pos) {
            // Преобразуем координаты (-1..16) в индексы массива (0..17)

            let x_index = (neighbor_pos.x + 1) as usize;
            let y_index = neighbor_pos.y as usize;
            let z_index = (neighbor_pos.z + 1) as usize;

            let neighbor_block = self.get_block(y_index, x_index, z_index).unwrap();
            return neighbor_block.material_type as u16 == MaterialType::AIR as u16;
        } else {
            // Нет соседа в этом чанке — считаем грань видимой, чтобы не пропадали блоки на границах.
            return true;
        }
    }
}

pub struct ChunkManager {
    pub chunks: Vec<Arc<RwLock<Chunk>>>,
    offset_index_map: HashMap<[i32; 3], usize>,
    index_offset: Vec<[i32; 3]>,
}

impl ChunkManager {
    pub fn new() -> Self {
        ChunkManager {
            chunks: Vec::new(),
            offset_index_map: HashMap::new(),
            index_offset: Vec::new(),
        }
    }

    pub fn add_chunk(&mut self, mut chunk: Chunk) {
        chunk.offset = [i32::MIN, i32::MIN, i32::MIN];
        self.index_offset.push(chunk.offset);
        self.chunks.push(Arc::new(RwLock::new(chunk)));
        // offset_index_map будет заполнен при update_chunk_offset
        debug_assert!(self.offset_index_map.get(&[i32::MIN, i32::MIN, i32::MIN]).is_none());
    }

    pub fn get_chunk(&self, index: usize) -> Option<Arc<RwLock<Chunk>>> {
        if index < self.chunks.len() {
            Some(self.chunks[index].clone())
        } else {
            None
        }
    }

    pub fn get_chunk_index_by_offset(&self, offset: &[i32; 3]) -> Option<usize> {
        self.offset_index_map.get(offset).copied()
    }

    // Получить материал блока в мировых координатах
    pub fn get_block_material(&self, world_pos: Vector3<i32>) -> Option<MaterialType> {
        let (chunk_offset, local_pos) = world_pos_to_chunk_and_local(world_pos);

        // Учитываем паддинг (local_pos 0..15 -> нужно -1..16)
        let x = local_pos.x + 1;
        let z = local_pos.z + 1;
        let y = local_pos.y;

        if !pos_in_chunk_bounds(Vector3::new(x, y, z)) {
            return None;
        }

        self.get_chunk_index_by_offset(&chunk_offset)
            .and_then(|index| {
                let chunk = self.chunks[index].read().unwrap();
                Some(
                    chunk
                        .get_block(y as usize, x as usize, z as usize)?
                        .material_type,
                )
            })
    }

    // Установить материал блока в мировых координатах
    pub fn set_block_material(
        &mut self,
        world_pos: Vector3<i32>,
        material: MaterialType,
    ) -> Vec<usize> {
        let (chunk_offset, local_pos) = world_pos_to_chunk_and_local(world_pos);

        // Учитываем паддинг (local_pos 0..15 -> нужно -1..16)
        let x = local_pos.x + 1;
        let z = local_pos.z + 1;
        let y = local_pos.y;

        if !pos_in_chunk_bounds(Vector3::new(x, y, z)) {
            println!("Position out of bounds: {:?}", world_pos);
            return Vec::new();
        }

        let mut touched = Vec::new();
        let mut neighbor_offsets = Vec::new();
        if local_pos.x == 0 {
            neighbor_offsets.push([chunk_offset[0] - 1, chunk_offset[1], chunk_offset[2]]);
        } else if local_pos.x == (CHUNK_AREA as i32 - 1) {
            neighbor_offsets.push([chunk_offset[0] + 1, chunk_offset[1], chunk_offset[2]]);
        }
        if local_pos.z == 0 {
            neighbor_offsets.push([chunk_offset[0], chunk_offset[1], chunk_offset[2] - 1]);
        } else if local_pos.z == (CHUNK_AREA as i32 - 1) {
            neighbor_offsets.push([chunk_offset[0], chunk_offset[1], chunk_offset[2] + 1]);
        }

        if let Some(index) = self.get_chunk_index_by_offset(&chunk_offset) {
            let mut chunk = self.chunks[index].write().unwrap();
            if let Some(block) = chunk.get_block_mut(y as usize, x as usize, z as usize) {
                block.update(material, chunk_offset);
                chunk.dirty = true;
                chunk.mark_dirty_y(y as usize);
                println!("Block updated at world position: {:?}", world_pos);
                touched.push(index);
            }
            drop(chunk);

            // Если блок на границе чанка — отмечаем соседние чанки как грязные, чтобы перерассчитать меш.
            for neigh_off in neighbor_offsets {
                if let Some(nidx) = self.get_chunk_index_by_offset(&neigh_off) {
                    if let Ok(mut neigh_chunk) = self.chunks[nidx].write() {
                        // Обновляем паддинг соседа, чтобы его грань стала видимой/скрытой корректно.
                        let origin = Vector3::new(
                            neigh_off[0] * CHUNK_AREA as i32,
                            neigh_off[1] * CHUNK_Y_SIZE as i32,
                            neigh_off[2] * CHUNK_AREA as i32,
                        );
                        let local_in_neigh = world_pos - origin;
                        let on_padding = local_in_neigh.x == -1
                            || local_in_neigh.x == CHUNK_AREA as i32
                            || local_in_neigh.z == -1
                            || local_in_neigh.z == CHUNK_AREA as i32;
                        if on_padding && pos_in_chunk_bounds(local_in_neigh) {
                            let nx = (local_in_neigh.x + 1) as usize;
                            let nz = (local_in_neigh.z + 1) as usize;
                            let ny = local_in_neigh.y as usize;
                            if let Some(pad_block) = neigh_chunk.get_block_mut(ny, nx, nz) {
                                pad_block.update(material, neigh_off);
                            }
                            neigh_chunk.mark_dirty_y(ny);
                        }
                        neigh_chunk.dirty = true;
                    }
                    touched.push(nidx);
                }
            }
            touched
        } else {
            println!("Chunk not found for world position: {:?}", world_pos);
            Vec::new()
        }
    }

    pub fn update_chunk_offset(&mut self, index: usize, new_offset: [i32; 3]) {
        if let Some(old_offset) = self.index_offset.get(index).copied() {
            self.offset_index_map.remove(&old_offset);
        }
        if index < self.index_offset.len() {
            self.index_offset[index] = new_offset;
        }
        self.offset_index_map.insert(new_offset, index);
        if let Some(chunk) = self.chunks.get(index) {
            if let Ok(mut chunk) = chunk.write() {
                chunk.offset = new_offset;
            }
        }
    }

    pub fn remove_chunk_from_map(&mut self, index: usize) {
        if let Some(old_offset) = self.index_offset.get(index).copied() {
            self.offset_index_map.remove(&old_offset);
            self.index_offset[index] = [0, 0, 0];
        }
    }
}

pub fn pos_in_chunk_bounds(pos: Vector3<i32>) -> bool {
    // Допускаем координаты от -1 до CHUNK_AREA (0..15 внутренняя область, -1 и 16 — паддинг)
    pos.x >= -1
        && pos.y >= 0
        && pos.z >= -1
        && pos.x <= CHUNK_AREA as i32
        && pos.y < CHUNK_Y_SIZE as i32
        && pos.z <= CHUNK_AREA as i32
}

fn world_pos_to_chunk_and_local(world_pos: Vector3<i32>) -> ([i32; 3], Vector3<i32>) {
    let chunk_x = world_pos.x.div_euclid(CHUNK_AREA as i32);
    let chunk_y = world_pos.y.div_euclid(CHUNK_Y_SIZE as i32);
    let chunk_z = world_pos.z.div_euclid(CHUNK_AREA as i32);

    let local_x = world_pos.x.rem_euclid(CHUNK_AREA as i32);
    let local_y = world_pos.y.rem_euclid(CHUNK_Y_SIZE as i32);
    let local_z = world_pos.z.rem_euclid(CHUNK_AREA as i32);

    (
        [chunk_x, chunk_y, chunk_z],
        Vector3::new(local_x, local_y, local_z),
    )
}

pub fn local_pos_to_world(offset: [i32; 3], local_pos: Vector3<i32>) -> Vector3<f32> {
    Vector3::new(
        local_pos.x as f32 + (offset[0] as f32 * CHUNK_AREA as f32),
        local_pos.y as f32 + (offset[1] as f32 * CHUNK_AREA as f32),
        local_pos.z as f32 + (offset[2] as f32 * CHUNK_AREA as f32),
    )
}

fn material_to_u8(mat: MaterialType) -> u8 {
    match mat {
        MaterialType::DIRT => 0,
        MaterialType::GRASS => 1,
        MaterialType::ROCK => 2,
        MaterialType::WATER => 3,
        MaterialType::AIR => 4,
        MaterialType::DEBUG => 5,
    }
}

fn material_from_u8(v: u8) -> MaterialType {
    match v {
        0 => MaterialType::DIRT,
        1 => MaterialType::GRASS,
        2 => MaterialType::ROCK,
        3 => MaterialType::WATER,
        4 => MaterialType::AIR,
        5 => MaterialType::DEBUG,
        _ => MaterialType::AIR,
    }
}

impl Chunk {
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut buf = Vec::with_capacity(self.blocks.len());
        for block in &self.blocks {
            buf.push(material_to_u8(block.material_type));
        }
        fs::write(path, buf)?;
        Ok(())
    }

    pub fn load_from(&mut self, path: &Path, offset: [i32; 3]) -> Result<()> {
        let data = fs::read(path)?;
        if data.len() != self.blocks.len() {
            bail!("chunk file has wrong size");
        }
        for (b, val) in self.blocks.iter_mut().zip(data.into_iter()) {
            b.update(material_from_u8(val), offset);
        }
        self.offset = offset;
        self.dirty = false;
        self.dirty_y_range = None;
        Ok(())
    }
}
