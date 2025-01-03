use {
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
};

#[derive(Clone)]
pub struct Checkbox {
    pub pane: SelectablePane,
    /// whether the checkbox is checked or not
    pub checked: Rc<RefCell<bool>>,
    /// activated when mouse is clicked down while over object
    pub clicked_down: Rc<RefCell<bool>>,

    /// rune to use for the checkmark
    /// recommended:  √ X x ✖
    pub checkmark: Rc<RefCell<char>>,

    /// function which executes when checkbox is checked or unchecked,
    /// bool is the new state of the checkbox (true = checked)
    pub clicked_fn: Rc<RefCell<CheckboxFn>>,
}

pub type CheckboxFn = Box<dyn FnMut(Context, bool) -> EventResponses>;

impl Checkbox {
    const KIND: &'static str = "checkbox";

    const STYLE: SelStyles = SelStyles {
        selected_style: Style::new_const(Color::BLACK, Color::YELLOW)
            .with_attrs(Attributes::new().with_bold()),
        ready_style: Style::new_const(Color::BLACK, Color::WHITE)
            .with_attrs(Attributes::new().with_bold()),
        unselectable_style: Style::new_const(Color::BLACK, Color::GREY13)
            .with_attrs(Attributes::new().with_bold()),
    };

    pub fn default_receivable_events() -> ReceivableEvents {
        ReceivableEvents(vec![(KB::KEY_ENTER.into())]) // when "active" hitting enter will click the button
    }

    pub fn new(ctx: &Context) -> Self {
        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_focused_receivable_events(Self::default_receivable_events())
            .with_styles(Self::STYLE)
            .with_dyn_width(DynVal::new_fixed(1))
            .with_dyn_height(DynVal::new_fixed(1));
        let cb = Checkbox {
            pane,
            checked: Rc::new(RefCell::new(false)),
            clicked_down: Rc::new(RefCell::new(false)),
            checkmark: Rc::new(RefCell::new('√')),
            clicked_fn: Rc::new(RefCell::new(Box::new(|_, _| EventResponses::default()))),
        };
        cb.pane.set_content_from_string(' ');
        cb.pane.set_content_style(cb.pane.get_current_style());

        let cb_ = cb.clone();
        cb.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                cb_.pane.set_content_style(cb_.pane.get_current_style());
            }));
        cb
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self.pane.set_content_style(self.pane.get_current_style());
        self
    }

    pub fn with_fn(self, clicked_fn: CheckboxFn) -> Self {
        self.set_fn(clicked_fn);
        self
    }

    pub fn set_fn(&self, clicked_fn: CheckboxFn) {
        *self.clicked_fn.borrow_mut() = clicked_fn;
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    // ----------------------------------------------

    pub fn text(&self) -> char {
        if *self.checked.borrow() {
            return *self.checkmark.borrow();
        }
        ' '
    }

    pub fn click(&self, ctx: &Context) -> EventResponses {
        let checked = !*self.checked.borrow();
        self.checked.replace(checked);
        self.pane.set_style(self.pane.get_current_style());
        self.pane.set_content_from_string(self.text());
        (self.clicked_fn.borrow_mut())(ctx.clone(), checked)
    }

    pub fn is_checked(&self) -> bool {
        *self.checked.borrow()
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Checkbox {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        if captured {
            return (true, resps);
        }
        if self.pane.get_selectability() == Selectability::Unselectable {
            return (false, resps);
        }
        match ev {
            Event::KeyCombo(ke) => {
                if self.pane.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }
                if ke[0] == KB::KEY_ENTER {
                    let resps_ = self.click(ctx);
                    resps.extend(resps_);
                    return (true, resps);
                }
            }
            Event::Mouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, resps);
                    }
                    MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        *self.clicked_down.borrow_mut() = false;
                        let resps_ = self.click(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }
            }
            _ => {}
        }
        (false, resps)
    }
}
