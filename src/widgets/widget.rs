use {
    super::{Label, SclLocation, SclVal},
    crate::{
        event::Event, Context, DrawCh, DrawChPos, DrawChs2D, Element, ElementID, EventResponse,
        EventResponses, Priority, ReceivableEventChanges, SortingHat, StandardPane, Style,
        UpwardPropagator, ZIndex,
    },
    std::{cell::RefCell, rc::Rc},
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

// Widgets are a basically simple elements. Differently from standard elements, a widget also
// stores its own scaled location, this is useful during the widget generation phase where multiple
// widgets are often created in tandam as a Widget group (see Widgets struct).
// Additionally the Widget trait also introduces a new attribute named Selectability which is integral to the
// operation of the WidgetPane Element.
//
//let Ok(v) = serde_json::to_string(&zafs) else {
pub const ATTR_SCL_WIDTH: &str = "widget_scl_width";
pub const ATTR_SCL_HEIGHT: &str = "widget_scl_height";
pub const ATTR_SCL_LOC_X: &str = "widget_scl_loc_x";
pub const ATTR_SCL_LOC_Y: &str = "widget_scl_loc_y";
pub const ATTR_SELECTABILITY: &str = "widget_selectability";

pub const WIDGET_Z_INDEX: ZIndex = 10;

pub trait Widget: Element {
    fn get_attr_scl_width(&self) -> SclVal {
        let Some(bz) = self.get_attribute(ATTR_SCL_WIDTH) else {
            return SclVal::default();
        };
        match serde_json::from_slice(&bz) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                SclVal::default()
            }
        }
    }
    fn set_attr_scl_width(&self, width: SclVal) {
        let bz = match serde_json::to_vec(&width) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                return;
            }
        };
        self.set_attribute(ATTR_SCL_WIDTH, bz)
    }
    fn get_attr_scl_height(&self) -> SclVal {
        let Some(bz) = self.get_attribute(ATTR_SCL_HEIGHT) else {
            return SclVal::default();
        };
        match serde_json::from_slice(&bz) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                SclVal::default()
            }
        }
    }
    fn set_attr_scl_height(&self, height: SclVal) {
        let bz = match serde_json::to_vec(&height) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                return;
            }
        };
        self.set_attribute(ATTR_SCL_HEIGHT, bz)
    }
    fn get_attr_scl_loc_x(&self) -> SclVal {
        let Some(bz) = self.get_attribute(ATTR_SCL_LOC_X) else {
            return SclVal::default();
        };
        match serde_json::from_slice(&bz) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                SclVal::default()
            }
        }
    }
    fn set_attr_scl_loc_x(&self, loc_x: SclVal) {
        let bz = match serde_json::to_vec(&loc_x) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                return;
            }
        };
        self.set_attribute(ATTR_SCL_LOC_X, bz)
    }
    fn get_attr_scl_loc_y(&self) -> SclVal {
        let Some(bz) = self.get_attribute(ATTR_SCL_LOC_Y) else {
            return SclVal::default();
        };
        match serde_json::from_slice(&bz) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                SclVal::default()
            }
        }
    }
    fn set_attr_scl_loc_y(&self, loc_y: SclVal) {
        let bz = match serde_json::to_vec(&loc_y) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                return;
            }
        };
        self.set_attribute(ATTR_SCL_LOC_Y, bz)
    }

    fn get_attr_selectability(&self) -> Selectability {
        let Some(bz) = self.get_attribute(ATTR_SELECTABILITY) else {
            return Selectability::Ready;
        };
        match serde_json::from_slice(&bz) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                Selectability::Ready
            }
        }
    }

    fn set_attr_selectability(&self, s: Selectability) {
        let bz = match serde_json::to_vec(&s) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                return;
            }
        };
        self.set_attribute(ATTR_SELECTABILITY, bz)
    }

    // ------------------

    fn get_selectability(&self) -> Selectability {
        self.get_attr_selectability()
    }

    // NOTE window creation in response to SetSelectability is currently not supported
    fn set_selectability(&self, ctx: &Context, s: Selectability) -> EventResponses {
        let mut resps = self.set_selectability_pre_hook(ctx, s);

        let attr_sel = self.get_attr_selectability();
        if attr_sel == s {
            return resps;
        }

        let mut rec = ReceivableEventChanges::default();
        match s {
            Selectability::Selected => {
                self.set_attr_selectability(s); // NOTE needs to happen before the next line or
                                                // else receivable will return the wrong value
                rec.add_evs(self.receivable())
            }
            Selectability::Ready | Selectability::Unselectable => {
                if let Selectability::Selected = attr_sel {
                    rec.remove_evs(self.receivable().iter().map(|(ev, _)| ev.clone()).collect());
                }
                self.set_attr_selectability(s); // NOTE needs to after before the prev line or else
                                                // receivable will return the wrong value
            }
        }

        resps.push(EventResponse::default().with_receivable_event_changes(rec));
        resps
    }

    // executed before the selectability is set
    fn set_selectability_pre_hook(&self, _ctx: &Context, _s: Selectability) -> EventResponses {
        EventResponses::default()
    }

    // get the scalable location of the widget
    fn get_scl_location(&self) -> SclLocation {
        let x1 = self.get_attr_scl_loc_x();
        let y1 = self.get_attr_scl_loc_y();
        let w = self.get_attr_scl_width();
        let h = self.get_attr_scl_height();
        let x2 = x1.clone().plus(w).minus_fixed(1);
        let y2 = y1.clone().plus(h).minus_fixed(1);
        SclLocation::new(x1, x2, y1, y2)
    }

    fn get_z_index(&self) -> ZIndex {
        WIDGET_Z_INDEX
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
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
#[derive(Clone, Copy, Debug)]
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

impl From<Vec<Box<dyn Widget>>> for Widgets {
    fn from(v: Vec<Box<dyn Widget>>) -> Self {
        Widgets(v)
    }
}

impl From<Box<dyn Widget>> for Widgets {
    fn from(w: Box<dyn Widget>) -> Self {
        Widgets(vec![w])
    }
}

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
    pub fn add_label(&mut self, ctx: &Context, l: Label, p: LabelPosition) {
        let (x, y) = self.label_position_to_xy(p, l.get_width(ctx), l.get_height(ctx));
        self.0.push(Box::new(l.at(x, y)));
    }

    pub fn with_label(self, hat: &SortingHat, ctx: &Context, l: &str) -> Self {
        // label to the right if a width of 1 otherwise label the top left
        if self.overall_loc().width(ctx) == 1 {
            self.with_right_top_label(hat, ctx, l)
        } else {
            self.with_above_left_label(hat, ctx, l)
        }
    }

    // XXX delete post-TRANSLATION
    //pub fn get_parent_ctx(&self) -> Context {
    //    if self.0.is_empty() {
    //        return Context::default();
    //    }
    //    (self.0[0].get_parent_ctx()).clone()
    //}

    pub fn with_above_left_label(mut self, hat: &SortingHat, ctx: &Context, l: &str) -> Self {
        self.add_label(ctx, Label::new(hat, ctx, l), LabelPosition::AboveThenLeft);
        self
    }

    pub fn with_above_right_label(mut self, hat: &SortingHat, ctx: &Context, l: &str) -> Self {
        self.add_label(ctx, Label::new(hat, ctx, l), LabelPosition::AboveThenRight);
        self
    }

    pub fn with_below_left_label(mut self, hat: &SortingHat, ctx: &Context, l: &str) -> Self {
        self.add_label(ctx, Label::new(hat, ctx, l), LabelPosition::BelowThenLeft);
        self
    }

    pub fn with_below_right_label(mut self, hat: &SortingHat, ctx: &Context, l: &str) -> Self {
        self.add_label(ctx, Label::new(hat, ctx, l), LabelPosition::BelowThenRight);
        self
    }

    pub fn with_left_top_label(mut self, hat: &SortingHat, ctx: &Context, l: &str) -> Self {
        self.add_label(ctx, Label::new(hat, ctx, l), LabelPosition::LeftThenTop);
        self
    }

    pub fn with_left_bottom_label(mut self, hat: &SortingHat, ctx: &Context, l: &str) -> Self {
        self.add_label(ctx, Label::new(hat, ctx, l), LabelPosition::LeftThenBottom);
        self
    }

    pub fn with_right_top_label(mut self, hat: &SortingHat, ctx: &Context, l: &str) -> Self {
        self.add_label(ctx, Label::new(hat, ctx, l), LabelPosition::RightThenTop);
        self
    }

    pub fn with_right_bottom_label(mut self, hat: &SortingHat, ctx: &Context, l: &str) -> Self {
        self.add_label(ctx, Label::new(hat, ctx, l), LabelPosition::RightThenBottom);
        self
    }

    // ---------------
    // vertical labels

    pub fn with_left_top_vertical_label(
        mut self, hat: &SortingHat, ctx: &Context, l: &str,
    ) -> Self {
        self.add_label(
            ctx,
            Label::new(hat, ctx, l)
                .with_rotated_text()
                .with_down_justification(),
            LabelPosition::LeftThenTop,
        );
        self
    }

    pub fn with_left_bottom_vertical_label(
        mut self, hat: &SortingHat, ctx: &Context, l: &str,
    ) -> Self {
        self.add_label(
            ctx,
            Label::new(hat, ctx, l)
                .with_rotated_text()
                .with_up_justification(),
            LabelPosition::LeftThenBottom,
        );
        self
    }

    pub fn with_right_top_vertical_label(
        mut self, hat: &SortingHat, ctx: &Context, l: &str,
    ) -> Self {
        self.add_label(
            ctx,
            Label::new(hat, ctx, l)
                .with_rotated_text()
                .with_down_justification(),
            LabelPosition::RightThenTop,
        );
        self
    }

    pub fn with_right_bottom_vertical_label(
        mut self, hat: &SortingHat, ctx: &Context, l: &str,
    ) -> Self {
        self.add_label(
            ctx,
            Label::new(hat, ctx, l)
                .with_rotated_text()
                .with_up_justification(),
            LabelPosition::RightThenBottom,
        );
        self
    }
}

//------------------------------------------------

#[derive(Clone)]
pub struct WidgetBase {
    pub sp: StandardPane,

    //pub last_ctx: Rc<RefCell<Context>>, // last parent context

    //pub selectedness: Selectability, // lol

    // size of the widget (NOT the content space)
    // These are scaling values and are used to generate the
    // exact location (when get_location is called).
    //pub width: SclVal,
    //pub height: SclVal,
    //pub loc_x: SclVal,
    //pub loc_y: SclVal,
    pub styles: Rc<RefCell<WBStyles>>,
    // the receivableEvents when this widget is active
    //pub receivable_events: Vec<Event>,
    //pub cmontent: DrawChs2D,      // [Y][X]DrawCh
    //pub content_max_width: usize, // max width of the content
    //pub content_x_offset: usize,
    //pub content_y_offset: usize,
}

impl WidgetBase {
    pub fn new(
        hat: &SortingHat, kind: &'static str, width: SclVal, height: SclVal, sty: WBStyles,
        mut receivable_events: Vec<Event>,
    ) -> Self {
        let evs = receivable_events
            .drain(..)
            .map(|ev| (ev, Priority::FOCUSED))
            .collect();
        let sp = StandardPane::new(hat, kind).with_self_receivable_events(evs);

        let wb = Self {
            sp,
            styles: Rc::new(RefCell::new(sty)),
        };
        wb.set_attr_scl_width(width);
        wb.set_attr_scl_height(height);
        wb.set_attr_scl_loc_x(SclVal::new_fixed(0));
        wb.set_attr_scl_loc_y(SclVal::new_fixed(0));
        wb.set_attr_selectability(Selectability::Ready);

        wb
    }

    pub fn at(&mut self, loc_x: SclVal, loc_y: SclVal) {
        self.set_attr_scl_loc_x(loc_x);
        self.set_attr_scl_loc_y(loc_y);
    }

    //-------------------------

    pub fn get_width(&self, ctx: &Context) -> usize {
        let scl_width = self.get_attr_scl_width();
        scl_width.get_val(ctx.get_width().into())
    }

    pub fn get_height(&self, ctx: &Context) -> usize {
        let scl_height = self.get_attr_scl_height();
        scl_height.get_val(ctx.get_height().into())
    }

    pub fn scroll_up(&mut self, ctx: &Context) {
        let view_offset_y = *self.sp.content_view_offset_y.borrow();
        self.set_content_y_offset(ctx, view_offset_y - 1);
    }

    pub fn scroll_down(&mut self, ctx: &Context) {
        let view_offset_y = *self.sp.content_view_offset_y.borrow();
        self.set_content_y_offset(ctx, view_offset_y + 1);
    }

    pub fn scroll_left(&mut self, ctx: &Context) {
        let view_offset_x = *self.sp.content_view_offset_x.borrow();
        self.set_content_x_offset(ctx, view_offset_x - 1);
    }

    pub fn scroll_right(&mut self, ctx: &Context) {
        let view_offset_x = *self.sp.content_view_offset_x.borrow();
        self.set_content_x_offset(ctx, view_offset_x + 1);
    }

    pub fn content_width(&self) -> usize {
        self.sp.content.borrow().width()
    }

    pub fn content_height(&self) -> usize {
        self.sp.content.borrow().height()
    }

    pub fn set_content_x_offset(&self, ctx: &Context, x: usize) {
        *self.sp.content_view_offset_x.borrow_mut() =
            if x > self.content_width() - self.get_width(ctx) {
                self.content_width() - self.get_width(ctx)
            } else {
                x
            };
    }

    pub fn set_content_y_offset(&self, ctx: &Context, y: usize) {
        *self.sp.content_view_offset_y.borrow_mut() =
            if y > self.content_height() - self.get_height(ctx) {
                self.content_height() - self.get_height(ctx)
            } else {
                y
            };
    }

    // sets content from string
    pub fn set_content_from_string(&self, ctx: &Context, s: &str) {
        let lines = s.split('\n');
        let mut rs: Vec<Vec<char>> = Vec::new();
        let sty = self.get_current_style();

        let mut width = self.get_width(ctx);
        let mut height = self.get_height(ctx);
        for line in lines {
            if width < line.len() {
                width = line.len();
            }
            rs.push(line.chars().collect());
        }
        if height < rs.len() {
            height = rs.len();
        }

        // initialize the content with blank characters
        // of the height and width of the widget
        *self.sp.content.borrow_mut() = DrawChs2D::new_empty_of_size(width, height, sty);

        // now fill in with actual content
        for y in 0..height {
            for x in 0..width {
                let r = if y < rs.len() && x < rs[y].len() {
                    rs[y][x]
                } else {
                    continue;
                };
                let dch = DrawCh::new(r, false, sty);
                self.sp.content.borrow_mut().0[y][x] = dch;
            }
        }
    }

    pub fn set_content(&self, content: DrawChs2D) {
        *self.sp.content.borrow_mut() = content;
    }

    // correct_offsets_to_view_position changes the content offsets within the
    // WidgetBase in order to bring the given view position into view.
    pub fn correct_offsets_to_view_position(&self, ctx: &Context, x: usize, y: usize) {
        let view_offset_y = *self.sp.content_view_offset_y.borrow();
        let view_offset_x = *self.sp.content_view_offset_x.borrow();

        // set y offset if cursor out of bounds
        if y >= view_offset_y + self.get_height(ctx) {
            //debug!("cor1");
            self.set_content_y_offset(ctx, y - self.get_height(ctx) + 1);
        } else if y < view_offset_y {
            //debug!("cor2");
            self.set_content_y_offset(ctx, y);
        }

        // correct the offset if the offset is now showing lines that don't exist in
        // the content
        //if view_offset_y + self.get_height(ctx) > self.content_height() - 1 {
        if view_offset_y + self.get_height(ctx) > self.content_height() {
            //debug!("cor3");
            self.set_content_y_offset(ctx, self.content_height());
        }

        // set x offset if cursor out of bounds
        if x >= view_offset_x + self.get_width(ctx) {
            self.set_content_x_offset(ctx, x - self.get_width(ctx) + 1);
        } else if x < view_offset_x {
            self.set_content_x_offset(ctx, x);
        }

        // correct the offset if the offset is now showing characters to the right
        // which don't exist in the content.
        //if view_offset_x + self.get_width(ctx) > self.content_width() - 1 {
        if view_offset_x + self.get_width(ctx) > self.content_width() {
            self.set_content_x_offset(ctx, self.content_width());
        }
    }

    pub fn disable(&self, ctx: &Context) -> EventResponses {
        self.set_selectability(ctx, Selectability::Unselectable)
    }

    pub fn enable(&self, ctx: &Context) -> EventResponses {
        self.set_selectability(ctx, Selectability::Ready)
    }

    pub fn get_current_style(&self) -> Style {
        match self.get_attr_selectability() {
            Selectability::Selected => self.styles.borrow().selected_style,
            Selectability::Ready => self.styles.borrow().ready_style,
            Selectability::Unselectable => self.styles.borrow().unselectable_style,
        }
    }

    pub fn set_styles(&self, styles: WBStyles) {
        *self.styles.borrow_mut() = styles;
    }
}

impl Widget for WidgetBase {}

impl Element for WidgetBase {
    fn kind(&self) -> &'static str {
        self.sp.kind()
    }
    fn id(&self) -> ElementID {
        self.sp.id()
    }

    // default implementation of Receivable, only receive when widget is active
    fn receivable(&self) -> Vec<(Event, Priority)> {
        let attr_sel = self.get_attr_selectability();
        if let Selectability::Selected = attr_sel {
            self.sp.receivable()
        } else {
            Vec::new()
        }
    }

    fn receive_event(&self, _ctx: &Context, _ev: Event) -> (bool, EventResponses) {
        (false, EventResponses::default())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.sp.change_priority(ctx, p)
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let sty = self.get_current_style(); // XXX this is different than standard_pane draw... unless this should be set somewhere else
        let h = self.get_height(ctx);
        let w = self.get_width(ctx);
        let view_offset_y = *self.sp.content_view_offset_y.borrow();
        let view_offset_x = *self.sp.content_view_offset_x.borrow();
        let content_height = self.sp.content.borrow().height();
        let content_width = self.sp.content.borrow().width();

        let mut chs = Vec::new();
        for y in view_offset_y..view_offset_y + h {
            for x in view_offset_x..view_offset_x + w {
                let ch = if y < content_height && x < content_width {
                    self.sp.content.borrow().0[y][x]
                } else {
                    DrawCh::new(' ', false, sty)
                };
                chs.push(DrawChPos::new(
                    ch,
                    (x - view_offset_x) as u16,
                    (y - view_offset_y) as u16,
                ));
            }
        }
        chs
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.sp.get_attribute(key)
    }

    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.sp.set_attribute(key, value)
    }

    fn set_upward_propagator(&self, up: Rc<RefCell<dyn UpwardPropagator>>) {
        self.sp.set_upward_propagator(up)
    }
}

// ---------------------------------------
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
