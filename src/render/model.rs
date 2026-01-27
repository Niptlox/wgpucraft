use crate::render::{buffer::Buffer, mesh::Mesh};

use super::{Vertex, buffer::DynamicBuffer};
/// Меш, отправленный на GPU.
pub struct Model<V: Vertex> {
    vbuf: Buffer<V>,
    ibuf: Buffer<u32>,
    pub num_indices: u32,
}

impl<V: Vertex> Model<V> {
    pub fn new(device: &wgpu::Device, mesh: &Mesh<V>) -> Option<Self> {
        if mesh.vertices().is_empty() || mesh.indices().is_empty() {
            return None;
        }

        let vbuf = Buffer::new(device, wgpu::BufferUsages::VERTEX, mesh.vertices());
        let ibuf = Buffer::new(device, wgpu::BufferUsages::INDEX, mesh.indices());

        Some(Self {
            vbuf,
            ibuf,
            num_indices: mesh.indices().len() as u32,
        })
    }

    pub fn vbuf(&self) -> &wgpu::Buffer {
        &self.vbuf.buff
    }
    pub fn ibuf(&self) -> &wgpu::Buffer {
        &self.ibuf.buff
    }
    pub fn len(&self) -> u16 {
        self.vbuf.len() as u16
    }
}

/// Меш, отправленный на GPU, с возможностью перевыделения буферов.
pub struct DynamicModel<V: Vertex> {
    vbuf: DynamicBuffer<V>,
    ibuf: DynamicBuffer<u32>,
    pub num_indices: u32,
    v_capacity: usize,
    i_capacity: usize,
    v_usage: wgpu::BufferUsages,
    i_usage: wgpu::BufferUsages,
}

impl<V: Vertex> DynamicModel<V> {
    pub fn new(device: &wgpu::Device, vertex_capacity: usize, index_capacity: usize) -> Self {
        let v_usage = wgpu::BufferUsages::VERTEX;
        let i_usage = wgpu::BufferUsages::INDEX;
        Self {
            vbuf: DynamicBuffer::new(device, vertex_capacity, v_usage),
            ibuf: DynamicBuffer::new(device, index_capacity, i_usage),
            num_indices: 0,
            v_capacity: vertex_capacity.max(1),
            i_capacity: index_capacity.max(1),
            v_usage,
            i_usage,
        }
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, verts: usize, indices: usize) {
        if verts > self.v_capacity {
            let new_cap = verts.next_power_of_two();
            self.vbuf = DynamicBuffer::new(device, new_cap, self.v_usage);
            self.v_capacity = new_cap;
        }
        if indices > self.i_capacity {
            let new_cap = indices.next_power_of_two();
            self.ibuf = DynamicBuffer::new(device, new_cap, self.i_usage);
            self.i_capacity = new_cap;
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &Mesh<V>) {
        self.ensure_capacity(device, mesh.vertices().len(), mesh.indices().len());
        self.vbuf.update(queue, mesh.vertices(), 0);
        self.ibuf.update(queue, mesh.indices(), 0);
        self.num_indices = mesh.indices().len() as u32;
    }

    /// Уменьшить буферы при возврате в пул, сохранив минимальный размер, чтобы избежать дёрганья.
    pub fn shrink_to(&mut self, device: &wgpu::Device, min_v: usize, min_i: usize) {
        let target_v = min_v.max(1).next_power_of_two();
        let target_i = min_i.max(1).next_power_of_two();
        if self.v_capacity > target_v {
            self.vbuf = DynamicBuffer::new(device, target_v, self.v_usage);
            self.v_capacity = target_v;
        }
        if self.i_capacity > target_i {
            self.ibuf = DynamicBuffer::new(device, target_i, self.i_usage);
            self.i_capacity = target_i;
        }
        self.num_indices = 0;
    }

    pub fn vbuf(&self) -> &wgpu::Buffer {
        &self.vbuf.buff
    }
    pub fn ibuf(&self) -> &wgpu::Buffer {
        &self.ibuf.buff
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.vbuf.len()
    }
}
