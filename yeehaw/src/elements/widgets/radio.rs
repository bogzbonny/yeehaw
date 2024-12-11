use {
    crate::{Keyboard as KB, Parent, *},
    crossterm::event::{MouseButton, MouseEventKind},
};

// TODO multiline text support for each radio
// TODO option to start with nothing selected

#[derive(Clone)]
pub struct RadioButtons {
    pub pane: SelectablePane,
    pub on_ch: Rc<RefCell<char>>,
    /// ch used for the selected
    pub off_ch: Rc<RefCell<char>>,
    /// ch used for the unselected
    pub radios: Rc<RefCell<Vec<String>>>,
    /// the text for each radio button
    pub clicked_down: Rc<RefCell<bool>>,
    /// activated when mouse is clicked down while over object
    pub selected: Rc<RefCell<usize>>,
    /// which radio button is selected

    /// function which executes when the radio selection is changed
    ///                                           (index, selected)
    #[allow(clippy::type_complexity)]
    pub radio_selected_fn: Rc<RefCell<dyn FnMut(Context, usize, String) -> EventResponses>>,
}

/// inspiration for some radios:
/// ◯ ◉ ◯ ○
/// ◯ ◯   ●
/// ⍟ ◉ ◯ ○

impl RadioButtons {
    const KIND: &'static str = "radio";

    const STYLE: SelStyles = SelStyles {
        selected_style: Style::new_const(Color::YELLOW, Color::TRANSPARENT),
        ready_style: Style::new_const(Color::WHITE, Color::TRANSPARENT),
        unselectable_style: Style::new_const(Color::GREY13, Color::TRANSPARENT),
    };

    pub fn default_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KB::KEY_UP.into(), Priority::Focused),
            (KB::KEY_DOWN.into(), Priority::Focused),
            (KB::KEY_J.into(), Priority::Focused),
            (KB::KEY_K.into(), Priority::Focused),
        ])
    }

    pub fn new(ctx: &Context, radios: Vec<String>) -> Self {
        let max_width = radios.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32 + 1; // +1 for the radio button
        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_self_receivable_events(Self::default_receivable_events())
            .with_styles(Self::STYLE)
            .with_dyn_width(DynVal::new_fixed(max_width))
            .with_dyn_height(DynVal::new_fixed(radios.len() as i32));

        let rb = RadioButtons {
            pane,
            on_ch: Rc::new(RefCell::new('⍟')),
            off_ch: Rc::new(RefCell::new('◯')),
            clicked_down: Rc::new(RefCell::new(false)),
            radios: Rc::new(RefCell::new(radios)),
            selected: Rc::new(RefCell::new(0)),
            radio_selected_fn: Rc::new(RefCell::new(|_, _, _| EventResponses::default())),
        };
        rb.update_content();

        let rb_ = rb.clone();
        rb.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                rb_.update_content();
            }));
        rb
    }

    // ----------------------------------------------
    // decorators

    pub fn with_radio_selected_fn(
        mut self, clicked_fn: Box<dyn FnMut(Context, usize, String) -> EventResponses>,
    ) -> Self {
        self.radio_selected_fn = Rc::new(RefCell::new(clicked_fn));
        self
    }

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    // ----------------------------------------------

    pub fn update_content(&self) {
        // need to re set the content in order to reflect active style
        let selected_i = *self.selected.borrow();
        let s =
            self.radios
                .borrow()
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, radio)| {
                    if i == selected_i {
                        acc.push(*self.on_ch.borrow());
                    } else {
                        acc.push(*self.off_ch.borrow());
                    }
                    acc.push_str(radio);
                    if i != self.radios.borrow().len() - 1 {
                        acc.push('\n');
                    }
                    acc
                });
        self.pane.set_style(self.pane.get_current_style());
        self.pane.set_content_from_string(&s);
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for RadioButtons {
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
                    _ if ke[0] == KB::KEY_DOWN || ke[0] == KB::KEY_J => {
                        if *self.selected.borrow() < self.radios.borrow().len() - 1 {
                            *self.selected.borrow_mut() += 1;
                            let sel_i = *self.selected.borrow();
                            let sel_str = self.radios.borrow()[sel_i].clone();
                            let resps_ =
                                self.radio_selected_fn.borrow_mut()(ctx.clone(), sel_i, sel_str);
                            resps.extend(resps_);
                            self.update_content();
                            return (true, resps);
                        }
                    }
                    _ if ke[0] == KB::KEY_UP || ke[0] == KB::KEY_K => {
                        if *self.selected.borrow() > 0 {
                            *self.selected.borrow_mut() -= 1;
                            let sel_i = *self.selected.borrow();
                            let sel_str = self.radios.borrow()[sel_i].clone();
                            let resps_ =
                                self.radio_selected_fn.borrow_mut()(ctx.clone(), sel_i, sel_str);
                            resps.extend(resps_);
                            self.update_content();
                            return (true, resps);
                        }
                    }
                    _ => {}
                }
                return (false, resps);
            }
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {}

                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        self.update_content();
                        return (true, resps);
                    }
                    MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        *self.clicked_down.borrow_mut() = false;
                        let y = me.row as usize;
                        if y < self.radios.borrow().len() {
                            *self.selected.borrow_mut() = y;
                            let resps_ = self.radio_selected_fn.borrow_mut()(
                                ctx.clone(),
                                y,
                                self.radios.borrow()[y].clone(),
                            );
                            resps.extend(resps_);
                            self.update_content();
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
