use crate::{Keyboard as KB, *};

// TODO increment functionality with up/down keys. Would need a way to set a default increment
// for each number type as it would be annoying to have to input this.

#[derive(Clone)]
pub struct SingleLineTextBox {
    pub tb: TextBox,
    pub hook: Rc<RefCell<Option<ValueChangedHook>>>,
}

type ValueChangedHook = Box<dyn FnMut(Context, bool, String) -> EventResponses>;

#[yeehaw_derive::impl_pane_basics_from(tb)]
impl SingleLineTextBox {
    pub fn new(ctx: &Context) -> Self {
        let tb = TextBox::new(ctx, "".to_string()).with_no_wordwrap(ctx);
        Self {
            tb,
            hook: Rc::new(RefCell::new(None)),
        }
    }

    // ---------------------------------------------------------
    // Decorators

    pub fn with_text(self, text: String) -> Self {
        self.tb.set_text(text);
        self
    }

    pub fn set_text(&self, text: String) {
        self.tb.set_text(text);
    }

    pub fn with_text_when_empty<S: Into<String>>(self, text: S) -> Self {
        self.tb.set_text_when_empty(text.into());
        self
    }

    pub fn with_dyn_width(self, width: DynVal) -> Self {
        self.tb.set_dyn_width(width);
        self
    }

    pub fn with_dyn_height(self, height: DynVal) -> Self {
        self.tb.set_dyn_height(height);
        self
    }

    pub fn with_hook(self, hook: Box<dyn FnMut(Context, bool, String) -> EventResponses>) -> Self {
        *self.hook.borrow_mut() = Some(hook);
        self
    }

    pub fn set_hook(&self, hook: Box<dyn FnMut(Context, bool, String) -> EventResponses>) {
        *self.hook.borrow_mut() = Some(hook);
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.tb.set_at(loc_x.into(), loc_y.into());
        self
    }

    pub fn set_at(&self, loc_x: DynVal, loc_y: DynVal) {
        self.tb.set_at(loc_x, loc_y);
    }

    // ---------------------------------------------------------
}

#[yeehaw_derive::impl_element_from(tb)]
impl Element for SingleLineTextBox {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::KeyCombo(ref ke) => {
                if self.tb.pane.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }

                match true {
                    _ if ke[0] == KB::KEY_ESC => {
                        if let Some(hook) = &mut *self.hook.borrow_mut() {
                            let resps = hook(ctx.clone(), true, self.tb.get_text());
                            self.tb.pane.set_selectability(Selectability::Ready, false);
                            return (true, resps);
                        }
                        (false, EventResponses::default())
                    }
                    _ if ke[0] == KB::KEY_ENTER => {
                        if let Some(hook) = &mut *self.hook.borrow_mut() {
                            let resps = hook(ctx.clone(), false, self.tb.get_text());
                            self.tb.pane.set_selectability(Selectability::Ready, false);
                            return (true, resps);
                        }
                        (false, EventResponses::default())
                    }
                    _ => self.tb.receive_event(ctx, ev),
                }
            }
            _ => self.tb.receive_event(ctx, ev),
        }
    }
}
