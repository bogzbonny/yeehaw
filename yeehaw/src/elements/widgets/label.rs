use crate::*;

#[derive(Clone)]
pub struct Label {
    pub pane: Pane,
    pub justification: Rc<RefCell<LabelJustification>>,
    pub text: Rc<RefCell<String>>,
}

#[derive(Clone, Copy)]
pub enum LabelJustification {
    Left,
    Right,
    Down,
    /// some wacky stuff
    Up,
}

/// label positions for around an element
///```text
///      1  2
///     5████7
///      ████
///     6████8
///      3  4
///```
#[derive(Clone, Copy, Debug)]
pub enum LabelPosition {
    /// 1
    AboveThenLeft,
    /// 2
    AboveThenRight,
    /// 3
    BelowThenLeft,
    /// 4
    BelowThenRight,
    /// 5
    LeftThenTop,
    /// 6
    LeftThenBottom,
    /// 7
    RightThenTop,
    /// 8
    RightThenBottom,
}

/// when "active" hitting enter will click the button
pub static LABEL_EV_COMBOS: SelfReceivableEvents = SelfReceivableEvents(vec![]);

impl Label {
    const KIND: &'static str = "label";

    pub fn new(ctx: &Context, text: &str) -> Self {
        Self::new_with_style(
            ctx,
            text,
            Style::default()
                .with_fg(Color::WHITE)
                .with_bg(Color::TRANSPARENT),
        )
    }

    pub fn new_with_style(ctx: &Context, text: &str, sty: Style) -> Self {
        let s = Size::get_text_size(text);
        let pane = Pane::new(ctx, Self::KIND)
            .with_self_receivable_events(LABEL_EV_COMBOS.clone())
            .with_style(sty)
            .with_dyn_width(DynVal::new_fixed(s.width as i32))
            .with_dyn_height(DynVal::new_fixed(s.height as i32));

        pane.set_content_from_string(text);
        Label {
            pane,
            justification: Rc::new(RefCell::new(LabelJustification::Left)),
            text: Rc::new(RefCell::new(text.to_string())),
        }
    }

    pub fn new_for_el(ctx: &Context, el_loc: DynLocation, text: &str) -> Self {
        let label = Self::new(ctx, text);

        // label to the right if a width of 1 otherwise label the top left
        if el_loc.width(ctx) == 1 {
            label.position_right_then_top(ctx, el_loc);
        } else {
            label.position_above_then_left(ctx, el_loc);
        }
        label
    }

    pub fn new_above_left_for_el(ctx: &Context, el_loc: DynLocation, text: &str) -> Self {
        let label = Self::new(ctx, text);
        label.position_above_then_left(ctx, el_loc);
        label
    }

    pub fn new_above_right_for_el(ctx: &Context, el_loc: DynLocation, text: &str) -> Self {
        let label = Self::new(ctx, text);
        label.position_above_then_right(ctx, el_loc);
        label
    }

    pub fn new_below_left_for_el(ctx: &Context, el_loc: DynLocation, text: &str) -> Self {
        let label = Self::new(ctx, text);
        label.position_below_then_left(ctx, el_loc);
        label
    }

    pub fn new_below_right_for_el(ctx: &Context, el_loc: DynLocation, text: &str) -> Self {
        let label = Self::new(ctx, text);
        label.position_below_then_right(ctx, el_loc);
        label
    }

    pub fn new_left_top_for_el(ctx: &Context, el_loc: DynLocation, text: &str) -> Self {
        let label = Self::new(ctx, text);
        label.position_left_then_top(ctx, el_loc);
        label
    }

    pub fn new_left_bottom_for_el(ctx: &Context, el_loc: DynLocation, text: &str) -> Self {
        let label = Self::new(ctx, text);
        label.position_left_then_bottom(ctx, el_loc);
        label
    }

    pub fn new_right_top_for_el(ctx: &Context, el_loc: DynLocation, text: &str) -> Self {
        let label = Self::new(ctx, text);
        label.position_right_then_top(ctx, el_loc);
        label
    }

    pub fn new_right_bottom_for_el(ctx: &Context, el_loc: DynLocation, text: &str) -> Self {
        let label = Self::new(ctx, text);
        label.position_right_then_bottom(ctx, el_loc);
        label
    }

    pub fn new_left_top_vertical_label_for_el(
        ctx: &Context, el_loc: DynLocation, text: &str,
    ) -> Self {
        let label = Self::new(ctx, text)
            .with_rotated_text()
            .with_down_justification();
        label.position_left_then_top(ctx, el_loc);
        label
    }

    pub fn new_left_bottom_vertical_label_for_el(
        ctx: &Context, el_loc: DynLocation, text: &str,
    ) -> Self {
        let label = Self::new(ctx, text)
            .with_rotated_text()
            .with_up_justification();
        label.position_left_then_bottom(ctx, el_loc);
        label
    }

    pub fn new_right_top_vertical_label_for_el(
        ctx: &Context, el_loc: DynLocation, text: &str,
    ) -> Self {
        let label = Self::new(ctx, text)
            .with_rotated_text()
            .with_down_justification();
        label.position_right_then_top(ctx, el_loc);
        label
    }

    pub fn new_right_bottom_vertical_label_for_el(
        ctx: &Context, el_loc: DynLocation, text: &str,
    ) -> Self {
        let label = Self::new(ctx, text)
            .with_rotated_text()
            .with_up_justification();
        label.position_right_then_bottom(ctx, el_loc);
        label
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

    /// Rotate the text by 90 degrees
    /// intended to be used with WithDownJustification or WithUpJustification
    pub fn with_rotated_text(self) -> Self {
        let rotated = self.pane.get_content().rotate_90_deg();
        self.pane.set_content(rotated);
        let old_height = self.pane.get_dyn_height();
        let old_width = self.pane.get_dyn_width();
        self.pane.set_dyn_width(old_height);
        self.pane.set_dyn_height(old_width);
        self
    }

    pub fn bold(self) -> Self {
        let sty = self.pane.get_style().with_bold();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn faded(self) -> Self {
        let sty = self.pane.get_style().with_faded();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn italic(self) -> Self {
        let sty = self.pane.get_style().with_italic();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn underlined(self) -> Self {
        let sty = self.pane.get_style().with_underlined();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn doubleunderlined(self) -> Self {
        let sty = self.pane.get_style().with_doubleunderlined();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn undercurled(self) -> Self {
        let sty = self.pane.get_style().with_undercurled();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn underdotted(self) -> Self {
        let sty = self.pane.get_style().with_underdotted();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn underdashed(self) -> Self {
        let sty = self.pane.get_style().with_underdashed();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn slowblink(self) -> Self {
        let sty = self.pane.get_style().with_slowblink();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn rapidblink(self) -> Self {
        let sty = self.pane.get_style().with_rapidblink();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn reverse(self) -> Self {
        let sty = self.pane.get_style().with_reverse();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn hidden(self) -> Self {
        let sty = self.pane.get_style().with_hidden();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn crossedout(self) -> Self {
        let sty = self.pane.get_style().with_crossedout();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn fraktur(self) -> Self {
        let sty = self.pane.get_style().with_fraktur();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn framed(self) -> Self {
        let sty = self.pane.get_style().with_framed();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn encircled(self) -> Self {
        let sty = self.pane.get_style().with_encircled();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }
    pub fn overlined(self) -> Self {
        let sty = self.pane.get_style().with_overlined();
        self.pane.set_content_style(sty.clone());
        self.pane.set_style(sty);
        self
    }

    pub fn with_style(self, sty: Style) -> Self {
        self.pane.set_style(sty);

        // this is necessary to actually update the content of the label w/
        // the new style
        // TODO: consider moving this somewhere else if it needs to be called in
        // many places
        self.pane
            .set_content_from_string(self.text.borrow().clone());
        self
    }

    pub fn get_text(&self) -> String {
        self.text.borrow().clone()
    }

    /// Updates the content and size of the label
    pub fn set_text(&self, text: String) {
        self.pane.set_content_from_string(&text);
        let s = Size::get_text_size(&text);
        self.pane.set_dyn_width(DynVal::new_fixed(s.width as i32));
        self.pane.set_dyn_height(DynVal::new_fixed(s.height as i32));
        *self.text.borrow_mut() = text;
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    pub fn set_at(&self, x: DynVal, y: DynVal) {
        self.pane.set_at(x, y);
    }

    /// get the label location from the label position
    fn label_position_to_xy(
        l: DynLocation,
        p: LabelPosition,
        label_width: usize,
        label_height: usize,
        //(x    , y     )
    ) -> (DynVal, DynVal) {
        match p {
            LabelPosition::AboveThenLeft => (l.start_x, l.start_y.minus_fixed(label_height as i32)),
            LabelPosition::AboveThenRight => (l.end_x, l.start_y.minus_fixed(label_height as i32)),
            LabelPosition::BelowThenLeft => (l.start_x, l.end_y.plus_fixed(1)),
            LabelPosition::BelowThenRight => (l.end_x, l.end_y.plus_fixed(1)),
            LabelPosition::LeftThenTop => (l.start_x.minus_fixed(label_width as i32), l.start_y),
            LabelPosition::LeftThenBottom => (l.start_x.minus_fixed(label_width as i32), l.end_y),
            LabelPosition::RightThenTop => (l.end_x.plus_fixed(1), l.start_y),
            LabelPosition::RightThenBottom => (l.end_x.plus_fixed(1), l.end_y),
        }
    }

    /// positions the label relative to the element location
    pub fn position_for(&self, ctx: &Context, el_loc: DynLocation, pos: LabelPosition) {
        let (x, y) = Self::label_position_to_xy(
            el_loc,
            pos,
            self.pane.get_width(ctx),
            self.pane.get_height(ctx),
        );
        self.set_at(x, y);
    }

    pub fn position_above_then_left(&self, ctx: &Context, el_loc: DynLocation) {
        self.position_for(ctx, el_loc, LabelPosition::AboveThenLeft);
    }

    pub fn position_above_then_right(&self, ctx: &Context, el_loc: DynLocation) {
        self.position_for(ctx, el_loc, LabelPosition::AboveThenRight);
    }

    pub fn position_below_then_left(&self, ctx: &Context, el_loc: DynLocation) {
        self.position_for(ctx, el_loc, LabelPosition::BelowThenLeft);
    }

    pub fn position_below_then_right(&self, ctx: &Context, el_loc: DynLocation) {
        self.position_for(ctx, el_loc, LabelPosition::BelowThenRight);
    }

    pub fn position_left_then_top(&self, ctx: &Context, el_loc: DynLocation) {
        self.position_for(ctx, el_loc, LabelPosition::LeftThenTop);
    }

    pub fn position_left_then_bottom(&self, ctx: &Context, el_loc: DynLocation) {
        self.position_for(ctx, el_loc, LabelPosition::LeftThenBottom);
    }

    pub fn position_right_then_top(&self, ctx: &Context, el_loc: DynLocation) {
        self.position_for(ctx, el_loc, LabelPosition::RightThenTop);
    }

    pub fn position_right_then_bottom(&self, ctx: &Context, el_loc: DynLocation) {
        self.position_for(ctx, el_loc, LabelPosition::RightThenBottom);
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Label {}
