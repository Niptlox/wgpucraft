use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ui::elements::UiElement;
use crate::ui::font::BitmapFont;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Val {
    Px(f32),
    Percent(f32),
}

impl Val {
    pub fn resolve(&self, parent: f32) -> f32 {
        match self {
            Val::Px(v) => *v,
            Val::Percent(p) => parent * *p,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RectSpec {
    pub x: Val,
    pub y: Val,
    pub w: Val,
    pub h: Val,
}

impl RectSpec {
    pub fn resolve(&self, parent: [f32; 2]) -> [f32; 4] {
        let pw = parent[0];
        let ph = parent[1];

        let rx = self.x.resolve(pw);
        let ry = self.y.resolve(ph);
        let rw = self.w.resolve(pw);
        let rh = self.h.resolve(ph);

        [rx, ry, rw, rh]
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Anchors {
    pub left: Option<Val>,
    pub right: Option<Val>,
    pub top: Option<Val>,
    pub bottom: Option<Val>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Layout {
    Absolute {
        rect: RectSpec,
        #[serde(default)]
        anchor: Option<Anchors>,
    },
    FlexRow {
        gap: f32,
        padding: f32,
        #[serde(default = "Align::default_align")]
        align: Align,
    },
    FlexColumn {
        gap: f32,
        padding: f32,
        #[serde(default = "Align::default_align")]
        align: Align,
    },
}

impl Align {
    fn default_align() -> Align {
        Align::Start
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiNode {
    pub id: Option<String>,
    pub layout: Layout,
    #[serde(default)]
    pub children: Vec<UiNode>,
    pub element: Option<UiElement>,
}

#[derive(Clone, Debug)]
pub struct ResolvedNode {
    pub id: Option<String>,
    pub rect: [f32; 4],
    pub element: Option<UiElement>,
}

#[derive(Clone)]
pub struct MeasureCtx {
    pub font: Option<Arc<BitmapFont>>,
    pub text_scale: f32,
}

impl Default for MeasureCtx {
    fn default() -> Self {
        Self {
            font: None,
            text_scale: 1.0,
        }
    }
}

impl UiNode {
    pub fn resolve_tree(&self, parent_size: [f32; 2], ctx: &MeasureCtx) -> Vec<ResolvedNode> {
        let mut out = Vec::new();
        self.compute_layout([0.0, 0.0], parent_size, ctx, &mut out);
        out
    }

    pub fn preferred_size(&self, parent_size: [f32; 2], ctx: &MeasureCtx) -> [f32; 2] {
        match &self.layout {
            Layout::Absolute { rect, .. } => {
                let r = rect.resolve(parent_size);
                [r[2], r[3]]
            }
            Layout::FlexColumn { gap, padding, .. } => {
                let inner_parent = [
                    (parent_size[0] - padding * 2.0).max(0.0),
                    (parent_size[1] - padding * 2.0).max(0.0),
                ];
                let mut height: f32 = 0.0;
                let mut width: f32 = 0.0;
                for child in &self.children {
                    let size = child.preferred_size(inner_parent, ctx);
                    height += size[1];
                    width = width.max(size[0]);
                }
                if !self.children.is_empty() {
                    height += *gap * (self.children.len() as f32 - 1.0);
                }
                [width + padding * 2.0, height + padding * 2.0]
            }
            Layout::FlexRow { gap, padding, .. } => {
                let inner_parent = [
                    (parent_size[0] - padding * 2.0).max(0.0),
                    (parent_size[1] - padding * 2.0).max(0.0),
                ];
                let mut width: f32 = 0.0;
                let mut height: f32 = 0.0;
                for child in &self.children {
                    let size = child.preferred_size(inner_parent, ctx);
                    width += size[0];
                    height = height.max(size[1]);
                }
                if !self.children.is_empty() {
                    width += *gap * (self.children.len() as f32 - 1.0);
                }
                [width + padding * 2.0, height + padding * 2.0]
            }
        }
    }

    fn compute_layout(
        &self,
        origin: [f32; 2],
        parent_size: [f32; 2],
        ctx: &MeasureCtx,
        out: &mut Vec<ResolvedNode>,
    ) {
        match &self.layout {
            Layout::Absolute { rect, anchor } => {
                let mut r = rect.resolve(parent_size);
                if let Some(anchor) = anchor {
                    Self::apply_anchor(&mut r, parent_size, anchor);
                }
                let rect_world = [origin[0] + r[0], origin[1] + r[1], r[2], r[3]];
                out.push(ResolvedNode {
                    id: self.id.clone(),
                    rect: rect_world,
                    element: self.element.clone(),
                });
                for child in &self.children {
                    child.compute_layout(
                        [rect_world[0], rect_world[1]],
                        [r[2], r[3]],
                        ctx,
                        out,
                    );
                }
            }
            Layout::FlexColumn { gap, padding, align } => {
                let inner_w = (parent_size[0] - padding * 2.0).max(0.0);
                if let Some(el) = &self.element {
                    out.push(ResolvedNode {
                        id: self.id.clone(),
                        rect: [origin[0], origin[1], parent_size[0], parent_size[1]],
                        element: Some(el.clone()),
                    });
                }
                let mut cursor_y = *padding;
                for child in &self.children {
                    let child_size = child.preferred_size([inner_w, parent_size[1]], ctx);
                    let width = match align {
                        Align::Stretch => inner_w,
                        Align::Start => child_size[0].min(inner_w),
                        Align::Center => child_size[0].min(inner_w),
                        Align::End => child_size[0].min(inner_w),
                    };
                    let height = child_size[1];
                    let x = match align {
                        Align::Stretch => *padding,
                        Align::Start => *padding,
                        Align::Center => *padding + (inner_w - width) * 0.5,
                        Align::End => *padding + (inner_w - width),
                    };
                    let rect_world = [
                        origin[0] + x,
                        origin[1] + cursor_y,
                        width,
                        height,
                    ];
                    child.compute_layout([rect_world[0], rect_world[1]], [width, height], ctx, out);
                    cursor_y += height + gap;
                }
            }
            Layout::FlexRow { gap, padding, align } => {
                let inner_h = (parent_size[1] - padding * 2.0).max(0.0);
                if let Some(el) = &self.element {
                    out.push(ResolvedNode {
                        id: self.id.clone(),
                        rect: [origin[0], origin[1], parent_size[0], parent_size[1]],
                        element: Some(el.clone()),
                    });
                }
                let mut cursor_x = *padding;
                for child in &self.children {
                    let child_size = child.preferred_size(parent_size, ctx);
                    let height = match align {
                        Align::Stretch => inner_h,
                        Align::Start => child_size[1].min(inner_h),
                        Align::Center => child_size[1].min(inner_h),
                        Align::End => child_size[1].min(inner_h),
                    };
                    let width = child_size[0];
                    let y = match align {
                        Align::Stretch => *padding,
                        Align::Start => *padding,
                        Align::Center => *padding + (inner_h - height) * 0.5,
                        Align::End => *padding + (inner_h - height),
                    };
                    let rect_world = [
                        origin[0] + cursor_x,
                        origin[1] + y,
                        width,
                        height,
                    ];
                    child.compute_layout([rect_world[0], rect_world[1]], [width, height], ctx, out);
                    cursor_x += width + gap;
                }
            }
        }
    }

    fn apply_anchor(rect: &mut [f32; 4], parent: [f32; 2], anchor: &Anchors) {
        if let (Some(left), Some(right)) = (anchor.left, anchor.right) {
            let l = left.resolve(parent[0]);
            let r = right.resolve(parent[0]);
            rect[0] = l;
            rect[2] = (parent[0] - l - r).max(0.0);
        } else if let Some(left) = anchor.left {
            rect[0] = left.resolve(parent[0]);
        } else if let Some(right) = anchor.right {
            rect[0] = parent[0] - right.resolve(parent[0]) - rect[2];
        }

        if let (Some(top), Some(bottom)) = (anchor.top, anchor.bottom) {
            let t = top.resolve(parent[1]);
            let b = bottom.resolve(parent[1]);
            rect[1] = t;
            rect[3] = (parent[1] - t - b).max(0.0);
        } else if let Some(top) = anchor.top {
            rect[1] = top.resolve(parent[1]);
        } else if let Some(bottom) = anchor.bottom {
            rect[1] = parent[1] - bottom.resolve(parent[1]) - rect[3];
        }
    }
}
