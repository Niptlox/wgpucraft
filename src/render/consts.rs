use crate::render::buffer::DynamicBuffer;
use bytemuck::Pod;

// Хэндл для набора значений на GPU, которые не меняются в течение одного render pass.

pub struct Consts<T: Copy + Pod> {
    buf: DynamicBuffer<T>,
}

impl<T: Copy + Pod> Consts<T> {
    // Создать новый `Const<T>`.
    pub fn new(device: &wgpu::Device, len: usize) -> Self {
        Self {
            // TODO: проверить, все ли константы должны уметь обновляться
            buf: DynamicBuffer::new(device, len, wgpu::BufferUsages::UNIFORM),
        }
    }

    // Обновить значение на стороне GPU, связанное с этим хэндлом.
    pub fn update(&mut self, queue: &wgpu::Queue, vals: &[T], offset: usize) {
        self.buf.update(queue, vals, offset)
    }

    pub fn buf(&self) -> &wgpu::Buffer {
        &self.buf.buff
    }
}

// Константы неизменны в пределах одного render pass, но могут меняться к следующему.
