use anyhow::Result;
use freetype::{Face, Library};
use std::{fs, path::Path};

pub struct BitmapFont {
    _lib: Library,
    face: Face,
    pixel_size: u32,
}

impl BitmapFont {
    pub fn load_from_path(
        path: impl AsRef<Path>,
        pixel_size: f32,
        _font_weight_px: u32,
    ) -> Result<Self> {
        let lib = Library::init()?;
        let bytes = fs::read(path)?;
        let face = lib.new_memory_face(bytes, 0)?;
        face.set_pixel_sizes(0, pixel_size as u32)?;
        Ok(Self {
            _lib: lib,
            face,
            pixel_size: pixel_size as u32,
        })
    }

    pub fn measure_text(&self, text: &str) -> (f32, f32) {
        let mut width = 0.0;
        let mut max_h: f32 = 0.0;
        for ch in text.chars() {
            let glyph_index = self.face.get_char_index(ch as usize);
            if self
                .face
                .load_glyph(glyph_index, freetype::face::LoadFlag::RENDER)
                .is_ok()
            {
                let glyph = self.face.glyph();
                width += (glyph.advance().x >> 6) as f32;
                max_h = max_h.max(glyph.bitmap().rows() as f32);
            } else {
                width += self.pixel_size as f32 * 0.5;
            }
        }
        (width, max_h.max(self.pixel_size as f32))
    }

    pub fn advance(&self, ch: char) -> f32 {
        let idx = self.face.get_char_index(ch as usize);
        if self
            .face
            .load_glyph(idx, freetype::face::LoadFlag::DEFAULT)
            .is_ok()
        {
            (self.face.glyph().advance().x >> 6) as f32
        } else {
            self.pixel_size as f32 * 0.5
        }
    }

    pub fn height(&self) -> usize {
        self.pixel_size as usize
    }
}
