use {
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct Toggle {
    pub pane: SelectablePane,
    pub left: Rc<RefCell<String>>,
    pub right: Rc<RefCell<String>>,
    pub left_selected: Rc<RefCell<bool>>,
    /// otherwise right is selected
    pub clicked_down: Rc<RefCell<bool>>,
    /// activated when mouse is clicked down while over button
    pub selected_sty: Rc<RefCell<Style>>,
    ///                                   selected
    pub toggled_fn: Rc<RefCell<dyn FnMut(Context, String) -> EventResponses>>,
}

impl Toggle {
    const KIND: &'static str = "widget_button";

    const STYLE: SelStyles = SelStyles {
        selected_style: Style::new_const(Color::BLACK, Color::LIGHT_YELLOW2),
        ready_style: Style::new_const(Color::BLACK, Color::WHITE),
        unselectable_style: Style::new_const(Color::BLACK, Color::GREY13),
    };

    /// for the selected toggle
    const DEFAULT_SELECTED_STY: Style = Style::new_const(Color::BLACK, Color::LIGHT_BLUE);

    pub fn default_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KB::KEY_ENTER.into(), Priority::Focused),
            (KB::KEY_LEFT.into(), Priority::Focused),
            (KB::KEY_RIGHT.into(), Priority::Focused),
            (KB::KEY_H.into(), Priority::Focused),
            (KB::KEY_L.into(), Priority::Focused),
        ])
    }

    pub fn new(
        ctx: &Context, left: String, right: String,
        toggeld_fn: Box<dyn FnMut(Context, String) -> EventResponses>,
    ) -> Self {
        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_self_receivable_events(Self::default_receivable_events())
            .with_styles(Self::STYLE)
            .with_dyn_width(DynVal::new_fixed(
                left.chars().count() as i32 + right.chars().count() as i32,
            ))
            .with_dyn_height(DynVal::new_fixed(1));

        pane.set_content_from_string(&(left.clone() + &right));

        Toggle {
            pane,
            left: Rc::new(RefCell::new(left)),
            right: Rc::new(RefCell::new(right)),
            left_selected: Rc::new(RefCell::new(true)),
            clicked_down: Rc::new(RefCell::new(false)),
            selected_sty: Rc::new(RefCell::new(Self::DEFAULT_SELECTED_STY)),
            toggled_fn: Rc::new(RefCell::new(toggeld_fn)),
        }
    }

    // ----------------------------------------------
    /// decorators

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.pane.set_at(loc_x, loc_y);
        self
    }

    // ----------------------------------------------

    pub fn selected(&self) -> String {
        if *self.left_selected.borrow() {
            return self.left.borrow().clone();
        }
        self.right.borrow().clone()
    }

    pub fn perform_toggle(&self, ctx: &Context) -> EventResponses {
        let l_sel = *self.left_selected.borrow();
        *self.left_selected.borrow_mut() = !l_sel;
        self.toggled_fn.borrow_mut()(ctx.clone(), self.selected())
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Toggle {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        if captured {
            return (true, resps);
        }
        match ev {
            Event::KeyCombo(ke) => {
                if self.pane.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, resps);
                }
                match true {
                    _ if ke[0] == KB::KEY_ENTER => {
                        let resps_ = self.perform_toggle(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ if ke[0] == KB::KEY_LEFT || ke[0] == KB::KEY_H => {
                        if !*self.left_selected.borrow() {
                            let resps_ = self.perform_toggle(ctx);
                            resps.extend(resps_);
                            return (true, resps);
                        }
                        return (true, resps);
                    }
                    _ if ke[0] == KB::KEY_RIGHT || ke[0] == KB::KEY_L => {
                        if *self.left_selected.borrow() {
                            let resps_ = self.perform_toggle(ctx);
                            resps.extend(resps_);
                            return (true, resps);
                        }
                        return (true, resps);
                    }
                    _ => {}
                }
                return (false, resps);
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
                        let x = me.column as usize;
                        let left_sel = *self.left_selected.borrow();
                        if (!left_sel && x < self.left.borrow().chars().count())
                            || (left_sel && x >= self.left.borrow().chars().count())
                        {
                            let resps_ = self.perform_toggle(ctx);
                            resps.extend(resps_);
                            return (true, resps);
                        }
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }
                return (false, resps);
            }
            _ => {}
        }
        (false, resps)
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        // need to re set the content in order to reflect active style
        let left = self.left.borrow();
        let right = self.right.borrow();
        let left_len = left.chars().count();
        let right_len = right.chars().count();
        self.pane.set_content_from_string(&(left.clone() + &right));
        if *self.left_selected.borrow() {
            for i in 0..left_len {
                self.pane.pane.pane.content.borrow_mut()[0][i].style =
                    self.selected_sty.borrow().clone();
            }
        } else {
            for i in left_len..left_len + right_len {
                self.pane.pane.pane.content.borrow_mut()[0][i].style =
                    self.selected_sty.borrow().clone();
            }
        }
        self.pane.drawing(ctx)
    }
}
