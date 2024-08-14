use {
    super::{Label, SclLocation, SclVal},
    crate::{
        event::Event, Context, DrawCh, DrawChPos, DrawChs2D, EventResponse, Location, Priority,
        ReceivableEventChanges, Style, ZIndex,
    },
};

//  WIDGET FARMER       ✲
//                         /|\      *
//  ⌂  ⌂  ⌂         ✲      \|/   /  *  \
//                 ✲            * time  *
//  water      ~  _|_  ~         \  *  /      ⌃
//  light        /   \              *       \   /
//  nutrience   / o o \   hi,             discovery
//  eneergy    /  ._   \  dont u d4re       /   \
//  darkness        \       munch my crops    ⌄
//                   -<<<-
//     |    |    |    |    |    |    |    |     f
//    \|/  \|/  \|/  \|/  \|/  \|/  \|/  \|/  \ o /
//    \|/  \|/  \:)  \|/  \|\  \|/  \|/  \|/  \ c /
//    \|/  \|/  \|/  \|/  \|/  \|/  \|/  \|/  \ u /
//     |    |    |    | oo |    |    |    |     s

// widgets are a basically really simple elements
// besides that a widget is aware of its location
pub trait Widget {
    // widgets can only receive events when they are active
    fn receivable(&self) -> Vec<Event>;

    fn get_parent_ctx(&self) -> &Context;
    fn set_parent_ctx(&mut self, parent_ctx: Context);

    // Draw the widget to the screen
    fn drawing(&self) -> Vec<DrawChPos>;

    fn set_styles(&mut self, styles: WBStyles);

    fn resize_event(&mut self, parent_ctx: Context);

    fn get_location(&self) -> Location;
    fn get_scl_location(&self) -> SclLocation;

    // NOTE the mouse event input is adjusted for the widgets location
    // (aka if you click on the top-left corner of the widget ev.Position()
    // will be 0, 0)
    fn receive_key_event(&mut self, ev: Event) -> (bool, EventResponse);
    fn receive_mouse_event(&mut self, ev: Event) -> (bool, EventResponse);

    // NOTE window creation in response to SetSelectability
    // is currently not supported
    fn get_selectability(&self) -> Selectability;
    fn set_selectability(&mut self, s: Selectability) -> EventResponse;

    // used in combination widgets (TODO confirm)
    fn to_widgets(self) -> Widgets;
}

const WIDGET_Z_INDEX: ZIndex = 10;

#[derive(Clone, Copy, PartialEq)]
pub enum Selectability {
    Selected,     // currently selected
    Ready,        // not selected but able to be selected
    Unselectable, // unselectable
}

// label positions
//      1  2
//     5████7
//      ████
//     6████8
//      3  4
pub enum LabelPosition {
    AboveThenLeft,   // 1
    AboveThenRight,  // 2
    BelowThenLeft,   // 3
    BelowThenRight,  // 4
    LeftThenTop,     // 5
    LeftThenBottom,  // 6
    RightThenTop,    // 7
    RightThenBottom, // 8
}

#[derive(Default)]
pub struct Widgets(pub Vec<Box<dyn Widget>>);

impl Widgets {
    // returns the smallest location which encompasses all
    // the sub-locations for all the contained widgets
    pub fn overall_loc(&self) -> SclLocation {
        if self.0.is_empty() {
            return SclLocation::default();
        }

        let mut l = SclLocation::default();
        for w in &self.0 {
            let wl_loc = w.get_scl_location();
            l.start_x = l.start_x.plus_min_of(wl_loc.start_x);
            l.end_x = l.end_x.plus_max_of(wl_loc.end_x);
            l.start_y = l.start_y.plus_min_of(wl_loc.start_y);
            l.end_y = l.end_y.plus_max_of(wl_loc.end_y);
        }
        l
    }

    // get the label location from the label position
    pub fn label_position_to_xy(
        &self,
        p: LabelPosition,
        label_width: usize,
        label_height: usize,
        //(x    , y     )
    ) -> (SclVal, SclVal) {
        let l = self.overall_loc();
        match p {
            LabelPosition::AboveThenLeft => (l.start_x, l.start_y.minus_fixed(label_height)),
            LabelPosition::AboveThenRight => (l.end_x, l.start_y.minus_fixed(label_height)),
            LabelPosition::BelowThenLeft => (l.start_x, l.end_y.plus_fixed(1)),
            LabelPosition::BelowThenRight => (l.end_x, l.end_y.plus_fixed(1)),
            LabelPosition::LeftThenTop => (l.start_x.minus_fixed(label_width), l.start_y),
            LabelPosition::LeftThenBottom => (l.start_x.minus_fixed(label_width), l.end_y),
            LabelPosition::RightThenTop => (l.end_x.plus_fixed(1), l.start_y),
            LabelPosition::RightThenBottom => (l.end_x.plus_fixed(1), l.end_y),
        }
    }

    //adds the label at the position provided
    pub fn add_label(&mut self, l: Label, p: LabelPosition) {
        let (x, y) = self.label_position_to_xy(p, l.get_width(), l.get_height());
        self.0.push(Box::new(l.at(x, y)));
    }

    pub fn with_label(self, l: String) -> Self {
        // label toi the right if a width of 1 otherwise label the top left
        if self.overall_loc().width(&(self).get_parent_ctx()) == 1 {
            self.with_right_top_label(l)
        } else {
            self.with_above_left_label(l)
        }
    }

    pub fn get_parent_ctx(&self) -> Context {
        if self.0.is_empty() {
            return Context::default();
        }
        (self.0[0].get_parent_ctx()).clone()
    }

    pub fn with_above_left_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l),
            LabelPosition::AboveThenLeft,
        );
        self
    }

    pub fn with_above_right_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l),
            LabelPosition::AboveThenRight,
        );
        self
    }

    pub fn with_below_left_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l),
            LabelPosition::BelowThenLeft,
        );
        self
    }

    pub fn with_below_right_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l),
            LabelPosition::BelowThenRight,
        );
        self
    }

    pub fn with_left_top_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l),
            LabelPosition::LeftThenTop,
        );
        self
    }

    pub fn with_left_bottom_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l),
            LabelPosition::LeftThenBottom,
        );
        self
    }

    pub fn with_right_top_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l),
            LabelPosition::RightThenTop,
        );
        self
    }

    pub fn with_right_bottom_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l),
            LabelPosition::RightThenBottom,
        );
        self
    }

    // ---------------
    // vertical labels

    pub fn with_left_top_vertical_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l)
                .with_rotated_text()
                .with_down_justification(),
            LabelPosition::LeftThenTop,
        );
        self
    }

    pub fn with_left_bottom_vertical_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l)
                .with_rotated_text()
                .with_up_justification(),
            LabelPosition::LeftThenBottom,
        );
        self
    }

    pub fn with_right_top_vertical_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l)
                .with_rotated_text()
                .with_down_justification(),
            LabelPosition::RightThenTop,
        );
        self
    }

    pub fn with_right_bottom_vertical_label(mut self, l: String) -> Self {
        self.add_label(
            Label::new(self.get_parent_ctx(), l)
                .with_rotated_text()
                .with_up_justification(),
            LabelPosition::RightThenBottom,
        );
        self
    }
}

//------------------------------------------------

pub struct WidgetBase {
    pub p_ctx: Context, // last parent context

    pub selectedness: Selectability, // lol

    // function called when a mouse event is successfully received
    //ReceiveMouseEventFn func(ev *tcell.EventMouse)

    // the receivableEvents when this widget is active
    pub receivable_events: Vec<Event>,

    // size of the widget (NOT the content space)
    pub width: SclVal,
    pub height: SclVal,
    pub loc_x: SclVal,
    pub loc_y: SclVal,

    pub content: DrawChs2D,       // [Y][X]DrawCh
    pub content_max_width: usize, // max width of the content
    pub content_x_offset: usize,
    pub content_y_offset: usize,
    pub styles: WBStyles,
}

impl WidgetBase {
    pub fn new(
        p_ctx: Context, width: SclVal, height: SclVal, sty: WBStyles, receivable_events: Vec<Event>,
    ) -> Self {
        Self {
            p_ctx,
            selectedness: Selectability::Ready,
            receivable_events,
            width,
            height,
            loc_x: SclVal::new_fixed(0),
            loc_y: SclVal::new_fixed(0),
            content: DrawChs2D::default(),
            content_max_width: 0,
            content_x_offset: 0,
            content_y_offset: 0,
            styles: sty,
        }
    }

    pub fn at(&mut self, loc_x: SclVal, loc_y: SclVal) {
        self.loc_x = loc_x;
        self.loc_y = loc_y;
    }

    //-------------------------

    pub fn get_width(&self) -> usize {
        self.width.get_val(self.p_ctx.get_width().into())
    }

    pub fn get_height(&self) -> usize {
        self.height.get_val(self.p_ctx.get_height().into())
    }

    pub fn get_parent_ctx(&self) -> &Context {
        &self.p_ctx
    }

    pub fn set_parent_ctx(&mut self, p_ctx: Context) {
        self.p_ctx = p_ctx;
    }

    pub fn resize_event(&mut self, p_ctx: Context) {
        self.p_ctx = p_ctx;
    }

    pub fn get_location(&self) -> Location {
        let w = self.get_width() as i32;
        let h = self.get_height() as i32;
        let x1 = self.loc_x.get_val(self.p_ctx.get_width().into()) as i32;
        let y1 = self.loc_y.get_val(self.p_ctx.get_height().into()) as i32;
        let x2 = x1 + w - 1;
        let y2 = y1 + h - 1;
        Location::new(x1, x2, y1, y2)
    }

    pub fn get_scl_location(&self) -> SclLocation {
        let x1 = self.loc_x.clone();
        let y1 = self.loc_y.clone();
        let x2 = x1.clone().plus(self.width.clone()).minus_fixed(1);
        let y2 = y1.clone().plus(self.height.clone()).minus_fixed(1);
        SclLocation::new(x1, x2, y1, y2)
    }

    pub fn scroll_up(&mut self) {
        self.set_content_y_offset(self.content_y_offset - 1);
    }

    pub fn scroll_down(&mut self) {
        self.set_content_y_offset(self.content_y_offset + 1);
    }

    pub fn scroll_left(&mut self) {
        self.set_content_x_offset(self.content_x_offset - 1);
    }

    pub fn scroll_right(&mut self) {
        self.set_content_x_offset(self.content_x_offset + 1);
    }

    pub fn content_width(&self) -> usize {
        self.content_max_width
    }

    pub fn content_height(&self) -> usize {
        self.content.height()
    }

    pub fn set_content_x_offset(&mut self, x: usize) {
        self.content_x_offset = if x > self.content_width() - self.get_width() {
            self.content_max_width - self.get_width()
        } else {
            x
        };
    }

    pub fn set_content_y_offset(&mut self, y: usize) {
        self.content_y_offset = if y > self.content_height() - self.get_height() {
            self.content_height() - self.get_height()
        } else {
            y
        };
    }

    // sets content from string
    pub fn set_content_from_string(&mut self, s: &str) {
        let lines = s.split('\n');
        let mut rs: Vec<Vec<char>> = Vec::new();
        let sty = self.get_current_style();

        let mut width = self.get_width();
        let mut height = self.get_height();
        for line in lines {
            if width < line.len() {
                width = line.len();
            }
            rs.push(line.chars().collect());
        }
        self.content_max_width = width;
        if height < rs.len() {
            height = rs.len();
        }

        // initialize the content with blank characters
        // of the height and width of the widget
        self.content = DrawChs2D::new_empty_of_size(width, height, sty);

        // now fill in with actual content
        for y in 0..height {
            for x in 0..width {
                let r = if y < rs.len() && x < rs[y].len() {
                    rs[y][x]
                } else {
                    continue;
                };
                let dch = DrawCh::new(r, false, sty);
                self.content.0[y][x] = dch;
            }
        }
    }

    pub fn set_content(&mut self, content: DrawChs2D) {
        self.content_max_width = content.width();
        self.content = content;
    }

    // correct_offsets_to_view_position changes the content offsets within the
    // WidgetBase in order to bring the given view position into view.
    pub fn correct_offsets_to_view_position(&mut self, x: usize, y: usize) {
        // set y offset if cursor out of bounds
        if y >= self.content_y_offset + self.get_height() {
            self.set_content_y_offset(y - self.get_height() + 1);
        } else if y < self.content_y_offset {
            self.set_content_y_offset(y);
        }

        // correct the offset if the offset is now showing lines that don't exist in
        // the content
        if self.content_y_offset + self.get_height() > self.content_height() - 1 {
            self.set_content_y_offset(self.content_height() - 1);
        }

        // set x offset if cursor out of bounds
        if x >= self.content_x_offset + self.get_width() {
            self.set_content_x_offset(x - self.get_width() + 1);
        } else if x < self.content_x_offset {
            self.set_content_x_offset(x);
        }

        // correct the offset if the offset is now showing characters to the right
        // which don't exist in the content.
        if self.content_x_offset + self.get_width() > self.content_width() - 1 {
            self.set_content_x_offset(self.content_width() - 1);
        }
    }

    // default implementation of Receivable, only receive when widget is active
    pub fn receivable(&self) -> Vec<Event> {
        if let Selectability::Selected = self.selectedness {
            self.receivable_events.clone()
        } else {
            Vec::new()
        }
    }

    pub fn get_selectability(&self) -> Selectability {
        self.selectedness
    }

    pub fn set_selectability(&mut self, s: Selectability) -> EventResponse {
        if self.selectedness == s {
            return EventResponse::default();
        }

        let mut rec = ReceivableEventChanges::default();
        match s {
            Selectability::Selected => {
                rec.add_evs_single_priority(self.receivable_events.clone(), Priority::FOCUSED);
            }
            Selectability::Ready | Selectability::Unselectable => {
                if let Selectability::Selected = self.selectedness {
                    rec.remove_evs(self.receivable_events.clone());
                }
            }
        }
        self.selectedness = s;
        EventResponse::default().with_receivable_event_changes(rec)
    }

    pub fn disable(&mut self) -> EventResponse {
        self.set_selectability(Selectability::Unselectable)
    }

    pub fn enable(&mut self) -> EventResponse {
        self.set_selectability(Selectability::Ready)
    }

    pub fn get_current_style(&self) -> Style {
        match self.selectedness {
            Selectability::Selected => self.styles.selected_style,
            Selectability::Ready => self.styles.ready_style,
            Selectability::Unselectable => self.styles.unselectable_style,
        }
    }

    pub fn drawing(&self) -> Vec<DrawChPos> {
        let sty = self.get_current_style();
        let h = self.get_height();
        let w = self.get_width();

        let mut chs = Vec::new();
        for y in self.content_y_offset..self.content_y_offset + h {
            for x in self.content_x_offset..self.content_x_offset + w {
                let ch = if y < self.content.height() && x < self.content.width() {
                    self.content.0[y][x]
                } else {
                    DrawCh::new(' ', false, sty)
                };
                chs.push(DrawChPos::new(
                    ch,
                    (x - self.content_x_offset) as u16,
                    (y - self.content_y_offset) as u16,
                ));
            }
        }
        chs
    }

    pub fn set_styles(&mut self, styles: WBStyles) {
        self.styles = styles;
    }

    pub fn receive_key_event(&mut self, _ev: Event) -> (bool, EventResponse) {
        (false, EventResponse::default())
    }

    pub fn receive_mouse_event(&self, _ev: Event) -> (bool, EventResponse) {
        (false, EventResponse::default())
    }
}

#[derive(Copy, Clone, Default)]
pub struct WBStyles {
    pub selected_style: Style,
    pub ready_style: Style,
    pub unselectable_style: Style,
}

impl WBStyles {
    pub fn new(selected_style: Style, ready_style: Style, unselectable_style: Style) -> WBStyles {
        WBStyles {
            selected_style,
            ready_style,
            unselectable_style,
        }
    }
}
