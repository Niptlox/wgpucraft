use crate::render::{mesh::Mesh, pipelines::hud::HUDVertex};

pub struct MeshBuilder {
    verts: Vec<HUDVertex>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            verts: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn push_quad(&mut self, rect: [f32; 4], screen: [f32; 2], uv: [f32; 4]) {
        let [x, y, w, h] = rect;
        let x0 = x;
        let y0 = y;
        let x1 = x + w;
        let y1 = y + h;

        let tl = to_clip(x0, y0, screen);
        let tr = to_clip(x1, y0, screen);
        let br = to_clip(x1, y1, screen);
        let bl = to_clip(x0, y1, screen);

        let base = self.verts.len() as u32;

        self.verts.extend_from_slice(&[
            HUDVertex {
                position: tl,
                uv: [uv[0], uv[1]],
            },
            HUDVertex {
                position: tr,
                uv: [uv[2], uv[1]],
            },
            HUDVertex {
                position: br,
                uv: [uv[2], uv[3]],
            },
            HUDVertex {
                position: bl,
                uv: [uv[0], uv[3]],
            },
        ]);

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn build(self) -> Mesh<HUDVertex> {
        Mesh {
            verts: self.verts,
            indices: self.indices,
        }
    }
}

pub fn quad_from_rect(rect: [f32; 4], screen: [f32; 2]) -> [HUDVertex; 4] {
    let [x, y, w, h] = rect;
    let x0 = x;
    let y0 = y;
    let x1 = x + w;
    let y1 = y + h;

    let tl = to_clip(x0, y0, screen);
    let tr = to_clip(x1, y0, screen);
    let br = to_clip(x1, y1, screen);
    let bl = to_clip(x0, y1, screen);

    [
        HUDVertex {
            position: tl,
            uv: [0.0, 0.0],
        },
        HUDVertex {
            position: tr,
            uv: [1.0, 0.0],
        },
        HUDVertex {
            position: br,
            uv: [1.0, 1.0],
        },
        HUDVertex {
            position: bl,
            uv: [0.0, 1.0],
        },
    ]
}

fn to_clip(x: f32, y: f32, screen: [f32; 2]) -> [f32; 2] {
    let nx = (x / screen[0]) * 2.0 - 1.0;
    let ny = 1.0 - (y / screen[1]) * 2.0;
    [nx, ny]
}
