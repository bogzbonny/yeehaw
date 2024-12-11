use {
    crate::{Keyboard as KB, *},
    std::{cmp::PartialOrd, fmt::Display, str::FromStr},
};

#[derive(Clone)]
pub struct NumbersTextBox<N> {
    pub tb: TextBox,
    pub value: Rc<RefCell<N>>,
    pub max_value: Rc<RefCell<Option<N>>>,
    /// if set, the minimum value the number can be
    pub min_value: Rc<RefCell<Option<N>>>,
    pub value_changed_hook: Rc<RefCell<Option<ValueChangedHook<N>>>>,
}

type ValueChangedHook<N> = Box<dyn FnMut(N) -> EventResponses>;

impl<N: Display + Clone + Copy + FromStr + PartialOrd + 'static> NumbersTextBox<N> {
    pub fn new(ctx: &Context, starting_value: N) -> Self {
        let tb = TextBox::new(ctx, format!("{}", starting_value)).with_width(DynVal::new_fixed(5));
        let ntb = Self {
            tb,
            value: Rc::new(RefCell::new(starting_value)),
            max_value: Rc::new(RefCell::new(None)),
            min_value: Rc::new(RefCell::new(None)),
            value_changed_hook: Rc::new(RefCell::new(None)),
        };

        let ntb_ = ntb.clone();
        ntb.tb
            .pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                let sel = ntb_.tb.pane.get_selectability();
                if sel != Selectability::Selected {
                    ntb_.restore_value();
                }
                ntb_.tb.post_hook_for_set_selectability();
            }));
        ntb
    }

    // ---------------------------------------------------------
    /// Decorators

    pub fn with_min(self, min: N) -> Self {
        *self.min_value.borrow_mut() = Some(min);
        self
    }

    pub fn without_min(self) -> Self {
        *self.min_value.borrow_mut() = None;
        self
    }

    pub fn with_max(self, max: N) -> Self {
        *self.max_value.borrow_mut() = Some(max);
        self
    }

    pub fn without_max(self) -> Self {
        *self.max_value.borrow_mut() = None;
        self
    }

    pub fn with_width(mut self, width: DynVal) -> Self {
        self.tb = self.tb.with_width(width);
        self
    }

    pub fn with_height(mut self, height: DynVal) -> Self {
        self.tb = self.tb.with_height(height);
        self
    }

    pub fn with_size(mut self, width: DynVal, height: DynVal) -> Self {
        self.tb = self.tb.with_size(width, height);
        self
    }

    pub fn with_cursor_style(mut self, style: Style) -> Self {
        self.tb = self.tb.with_cursor_style(style);
        self
    }

    pub fn with_value_changed_hook(self, hook: ValueChangedHook<N>) -> Self {
        *self.value_changed_hook.borrow_mut() = Some(hook);
        self
    }

    pub fn with_styles(mut self, styles: SelStyles) -> Self {
        self.tb = self.tb.with_styles(styles);
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.tb.set_at(loc_x.into(), loc_y.into());
        self
    }

    // ---------------------------------------------------------

    pub fn change_value(&self, mut new_value: N) {
        // correct bounds on value
        if let Some(min) = *self.min_value.borrow() {
            new_value = num_traits::clamp_min(new_value, min);
        }
        if let Some(max) = *self.max_value.borrow() {
            new_value = num_traits::clamp_max(new_value, max);
        }

        *self.value.borrow_mut() = new_value;
        self.tb.set_text(format!("{}", new_value));

        self.value_changed_hook
            .borrow_mut()
            .as_mut()
            .map(|hook| hook(new_value));

        self.tb.set_cursor_pos(self.tb.get_text().chars().count());
    }

    pub fn update_value_from_tb(&self) {
        let value_str = self.tb.get_text();
        let value = value_str.parse::<N>();
        if let Ok(value) = value {
            self.change_value(value);
        } else {
            let old_value = *self.value.borrow();
            self.change_value(old_value);
        }
    }

    pub fn restore_value(&self) {
        let old_value = *self.value.borrow();
        self.change_value(old_value);
    }
}

#[yeehaw_derive::impl_element_from(tb)]
impl<N: Display + Clone + Copy + FromStr + PartialOrd + 'static> Element for NumbersTextBox<N> {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::KeyCombo(ref ke) => {
                if self.tb.pane.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }

                match true {
                    //_ if ke[0] == KB::KEY_UP => {
                    //    let old_value = *self.value.borrow();
                    //    self.change_value(ctx, old_value + *self.button_increment.borrow());
                    //    (true, EventResponses::default())
                    //}
                    //_ if ke[0] == KB::KEY_DOWN => {
                    //    let old_value = *self.value.borrow();
                    //    self.change_value(ctx, old_value - *self.button_increment.borrow());
                    //    (true, EventResponses::default())
                    //}
                    _ if ke[0] == KB::KEY_ENTER => {
                        self.update_value_from_tb();
                        (true, EventResponses::default())
                    }
                    _ if ke[0] == KB::KEY_SHIFT_ENTER => (true, EventResponses::default()),
                    _ => self.tb.receive_event(ctx, ev),
                }
            }
            _ => self.tb.receive_event(ctx, ev),
        }
    }
}
