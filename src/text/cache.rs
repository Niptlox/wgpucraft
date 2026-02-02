use std::collections::HashMap;

use anyhow::Result;

use crate::text::{atlas::GlyphAtlasEntry, font::{GlyphBitmap, GlyphKey}};
use wgpu::Queue;

pub struct CachedGlyph {
    pub atlas: GlyphAtlasEntry,
}

pub struct GlyphCache {
    entries: HashMap<GlyphKey, CachedGlyph>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &GlyphKey) -> Option<&CachedGlyph> {
        self.entries.get(key)
    }

    pub fn get_or_insert<F: FnOnce() -> Result<Option<GlyphBitmap>>>(
        &mut self,
        queue: &Queue,
        atlas: &mut crate::text::atlas::GlyphAtlas,
        key: GlyphKey,
        rasterize: F,
    ) -> Result<Option<&CachedGlyph>> {
        if self.entries.contains_key(&key) {
            return Ok(self.entries.get(&key));
        }
        if let Some(bitmap) = rasterize()? {
            let entry = atlas.add_glyph(
                queue,
                &bitmap.buffer,
                bitmap.width,
                bitmap.height,
                [bitmap.bearing_x, bitmap.bearing_y],
                bitmap.advance,
            )?;
            self.entries.insert(key, CachedGlyph { atlas: entry });
            Ok(self.entries.get(&key))
        } else {
            Ok(None)
        }
    }
}
