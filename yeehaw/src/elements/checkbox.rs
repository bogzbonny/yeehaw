use {
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct Checkbox {
    pub pane: SelectablePane,
    pub checked: Rc<RefCell<bool>>,
    /// whether the checkbox is checked or not
    pub clicked_down: Rc<RefCell<bool>>,
    /// activated when mouse is clicked down while over object

    /// rune to use for the checkmark
    /// recommended:  √ X x ✖
    pub checkmark: Rc<RefCell<char>>,

    /// function which executes when checkbox is checked or unchecked,
    /// bool is the new state of the checkbox (true = checked)
    pub clicked_fn: Rc<RefCell<dyn FnMut(Context, bool) -> EventResponses>>,
}

impl Checkbox {
    const KIND: &'static str = "widget_checkbox";

    const STYLE: SelStyles = SelStyles {
        //selected_style: Style::new_const(Color::BLACK, Color::LIGHT_YELLOW2)
        selected_style: Style::new_const(Color::BLACK, Color::YELLOW)
            .with_attr(Attributes::new().with_bold()),
        ready_style: Style::new_const(Color::BLACK, Color::WHITE)
            .with_attr(Attributes::new().with_bold()),
        unselectable_style: Style::new_const(Color::BLACK, Color::GREY13)
            .with_attr(Attributes::new().with_bold()),
    };

    pub fn default_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![(KB::KEY_ENTER.into(), Priority::Focused)]) // / when "active" hitting enter will click the button
    }

    pub fn new(ctx: &Context) -> Self {
        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_self_receivable_events(Self::default_receivable_events())
            .with_styles(Self::STYLE)
            .with_dyn_width(DynVal::new_fixed(1))
            .with_dyn_height(DynVal::new_fixed(1));
        let cb = Checkbox {
            pane,
            checked: Rc::new(RefCell::new(false)),
            clicked_down: Rc::new(RefCell::new(false)),
            checkmark: Rc::new(RefCell::new('√')),
            clicked_fn: Rc::new(RefCell::new(|_, _| EventResponses::default())),
        };

        let cb_ = cb.clone();
        cb.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                cb_.pane.set_content_style(cb_.pane.get_current_style());
            }));
        cb
    }

    // ----------------------------------------------
    /// decorators

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self
    }

    pub fn with_clicked_fn(
        mut self, clicked_fn: Box<dyn FnMut(Context, bool) -> EventResponses>,
    ) -> Self {
        self.clicked_fn = Rc::new(RefCell::new(clicked_fn));
        self
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.pane.set_at(loc_x, loc_y);
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
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Checkbox {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        if captured {
            return (true, resps);
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
