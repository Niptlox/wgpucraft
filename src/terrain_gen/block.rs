use cgmath::Vector3;

use crate::render::atlas::MaterialType;

use crate::render::pipelines::terrain::BlockVertex;

pub fn quad_vertex(
    pos: [i8; 3],
    material_type: MaterialType,
    texture_corners: [u32; 2],
    position: [i32; 3],
    quad_side: Direction,
) -> BlockVertex {
    let tc = material_type.get_texture_coordinates(texture_corners, quad_side);
    BlockVertex {
        pos: [
            pos[0] as f32 + position[0] as f32,
            pos[1] as f32 + position[1] as f32,
            pos[2] as f32 + position[2] as f32,
        ],
        texture_coordinates: [tc[0] as f32, tc[1] as f32],
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    TOP,
    BOTTOM,
    RIGHT,
    LEFT,
    FRONT,
    BACK,
}

impl Direction {
    pub const ALL: [Self; 6] = [
        Self::TOP,
        Self::BOTTOM,
        Self::RIGHT,
        Self::LEFT,
        Self::FRONT,
        Self::BACK,
    ];

    pub fn to_vec(self) -> Vector3<i32> {
        match self {
            Direction::TOP => Vector3::new(0, 1, 0),
            Direction::BOTTOM => Vector3::new(0, -1, 0),
            Direction::RIGHT => Vector3::new(1, 0, 0),
            Direction::LEFT => Vector3::new(-1, 0, 0),
            Direction::FRONT => Vector3::new(0, 0, 1),
            Direction::BACK => Vector3::new(0, 0, -1),
        }
    }

    pub fn get_vertices(self, material_type: MaterialType, position: [i32; 3]) -> [BlockVertex; 4] {
        match self {
            Direction::TOP => [
                quad_vertex([0, 1, 0], material_type, [0, 0], position, self),
                quad_vertex([0, 1, 1], material_type, [0, 1], position, self),
                quad_vertex([1, 1, 1], material_type, [1, 1], position, self),
                quad_vertex([1, 1, 0], material_type, [1, 0], position, self),
            ],
            Direction::BOTTOM => [
                quad_vertex([0, 0, 1], material_type, [0, 0], position, self),
                quad_vertex([0, 0, 0], material_type, [0, 1], position, self),
                quad_vertex([1, 0, 0], material_type, [1, 1], position, self),
                quad_vertex([1, 0, 1], material_type, [1, 0], position, self),
            ],
            Direction::RIGHT => [
                quad_vertex([1, 1, 1], material_type, [0, 0], position, self),
                quad_vertex([1, 0, 1], material_type, [0, 1], position, self),
                quad_vertex([1, 0, 0], material_type, [1, 1], position, self),
                quad_vertex([1, 1, 0], material_type, [1, 0], position, self),
            ],
            Direction::LEFT => [
                quad_vertex([0, 1, 0], material_type, [0, 0], position, self),
                quad_vertex([0, 0, 0], material_type, [0, 1], position, self),
                quad_vertex([0, 0, 1], material_type, [1, 1], position, self),
                quad_vertex([0, 1, 1], material_type, [1, 0], position, self),
            ],
            Direction::FRONT => [
                quad_vertex([0, 1, 1], material_type, [0, 0], position, self),
                quad_vertex([0, 0, 1], material_type, [0, 1], position, self),
                quad_vertex([1, 0, 1], material_type, [1, 1], position, self),
                quad_vertex([1, 1, 1], material_type, [1, 0], position, self),
            ],
            Direction::BACK => [
                quad_vertex([1, 1, 0], material_type, [0, 0], position, self),
                quad_vertex([1, 0, 0], material_type, [0, 1], position, self),
                quad_vertex([0, 0, 0], material_type, [1, 1], position, self),
                quad_vertex([0, 1, 0], material_type, [1, 0], position, self),
            ],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Quad {
    pub vertices: [BlockVertex; 4],
    pub side: Direction,
}

impl Quad {
    pub fn new(material_type: MaterialType, quad_side: Direction, position: [i32; 3]) -> Self {
        Self {
            vertices: quad_side.get_vertices(material_type, position),
            side: quad_side,
        }
    }

    pub fn get_indices_v(&self, vertex_offset: u32) -> [u32; 6] {
        [
            vertex_offset,
            vertex_offset + 1,
            vertex_offset + 2,
            vertex_offset + 2,
            vertex_offset + 3,
            vertex_offset,
        ]
    }

    pub fn get_indices(&self, i: u32) -> [u32; 6] {
        let displacement = i * 4;
        [
            0 + displacement,
            1 + displacement,
            2 + displacement,
            2 + displacement,
            3 + displacement,
            0 + displacement,
        ]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Block {
    pub material_type: MaterialType,
}

impl Block {
    pub fn new(material_type: MaterialType) -> Self {
        Self {
            material_type,
        }
    }

    pub fn is_transparent(&self) -> bool {
        self.material_type == MaterialType::AIR || self.material_type == MaterialType::WATER
    }

    pub fn is_solid(&self) -> bool {
        !self.is_transparent()
    }

    pub fn update(&mut self, new_material_type: MaterialType) {
        self.material_type = new_material_type;
    }
}

impl Default for Block {
    fn default() -> Self {
        Block {
            material_type: MaterialType::AIR,
        }
    }
}
