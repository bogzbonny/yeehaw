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
    /// otherwise right is selected
    pub left_selected: Rc<RefCell<bool>>,
    /// activated when mouse is clicked down while over button
    pub clicked_down: Rc<RefCell<bool>>,
    /// the style of the selected side
    pub selected_sty: Rc<RefCell<Style>>,
    ///                                   selected
    pub toggled_fn: Rc<RefCell<ToggleFn>>,
}

pub type ToggleFn = Box<dyn FnMut(Context, Toggle) -> EventResponses>;

impl Toggle {
    const KIND: &'static str = "toggle";

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

    pub fn new<S: Into<String>>(ctx: &Context, left: S, right: S) -> Self {
        let (left, right) = (left.into(), right.into());
        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_self_receivable_events(Self::default_receivable_events())
            .with_styles(Self::STYLE)
            .with_dyn_width(DynVal::new_fixed(
                left.chars().count() as i32 + right.chars().count() as i32,
            ))
            .with_dyn_height(DynVal::new_fixed(1));

        pane.set_content_from_string(&(left.clone() + &right));

        let t = Toggle {
            pane,
            left: Rc::new(RefCell::new(left)),
            right: Rc::new(RefCell::new(right)),
            left_selected: Rc::new(RefCell::new(true)),
            clicked_down: Rc::new(RefCell::new(false)),
            selected_sty: Rc::new(RefCell::new(Self::DEFAULT_SELECTED_STY)),
            toggled_fn: Rc::new(RefCell::new(Box::new(|_, _| EventResponses::default()))),
        };
        t.update_content();

        let t_ = t.clone();
        t.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                t_.update_content();
            }));
        t
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    pub fn with_fn(self, toggle_fn: ToggleFn) -> Self {
        *self.toggled_fn.borrow_mut() = toggle_fn;
        self
    }

    pub fn set_fn(&self, toggle_fn: ToggleFn) {
        *self.toggled_fn.borrow_mut() = toggle_fn;
    }

    // ----------------------------------------------

    pub fn is_left(&self) -> bool {
        *self.left_selected.borrow()
    }

    pub fn selected(&self) -> String {
        if *self.left_selected.borrow() {
            return self.left.borrow().clone();
        }
        self.right.borrow().clone()
    }

    pub fn perform_toggle(&self, ctx: &Context) -> EventResponses {
        let l_sel = *self.left_selected.borrow();
        *self.left_selected.borrow_mut() = !l_sel;
        let resps = self.toggled_fn.borrow_mut()(ctx.clone(), self.clone());
        self.update_content();
        resps
    }

    pub fn update_content(&self) {
        // need to re set the content in order to reflect active style
        self.pane.set_style(self.pane.get_current_style());
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
}
