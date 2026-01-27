use std::{collections::HashMap, fs, path::Path};

use fontdue::Font;
use anyhow;

#[derive(Debug)]
pub struct BitmapFont {
    glyphs: HashMap<u32, Glyph>,
    height: usize,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub width: usize,
    pub height: usize,
    pub advance: f32,
    pub bitmap: Vec<u8>, // alpha mask, row-major
}

impl BitmapFont {
    pub fn load_from_path(
        path: impl AsRef<Path>,
        font_size: f32,
        font_weight_px: u32,
    ) -> anyhow::Result<Self> {
        let bytes = fs::read(path)?;
        let font = Font::from_bytes(bytes, fontdue::FontSettings::default())
            .map_err(|e| anyhow::anyhow!(e))?;
        let px_size = font_size.max(4.0);

        let mut glyphs = HashMap::new();
        let charset: Vec<u32> = (32u32..127)
            .chain(1024..1120) // basic Cyrillic block for RU strings
            .chain([1025u32, 1105u32]) // Ё ё
            .collect();

        for cp in charset {
            if let Some(ch) = std::char::from_u32(cp) {
                let (metrics, mut bitmap) = font.rasterize(ch, px_size);
                if metrics.width == 0 || metrics.height == 0 {
                    continue;
                }
                if font_weight_px > 0 {
                    bitmap = thicken(&bitmap, metrics.width, metrics.height, font_weight_px);
                }
                glyphs.insert(
                    cp,
                    Glyph {
                        width: metrics.width,
                        height: metrics.height,
                        advance: metrics.advance_width,
                        bitmap,
                    },
                );
            }
        }

        Ok(Self {
            glyphs,
            height: px_size as usize,
        })
    }

    pub fn bitmap(&self, ch: char) -> Option<&Glyph> {
        let cp = ch.to_uppercase().next().unwrap_or(ch) as u32;
        self.glyphs.get(&cp)
    }

    pub fn advance(&self, ch: char) -> f32 {
        let cp = ch.to_uppercase().next().unwrap_or(ch) as u32;
        self.glyphs
            .get(&cp)
            .map(|g| g.advance)
            .unwrap_or(self.height as f32 * 0.6)
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn measure_text(&self, text: &str) -> (f32, f32) {
        let width: f32 = text.chars().map(|c| self.advance(c)).sum();
        (width, self.height as f32)
    }
}

fn thicken(bitmap: &[u8], width: usize, height: usize, weight: u32) -> Vec<u8> {
    if weight == 0 {
        return bitmap.to_vec();
    }
    let mut out = bitmap.to_vec();
    let w = width as i32;
    let h = height as i32;
    let r = weight as i32;
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if bitmap[idx] > 0 {
                for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && nx < w && ny >= 0 && ny < h {
                            let nidx = (ny * w + nx) as usize;
                            out[nidx] = out[nidx].max(255);
                        }
                    }
                }
            }
        }
    }
    out
}
