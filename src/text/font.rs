use anyhow::Result;
use freetype::{Face, Library, face::LoadFlag};
use std::{path::Path, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontHandle(pub usize);

pub struct Font {
    pub id: FontHandle,
    pub face: Face,
}

pub struct FontManager {
    library: Library,
    fonts: Vec<Arc<Font>>,
}

impl FontManager {
    pub fn new() -> Result<Self> {
        let library = Library::init()?;
        Ok(Self {
            library,
            fonts: Vec::new(),
        })
    }

    pub fn load_font(&mut self, path: impl AsRef<Path>) -> Result<FontHandle> {
        let path_ref = path.as_ref();
        let face = self.library.new_face(path_ref, 0)?;
        let handle = FontHandle(self.fonts.len());
        let font = Arc::new(Font { id: handle, face });
        self.fonts.push(font);
        Ok(handle)
    }

    pub fn get(&self, handle: FontHandle) -> Arc<Font> {
        self.fonts[handle.0].clone()
    }
}

#[derive(Clone, Debug)]
pub struct GlyphBitmap {
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: f32,
    pub buffer: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderMode {
    Normal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: FontHandle,
    pub glyph_id: u32,
    pub pixel_size: u32,
    pub render_mode: RenderMode,
}

impl Font {
    pub fn set_pixel_size(&self, px: u32) -> Result<()> {
        self.face.set_pixel_sizes(0, px)?;
        Ok(())
    }

    pub fn load_glyph_bitmap(&self, glyph_id: u32, pixel_size: u32) -> Result<Option<GlyphBitmap>> {
        self.face.set_pixel_sizes(0, pixel_size)?;
        let glyph_index = glyph_id;
        self.face.load_glyph(
            glyph_index as u32,
            LoadFlag::RENDER | LoadFlag::TARGET_NORMAL,
        )?;
        let glyph_slot = self.face.glyph();
        let bitmap = glyph_slot.bitmap();
        let width = bitmap.width() as u32;
        let height = bitmap.rows() as u32;
        if width == 0 || height == 0 {
            return Ok(None);
        }
        let buffer = bitmap.buffer().to_vec();
        let advance = (glyph_slot.advance().x >> 6) as f32;
        Ok(Some(GlyphBitmap {
            width,
            height,
            bearing_x: glyph_slot.bitmap_left(),
            bearing_y: glyph_slot.bitmap_top(),
            advance,
            buffer,
        }))
    }

    pub fn glyph_index_for_char(&self, ch: char) -> u32 {
        self.face.get_char_index(ch as usize)
    }

    pub fn ascent(&self, pixel_size: u32) -> Result<f32> {
        self.face.set_pixel_sizes(0, pixel_size)?;
        Ok(self
            .face
            .size_metrics()
            .map(|m| (m.ascender >> 6) as f32)
            .unwrap_or(0.0))
    }

    pub fn descent(&self, pixel_size: u32) -> Result<f32> {
        self.face.set_pixel_sizes(0, pixel_size)?;
        Ok(self
            .face
            .size_metrics()
            .map(|m| (m.descender >> 6) as f32)
            .unwrap_or(0.0))
    }

    pub fn line_gap(&self, pixel_size: u32) -> Result<f32> {
        self.face.set_pixel_sizes(0, pixel_size)?;
        Ok(self
            .face
            .size_metrics()
            .map(|m| {
                (m.height >> 6) as f32
                    - (self
                        .face
                        .size_metrics()
                        .map(|m| (m.ascender - m.descender) >> 6)
                        .unwrap_or(0) as f32)
            })
            .unwrap_or(0.0))
    }
}
