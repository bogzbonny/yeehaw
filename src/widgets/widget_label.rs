use {
    super::{common, SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponse, Priority,
        ReceivableEventChanges, RgbColour, SortingHat, Style, UpwardPropagator,
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
    selected_style: Style::new(),
    ready_style: Style::new(),
    unselectable_style: Style::new().with_fg(RgbColour::WHITE),
};

impl Label {
    const KIND: &'static str = "widget_label";

    pub fn new(hat: &SortingHat, ctx: &Context, text: &str) -> Self {
        let (w, h) = common::get_text_size(text);
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            ctx.clone(),
            SclVal::new_fixed(w),
            SclVal::new_fixed(h),
            LABEL_STYLE,
            LABEL_EV_COMBOS.clone(),
        );
        _ = wb.set_selectability(Selectability::Unselectable);
        wb.set_content_from_string(text);
        Label {
            base: wb,
            justification: Rc::new(RefCell::new(LabelJustification::Left)),
            text: Rc::new(RefCell::new(text.to_string())),
        }
    }

    pub fn get_width(&self) -> usize {
        self.base.get_width()
    }

    pub fn get_height(&self) -> usize {
        self.base.get_height()
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
        let rotated = self.base.sp.content.borrow().rotate_90_deg();
        *self.base.sp.content.borrow_mut() = rotated;
        let old_height = self.base.get_attr_scl_height();
        let old_width = self.base.get_attr_scl_width();
        self.base.set_attr_scl_width(old_height);
        self.base.set_attr_scl_height(old_width);
        self
    }

    pub fn with_style(self, sty: Style) -> Self {
        self.base.styles.borrow_mut().unselectable_style = sty;

        // this is necessary to actually update the content of the label w/
        // the new style
        // TODO: consider moving this somewhere else if it needs to be called in
        // many places
        self.base.set_content_from_string(&self.text.borrow());
        self
    }

    pub fn get_text(&self) -> String {
        self.text.borrow().clone()
    }

    // Updates the content and size of the label
    pub fn set_text(&self, text: String) {
        self.base.set_content_from_string(&text);
        let (w, h) = common::get_text_size(&text);
        self.base.set_attr_scl_width(SclVal::new_fixed(w));
        self.base.set_attr_scl_height(SclVal::new_fixed(h));
        *self.text.borrow_mut() = text;
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(mut self) -> Widgets {
        let mut x = self.base.get_attr_scl_loc_x();
        let mut y = self.base.get_attr_scl_loc_y();
        let w = self.base.get_attr_scl_width();
        let h = self.base.get_attr_scl_height();
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

//fn kind(&self) -> &'static str;
//fn id(&self) -> &ElementID;
//fn receivable(&self) -> Vec<(Event, Priority)>;
//fn receive_event(&mut self, ctx: &Context, ev: Event) -> (bool, EventResponse);
//fn change_priority(&mut self, ctx: &Context, p: Priority) -> ReceivableEventChanges;
//fn drawing(&self, ctx: &Context) -> Vec<DrawChPos>;
//fn get_attribute(&self, key: &str) -> Option<&[u8]>;
//fn set_attribute(&mut self, key: &str, value: Vec<u8>);
//fn set_upward_propagator(&mut self, up: Rc<RefCell<dyn UpwardPropagator>>);

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
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponse) {
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
    fn set_upward_propagator(&self, up: Rc<RefCell<dyn UpwardPropagator>>) {
        self.base.set_upward_propagator(up)
    }
}
