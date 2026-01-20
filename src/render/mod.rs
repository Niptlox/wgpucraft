pub mod atlas;
pub mod binding;
pub mod buffer;
pub mod consts;
pub mod mesh;
pub mod model;
pub mod pipelines;
pub mod renderer;
pub mod texture;

pub trait Vertex: Copy + bytemuck::Pod {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
