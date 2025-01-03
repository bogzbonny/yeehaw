use crate::*;

/// displays the size
#[derive(Clone)]
pub struct DebugSizePane {
    pub pane: Pane,
    pub text: Rc<RefCell<String>>,
    pub text_sty: Rc<RefCell<Option<Style>>>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl DebugSizePane {
    pub fn new(ctx: &Context) -> DebugSizePane {
        DebugSizePane {
            pane: Pane::new(ctx, "debug_size_pane"),
            text: Rc::new(RefCell::new(String::new())),
            text_sty: Rc::new(RefCell::new(None)),
        }
        .with_dyn_height(DynVal::FULL)
        .with_dyn_width(DynVal::FULL)
    }

    pub fn with_text(self, text: String) -> Self {
        *self.text.borrow_mut() = text;
        self
    }

    pub fn with_text_style(self, sty: Style) -> Self {
        *self.text_sty.borrow_mut() = Some(sty);
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for DebugSizePane {
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        let size = dr.size;
        let s = format!("{}x{} {}", size.width, size.height, self.text.borrow());
        let sty = if let Some(sty) = &*self.text_sty.borrow() {
            sty.clone()
        } else {
            self.pane.get_style()
        };
        let content = DrawChs2D::from_string(s, sty);
        if !force_update && *self.pane.get_content() == content {
            return vec![];
        }
        self.pane.set_content(content);
        self.pane.drawing(ctx, dr, force_update)
    }
}
