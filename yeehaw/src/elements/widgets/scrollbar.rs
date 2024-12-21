use {
    crate::{Keyboard as KB, *},
    crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    std::cmp::Ordering,
    std::ops::{Deref, DerefMut},
};

/// NOTE the code in this file is structured in a "zipper" fashion between vertical and horizontal
/// scrollbar, although this increases the line count (extra impl lines everywhere) it is useful for
/// ensuring that the two scrollbars have consistent code, as similar code is always grouped together.

/// up is backwards, down is forwards
#[derive(Clone)]
pub struct VerticalScrollbar(Scrollbar);

/// left is backwards, right is forwards
#[derive(Clone)]
pub struct HorizontalScrollbar(Scrollbar);

impl Deref for VerticalScrollbar {
    type Target = Scrollbar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VerticalScrollbar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for HorizontalScrollbar {
    type Target = Scrollbar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HorizontalScrollbar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub enum VerticalSBPositions {
    None,
    ToTheLeft,
    ToTheRight,
}

#[derive(Clone, Copy, Debug)]
pub enum HorizontalSBPositions {
    None,
    Above,
    Below,
}

impl VerticalScrollbar {
    const KIND: &'static str = "vertical_scrollbar";
    pub fn default_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KB::KEY_UP.into()),
            (KB::KEY_DOWN.into()),
            (KB::KEY_SPACE.into()),
        ])
    }
    pub fn new(
        ctx: &Context, scrollable_view_height: DynVal, scrollable_view_size: Size,
        scrollable_height: usize,
    ) -> Self {
        let pane = Pane::new(ctx, Self::KIND)
            .with_self_receivable_events(Self::default_receivable_events())
            .with_style(Scrollbar::STYLE)
            .with_dyn_width(DynVal::new_fixed(1))
            .with_dyn_height(scrollable_view_height.clone());

        VerticalScrollbar(Scrollbar {
            pane,
            scrollable_domain_chs: Rc::new(RefCell::new(scrollable_height)),
            scrollable_view_chs: Rc::new(RefCell::new(scrollable_view_height.clone())),
            scrollable_view_size: Rc::new(RefCell::new(scrollable_view_size)),
            scrollable_position: Rc::new(RefCell::new(0)),
            has_arrows: Rc::new(RefCell::new(true)),
            sb_sty: Rc::new(RefCell::new(ScrollbarSty::vertical_block(Scrollbar::STYLE))),
            position_changed_hook: Rc::new(RefCell::new(None)),
            currently_dragging: Rc::new(RefCell::new(false)),
            start_drag_position: Rc::new(RefCell::new(0)),
            jump_scroll_percent: Rc::new(RefCell::new(10)),
            jump_scroll_min_amount: Rc::new(RefCell::new(3)),
        })
    }

    /// is scrollable_height is None, then it is considererd unchanged
    pub fn set_dyn_height(
        &self,
        view_height: DynVal,
        scrollbar_length: DynVal,
        scrollable_height: Option<usize>, // None = unchanged
    ) {
        *self.scrollable_view_chs.borrow_mut() = view_height;
        self.pane.set_dyn_height(scrollbar_length.clone());
        if let Some(scrollable_height) = scrollable_height {
            *self.scrollable_domain_chs.borrow_mut() = scrollable_height;
        }
    }

    // ----------------------------------------------
    // decorators

    pub fn without_keyboard_events(self) -> Self {
        self.pane
            .set_self_receivable_events(SelfReceivableEvents::default());
        self
    }

    pub fn with_scrollbar_sty(self, sb_sty: ScrollbarSty) -> Self {
        *self.sb_sty.borrow_mut() = sb_sty;
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    pub fn set_at(&self, loc_x: DynVal, loc_y: DynVal) {
        self.pane.set_at(loc_x, loc_y);
    }

    pub fn without_arrows(self) -> Self {
        *self.has_arrows.borrow_mut() = false;
        self
    }

    pub fn set_scrollable_view_size(&self, scrollable_view_size: Size) {
        *self.scrollable_view_size.borrow_mut() = scrollable_view_size;
    }
}

impl HorizontalScrollbar {
    const KIND: &'static str = "horizontal_scrollbar";
    pub fn default_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KB::KEY_LEFT.into()),
            (KB::KEY_RIGHT.into()),
        ])
    }
    pub fn new(
        ctx: &Context, scrollable_view_width: DynVal, scrollable_view_size: Size,
        scrollable_width: usize,
    ) -> Self {
        let pane = Pane::new(ctx, Self::KIND)
            .with_self_receivable_events(Self::default_receivable_events())
            .with_style(Scrollbar::STYLE)
            .with_dyn_height(DynVal::new_fixed(1))
            .with_dyn_width(scrollable_view_width.clone());

        HorizontalScrollbar(Scrollbar {
            pane,
            scrollable_domain_chs: Rc::new(RefCell::new(scrollable_width)),
            scrollable_view_chs: Rc::new(RefCell::new(scrollable_view_width.clone())),
            scrollable_view_size: Rc::new(RefCell::new(scrollable_view_size)),
            scrollable_position: Rc::new(RefCell::new(0)),
            has_arrows: Rc::new(RefCell::new(true)),
            sb_sty: Rc::new(RefCell::new(ScrollbarSty::horizontal_block(
                Scrollbar::STYLE,
            ))),
            position_changed_hook: Rc::new(RefCell::new(None)),
            currently_dragging: Rc::new(RefCell::new(false)),
            start_drag_position: Rc::new(RefCell::new(0)),
            jump_scroll_percent: Rc::new(RefCell::new(10)),
            jump_scroll_min_amount: Rc::new(RefCell::new(3)),
        })
    }

    /// is scrollable_width is None, then it is considererd unchanged
    pub fn set_dyn_width(
        &self,
        view_width: DynVal,
        scrollbar_length: DynVal,
        scrollable_width: Option<usize>, // None = unchanged
    ) {
        *self.scrollable_view_chs.borrow_mut() = view_width;
        self.pane.set_dyn_width(scrollbar_length.clone());
        if let Some(scrollable_width) = scrollable_width {
            *self.scrollable_domain_chs.borrow_mut() = scrollable_width;
        }
    }

    // ----------------------------------------------
    // decorators

    pub fn without_keyboard_events(self) -> Self {
        self.pane
            .set_self_receivable_events(SelfReceivableEvents::default());
        self
    }

    pub fn with_scrollbar_sty(self, sb_sty: ScrollbarSty) -> Self {
        *self.sb_sty.borrow_mut() = sb_sty;
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    pub fn set_at(&self, loc_x: DynVal, loc_y: DynVal) {
        self.pane.set_at(loc_x, loc_y);
    }

    pub fn without_arrows(self) -> Self {
        *self.has_arrows.borrow_mut() = false;
        self
    }

    pub fn set_scrollable_view_size(&self, scrollable_view_size: Size) {
        *self.scrollable_view_size.borrow_mut() = scrollable_view_size;
    }
}

// ------------------------------------------------------------------

/// The Scrollbar is a pane type of common logic to build the
/// vertical and horizontal scrollbars off of.
///
/// For vertical scrollbars:
///   - "backwards" should be thought of as "up" and
///   - "forwards" should be thought of as "down".
///
/// For horizontal scrollbars:
///   - "backwards" should be thought of as "left" and
///   - "forwards" should be thought of as "right".
#[derive(Clone)]
pub struct Scrollbar {
    pub pane: Pane,

    /// The ScrollableDomainChs is the scrollable dimension in true characters.
    /// It is AFFECTED by the scrollbar and NOT the literal area of the scrollbar
    /// itself.
    pub scrollable_domain_chs: Rc<RefCell<usize>>,
    /// how large the area is that can be scrolled

    /// how much of the scrollable area is visible in true chars.
    pub scrollable_view_chs: Rc<RefCell<DynVal>>,

    /// used during the construction of the context during
    /// hook calls to the view
    pub scrollable_view_size: Rc<RefCell<Size>>,

    /// how far down the area is scrolled from the beginning of the actual content in true chars.
    /// The ScrollablePosition will be the first line of the area scrolled to.
    /// NOTE this is not the position within the scrollbar but the position within the actual content.
    pub scrollable_position: Rc<RefCell<usize>>,

    pub has_arrows: Rc<RefCell<bool>>,
    /// if the scrollbar has arrows
    pub sb_sty: Rc<RefCell<ScrollbarSty>>,

    /// function the scrollbar will call everytime there is a position change
    #[allow(clippy::type_complexity)]
    pub position_changed_hook: Rc<RefCell<Option<Box<dyn FnMut(Context, usize)>>>>,

    /// is the scrollbar currently being dragged?
    pub currently_dragging: Rc<RefCell<bool>>,
    pub start_drag_position: Rc<RefCell<usize>>,
    /// in true characters

    /// The percent (0-100) of the total scrollable domain
    /// to scroll when a click in the scrollbar whitespace is made.
    pub jump_scroll_percent: Rc<RefCell<usize>>,

    /// minimum amount to scroll during a jump scroll
    pub jump_scroll_min_amount: Rc<RefCell<usize>>,
}

pub struct ScrollbarSty {
    pub backwards_arrow: DrawCh,
    pub forwards_arrow: DrawCh,
    pub empty_block: DrawCh,
    pub full_block: DrawCh,
    pub forwards_half_block: DrawCh,
    pub backwards_half_block: DrawCh,
    pub unnessecary: DrawCh,
}

impl ScrollbarSty {
    pub fn horizontal_block(sty: Style) -> Self {
        ScrollbarSty {
            backwards_arrow: DrawCh::new('◀', sty.clone()),
            forwards_arrow: DrawCh::new('▶', sty.clone()),
            empty_block: DrawCh::new(' ', sty.clone()),
            full_block: DrawCh::new('█', sty.clone()),
            forwards_half_block: DrawCh::new('▐', sty.clone()),
            backwards_half_block: DrawCh::new('▌', sty.clone()),
            unnessecary: DrawCh::new('░', sty),
        }
    }

    pub fn horizontal_for_thin_box(sty: Style) -> Self {
        ScrollbarSty {
            backwards_arrow: DrawCh::new('◁', sty.clone()),
            forwards_arrow: DrawCh::new('▷', sty.clone()),
            empty_block: DrawCh::new('─', sty.clone()),
            full_block: DrawCh::new('━', sty.clone()),
            forwards_half_block: DrawCh::new('╼', sty.clone()),
            backwards_half_block: DrawCh::new('╾', sty.clone()),
            unnessecary: DrawCh::new('━', sty),
        }
    }

    pub fn vertical_block(sty: Style) -> Self {
        ScrollbarSty {
            backwards_arrow: DrawCh::new('▲', sty.clone()),
            forwards_arrow: DrawCh::new('▼', sty.clone()),
            empty_block: DrawCh::new(' ', sty.clone()),
            full_block: DrawCh::new('█', sty.clone()),
            forwards_half_block: DrawCh::new('▄', sty.clone()),
            backwards_half_block: DrawCh::new('▀', sty.clone()),
            unnessecary: DrawCh::new('░', sty),
        }
    }

    pub fn vertical_for_thin_box(sty: Style) -> Self {
        ScrollbarSty {
            backwards_arrow: DrawCh::new('△', sty.clone()),
            forwards_arrow: DrawCh::new('▽', sty.clone()),
            empty_block: DrawCh::new('│', sty.clone()),
            full_block: DrawCh::new('┃', sty.clone()),
            forwards_half_block: DrawCh::new('╽', sty.clone()),
            backwards_half_block: DrawCh::new('╿', sty.clone()),
            unnessecary: DrawCh::new('┃', sty),
        }
    }

    pub fn with_empty_block(mut self, c: char) -> Self {
        self.empty_block.ch = c.into();
        self
    }

    pub fn with_unnessecary(mut self, c: char) -> Self {
        self.unnessecary.ch = c.into();
        self
    }
}

pub enum SBRelPosition {
    None,
    Before,
    On,
    After,
}

impl Scrollbar {
    const STYLE: Style = Style::new_const(Color::WHITE, Color::GREY13);

    pub fn set_scrollable_view_size(&self, scrollable_view_size: Size) {
        *self.scrollable_view_size.borrow_mut() = scrollable_view_size;
    }

    /// if the Scrollbar currently cannot be used due to insufficient domain.
    pub fn is_currently_unnecessary(&self, p_size: usize) -> bool {
        (*self.scrollable_domain_chs.borrow() as i32)
            <= self.scrollable_view_chs.borrow().get_val(p_size as u16)
    }

    pub fn jump_scroll_amount(&self) -> usize {
        let js = *self.scrollable_domain_chs.borrow() * *self.jump_scroll_percent.borrow() / 100;
        if js < *self.jump_scroll_min_amount.borrow() {
            *self.jump_scroll_min_amount.borrow()
        } else {
            js
        }
    }

    /// scroll to the position within the scrollable domain.
    pub fn scroll_to_position(&self, ctx: &Context, p_size: usize, mut position: usize) {
        let sc_dom_len = *self.scrollable_domain_chs.borrow();
        let sc_view_len = self.scrollable_view_chs.borrow().get_val(p_size as u16) as usize;
        if position > sc_dom_len.saturating_sub(sc_view_len) {
            position = sc_dom_len.saturating_sub(sc_view_len)
        }
        *self.scrollable_position.borrow_mut() = position;
        if let Some(hook) = self.position_changed_hook.borrow_mut().as_mut() {
            let mut ctx = ctx.clone();
            ctx.size = *self.scrollable_view_size.borrow();
            hook(ctx, position);
        }
    }

    pub fn can_scroll_forwards(&self, p_size: usize) -> bool {
        let sc_pos = *self.scrollable_position.borrow();
        let sc_dom_chs = *self.scrollable_domain_chs.borrow();
        let sc_view_chs = self.scrollable_view_chs.borrow().get_val(p_size as u16) as usize;
        sc_pos < sc_dom_chs.saturating_sub(sc_view_chs)
    }

    pub fn jump_scroll_backwards(&self, ctx: &Context, p_size: usize) {
        let pos = self
            .scrollable_position
            .borrow()
            .saturating_sub(self.jump_scroll_amount());
        self.scroll_to_position(ctx, p_size, pos);
    }

    pub fn jump_scroll_forwards(&self, ctx: &Context, p_size: usize) {
        let pos = *self.scrollable_position.borrow() + self.jump_scroll_amount();
        self.scroll_to_position(ctx, p_size, pos);
    }

    pub fn can_scroll_backwards(&self) -> bool {
        *self.scrollable_position.borrow() > 0
    }

    /// scroll backwards through the actual content by 1 ch
    pub fn scroll_backwards(&self, ctx: &Context) {
        if !self.can_scroll_backwards() {
            return;
        }
        *self.scrollable_position.borrow_mut() -= 1;
        if let Some(hook) = self.position_changed_hook.borrow_mut().as_mut() {
            let mut ctx = ctx.clone();
            ctx.size = *self.scrollable_view_size.borrow();
            hook(ctx, *self.scrollable_position.borrow());
        }
    }

    /// scroll forwards through the actual content by 1 ch
    pub fn scroll_forwards(&self, ctx: &Context, p_size: usize) {
        if !self.can_scroll_forwards(p_size) {
            return;
        }
        *self.scrollable_position.borrow_mut() += 1;
        if let Some(hook) = self.position_changed_hook.borrow_mut().as_mut() {
            let mut ctx = ctx.clone();
            ctx.size = *self.scrollable_view_size.borrow();
            hook(ctx, *self.scrollable_position.borrow());
        }
    }

    /// the scrollbar domain is the total space which the scroll bar may occupy (both the actual bar
    /// and the movement space above and below it) measured in half-increments but not including the
    /// arrow spaces. Each half-increment represents half a character, as the scrollbar may use half
    /// characters to represent its position.
    pub fn scrollbar_domain_in_half_increments(&self, p_size: usize) -> usize {
        // minus 2 for the backwards and forwards arrows
        let arrows = if *self.has_arrows.borrow() { 2 } else { 0 };
        let sc_len_chs = p_size;
        // times 2 for half characters
        2 * (sc_len_chs.saturating_sub(arrows))
    }

    pub fn scroll_bar_size_and_domain_in_half_increments(&self, p_size: usize) -> (usize, usize) {
        let domain_incr = self.scrollbar_domain_in_half_increments(p_size);
        let percent_viewable = (self.scrollable_view_chs.borrow().get_val(p_size as u16) as f64)
            / (*self.scrollable_domain_chs.borrow() as f64);

        // scrollbar size in half increments
        let mut scrollbar_incr = (percent_viewable * domain_incr as f64).round() as usize;

        // minimum size of 1 half-increment
        if scrollbar_incr < 1 {
            scrollbar_incr = 1;
        }

        // safeguard
        if scrollbar_incr > domain_incr {
            scrollbar_incr = domain_incr;
        }

        (scrollbar_incr, domain_incr)
    }

    /// the number of true view characters per full scrollbar character (aka 2
    /// half-increments)
    pub fn true_chs_per_scrollbar_character(&self, p_size: usize) -> usize {
        let (scrollbar_incr, _) = self.scroll_bar_size_and_domain_in_half_increments(p_size);
        (p_size as f64 / (scrollbar_incr as f64 / 2.0)) as usize
    }

    /// Get an array of half-increments of the scrollbar domain area
    fn scroll_bar_domain_array_of_half_increments(&self, p_size: usize) -> Vec<bool> {
        let (scrollbar_incr, domain_incr) =
            self.scroll_bar_size_and_domain_in_half_increments(p_size);

        // total increments within the scrollbar domain for space above and below the bar
        let total_spacer_incr = domain_incr.saturating_sub(scrollbar_incr);

        let true_chs_above = *self.scrollable_position.borrow();

        let sc_dom_chs = *self.scrollable_domain_chs.borrow();
        let sc_view_chs = self.scrollable_view_chs.borrow().get_val(p_size as u16) as usize;
        let diff = sc_dom_chs.saturating_sub(sc_view_chs) as f64;
        let incr_above = (true_chs_above as f64 / diff) * total_spacer_incr as f64;
        let mut incr_above = incr_above.round() as usize;

        // correct incase the rounding gives an extra increment
        if incr_above + scrollbar_incr > domain_incr {
            incr_above = domain_incr.saturating_sub(scrollbar_incr);
        }

        // -----------------------------------------------
        // determine whether each increment is a filled.
        if domain_incr == 0 {
            //debug!("----------------------------\n")
            //debug!("incrAbove: %v, scrollbarIncr: %v, domainIncr: %v\n", incrAbove, scrollbarIncr, domainIncr)
            //debug!("totalSpacerIncr: %v, trueChsAbove: %v, sb.ScrollableDomainChs: %v\n", totalSpacerIncr, trueChsAbove, sb.ScrollableDomainChs)
            //debug!("pSize: %v, sb.ScrollableViewChs.GetVal(pSize)): %v\n", pSize, sb.ScrollableViewChs.GetVal(pSize))
            return vec![];
        }

        let mut incr_filled = vec![false; domain_incr];
        incr_filled
            .iter_mut()
            .skip(incr_above)
            .take(scrollbar_incr)
            .for_each(|i| *i = true);
        incr_filled
    }

    fn last_incr_filled(incr_filled: &[bool]) -> Option<usize> {
        (0..incr_filled.len()).rev().find(|&i| incr_filled[i])
    }

    fn first_incr_filled(incr_filled: &[bool]) -> Option<usize> {
        (0..incr_filled.len()).find(|&i| incr_filled[i])
    }

    /// used for mouse dragging the scrollbar. What the incrementIsFilled should look
    /// like if it dragged down by one rune (aka 2 half increments)
    pub fn drag_forwards_by_1_ch(&self, ctx: &Context, p_size: usize) {
        let start_incrs = self.scroll_bar_domain_array_of_half_increments(p_size);
        let last_filled = Self::last_incr_filled(&start_incrs);
        let Some(last_filled) = last_filled else {
            return;
        };
        let mut goal_last_filled = last_filled + 2;
        if goal_last_filled >= start_incrs.len() {
            goal_last_filled = start_incrs.len().saturating_sub(1);
        }
        loop {
            // safegaurd against infinite loop
            if !self.can_scroll_forwards(p_size) {
                return;
            }
            self.scroll_forwards(ctx, p_size);
            let current_incr = self.scroll_bar_domain_array_of_half_increments(p_size);
            let curr_last_filled = Self::last_incr_filled(&current_incr);
            // NOTE scroll forwards scrolls through the actual content by 1 ch, therefor under
            // certain circumstances it is possible for curr_last_filled to jump over the goal_last_filled
            if curr_last_filled >= Some(goal_last_filled) {
                return;
            }
        }
    }

    /// Same as DragForwardsBy1Ch but in the backwards direction
    pub fn drag_backwards_by_1_ch(&self, ctx: &Context, p_size: usize) {
        let start_incrs = self.scroll_bar_domain_array_of_half_increments(p_size);
        let first_filled = Self::first_incr_filled(&start_incrs);
        let Some(first_filled) = first_filled else {
            return;
        };
        let goal_first_filled = first_filled.saturating_sub(2);
        loop {
            // safegaurd against infinite loop
            if !self.can_scroll_backwards() {
                return;
            }
            self.scroll_backwards(ctx);
            let current_incr = self.scroll_bar_domain_array_of_half_increments(p_size);
            let curr_first_filled = Self::first_incr_filled(&current_incr);
            // NOTE scroll forwards scrolls through the actual content by 1 ch, therefor under
            // certain circumstances it is possible for curr_last_filled to jump before the goal_last_filled
            if curr_first_filled <= Some(goal_first_filled) {
                return;
            }
        }
    }

    pub fn scrollbar_domain_array_of_runes(&self, p_size: usize) -> Vec<DrawCh> {
        let incr_filled = self.scroll_bar_domain_array_of_half_increments(p_size);
        let mut rs = vec![];
        // determine the characters based on the filled increments
        for i in 0..incr_filled.len() {
            if i % 2 == 1 {
                match (incr_filled[i - 1], incr_filled[i]) {
                    (true, true) => rs.push(self.sb_sty.borrow().full_block.clone()),
                    (true, false) => rs.push(self.sb_sty.borrow().backwards_half_block.clone()),
                    (false, true) => rs.push(self.sb_sty.borrow().forwards_half_block.clone()),
                    (false, false) => rs.push(self.sb_sty.borrow().empty_block.clone()),
                }
            }
        }
        rs
    }

    pub fn drawing_runes(&self, p_size: usize) -> Vec<DrawCh> {
        let mut chs = vec![];
        if self.is_currently_unnecessary(p_size) {
            for _ in 0..p_size {
                chs.push(self.sb_sty.borrow().unnessecary.clone());
            }
        } else {
            if *self.has_arrows.borrow() {
                chs.push(self.sb_sty.borrow().backwards_arrow.clone());
            }
            chs.append(&mut self.scrollbar_domain_array_of_runes(p_size));
            if *self.has_arrows.borrow() {
                chs.push(self.sb_sty.borrow().forwards_arrow.clone());
            }
        }
        chs
    }

    /// Call this when the position has been changed external to the scrollbar
    /// new_view_offset is the new position of the view in full characters
    /// new_view_domain is the number of full characters of the full scrollable domain
    pub fn external_change(
        &self, new_view_offset: usize, new_domain_chs: usize, scrollable_view_size: Size,
    ) {
        *self.scrollable_position.borrow_mut() = new_view_offset;
        *self.scrollable_domain_chs.borrow_mut() = new_domain_chs;
        *self.scrollable_view_size.borrow_mut() = scrollable_view_size;
    }

    /// is the provided position before, on, or after the scrollbar?
    pub fn position_relative_to_scrollbar(&self, p_size: usize, mut pos: usize) -> SBRelPosition {
        // last pos the actual scrollbar may be in
        let mut last_scrollbar_pos = p_size.saturating_sub(1);

        if *self.has_arrows.borrow() {
            let sc_len_chs = p_size;
            if pos == 0 || pos == sc_len_chs.saturating_sub(1) {
                return SBRelPosition::None;
            }
            pos -= 1; // account for the backwards arrow
            last_scrollbar_pos = sc_len_chs.saturating_sub(3); // account for the forwards arrow
        }

        let rs = self.scrollbar_domain_array_of_runes(p_size);
        if pos >= rs.len() {
            return SBRelPosition::After;
        }

        let mut first_full: Option<usize> = None;
        let mut last_full = 0;
        let mut backwards_half_pos = 0;
        let mut forwards_half_pos = 0;
        for (i, r) in rs.iter().enumerate() {
            if *r == self.sb_sty.borrow().full_block {
                if first_full.is_none() {
                    first_full = Some(i);
                }
                last_full = i;
            }
            if *r == self.sb_sty.borrow().backwards_half_block {
                backwards_half_pos = i;
            }
            if *r == self.sb_sty.borrow().forwards_half_block {
                forwards_half_pos = i;
            }
        }

        // edge cases for when very near the end
        if pos == 0 && forwards_half_pos == 0 {
            return SBRelPosition::Before;
        }
        if pos == last_scrollbar_pos && backwards_half_pos == last_scrollbar_pos {
            return SBRelPosition::After;
        }

        match first_full {
            None => match pos {
                _ if backwards_half_pos == pos || forwards_half_pos == pos => SBRelPosition::On,
                _ if pos < forwards_half_pos || pos < backwards_half_pos => SBRelPosition::Before,
                _ if pos > forwards_half_pos || pos > backwards_half_pos => SBRelPosition::After,
                _ => SBRelPosition::None,
            },
            Some(first_full) => match pos {
                _ if pos < first_full => SBRelPosition::Before,
                _ if pos > last_full => SBRelPosition::After,
                _ => SBRelPosition::On,
            },
        }
    }
}

// -------------------------------------------------------------------
/// Specific implementations for the vertical and horizontal scrollbars

impl VerticalScrollbar {
    pub fn external_change(
        &self, new_view_offset: usize, new_domain_chs: usize, scrollable_view_size: Size,
    ) {
        *self.scrollable_position.borrow_mut() = new_view_offset;
        *self.scrollable_domain_chs.borrow_mut() = new_domain_chs;
        *self.scrollable_view_size.borrow_mut() = scrollable_view_size;
    }
}
impl HorizontalScrollbar {
    pub fn external_change(
        &self, new_view_offset: usize, new_domain_chs: usize, scrollable_view_size: Size,
    ) {
        *self.scrollable_position.borrow_mut() = new_view_offset;
        *self.scrollable_domain_chs.borrow_mut() = new_domain_chs;
        *self.scrollable_view_size.borrow_mut() = scrollable_view_size;
    }
}

impl VerticalScrollbar {
    pub fn get_content(&self, ctx: &Context) -> DrawChs2D {
        let chs = self.drawing_runes(ctx.get_height().into());
        DrawChs2D::from_draw_chs_vertical(chs)
    }
}
impl HorizontalScrollbar {
    pub fn get_content(&self, ctx: &Context) -> DrawChs2D {
        let chs = self.drawing_runes(ctx.get_width().into());
        DrawChs2D::from_draw_chs_horizontal(chs)
    }
}

impl VerticalScrollbar {
    pub fn receive_key_event(&self, ev: Vec<KeyEvent>, ctx: &Context) -> (bool, EventResponses) {
        if ev.is_empty() {
            return (false, EventResponses::default());
        }

        match true {
            _ if ev[0] == KB::KEY_UP => {
                self.scroll_backwards(ctx);
                (true, EventResponses::default())
            }
            _ if ev[0] == KB::KEY_DOWN => {
                self.scroll_forwards(ctx, ctx.get_height().into());
                (true, EventResponses::default())
            }
            _ if ev[0] == KB::KEY_SPACE => {
                self.jump_scroll_forwards(ctx, ctx.get_height().into());
                (true, EventResponses::default())
            }
            _ => (false, EventResponses::default()),
        }
    }
}

impl HorizontalScrollbar {
    pub fn receive_key_event(&self, ev: Vec<KeyEvent>, ctx: &Context) -> (bool, EventResponses) {
        if ev.is_empty() {
            return (false, EventResponses::default());
        }

        match true {
            _ if ev[0] == KB::KEY_LEFT => {
                self.scroll_backwards(ctx);
                (true, EventResponses::default())
            }
            _ if ev[0] == KB::KEY_RIGHT => {
                self.scroll_forwards(ctx, ctx.get_width().into());
                (true, EventResponses::default())
            }
            _ => (false, EventResponses::default()),
        }
    }
}

impl VerticalScrollbar {
    pub fn receive_mouse_event(&self, ctx: &Context, ev: MouseEvent) -> (bool, EventResponses) {
        let curr_dragging = *self.currently_dragging.borrow();
        let h = ctx.get_height();
        match ev.kind {
            MouseEventKind::ScrollDown => {
                self.scroll_forwards(ctx, h.into());
                (true, EventResponses::default())
            }
            MouseEventKind::ScrollUp => {
                self.scroll_backwards(ctx);
                (true, EventResponses::default())
            }
            MouseEventKind::Up(MouseButton::Left) => {
                *self.currently_dragging.borrow_mut() = false;
                (true, EventResponses::default())
            }
            MouseEventKind::Drag(MouseButton::Left) if curr_dragging => {
                self.drag_while_dragging(ctx, ev)
            }
            MouseEventKind::Down(MouseButton::Left) if !curr_dragging => {
                let y = ev.row as usize;
                let has_arrows = *self.has_arrows.borrow();
                match true {
                    _ if has_arrows && y == 0 => {
                        self.scroll_backwards(ctx);
                        *self.currently_dragging.borrow_mut() = false;
                    }
                    _ if has_arrows && y == (h as usize).saturating_sub(1) => {
                        self.scroll_forwards(ctx, h.into());
                        *self.currently_dragging.borrow_mut() = false;
                    }
                    _ => match self.position_relative_to_scrollbar(h.into(), y) {
                        SBRelPosition::Before => {
                            self.jump_scroll_backwards(ctx, h.into());
                            *self.currently_dragging.borrow_mut() = false;
                        }
                        SBRelPosition::After => {
                            self.jump_scroll_forwards(ctx, h.into());
                            *self.currently_dragging.borrow_mut() = false;
                        }
                        SBRelPosition::On => {
                            *self.currently_dragging.borrow_mut() = true;
                            *self.start_drag_position.borrow_mut() = y;
                        }
                        SBRelPosition::None => {
                            *self.currently_dragging.borrow_mut() = false;
                        }
                    },
                }
                (true, EventResponses::default())
            }
            _ => {
                *self.currently_dragging.borrow_mut() = false;
                (false, EventResponses::default())
            }
        }
    }

    pub fn drag_while_dragging(&self, ctx: &Context, ev: MouseEvent) -> (bool, EventResponses) {
        let h = ctx.get_height();
        let y = ev.row as usize;
        let start_drag_pos = *self.start_drag_position.borrow();
        if y == start_drag_pos {
            return (false, EventResponses::default());
        }

        // only allow dragging if the scrollbar is 1 away from the last
        // drag position
        if !(y == start_drag_pos.saturating_sub(1) || y == start_drag_pos + 1) {
            *self.currently_dragging.borrow_mut() = false;
            return self.receive_mouse_event(ctx, ev); // resend the event as a non-dragging event
        }

        // consider dragging on the arrow keys to be a drag ONLY if the
        // mouse is already a single character away from each
        // otherwise, cancel the drag and perform a single scroll
        if *self.has_arrows.borrow() {
            if y == 0 && start_drag_pos != 1 {
                *self.currently_dragging.borrow_mut() = false;
                self.scroll_backwards(ctx);
                return (true, EventResponses::default());
            }
            let sb_len_chs = h as usize;
            if y == sb_len_chs.saturating_sub(1) && start_drag_pos != sb_len_chs.saturating_sub(2) {
                *self.currently_dragging.borrow_mut() = false;
                self.scroll_forwards(ctx, h.into());
                return (true, EventResponses::default());
            }
        }

        match y.cmp(&start_drag_pos) {
            Ordering::Greater => {
                self.drag_forwards_by_1_ch(ctx, h.into());
            }
            Ordering::Less => {
                self.drag_backwards_by_1_ch(ctx, h.into());
            }
            Ordering::Equal => {}
        }
        *self.start_drag_position.borrow_mut() = y;
        (false, EventResponses::default())
    }

    pub fn receive_external_mouse_event(
        &self, ctx: &Context, ev: RelMouseEvent,
    ) -> (bool, EventResponses) {
        let curr_dragging = *self.currently_dragging.borrow();
        if !curr_dragging {
            return (false, EventResponses::default());
        }
        match ev.kind {
            MouseEventKind::Drag(MouseButton::Left) => self.drag_while_dragging(ctx, ev.into()),
            _ => {
                *self.currently_dragging.borrow_mut() = false;
                (false, EventResponses::default())
            }
        }
    }
}

impl HorizontalScrollbar {
    pub fn receive_mouse_event(&self, ctx: &Context, ev: MouseEvent) -> (bool, EventResponses) {
        let curr_dragging = *self.currently_dragging.borrow();
        let w = ctx.get_width();
        match ev.kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft => {
                self.scroll_backwards(ctx);
                (true, EventResponses::default())
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollRight => {
                self.scroll_forwards(ctx, w.into());
                (true, EventResponses::default())
            }
            MouseEventKind::Up(MouseButton::Left) => {
                *self.currently_dragging.borrow_mut() = false;
                (true, EventResponses::default())
            }

            MouseEventKind::Drag(MouseButton::Left) if curr_dragging => {
                self.drag_while_dragging(ctx, ev)
            }
            MouseEventKind::Down(MouseButton::Left) if !curr_dragging => {
                let x = ev.column as usize;
                let has_arrows = *self.has_arrows.borrow();
                match true {
                    _ if has_arrows && x == 0 => {
                        self.scroll_backwards(ctx);
                        *self.currently_dragging.borrow_mut() = false;
                    }
                    _ if has_arrows && x == (w as usize).saturating_sub(1) => {
                        self.scroll_forwards(ctx, w.into());
                        *self.currently_dragging.borrow_mut() = false;
                    }
                    _ => match self.position_relative_to_scrollbar(w.into(), x) {
                        SBRelPosition::Before => {
                            self.jump_scroll_backwards(ctx, w.into());
                            *self.currently_dragging.borrow_mut() = false;
                        }
                        SBRelPosition::After => {
                            self.jump_scroll_forwards(ctx, w.into());
                            *self.currently_dragging.borrow_mut() = false;
                        }
                        SBRelPosition::On => {
                            *self.currently_dragging.borrow_mut() = true;
                            *self.start_drag_position.borrow_mut() = x;
                        }
                        SBRelPosition::None => {
                            *self.currently_dragging.borrow_mut() = false;
                        }
                    },
                }
                (true, EventResponses::default())
            }
            _ => {
                *self.currently_dragging.borrow_mut() = false;
                (false, EventResponses::default())
            }
        }
    }

    pub fn drag_while_dragging(&self, ctx: &Context, ev: MouseEvent) -> (bool, EventResponses) {
        let w = ctx.get_width();
        let x = ev.column as usize;
        let start_drag_pos = *self.start_drag_position.borrow();
        if x == start_drag_pos {
            return (false, EventResponses::default());
        }
        let has_arrows = *self.has_arrows.borrow();
        if has_arrows {
            if x == 0 && start_drag_pos != 1 {
                *self.currently_dragging.borrow_mut() = false;
                self.scroll_backwards(ctx);
                return (true, EventResponses::default());
            }
            let sb_len_chs = w;
            if x == (sb_len_chs as usize).saturating_sub(1)
                && start_drag_pos != (sb_len_chs as usize).saturating_sub(2)
            {
                *self.currently_dragging.borrow_mut() = false;
                self.scroll_forwards(ctx, w.into());
                return (true, EventResponses::default());
            }
        }

        match x.cmp(&start_drag_pos) {
            Ordering::Greater => {
                self.drag_forwards_by_1_ch(ctx, w.into());
            }
            Ordering::Less => {
                self.drag_backwards_by_1_ch(ctx, w.into());
            }
            Ordering::Equal => {}
        }
        *self.start_drag_position.borrow_mut() = x;
        (false, EventResponses::default())
    }

    pub fn receive_external_mouse_event(
        &self, ctx: &Context, ev: RelMouseEvent,
    ) -> (bool, EventResponses) {
        let curr_dragging = *self.currently_dragging.borrow();
        if !curr_dragging {
            return (false, EventResponses::default());
        }
        match ev.kind {
            MouseEventKind::Drag(MouseButton::Left) => self.drag_while_dragging(ctx, ev.into()),
            _ => {
                *self.currently_dragging.borrow_mut() = false;
                (false, EventResponses::default())
            }
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for VerticalScrollbar {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::KeyCombo(ke) => self.receive_key_event(ke, ctx),
            Event::Mouse(me) => self.receive_mouse_event(ctx, me),
            Event::ExternalMouse(me) => self.receive_external_mouse_event(ctx, me),
            _ => (false, EventResponses::default()),
        }
    }
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        let content = self.get_content(ctx);
        self.pane.set_content_if_diff(content);
        self.pane.drawing(ctx, force_update)
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for HorizontalScrollbar {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::KeyCombo(ke) => self.receive_key_event(ke, ctx),
            Event::Mouse(me) => self.receive_mouse_event(ctx, me),
            Event::ExternalMouse(me) => self.receive_external_mouse_event(ctx, me),
            _ => (false, EventResponses::default()),
        }
    }
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        let content = self.get_content(ctx);
        self.pane.set_content_if_diff(content);
        self.pane.drawing(ctx, force_update)
    }
}

/// test
#[cfg(test)]
mod tests {
    use {super::*, std::fmt::Write};

    #[test]
    fn test_scrollbar_drawing() {
        let w = 10;
        let sub = 2;
        let hat = crate::SortingHat::default();
        let (ev_tx, _) = tokio::sync::mpsc::channel(1);
        let ctx = Context::new_context_for_screen_no_dur(&hat, ev_tx)
            .with_height(1)
            .with_width(w);

        let width = DynVal::FULL.minus(sub.into());
        let width_val = width.get_val(ctx.get_width());
        assert_eq!(width_val, w as i32 - sub);
        let sb = HorizontalScrollbar::new(&ctx, width, ctx.size, w as usize * 2);
        assert!(*sb.has_arrows.borrow());

        let dr = sb.drawing_runes(ctx.get_width().into()).iter().fold(
            String::new(),
            |mut output, dc| {
                let _ = write!(output, "{}", dc.ch);
                output
            },
        );
        assert_eq!(dr.to_string(), "◀███     ▶");
    }
}
