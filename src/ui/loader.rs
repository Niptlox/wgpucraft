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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn walk_ids(node: &UiNode, acc: &mut HashSet<String>) {
        if let Some(id) = &node.id {
            acc.insert(id.clone());
        }
        for child in &node.children {
            walk_ids(child, acc);
        }
    }

    #[test]
    fn ron_menu_files_parse() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/ui");
        let files = ["menu_main.ron", "menu_settings.ron", "menu_advanced.ron"];
        for f in files {
            let path = root.join(f);
            let ui = load_ron(&path).expect("RON should parse");
            assert!(ui.id.is_some(), "root id must be set for {}", f);
            // ensure ids are unique within the tree
            let mut ids = HashSet::new();
            walk_ids(&ui, &mut ids);
            assert_eq!(ids.len(), ids.iter().count(), "duplicate ids in {}", f);
        }
    }

    #[test]
    fn minimal_inline_ron_parses() {
        let ron = r#"(id:Some("root"), layout:Absolute(rect:(x:Px(0.0), y:Px(0.0), w:Percent(1.0), h:Percent(1.0)), anchor:None), children:[], element:None)"#;
        let _ui: UiNode = ron::from_str(ron).expect("inline RON should parse");
    }

    #[test]
    fn panel_variant_parses() {
        let value = crate::ui::UiElement::Panel {
            color: [1, 2, 3, 4],
        };
        let ron_str = ron::to_string(&value).expect("serialize panel");
        println!("serialized panel: {}", ron_str);
        let _el: crate::ui::UiElement =
            ron::from_str(&ron_str).expect("panel variant should parse when round-tripped");

        let btn = crate::ui::UiElement::Button(crate::ui::ButtonSpec {
            text: "T".into(),
            detail: Some("D".into()),
            padding: 12.0,
            min_height: 52.0,
        });
        let btn_str = ron::to_string(&btn).expect("serialize button");
        println!("serialized button: {}", btn_str);
        let _btn: crate::ui::UiElement =
            ron::from_str(&btn_str).expect("button variant should parse when round-tripped");
    }
}
