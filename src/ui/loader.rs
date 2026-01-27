use std::path::Path;

use crate::ui::layout::UiNode;

pub fn load_ron(path: impl AsRef<Path>) -> anyhow::Result<UiNode> {
    let raw = std::fs::read_to_string(path)?;
    let node: UiNode = ron::from_str(&raw)?;
    Ok(node)
}

pub fn load_json(path: impl AsRef<Path>) -> anyhow::Result<UiNode> {
    let raw = std::fs::read_to_string(path)?;
    let node: UiNode = serde_json::from_str(&raw)?;
    Ok(node)
}
