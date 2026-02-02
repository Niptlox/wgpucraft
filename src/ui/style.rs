use anyhow::Result;
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct MenuStyle {
    pub panel_color: (u8, u8, u8, u8),
    pub button_padding: f32,
    pub button_height: f32,
    pub gap: f32,
    pub padding: f32,
    pub title_size: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StyleSheet {
    pub menu: MenuStyle,
}

impl StyleSheet {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let raw = fs::read_to_string(path)?;
        let sheet: StyleSheet = ron::from_str(&raw)?;
        Ok(sheet)
    }
}
