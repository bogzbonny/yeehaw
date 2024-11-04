use {
    super::{Button, Selectability, TextBox, WBStyles, Widget, Widgets},
    crate::{
        Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event, EventResponses,
        KeyPossibility, Keyboard as KB, Parent, Priority, ReceivableEvent, ReceivableEventChanges,
        SelfReceivableEvents, Style,
    },
    std::{cell::RefCell, rc::Rc},
};

// TODO create number finalized hook
// TODO allow integer OR float values

#[derive(Clone)]
pub struct NumbersTextBox {
    pub tb: TextBox,
    pub value: Rc<RefCell<i64>>,
    pub has_buttons: Rc<RefCell<bool>>, // if true, adds up/down buttons to the right of the text box
    pub button_increment: Rc<RefCell<i64>>, // how much to increment/decrement the value by when the up/down buttons are pressed
    pub max_value: Rc<RefCell<Option<i64>>>, // if set, the maximum value the number can be
    pub min_value: Rc<RefCell<Option<i64>>>, // if set, the minimum value the number can be
}

impl NumbersTextBox {
    // for number textboxes which are editable
    pub fn editable_receivable_events() -> Vec<ReceivableEvent> {
        vec![
            KeyPossibility::Chars.into(),
            KB::KEY_BACKSPACE.into(),
            KB::KEY_ENTER.into(),
            KB::KEY_SHIFT_ENTER.into(),
            KB::KEY_LEFT.into(),
            KB::KEY_RIGHT.into(),
            KB::KEY_UP.into(),
            KB::KEY_DOWN.into(),
        ]
    }

    pub fn new(ctx: &Context, starting_value: i64) -> Self {
        let tb = TextBox::new(ctx, format!("{}", starting_value)).with_width(DynVal::new_fixed(5));
        Self {
            tb,
            value: Rc::new(RefCell::new(starting_value)),
            has_buttons: Rc::new(RefCell::new(true)),
            button_increment: Rc::new(RefCell::new(1)),
            max_value: Rc::new(RefCell::new(None)),
            min_value: Rc::new(RefCell::new(None)),
        }
    }

    // ---------------------------------------------------------
    // Decorators

    pub fn with_buttons(self) -> Self {
        *self.has_buttons.borrow_mut() = true;
        self
    }

    pub fn without_buttons(self) -> Self {
        *self.has_buttons.borrow_mut() = false;
        self
    }

    pub fn with_min(self, min: i64) -> Self {
        *self.min_value.borrow_mut() = Some(min);
        self
    }

    pub fn without_min(self) -> Self {
        *self.min_value.borrow_mut() = None;
        self
    }

    pub fn with_max(self, max: i64) -> Self {
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

    pub fn with_styles(mut self, styles: WBStyles) -> Self {
        self.tb = self.tb.with_styles(styles);
        self
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.tb.set_at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(&self, ctx: &Context) -> Widgets {
        let (x, y) = (
            self.tb.base.get_dyn_start_x(),
            self.tb.base.get_dyn_start_y(),
        );

        let mut out: Vec<Box<dyn Widget>> = vec![];
        if *self.has_buttons.borrow() {
            let incr = *self.button_increment.borrow();
            let self_ = self.clone();
            let up_btn = Button::new(
                ctx,
                "▲",
                Box::new(move |_, ctx| {
                    let old_value = *self_.value.borrow();
                    self_.change_value(&ctx, old_value + incr);
                    EventResponses::default()
                }),
            )
            .basic_button(None);
            let self_ = self.clone();
            let down_btn = Button::new(
                ctx,
                "▼",
                Box::new(move |_, ctx| {
                    let old_value = *self_.value.borrow();
                    self_.change_value(&ctx, old_value - incr);
                    EventResponses::default()
                }),
            )
            .basic_button(None);

            let up_btn_x = x.clone().plus(self.tb.base.get_dyn_width());
            let down_btn_x = up_btn_x.clone().plus_fixed(1);
            out.push(Box::new(up_btn.at(up_btn_x, y.clone())));
            out.push(Box::new(down_btn.at(down_btn_x, y.clone())));
        }

        out.push(Box::new(self.clone()));
        Widgets(out)
    }

    // ---------------------------------------------------------

    pub fn change_value(&self, ctx: &Context, mut new_value: i64) {
        // correct bounds on value
        if let Some(min) = *self.min_value.borrow() {
            new_value = new_value.max(min)
        }
        if let Some(max) = *self.max_value.borrow() {
            new_value = new_value.min(max)
        }
        *self.value.borrow_mut() = new_value;
        self.tb.set_text(format!("{}", new_value));

        self.tb
            .set_cursor_pos(ctx, self.tb.get_text().chars().count());

        //if let Some(hook) = &mut *self.tb.text_changed_hook.borrow_mut() {
        //    let resp = hook(ctx.clone(), self.tb.get_text());
        //    debug_assert!(resp.is_empty());
        //}
    }

    pub fn update_value_from_tb(&self, ctx: &Context) {
        let value_str = self.tb.get_text();
        let value = value_str.parse::<i64>();
        if let Ok(value) = value {
            self.change_value(ctx, value);
        } else {
            let old_value = *self.value.borrow();
            self.change_value(ctx, old_value);
        }
    }

    pub fn restore_value(&self, ctx: &Context) {
        let old_value = *self.value.borrow();
        self.change_value(ctx, old_value);
    }
}

impl Widget for NumbersTextBox {
    fn set_selectability_pre_hook(&self, ctx: &Context, s: Selectability) -> EventResponses {
        if self.tb.get_selectability() == Selectability::Selected && s != Selectability::Selected {
            self.restore_value(ctx);
        }
        self.tb.set_selectability_pre_hook(ctx, s)
    }
}

impl Element for NumbersTextBox {
    fn kind(&self) -> &'static str {
        self.tb.kind()
    }
    fn id(&self) -> ElementID {
        self.tb.id()
    }
    fn receivable(&self) -> SelfReceivableEvents {
        self.tb.receivable()
    }

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::KeyCombo(ref ke) => {
                if self.tb.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }

                match true {
                    _ if ke[0] == KB::KEY_UP => {
                        let old_value = *self.value.borrow();
                        self.change_value(ctx, old_value + *self.button_increment.borrow());
                        (true, EventResponses::default())
                    }
                    _ if ke[0] == KB::KEY_DOWN => {
                        let old_value = *self.value.borrow();
                        self.change_value(ctx, old_value - *self.button_increment.borrow());
                        (true, EventResponses::default())
                    }
                    _ if ke[0] == KB::KEY_ENTER => {
                        self.update_value_from_tb(ctx);
                        (true, EventResponses::default())
                    }
                    _ if ke[0] == KB::KEY_SHIFT_ENTER => {
                        (true, EventResponses::default())
                    }
                    _ => self.tb.receive_event(ctx, ev),
                }
            }
            _ => self.tb.receive_event(ctx, ev),
        }
    }

    fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
        self.tb.change_priority(p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.tb.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.tb.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.tb.set_attribute(key, value)
    }
    fn set_parent(&self, up: Box<dyn Parent>) {
        self.tb.set_parent(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.tb.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.tb.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.tb.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.tb.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.tb.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.tb.get_visible()
    }
}

/*
*/