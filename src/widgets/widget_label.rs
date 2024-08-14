use {
    super::{common, SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponse, Priority,
        ReceivableEventChanges, RgbColour, SortingHat, Style, UpwardPropagator,
    },
    std::{cell::RefCell, rc::Rc},
};

pub struct Label {
    pub base: WidgetBase,
    pub justification: LabelJustification,
    pub text: String,
}

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

    pub fn new(hat: &SortingHat, ctx: &Context, text: String) -> Self {
        let (w, h) = common::get_text_size(&text);
        let mut wb = WidgetBase::new(
            hat,
            Self::KIND,
            ctx.clone(),
            SclVal::new_fixed(w),
            SclVal::new_fixed(h),
            LABEL_STYLE,
            LABEL_EV_COMBOS.clone(),
        );
        _ = wb.set_selectability(Selectability::Unselectable);
        wb.set_content_from_string(&text);
        Label {
            base: wb,
            justification: LabelJustification::Left,
            text,
        }
    }

    pub fn get_width(&self) -> usize {
        self.base.get_width()
    }

    pub fn get_height(&self) -> usize {
        self.base.get_height()
    }

    pub fn with_left_justification(mut self) -> Self {
        self.justification = LabelJustification::Left;
        self
    }

    pub fn with_right_justification(mut self) -> Self {
        self.justification = LabelJustification::Right;
        self
    }

    pub fn with_down_justification(mut self) -> Self {
        self.justification = LabelJustification::Down;
        self
    }

    pub fn with_up_justification(mut self) -> Self {
        self.justification = LabelJustification::Up;
        self
    }

    // Rotate the text by 90 degrees
    // intended to be used with WithDownJustification or WithUpJustification
    pub fn with_rotated_text(mut self) -> Self {
        self.base.sp.content = self.base.sp.content.rotate_90_deg();
        let old_height = self.base.height.clone();
        let old_width = self.base.width.clone();
        self.base.width = old_height;
        self.base.height = old_width;
        self
    }

    pub fn with_style(mut self, sty: Style) -> Self {
        self.base.styles.unselectable_style = sty;

        // this is necessary to actually update the content of the label w/
        // the new style
        // TODO: consider moving this somewhere else if it needs to be called in
        // many places
        self.base.set_content_from_string(&self.text);
        self
    }

    // Updates the content and size of the label
    pub fn set_text(&mut self, text: String) {
        self.base.set_content_from_string(&text);
        let (w, h) = common::get_text_size(&text);
        self.base.width = SclVal::new_fixed(w);
        self.base.height = SclVal::new_fixed(h);
        self.text = text;
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(mut self) -> Widgets {
        let mut x = self.base.loc_x.clone();
        let mut y = self.base.loc_y.clone();
        match self.justification {
            LabelJustification::Left => {}
            LabelJustification::Right => {
                x = x.minus(self.base.width.clone().minus_fixed(1));
            }
            LabelJustification::Down => {}
            LabelJustification::Up => {
                y = y.minus(self.base.height.clone().minus_fixed(1));
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
    fn id(&self) -> &ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }
    fn receive_event(&mut self, ctx: &Context, ev: Event) -> (bool, EventResponse) {
        self.base.receive_event(ctx, ev)
    }
    fn change_priority(&mut self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.base.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<&[u8]> {
        self.base.get_attribute(key)
    }
    fn set_attribute(&mut self, key: &str, value: Vec<u8>) {
        self.base.set_attribute(key, value)
    }
    fn set_upward_propagator(&mut self, up: Rc<RefCell<dyn UpwardPropagator>>) {
        self.base.set_upward_propagator(up)
    }
}
