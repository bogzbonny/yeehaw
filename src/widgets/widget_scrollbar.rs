use {
    super::{SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponse, Keyboard as KB, Priority,
        ReceivableEventChanges, RgbColour, SortingHat, Style, UpwardPropagator, YHAttributes,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::ops::{Deref, DerefMut},
    std::{cell::RefCell, rc::Rc},
};

// TODO Better handling of dragging if the mouse leaves the scrollbar
// location but is still dragging

// NOTE the code in this file is structured in a "zipper" fashion between vertical and horizontal
// scrollbar, although this increases the line count (extra impl lines everywhere) it is useful for
// ensuring that the two scrollbars have consistent code, as similar code is always grouped together.

// up is backwards, down is forwards
pub struct VerticalScrollbar(Scrollbar);

// left is backwards, right is forwards
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

pub enum ScrollbarPositions {
    NoScrollbar,
    LeftScrollbar,
    RightScrollbar,
    TopScrollbar,
    BottomScrollbar,
}

impl VerticalScrollbar {
    const KIND: &'static str = "widget_vertical_scrollbar";
    pub fn default_receivable_events() -> Vec<Event> {
        vec![KB::KEY_UP.into(), KB::KEY_DOWN.into(), KB::KEY_SPACE.into()]
    }
    pub fn new(
        hat: &SortingHat, ctx: &Context, scrollable_view_height: SclVal, scrollable_height: usize,
    ) -> Self {
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            ctx.clone(),
            SclVal::new_fixed(1),
            scrollable_view_height.clone(),
            Scrollbar::STYLE,
            Self::default_receivable_events(),
        );
        VerticalScrollbar(Scrollbar {
            base: wb,
            scrollable_domain_chs: Rc::new(RefCell::new(scrollable_height)),
            scrollable_view_chs: Rc::new(RefCell::new(scrollable_view_height.clone())),
            scrollbar_length_chs: Rc::new(RefCell::new(scrollable_view_height)),
            scrollable_position: Rc::new(RefCell::new(0)),
            has_arrows: Rc::new(RefCell::new(true)),
            backwards_arrow: Rc::new(RefCell::new('▲')),
            forwards_arrow: Rc::new(RefCell::new('▼')),
            empty_block: Rc::new(RefCell::new(' ')),
            full_block: Rc::new(RefCell::new('█')),
            forwards_half_block: Rc::new(RefCell::new('▄')),
            backwards_half_block: Rc::new(RefCell::new('▀')),
            unnessecary: Rc::new(RefCell::new('░')),
            position_changed_hook: Rc::new(RefCell::new(None)),
            currently_dragging: Rc::new(RefCell::new(false)),
            start_drag_position: Rc::new(RefCell::new(0)),
            jump_scroll_percent: Rc::new(RefCell::new(10)),
            jump_scroll_min_amount: Rc::new(RefCell::new(3)),
        })
    }

    pub fn set_height(
        &self, view_height: SclVal, scrollbar_length: SclVal, scrollable_height: usize,
    ) {
        *self.scrollable_view_chs.borrow_mut() = view_height;
        self.base.set_attr_scl_height(scrollbar_length.clone());
        *self.scrollbar_length_chs.borrow_mut() = scrollbar_length;
        *self.scrollable_domain_chs.borrow_mut() = scrollable_height;
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }

    pub fn without_arrows(self) -> Self {
        *self.has_arrows.borrow_mut() = false;
        self
    }
}

impl HorizontalScrollbar {
    const KIND: &'static str = "widget_horizontal_scrollbar";
    pub fn default_receivable_events() -> Vec<Event> {
        vec![KB::KEY_LEFT.into(), KB::KEY_RIGHT.into()]
    }
    pub fn new(
        hat: &SortingHat, ctx: &Context, scrollable_view_width: SclVal, scrollable_width: usize,
    ) -> Self {
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            ctx.clone(),
            scrollable_view_width.clone(),
            SclVal::new_fixed(1),
            Scrollbar::STYLE,
            Self::default_receivable_events(),
        );
        HorizontalScrollbar(Scrollbar {
            base: wb,
            scrollable_domain_chs: Rc::new(RefCell::new(scrollable_width)),
            scrollable_view_chs: Rc::new(RefCell::new(scrollable_view_width.clone())),
            scrollbar_length_chs: Rc::new(RefCell::new(scrollable_view_width)),
            scrollable_position: Rc::new(RefCell::new(0)),
            has_arrows: Rc::new(RefCell::new(true)),
            backwards_arrow: Rc::new(RefCell::new('◀')),
            forwards_arrow: Rc::new(RefCell::new('▶')),
            empty_block: Rc::new(RefCell::new(' ')),
            full_block: Rc::new(RefCell::new('█')),
            forwards_half_block: Rc::new(RefCell::new('▐')),
            backwards_half_block: Rc::new(RefCell::new('▌')),
            unnessecary: Rc::new(RefCell::new('░')),
            position_changed_hook: Rc::new(RefCell::new(None)),
            currently_dragging: Rc::new(RefCell::new(false)),
            start_drag_position: Rc::new(RefCell::new(0)),
            jump_scroll_percent: Rc::new(RefCell::new(10)),
            jump_scroll_min_amount: Rc::new(RefCell::new(3)),
        })
    }

    pub fn set_width(&self, view_width: SclVal, scrollbar_length: SclVal, scrollable_width: usize) {
        *self.scrollable_view_chs.borrow_mut() = view_width;
        self.base.set_attr_scl_width(scrollbar_length.clone());
        *self.scrollbar_length_chs.borrow_mut() = scrollbar_length;
        *self.scrollable_domain_chs.borrow_mut() = scrollable_width;
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }

    pub fn without_arrows(self) -> Self {
        *self.has_arrows.borrow_mut() = false;
        self
    }
}

// ------------------------------------------------------------------

// The Scrollbar is a base type of common logic to build the
// vertical and horizontal scrollbars off of.
//
// For vertical scrollbars:
//   - "backwards" should be thought of as "up" and
//   - "forwards" should be thought of as "down".
//
// For horizontal scrollbars:
//   - "backwards" should be thought of as "left" and
//   - "forwards" should be thought of as "right".
#[derive(Clone)]
pub struct Scrollbar {
    pub base: WidgetBase,

    // The ScrollableDomainChs is the scrollable dimension in true characters.
    // It is AFFECTED by the scrollbar and NOT the literal area of the scrollbar
    // itself.
    pub scrollable_domain_chs: Rc<RefCell<usize>>, // how large the area is that can be scrolled

    // how much of the scrollable area is visible in true chars.
    pub scrollable_view_chs: Rc<RefCell<SclVal>>,

    // Length of the actual scrollbar (and arrows) in true characters.
    // Typically this is the same as ScrollableViewChs, however some situations
    // call for a different size scrollbar than the scrollable view, such as the
    // dropdown menu with a scrollbar below the dropdown-arrow.
    pub scrollbar_length_chs: Rc<RefCell<SclVal>>,

    // how far down the area is scrolled from the top in true chars.
    // The ScrollablePosition will be the first line of the area scrolled to.
    pub scrollable_position: Rc<RefCell<usize>>,

    pub has_arrows: Rc<RefCell<bool>>, // if the scrollbar has arrows

    pub backwards_arrow: Rc<RefCell<char>>,
    pub forwards_arrow: Rc<RefCell<char>>,
    pub empty_block: Rc<RefCell<char>>,
    pub full_block: Rc<RefCell<char>>,
    pub forwards_half_block: Rc<RefCell<char>>,
    pub backwards_half_block: Rc<RefCell<char>>,
    pub unnessecary: Rc<RefCell<char>>, // for when the scrollbar ought not to exist

    // function the scrollbar will call everytime there is a position change
    pub position_changed_hook: Rc<RefCell<Option<Rc<RefCell<dyn FnMut(usize)>>>>>,

    // is the scrollbar currently being dragged?
    pub currently_dragging: Rc<RefCell<bool>>,
    pub start_drag_position: Rc<RefCell<usize>>, // in true characters

    // The percent (0-100) of the total scrollable domain
    // to scroll when a click in the scrollbar whitespace is made.
    pub jump_scroll_percent: Rc<RefCell<usize>>,

    // minimum amount to scroll during a jump scroll
    pub jump_scroll_min_amount: Rc<RefCell<usize>>,
}

pub enum SBRelPosition {
    None,
    Before,
    On,
    After,
}

impl Scrollbar {
    const STYLE: WBStyles = WBStyles {
        //
        selected_style: Style::new()
            .with_bg(RgbColour::LIGHT_YELLOW2)
            .with_fg(RgbColour::BLACK),
        ready_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::WHITE),
        unselectable_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::WHITE),
    };

    // if the Scrollbar currently cannot be used due to insufficient domain.
    pub fn is_currently_unnecessary(&self, p_size: usize) -> bool {
        *self.scrollable_domain_chs.borrow() <= self.scrollable_view_chs.borrow().get_val(p_size)
    }

    pub fn jump_scroll_amount(&self) -> usize {
        let js = *self.scrollable_domain_chs.borrow() * *self.jump_scroll_percent.borrow() / 100;
        if js < *self.jump_scroll_min_amount.borrow() {
            *self.jump_scroll_min_amount.borrow()
        } else {
            js
        }
    }

    // scroll to the position within the scrollable domain.
    pub fn scroll_to_position(&self, p_size: usize, mut position: usize) {
        let sc_dom_len = *self.scrollable_domain_chs.borrow();
        let sc_view_len = self.scrollable_view_chs.borrow().get_val(p_size);
        if position > sc_dom_len - sc_view_len {
            position = sc_dom_len - sc_view_len
        }
        *self.scrollable_position.borrow_mut() = position;
        if let Some(hook) = self.position_changed_hook.borrow().as_ref() {
            hook.borrow_mut()(position);
        }
    }

    pub fn jump_scroll_backwards(&self, p_size: usize) {
        self.scroll_to_position(
            p_size,
            *self.scrollable_position.borrow() - self.jump_scroll_amount(),
        );
    }

    pub fn jump_scroll_forwards(&self, p_size: usize) {
        self.scroll_to_position(
            p_size,
            *self.scrollable_position.borrow() + self.jump_scroll_amount(),
        );
    }

    pub fn can_scroll_backwards(&self) -> bool {
        *self.scrollable_position.borrow() > 0
    }

    pub fn scroll_backwards(&self) {
        if !self.can_scroll_backwards() {
            return;
        }
        *self.scrollable_position.borrow_mut() -= 1;
        if let Some(hook) = self.position_changed_hook.borrow().as_ref() {
            hook.borrow_mut()(*self.scrollable_position.borrow());
        }
    }

    pub fn can_scroll_forwards(&self, p_size: usize) -> bool {
        *self.scrollable_position.borrow()
            < *self.scrollable_domain_chs.borrow()
                - self.scrollable_view_chs.borrow().get_val(p_size)
    }

    pub fn scroll_forwards(&self, p_size: usize) {
        if !self.can_scroll_forwards(p_size) {
            return;
        }
        *self.scrollable_position.borrow_mut() += 1;
        if let Some(hook) = self.position_changed_hook.borrow().as_ref() {
            hook.borrow_mut()(*self.scrollable_position.borrow());
        }
    }

    // the scrollbar domain is the total space which the scroll bar may occupy (both
    // the bar and the empty space above and below it) measured in half-increments.
    // Each half-increment represents half a character, as the scrollbar may use
    // half characters to represent its position.
    pub fn scrollbar_domain_in_half_increments(&self, p_size: usize) -> usize {
        // minus 2 for the backwards and forwards arrows
        let arrows = if *self.has_arrows.borrow() { 2 } else { 0 };
        let sc_len_chs = self.scrollbar_length_chs.borrow().get_val(p_size);
        // times 2 for half characters
        2 * (sc_len_chs.saturating_sub(arrows))
    }

    pub fn scroll_bar_size_in_half_increments(&self, p_size: usize) -> usize {
        let domain_incr = self.scrollbar_domain_in_half_increments(p_size);
        let percent_viewable = (self.scrollable_view_chs.borrow().get_val(p_size) as f64)
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

        scrollbar_incr
    }

    // the number of true view characters per full scrollbar character (aka 2
    // half-increments)
    pub fn true_chs_per_scrollbar_character(&self, p_size: usize) -> usize {
        let scrollbar_incr = self.scroll_bar_size_in_half_increments(p_size);
        (self.scrollbar_length_chs.borrow().get_val(p_size) as f64 / (scrollbar_incr as f64 / 2.0))
            as usize
    }

    pub fn set_selectability(&self, s: Selectability) -> EventResponse {
        *self.currently_dragging.borrow_mut() = false;
        self.base.set_selectability(s)
    }

    // Get an array of half-increments of the scrollbar domain area
    fn scroll_bar_domain_array_of_half_increments(&self, p_size: usize) -> Vec<bool> {
        //domainIncr := sb.ScrollBarDomainInHalfIncrements(pSize)
        //scrollbarIncr := sb.ScrollBarSizeInHalfIncrements(pSize)
        let domain_incr = self.scrollbar_domain_in_half_increments(p_size);
        let scrollbar_incr = self.scroll_bar_size_in_half_increments(p_size);

        // total increments within the scrollbar domain for space above and below the bar
        //totalSpacerIncr := domainIncr - scrollbarIncr
        let total_spacer_incr = domain_incr.saturating_sub(scrollbar_incr);

        //trueChsAbove := sb.ScrollablePosition
        let true_chs_above = *self.scrollable_position.borrow();

        //incrAbove := int(math.Round(
        //    float64(trueChsAbove) /
        //        float64(sb.ScrollableDomainChs-sb.ScrollableViewChs.GetVal(pSize)) * float64(totalSpacerIncr),
        //))
        let sc_dom_chs = *self.scrollable_domain_chs.borrow();
        let sc_view_chs = self.scrollable_view_chs.borrow().get_val(p_size);
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
        for i in incr_above..incr_above + scrollbar_incr {
            incr_filled[i] = true;
        }
        incr_filled
    }

    fn last_incr_filled(incr_filled: &[bool]) -> Option<usize> {
        for i in (0..incr_filled.len()).rev() {
            if incr_filled[i] {
                return Some(i);
            }
        }
        None
    }

    fn first_incr_filled(incr_filled: &[bool]) -> Option<usize> {
        for i in 0..incr_filled.len() {
            if incr_filled[i] {
                return Some(i);
            }
        }
        None
    }

    // used for mouse dragging the scrollbar. What the incrementIsFilled should look
    // like if it dragged down by one rune (aka 2 half increments)
    pub fn drag_forwards_by_1_ch(&self, p_size: usize) {
        let start_incrs = self.scroll_bar_domain_array_of_half_increments(p_size);
        let last_filled = Self::last_incr_filled(&start_incrs);
        let Some(last_filled) = last_filled else {
            return;
        };
        let mut goal_last_filled = last_filled + 2;
        if goal_last_filled >= start_incrs.len() {
            goal_last_filled = start_incrs.len() - 1;
        }
        loop {
            // safegaurd against infinite loop
            if !self.can_scroll_forwards(p_size) {
                return;
            }
            self.scroll_forwards(p_size);
            let current_incr = self.scroll_bar_domain_array_of_half_increments(p_size);
            let curr_last_filled = Self::last_incr_filled(&current_incr);
            if curr_last_filled == Some(goal_last_filled) {
                return;
            }
        }
    }

    // Same as DragForwardsBy1Ch but in the backwards direction
    pub fn drag_backwards_by_1_ch(&self, p_size: usize) {
        let start_incrs = self.scroll_bar_domain_array_of_half_increments(p_size);
        let first_filled = Self::first_incr_filled(&start_incrs);
        let Some(first_filled) = first_filled else {
            return;
        };
        let mut goal_first_filled = first_filled - 2;
        if goal_first_filled < 0 {
            goal_first_filled = 0;
        }
        loop {
            // safegaurd against infinite loop
            if !self.can_scroll_backwards() {
                return;
            }
            self.scroll_backwards();
            let current_incr = self.scroll_bar_domain_array_of_half_increments(p_size);
            let curr_first_filled = Self::first_incr_filled(&current_incr);
            if curr_first_filled == Some(goal_first_filled) {
                return;
            }
        }
    }

    pub fn scrollbar_domain_array_of_runes(&self, p_size: usize) -> Vec<char> {
        let incr_filled = self.scroll_bar_domain_array_of_half_increments(p_size);
        let mut rs = vec![];
        // determine the characters based on the filled increments
        for i in 0..incr_filled.len() {
            if i % 2 == 1 {
                match (incr_filled[i - 1], incr_filled[i]) {
                    (true, true) => rs.push(*self.full_block.borrow()),
                    (true, false) => rs.push(*self.backwards_half_block.borrow()),
                    (false, true) => rs.push(*self.forwards_half_block.borrow()),
                    (false, false) => rs.push(*self.empty_block.borrow()),
                }
            }
        }
        rs
    }

    pub fn drawing_runes(&self, p_size: usize) -> Vec<char> {
        let mut chs = vec![];
        if self.is_currently_unnecessary(p_size) {
            for _ in 0..self.scrollbar_length_chs.borrow().get_val(p_size) {
                chs.push(*self.unnessecary.borrow());
            }
        } else {
            if *self.has_arrows.borrow() {
                chs.push(*self.backwards_arrow.borrow());
            }
            chs.append(&mut self.scrollbar_domain_array_of_runes(p_size));
            if *self.has_arrows.borrow() {
                chs.push(*self.forwards_arrow.borrow());
            }
        }
        chs
    }

    // Call this when the position has been changed external to the scrollbar
    // new_view_offset is the new position of the view in full characters
    // new_view_domain is the number of full characters of the full scrollable domain
    pub fn external_change(&self, p_size: usize, new_view_offset: usize, new_domain_chs: usize) {
        *self.scrollable_position.borrow_mut() = new_view_offset;
        *self.scrollable_domain_chs.borrow_mut() = new_domain_chs;
        self.update_selectibility(p_size);
    }

    // process for the selectibility of the scrollbar
    pub fn update_selectibility(&self, p_size: usize) {
        if self.is_currently_unnecessary(p_size) {
            *self.currently_dragging.borrow_mut() = false;
            let _ = self.set_selectability(Selectability::Unselectable);
        } else {
            let _ = self.set_selectability(Selectability::Ready);
        }
    }

    // is the provided position before, on, or after the scrollbar?
    pub fn position_relative_to_scrollbar(&self, p_size: usize, mut pos: usize) -> SBRelPosition {
        // last pos the actual scrollbar may be in
        let mut last_scrollbar_pos = self.scrollbar_length_chs.borrow().get_val(p_size) - 1;

        if *self.has_arrows.borrow() {
            let sc_len_chs = self.scrollbar_length_chs.borrow().get_val(p_size);
            if pos == 0 || pos == sc_len_chs - 1 {
                return SBRelPosition::None;
            }
            pos -= 1; // account for the backwards arrow
            last_scrollbar_pos = sc_len_chs - 3; // account for the forwards arrow
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
            if *r == *self.full_block.borrow() {
                if first_full.is_none() {
                    first_full = Some(i);
                }
                last_full = i;
            }
            if *r == *self.backwards_half_block.borrow() {
                backwards_half_pos = i;
            }
            if *r == *self.forwards_half_block.borrow() {
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
// Specific implementations for the vertical and horizontal scrollbars

impl VerticalScrollbar {
    pub fn external_change(&self, new_view_offset: usize, new_domain_chs: usize) {
        *self.scrollable_position.borrow_mut() = new_view_offset;
        *self.scrollable_domain_chs.borrow_mut() = new_domain_chs;
        self.update_selectibility(self.base.get_last_ctx().get_height().into());
    }
}
impl HorizontalScrollbar {
    pub fn external_change(&self, new_view_offset: usize, new_domain_chs: usize) {
        *self.scrollable_position.borrow_mut() = new_view_offset;
        *self.scrollable_domain_chs.borrow_mut() = new_domain_chs;
        self.update_selectibility(self.base.get_last_ctx().get_width().into());
    }
}

impl VerticalScrollbar {
    pub fn resize_event(&self, ctx: &Context) {
        self.update_selectibility(ctx.get_height().into());
    }
}
impl HorizontalScrollbar {
    pub fn resize_event(&self, ctx: &Context) {
        self.update_selectibility(ctx.get_width().into());
    }
}

impl VerticalScrollbar {
    pub fn drawing_(&self, ctx: &Context) -> Vec<DrawChPos> {
        let chs = self.drawing_runes(ctx.get_height().into());

        // compile the runes into a vertical string
        let mut v_str = String::new();
        for (i, ch) in chs.iter().enumerate() {
            v_str.push(*ch);
            if i != chs.len() - 1 {
                v_str.push('\n');
            }
        }
        self.base.set_content_from_string(&v_str);
        self.base.drawing(ctx)
    }
}
impl HorizontalScrollbar {
    pub fn drawing_(&self, ctx: &Context) -> Vec<DrawChPos> {
        let h_str = self
            .drawing_runes(ctx.get_width().into())
            .iter()
            .collect::<String>();
        self.base.set_content_from_string(&h_str);
        self.base.drawing(ctx)
    }
}

// XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
impl VerticalScrollbar {
    pub fn receive_key_event(&self, ev: Vec<KeyPossibility>) -> (bool, EventResponse) {
        if self.selectedness() != Selectability::Selected {
            return (false, EventResponse::new());
        }

        for k in ev {
            match k {
                KeyPossibility::Up => {
                    self.scroll_backwards();
                    return (true, EventResponse::new());
                }
                KeyPossibility::Down => {
                    self.scroll_forwards(self.base.get_last_ctx().get_height().into());
                    return (true, EventResponse::new());
                }
                KeyPossibility::Space => {
                    self.jump_scroll_forwards(self.base.get_last_ctx().get_height().into());
                    return (true, EventResponse::new());
                }
            }
        }
        (false, EventResponse::new())
    }
}

/*
func (vsb *VerticalScrollbar) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
    ctx := vsb.GetParentCtx()
    if vsb.Selectedness != Selected {
        return false, yh.NewEventResponse()
    }

    switch {
    case yh.UpEKC.Matches(evs):
        vsb.ScrollBackwards()
        return true, yh.NewEventResponse()
    case yh.DownEKC.Matches(evs):
        vsb.ScrollForwards(ctx.GetHeight())
        return true, yh.NewEventResponse()
    case yh.SpaceEKC.Matches(evs):
        vsb.JumpScrollForwards(ctx.GetHeight())
        return true, yh.NewEventResponse()

    }
    return false, yh.NewEventResponse()
}

func (hsb *HorizontalScrollbar) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
    ctx := hsb.GetParentCtx()
    if hsb.Selectedness != Selected {
        return false, yh.NewEventResponse()
    }

    switch {
    case yh.LeftEKC.Matches(evs):
        hsb.ScrollBackwards()
        return true, yh.NewEventResponse()
    case yh.RightEKC.Matches(evs):
        hsb.ScrollForwards(ctx.GetWidth())
        return true, yh.NewEventResponse()
    }
    return false, yh.NewEventResponse()
}

func (vsb *VerticalScrollbar) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
    yh.Debug("VerticalScrollbar received mouse event: %v\n", ev)
    if vsb.Selectedness == Unselectable {
        return false, yh.NewEventResponse()
    }

    ctx := vsb.GetParentCtx()
    h := ctx.GetHeight()

    if ev.Buttons() == tcell.WheelDown {
        vsb.ScrollForwards(h)
        return true, yh.NewEventResponse()
    }

    if ev.Buttons() == tcell.WheelUp {
        vsb.ScrollBackwards()
        return true, yh.NewEventResponse()
    }

    if ev.Buttons() == tcell.Button1 { //left click
        _, y := ev.Position()

        if vsb.CurrentlyDragging {
            if y == vsb.StartDragPosition {
                return false, yh.NewEventResponse()
            }

            // only allow dragging if the scrollbar is 1 away from the last
            // drag position
            if !(y == vsb.StartDragPosition-1 || y == vsb.StartDragPosition+1) {
                vsb.CurrentlyDragging = false
                return vsb.ReceiveMouseEvent(ev)
            }

            // consider dragging on the arrow keys to be a drag ONLY if the
            // mouse is already a single character away from each
            // otherwise, cancel the drag and perform a single scroll
            if vsb.HasArrows {
                if y == 0 && vsb.StartDragPosition != 1 {
                    vsb.CurrentlyDragging = false
                    vsb.ScrollBackwards()
                    return true, yh.NewEventResponse()
                } else if y == vsb.ScrollbarLengthChs.GetVal(h)-1 &&
                    vsb.StartDragPosition != vsb.ScrollbarLengthChs.GetVal(h)-2 {

                    vsb.CurrentlyDragging = false
                    vsb.ScrollForwards(h)
                    return true, yh.NewEventResponse()
                }
            }

            change := y - vsb.StartDragPosition
            if change > 0 {
                vsb.DragForwardsBy1Ch(h)
            } else if change < 0 {
                vsb.DragBackwardsBy1Ch(h)
            }

            vsb.StartDragPosition = y
        } else {
            switch {
            case vsb.HasArrows && y == 0:
                vsb.ScrollBackwards()
                vsb.CurrentlyDragging = false
            case vsb.HasArrows && y == vsb.ScrollbarLengthChs.GetVal(h)-1:
                vsb.ScrollForwards(h)
                vsb.CurrentlyDragging = false
            default:
                rel := vsb.PositionRelativeToScrollbar(h, y)
                switch rel {
                case before:
                    vsb.JumpScrollBackwards(h)
                    vsb.CurrentlyDragging = false
                case after:
                    vsb.JumpScrollForwards(h)
                    vsb.CurrentlyDragging = false
                case on:
                    vsb.CurrentlyDragging = true
                    vsb.StartDragPosition = y
                }
            }
        }

    } else {
        vsb.CurrentlyDragging = false
    }
    return true, yh.NewEventResponse()
}

func (hsb *HorizontalScrollbar) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
    if hsb.Selectedness == Unselectable {
        return false, yh.NewEventResponse()
    }

    ctx := hsb.GetParentCtx()
    w := ctx.GetWidth()

    if ev.Buttons() == tcell.WheelLeft || ev.Buttons() == tcell.WheelUp {
        hsb.ScrollForwards(w)
        return true, yh.NewEventResponse()
    }

    if ev.Buttons() == tcell.WheelRight || ev.Buttons() == tcell.WheelDown {
        hsb.ScrollBackwards()
        return true, yh.NewEventResponse()
    }

    if ev.Buttons() == tcell.Button1 { //left click
        x, _ := ev.Position()

        if hsb.CurrentlyDragging {
            if x != hsb.StartDragPosition {

                // consider dragging on the arrow keys to be a drag ONLY if the
                // mouse is already a single character away from each
                // otherwise, cancel the drag and perform a single scroll
                if hsb.HasArrows {
                    if x == 0 && hsb.StartDragPosition != 1 {
                        hsb.CurrentlyDragging = false
                        hsb.ScrollBackwards()
                        return true, yh.NewEventResponse()
                    } else if x == hsb.ScrollbarLengthChs.GetVal(w)-1 &&
                        hsb.StartDragPosition != hsb.ScrollbarLengthChs.GetVal(w)-2 {

                        hsb.CurrentlyDragging = false
                        hsb.ScrollForwards(w)
                        return true, yh.NewEventResponse()
                    }
                }

                change := x - hsb.StartDragPosition
                if change > 0 {
                    hsb.DragForwardsBy1Ch(w)
                } else if change < 0 {
                    hsb.DragBackwardsBy1Ch(w)
                }
                //newPosition := hsb.ScrollablePosition + (change * hsb.TrueChsPerScrollbarCharacter())
                //hsb.ScrollToPosition(newPosition)
            }
            hsb.StartDragPosition = x
        } else {
            switch {
            case hsb.HasArrows && x == 0:
                hsb.ScrollBackwards()
                hsb.CurrentlyDragging = false
            case hsb.HasArrows && x == hsb.ScrollbarLengthChs.GetVal(w)-1:
                hsb.ScrollForwards(w)
                hsb.CurrentlyDragging = false
            default:
                rel := hsb.PositionRelativeToScrollbar(w, x)
                switch rel {
                case before:
                    hsb.JumpScrollBackwards(w)
                    hsb.CurrentlyDragging = false
                case after:
                    hsb.JumpScrollForwards(w)
                    hsb.CurrentlyDragging = false
                case on:
                    hsb.CurrentlyDragging = true
                    hsb.StartDragPosition = x
                }
            }
        }

    } else {
        hsb.CurrentlyDragging = false
    }
    return true, yh.NewEventResponse()
}
*/

impl Widget for Scrollbar {}

impl Element for Scrollbar {
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
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponse::default());
                }
                if ke[0].matches(&KB::KEY_ENTER) {
                    return (true, self.click());
                }
            }
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    return (true, self.click());
                }
            }
            _ => {}
        }
        (false, EventResponse::default())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        // need to re set the content in order to reflect active style
        self.base.set_content_from_string(&self.text());
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
