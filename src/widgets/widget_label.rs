use {
    super::{common, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Color, Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Priority, ReceivableEventChanges, SortingHat, Style, UpwardPropagator,
    },
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct Label {
    pub base: WidgetBase,
    pub justification: Rc<RefCell<LabelJustification>>,
    pub text: Rc<RefCell<String>>,
}

#[derive(Clone, Copy)]
pub enum LabelJustification {
    Left,
    Right,
    Down, // some wacky stuff
    Up,
}

// when "active" hitting enter will click the button
pub static LABEL_EV_COMBOS: Vec<Event> = Vec::new();

pub static LABEL_STYLE: WBStyles = WBStyles {
    selected_style: Style::default_const(),
    ready_style: Style::default_const(),
    unselectable_style: Style::new(Some(Color::WHITE), None, None),
};

impl Label {
    const KIND: &'static str = "widget_label";

    pub fn new(hat: &SortingHat, ctx: &Context, text: &str) -> Self {
        let (w, h) = common::get_text_size(text);
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            DynVal::new_fixed(w as i32),
            DynVal::new_fixed(h as i32),
            LABEL_STYLE.clone(),
            LABEL_EV_COMBOS.clone(),
        );
        _ = wb.set_selectability(ctx, Selectability::Unselectable);
        wb.set_content_from_string(ctx, text);
        Label {
            base: wb,
            justification: Rc::new(RefCell::new(LabelJustification::Left)),
            text: Rc::new(RefCell::new(text.to_string())),
        }
    }

    pub fn get_width_val(&self, ctx: &Context) -> usize {
        self.base.get_width_val(ctx)
    }

    pub fn get_height_val(&self, ctx: &Context) -> usize {
        self.base.get_height_val(ctx)
    }

    pub fn with_left_justification(self) -> Self {
        *self.justification.borrow_mut() = LabelJustification::Left;
        self
    }

    pub fn with_right_justification(self) -> Self {
        *self.justification.borrow_mut() = LabelJustification::Right;
        self
    }

    pub fn with_down_justification(self) -> Self {
        *self.justification.borrow_mut() = LabelJustification::Down;
        self
    }

    pub fn with_up_justification(self) -> Self {
        *self.justification.borrow_mut() = LabelJustification::Up;
        self
    }

    // Rotate the text by 90 degrees
    // intended to be used with WithDownJustification or WithUpJustification
    pub fn with_rotated_text(self) -> Self {
        let rotated = self.base.pane.content.borrow().rotate_90_deg();
        *self.base.pane.content.borrow_mut() = rotated;
        let old_height = self.base.get_dyn_height();
        let old_width = self.base.get_dyn_width();
        self.base.set_dyn_width(old_height);
        self.base.set_dyn_height(old_width);
        self
    }

    pub fn with_style(self, ctx: &Context, sty: Style) -> Self {
        self.base.styles.borrow_mut().unselectable_style = sty;

        // this is necessary to actually update the content of the label w/
        // the new style
        // TODO: consider moving this somewhere else if it needs to be called in
        // many places
        self.base.set_content_from_string(ctx, &self.text.borrow());
        self
    }

    pub fn get_text(&self) -> String {
        self.text.borrow().clone()
    }

    // Updates the content and size of the label
    pub fn set_text(&self, ctx: &Context, text: String) {
        self.base.set_content_from_string(ctx, &text);
        let (w, h) = common::get_text_size(&text);
        self.base.set_dyn_width(DynVal::new_fixed(w as i32));
        self.base.set_dyn_height(DynVal::new_fixed(h as i32));
        *self.text.borrow_mut() = text;
    }

    pub fn at(mut self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(mut self) -> Widgets {
        let mut x = self.base.get_dyn_start_x();
        let mut y = self.base.get_dyn_start_y();
        let w = self.base.get_dyn_width();
        let h = self.base.get_dyn_height();
        match *self.justification.borrow() {
            LabelJustification::Left => {}
            LabelJustification::Right => {
                x = x.minus(w.minus_fixed(1));
            }
            LabelJustification::Down => {}
            LabelJustification::Up => {
                y = y.minus(h.minus_fixed(1));
            }
        }
        self.base.at(x, y);
        Widgets(vec![Box::new(self)])
    }
}
impl Widget for Label {}

impl Element for Label {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.base.receive_event(ctx, ev)
    }
    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.base.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.base.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.base.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.base.set_upward_propagator(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.base.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.base.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.base.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.base.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.base.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.base.get_visible()
    }
}
