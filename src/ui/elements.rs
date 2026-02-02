use serde::{Deserialize, Serialize};

use crate::ui::MeasureCtx;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabelSpec {
    pub text: String,
    #[serde(default = "LabelSpec::default_font_size")]
    pub font_size: f32,
}

impl LabelSpec {
    fn default_font_size() -> f32 {
        16.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::BitmapFont;
    use std::sync::Arc;

    #[test]
    fn label_pref_size_scales_with_font() {
        let font = Arc::new(
            BitmapFont::load_from_path("assets/fonts/OpenLukyanov.ttf", 16.0, 400)
                .expect("font available for tests"),
        );
        let ctx = MeasureCtx {
            font: Some(font),
            text_scale: 1.0,
        };
        let spec = LabelSpec {
            text: "Test".into(),
            font_size: 16.0,
        };
        let el = UiElement::Label(spec);
        let sz = el.preferred_size(&ctx);
        // height should be at least font size, width positive
        assert!(sz[0] > 0.0);
        assert!(sz[1] >= 16.0);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonSpec {
    pub text: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default = "ButtonSpec::default_padding")]
    pub padding: f32,
    #[serde(default = "ButtonSpec::default_height")]
    pub min_height: f32,
}

impl ButtonSpec {
    fn default_padding() -> f32 {
        8.0
    }

    fn default_height() -> f32 {
        48.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UiElement {
    Label(LabelSpec),
    Button(ButtonSpec),
    Panel {
        #[serde(default = "UiElement::default_panel_color")]
        color: [u8; 4],
    },
    Image {
        #[serde(default)]
        uv: [f32; 4],
    },
    Spacer {
        size: f32,
    },
}

impl UiElement {
    fn default_panel_color() -> [u8; 4] {
        [24, 26, 30, 200]
    }

    pub fn preferred_size(&self, ctx: &MeasureCtx) -> [f32; 2] {
        match self {
            UiElement::Label(label) => {
                if let Some(font) = &ctx.font {
                    let (w, h) = font.measure_text(&label.text);
                    let scale = (label.font_size / font.height() as f32) * ctx.text_scale;
                    [w * scale, h * scale]
                } else {
                    let scale = ctx.text_scale;
                    [
                        label.text.len() as f32 * 8.0 * scale,
                        label.font_size * scale,
                    ]
                }
            }
            UiElement::Button(button) => {
                if let Some(font) = &ctx.font {
                    let (w, h) = font.measure_text(&button.text);
                    let detail_w = button
                        .detail
                        .as_ref()
                        .map(|d| font.measure_text(d).0)
                        .unwrap_or(0.0);
                    let text_w = w.max(detail_w);
                    let text_h = if button.detail.is_some() { h * 2.0 } else { h };
                    let scale = ctx.text_scale;
                    [
                        text_w * scale + button.padding * 2.0,
                        (text_h * scale + button.padding * 2.0).max(button.min_height * scale),
                    ]
                } else {
                    let scale = ctx.text_scale;
                    [
                        (button.text.len() as f32) * 10.0 * scale + button.padding * 2.0,
                        button.min_height * scale,
                    ]
                }
            }
            UiElement::Panel { .. } => [0.0, 0.0],
            UiElement::Image { .. } => [0.0, 0.0],
            UiElement::Spacer { size } => [0.0, *size],
        }
    }
}
