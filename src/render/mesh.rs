use crate::terrain_gen::block::Quad;

use super::{Vertex, pipelines::terrain::BlockVertex};

#[derive(Clone)]

/// Меш на CPU, хранящий вершины и индексы.
pub struct Mesh<V: Vertex> {
    pub verts: Vec<V>,
    pub indices: Vec<u32>,
}

impl<V: Vertex> Mesh<V> {
    /// Создать новый `Mesh`.
    pub fn new() -> Self {
        Self {
            verts: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Очистить вершины, сохраняя выделенную память вектора.
    pub fn clear(&mut self) {
        self.verts.clear();
    }

    /// Получить срез ссылок на вершины.
    pub fn vertices(&self) -> &[V] {
        &self.verts
    }

    pub fn push(&mut self, vert: V) {
        self.verts.push(vert);
    }

    // Добавить набор индексов
    pub fn push_indices(&mut self, indices: &[u32]) {
        self.indices.extend_from_slice(indices);
    }

    // Вернуть индексы
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn add_quad(&mut self, quad: &Quad)
    where
        Vec<V>: Extend<BlockVertex>,
    {
        let base_index = self.verts.len() as u32;
        self.verts.extend(quad.vertices);
        self.indices.extend(&quad.get_indices(base_index));
    }

    pub fn clone(&self) -> Self {
        Self {
            verts: self.verts.clone(),
            indices: self.indices.clone(),
        }
    }

    pub fn iter_verts(&self) -> std::slice::Iter<V> {
        self.verts.iter()
    }

    pub fn iter_indices(&self) -> std::vec::IntoIter<u32> {
        self.indices.clone().into_iter()
    }
}
