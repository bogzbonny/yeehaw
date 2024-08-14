use {
    super::{common, SclLocation, SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{Context, DrawChPos, Event, EventResponse, Location, RgbColour, Style},
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
    pub fn new(p_ctx: Context, text: String) -> Self {
        let (w, h) = common::get_text_size(&text);
        let mut wb = WidgetBase::new(
            p_ctx,
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
        self.base.content = self.base.content.rotate_90_deg();
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
}

/*
OLD GO CODE TO TRANSLATE

func (l *Label) ToWidgets() Widgets {
    x, y := l.LocX, l.LocY
    switch l.Justification {
    case JustifyLeft: // do nothing
    case JustifyRight:
        x = x.Minus(l.Width.MinusStatic(1))
    case JustifyDown: // do nothing
    case JustifyUp:
        y = y.Minus(l.Height.MinusStatic(1))
    }
    l.At(x, y)
    return Widgets{l}
}

func (l *Label) ReceiveKeyEventCombo(_ []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
    return false, yh.NewEventResponse()
}

func (l *Label) ReceiveMouseEvent(_ *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
    return false, yh.NewEventResponse()
}
*/

impl Widget for Label {
    fn receivable(&self) -> Vec<Event> {
        self.base.receivable()
    }
    fn get_parent_ctx(&self) -> &Context {
        self.base.get_parent_ctx()
    }
    fn set_parent_ctx(&mut self, parent_ctx: Context) {
        self.base.set_parent_ctx(parent_ctx);
    }
    fn drawing(&self) -> Vec<DrawChPos> {
        self.base.drawing()
    }
    fn set_styles(&mut self, styles: WBStyles) {
        self.base.set_styles(styles);
    }
    fn resize_event(&mut self, parent_ctx: Context) {
        self.base.resize_event(parent_ctx);
    }
    fn get_location(&self) -> Location {
        self.base.get_location()
    }
    fn get_scl_location(&self) -> SclLocation {
        self.base.get_scl_location()
    }
    fn receive_key_event(&mut self, ev: Event) -> (bool, EventResponse) {
        self.base.receive_key_event(ev)
    }
    fn receive_mouse_event(&mut self, ev: Event) -> (bool, EventResponse) {
        self.base.receive_mouse_event(ev)
    }
    fn get_selectability(&self) -> Selectability {
        self.base.get_selectability()
    }
    fn set_selectability(&mut self, s: Selectability) -> EventResponse {
        self.base.set_selectability(s)
    }
    fn to_widgets(mut self) -> Widgets {
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
