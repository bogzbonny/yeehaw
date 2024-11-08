use {
    crate::{
        Context, DrawCh, DrawChPos, DrawChs2D, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Pane, Parent, Priority, ReceivableEventChanges, SelfReceivableEvents,
        Style, ZIndex,
    },
    std::{cell::RefCell, rc::Rc},
};

/// displays the size
#[derive(Clone)]
pub struct DebugSizePane {
    pub pane: Pane,
    pub sty: Rc<RefCell<Style>>,
    pub text: Rc<RefCell<String>>,
}

impl DebugSizePane {
    pub fn new(ctx: &Context) -> DebugSizePane {
        DebugSizePane {
            pane: Pane::new(ctx, "debug_size_pane"),
            sty: Rc::new(RefCell::new(Style::default())),
            text: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn with_text(self, text: String) -> Self {
        *self.text.borrow_mut() = text;
        self
    }

    pub fn with_height(self, h: DynVal) -> Self {
        self.pane.set_dyn_height(h);
        self
    }

    pub fn with_style(self, style: Style) -> Self {
        *self.sty.borrow_mut() = style;
        self
    }

    pub fn with_default_ch(self, ch: DrawCh) -> Self {
        self.pane.set_default_ch(ch);
        self
    }

    pub fn with_width(self, w: DynVal) -> Self {
        self.pane.set_dyn_width(w);
        self
    }

    pub fn with_z(self, z: ZIndex) -> Self {
        self.pane.set_z(z);
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for DebugSizePane {
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let size = ctx.s;
        let s = format!("{}x{} {}", size.width, size.height, self.text.borrow());
        let sty = self.sty.borrow().clone();
        let content = DrawChs2D::from_string(s, sty);
        self.pane.set_content(content);
        self.pane.drawing(ctx)
    }
}
