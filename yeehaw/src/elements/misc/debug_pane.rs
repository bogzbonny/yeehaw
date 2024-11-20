use {
    crate::*,
    std::{cell::RefCell, rc::Rc},
};

/// displays the size
#[derive(Clone)]
pub struct DebugSizePane {
    pub pane: Pane,
    pub text: Rc<RefCell<String>>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl DebugSizePane {
    pub fn new(ctx: &Context) -> DebugSizePane {
        DebugSizePane {
            pane: Pane::new(ctx, "debug_size_pane"),
            text: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn with_text(self, text: String) -> Self {
        *self.text.borrow_mut() = text;
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for DebugSizePane {
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let size = ctx.s;
        let s = format!("{}x{} {}", size.width, size.height, self.text.borrow());
        let sty = self.pane.get_style();
        let content = DrawChs2D::from_string(s, sty);
        self.pane.set_content(content);
        self.pane.drawing(ctx)
    }
}
