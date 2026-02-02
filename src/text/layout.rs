use glam::Vec2;

use crate::text::font::{Font, GlyphKey, RenderMode};

#[derive(Clone, Debug)]
pub struct PlacedGlyph {
    pub key: GlyphKey,
    pub position: Vec2, // baseline based
}

pub fn layout_line(text: &str, font: &Font, px: u32) -> Vec<PlacedGlyph> {
    let mut out = Vec::new();
    let mut cursor_x = 0.0f32;
    for ch in text.chars() {
        let glyph_id = font.glyph_index_for_char(ch);
        let key = GlyphKey {
            font_id: font.id,
            glyph_id,
            pixel_size: px,
            render_mode: RenderMode::Normal,
        };
        out.push(PlacedGlyph {
            key,
            position: Vec2::new(cursor_x, 0.0),
        });
        if let Ok(Some(glyph)) = font.load_glyph_bitmap(glyph_id, px) {
            cursor_x += glyph.advance;
        } else {
            cursor_x += px as f32 * 0.5;
        }
    }
    out
}
