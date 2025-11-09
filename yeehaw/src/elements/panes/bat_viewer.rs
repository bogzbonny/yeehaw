use crate::*;

#[derive(Clone)]
pub struct BatViewer {
    pane: PaneScrollable,
}

impl BatViewer {
    pub fn new(ctx: &Context, width: usize, height: usize) -> Self {
        Self {
            pane: PaneScrollable::new(ctx, width, height),
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for BatViewer {}
