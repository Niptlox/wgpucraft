use super::HUDVertex;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IconType {
    ROCK,
    GRASS,
    DIRT,
    WATER,
    DEBUG,
}

const ICON_GRID: (f32, f32) = (16.0, 16.0); // сетка 16x16 иконок

impl IconType {
    pub fn all() -> Vec<IconType> {
        vec![
            IconType::ROCK,
            IconType::GRASS,
            IconType::DIRT,
            IconType::WATER,
            IconType::DEBUG,
        ]
    }

    fn get_uv_cords(&self, atlas_size: (f32, f32)) -> [f32; 4] {
        let (x, y) = match self {
            IconType::ROCK => (3, 0),
            IconType::GRASS => (1, 0),
            IconType::DIRT => (2, 0),
            IconType::WATER => (9, 0),
            IconType::DEBUG => (0, 7),
        };

        let tile_w = atlas_size.0 / ICON_GRID.0;
        let tile_h = atlas_size.1 / ICON_GRID.1;
        let u_min = (x as f32 * tile_w) / atlas_size.0;
        let v_min = (y as f32 * tile_h) / atlas_size.1;
        let u_max = u_min + (tile_w / atlas_size.0);
        let v_max = v_min + (tile_h / atlas_size.1);

        [u_min, v_min, u_max, v_max]
    }

    pub fn next(self) -> Self {
        match self {
            IconType::ROCK => IconType::GRASS,
            IconType::GRASS => IconType::DIRT,
            IconType::DIRT => IconType::WATER,
            IconType::WATER => IconType::DEBUG,
            IconType::DEBUG => IconType::ROCK, // циклический переход
        }
    }

    pub fn prev(self) -> Self {
        match self {
            IconType::ROCK => IconType::DEBUG,
            IconType::DEBUG => IconType::WATER,
            IconType::WATER => IconType::DIRT,
            IconType::DIRT => IconType::GRASS,
            IconType::GRASS => IconType::ROCK,
        }
    }

    pub fn get_vertex_quad(
        &self,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
        aspect_correction: f32,
        atlas_size: (f32, f32),
    ) -> ([HUDVertex; 4], [u32; 6]) {
        let uv = self.get_uv_cords(atlas_size);
        let half_width = (width / 2.0) * aspect_correction;
        let half_height = height / 2.0;

        let vertices = [
            HUDVertex {
                position: [center_x - half_width, center_y - half_height],
                uv: [uv[0], uv[3]], // v_min и v_max меняем местами
            },
            HUDVertex {
                position: [center_x + half_width, center_y - half_height],
                uv: [uv[2], uv[3]],
            },
            HUDVertex {
                position: [center_x + half_width, center_y + half_height],
                uv: [uv[2], uv[1]],
            },
            HUDVertex {
                position: [center_x - half_width, center_y + half_height],
                uv: [uv[0], uv[1]],
            },
        ];

        // Индексы для рендера двух треугольников (квад)
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }

    pub fn to_material(&self) -> crate::render::atlas::MaterialType {
        match self {
            IconType::ROCK => crate::render::atlas::MaterialType::ROCK,
            IconType::GRASS => crate::render::atlas::MaterialType::GRASS,
            IconType::DIRT => crate::render::atlas::MaterialType::DIRT,
            IconType::WATER => crate::render::atlas::MaterialType::WATER,
            IconType::DEBUG => crate::render::atlas::MaterialType::DEBUG,
        }
    }

    pub fn from_material(mat: crate::render::atlas::MaterialType) -> Option<Self> {
        match mat {
            crate::render::atlas::MaterialType::ROCK => Some(IconType::ROCK),
            crate::render::atlas::MaterialType::GRASS => Some(IconType::GRASS),
            crate::render::atlas::MaterialType::DIRT => Some(IconType::DIRT),
            crate::render::atlas::MaterialType::WATER => Some(IconType::WATER),
            crate::render::atlas::MaterialType::DEBUG => Some(IconType::DEBUG),
            crate::render::atlas::MaterialType::AIR => None,
        }
    }
}
