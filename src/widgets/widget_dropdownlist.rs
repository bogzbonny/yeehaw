use {
    super::{Selectability, VerticalScrollbar, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        element::RelocationRequest, Context, DrawCh, DrawChPos, Element, ElementID, Event,
        EventResponse, EventResponses, Keyboard as KB, Priority, ReceivableEventChanges, RgbColour,
        SclLocationSet, SclVal, SortingHat, Style, UpwardPropagator, ZIndex,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

//TODO multiline dropdown entry support

#[derive(Clone)]
pub struct DropdownList {
    pub base: WidgetBase,
    pub entries: Rc<RefCell<Vec<String>>>,
    pub left_padding: Rc<RefCell<usize>>,
    pub specified_width: Rc<RefCell<Option<SclVal>>>, // width explicitly set by the caller
    pub selected: Rc<RefCell<usize>>,                 // the entry which has been selected
    pub cursor: Rc<RefCell<usize>>, // the entry that is currently hovered while open
    pub open: Rc<RefCell<bool>>,    // if the list is open
    pub max_expanded_height: Rc<RefCell<usize>>, // the max height of the entire dropdown list when expanded
    pub dropdown_arrow: Rc<RefCell<DrawCh>>,     // ▼
    pub cursor_style: Rc<RefCell<Style>>,        // style for the selected entry
    #[allow(clippy::type_complexity)]
    pub selection_made_fn: Rc<RefCell<Box<dyn FnMut(Context, String) -> EventResponses>>>, // function which executes when button moves from pressed -> unpressed
    pub scrollbar: VerticalScrollbar, // embedded scrollbar in dropdown list
}

impl DropdownList {
    const KIND: &'static str = "widget_dropdownlist";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new()
            .with_bg(RgbColour::YELLOW)
            .with_fg(RgbColour::BLACK),
        ready_style: Style::new()
            .with_bg(RgbColour::WHITE)
            .with_fg(RgbColour::BLACK),
        unselectable_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::BLACK),
    };

    const STYLE_SCROLLBAR: WBStyles = WBStyles {
        selected_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::WHITE),
        ready_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::WHITE),
        unselectable_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::WHITE),
    };

    const STYLE_DD_CURSOR: Style = Style::new().with_bg(RgbColour::BLUE);

    const DEFAULT_DROPDOWN_ARROW: DrawCh = DrawCh::new(
        '▼',
        false,
        Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::BLACK),
    );

    // needs to be slightly above other widgets to select properly
    // if widgets overlap
    const Z_INDEX: i32 = super::widget::WIDGET_Z_INDEX - 1;

    pub fn default_receivable_events() -> Vec<Event> {
        vec![
            KB::KEY_ENTER.into(),
            KB::KEY_DOWN.into(),
            KB::KEY_UP.into(),
            KB::KEY_K.into(),
            KB::KEY_J.into(),
            KB::KEY_SPACE.into(),
        ]
    }

    pub fn new(
        hat: &SortingHat, _ctx: &Context, entries: Vec<String>,
        selection_made_fn: Box<dyn FnMut(Context, String) -> EventResponses>,
    ) -> Self {
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            SclVal::new_fixed(0), // NOTE width is set later
            SclVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        let sb = VerticalScrollbar::new(hat, SclVal::new_fixed(0), 0)
            .without_arrows()
            .with_styles(Self::STYLE_SCROLLBAR);

        //wire the scrollbar to the dropdown list
        let wb_ = wb.clone();
        let hook = Box::new(move |ctx, y| wb_.set_content_y_offset(&ctx, y));
        *sb.position_changed_hook.borrow_mut() = Some(hook);

        let d = DropdownList {
            base: wb,
            entries: Rc::new(RefCell::new(entries)),
            left_padding: Rc::new(RefCell::new(1)),
            specified_width: Rc::new(RefCell::new(None)),
            selected: Rc::new(RefCell::new(0)),
            cursor: Rc::new(RefCell::new(0)),
            open: Rc::new(RefCell::new(false)),
            max_expanded_height: Rc::new(RefCell::new(10)),
            dropdown_arrow: Rc::new(RefCell::new(Self::DEFAULT_DROPDOWN_ARROW)),
            cursor_style: Rc::new(RefCell::new(Self::STYLE_DD_CURSOR)),
            selection_made_fn: Rc::new(RefCell::new(selection_made_fn)),
            scrollbar: sb,
        };
        d.base.set_scl_width(d.calculate_scl_width());
        d
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn with_arrow(self, ch: DrawCh) -> Self {
        *self.dropdown_arrow.borrow_mut() = ch;
        self
    }

    pub fn with_width(self, width: SclVal) -> Self {
        *self.specified_width.borrow_mut() = Some(width);
        self.base.set_scl_width(self.calculate_scl_width());
        self
    }

    pub fn with_left_padding(self, padding: usize) -> Self {
        *self.left_padding.borrow_mut() = padding;
        self.base.set_scl_width(self.calculate_scl_width());
        self
    }

    pub fn with_max_expanded_height(self, height: usize) -> Self {
        *self.max_expanded_height.borrow_mut() = height;
        self.scrollbar.set_height(
            SclVal::new_fixed(height as i32), // view height (same as the dropdown list height)
            SclVal::new_fixed(height.saturating_sub(1) as i32), // scrollbar height (1 less, b/c scrollbar's below the drop-arrow)
            self.entries.borrow().len(),                        // scrollable domain
        );
        self
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }

    // ----------------------------------------------

    pub fn correct_offsets(&self, ctx: &Context) {
        let cursor_pos = *self.cursor.borrow();
        self.base
            .correct_offsets_to_view_position(ctx, 0, cursor_pos);
        self.scrollbar.external_change(
            ctx,
            *self.base.pane.content_view_offset_y.borrow(),
            self.base.content_height(),
        );
    }

    pub fn calculate_scl_width(&self) -> SclVal {
        if let Some(ref w) = *self.specified_width.borrow() {
            return w.clone();
        }
        let left_padding = *self.left_padding.borrow() as i32;
        let max_entry_width = self
            .entries
            .borrow()
            .iter()
            .map(|r| r.chars().count())
            .max()
            .unwrap_or(0) as i32;
        let arrow_width = 1;
        SclVal::new_fixed(left_padding + max_entry_width + arrow_width)
    }

    pub fn padded_entry_text(&self, ctx: &Context, i: usize) -> String {
        let entry = self.entries.borrow()[i].clone();
        let entry_len = entry.chars().count();
        //let width = self.base.get_width(ctx);
        let width = self.calculate_scl_width().get_val(ctx.get_width());
        let left_padding = *self.left_padding.borrow();
        let right_padding = width.saturating_sub(entry_len as i32 + left_padding as i32);
        let pad_left = " ".repeat(left_padding);
        let pad_right = " ".repeat(right_padding as usize);
        format!("{}{}{}", pad_left, entry, pad_right)
    }

    // doesn't include the arrow text
    pub fn text(&self, ctx: &Context) -> String {
        if !*self.open.borrow() {
            return self.padded_entry_text(ctx, *self.selected.borrow());
        }
        let mut out = String::new();
        let entries_len = self.entries.borrow().len();
        for i in 0..entries_len {
            out += &self.padded_entry_text(ctx, i);
            if i != entries_len.saturating_sub(1) {
                out += "\n";
            }
        }
        out
    }

    // the height of the dropdown list while expanded
    pub fn expanded_height(&self) -> usize {
        let max_height = *self.max_expanded_height.borrow();
        if self.entries.borrow().len() > max_height {
            return max_height;
        }
        self.entries.borrow().len()
    }

    // whether or not the dropdown list should display a scrollbar
    pub fn display_scrollbar(&self) -> bool {
        self.entries.borrow().len() > self.expanded_height()
    }

    pub fn perform_open(&self, ctx: &Context) -> EventResponse {
        *self.open.borrow_mut() = true;
        *self.cursor.borrow_mut() = *self.selected.borrow();
        let h = self.expanded_height() as i32;
        self.base.set_scl_height(SclVal::new_fixed(h));

        // must set the content for the offsets to be correct
        self.base.set_content_from_string(ctx, &self.text(ctx));
        self.correct_offsets(ctx);

        let rr = RelocationRequest::new_down(h - 1);
        EventResponse::default().with_relocation(rr)
    }

    pub fn perform_close(&self, ctx: &Context, escaped: bool) -> EventResponses {
        *self.open.borrow_mut() = false;
        *self.base.pane.content_view_offset_y.borrow_mut() = 0;
        self.scrollbar
            .external_change(ctx, 0, self.base.content_height());
        self.base.set_scl_height(SclVal::new_fixed(1));
        let mut resps = if !escaped && *self.selected.borrow() != *self.cursor.borrow() {
            *self.selected.borrow_mut() = *self.cursor.borrow();
            (self.selection_made_fn.borrow_mut())(
                ctx.clone(),
                self.entries.borrow()[*self.selected.borrow()].clone(),
            )
        } else {
            EventResponses::default()
        };

        let rr = RelocationRequest::new_down(-(self.expanded_height() as i32 - 1));
        let resp = EventResponse::default().with_relocation(rr);
        resps.push(resp);
        resps
    }

    pub fn cursor_up(&self, ctx: &Context) {
        if *self.cursor.borrow() > 0 {
            *self.cursor.borrow_mut() -= 1;
        }
        self.correct_offsets(ctx);
    }

    pub fn cursor_down(&self, ctx: &Context) {
        if *self.cursor.borrow() < self.entries.borrow().len().saturating_sub(1) {
            *self.cursor.borrow_mut() += 1;
        }
        self.correct_offsets(ctx);
    }
}

impl Widget for DropdownList {
    fn get_z_index(&self) -> ZIndex {
        Self::Z_INDEX // slightly lower than the rest of the widgets so that the dropdown list will sit above the other widgets
    }

    fn set_selectability_pre_hook(&self, ctx: &Context, s: Selectability) -> EventResponses {
        if self.base.get_selectability() == Selectability::Selected
            && s != Selectability::Selected
            && *self.open.borrow()
        {
            return self.perform_close(ctx, true);
        }
        EventResponses::default()
    }
}

impl Element for DropdownList {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }
                let open = *self.open.borrow();
                return match true {
                    _ if !open
                        && (ke[0].matches_key(&KB::KEY_ENTER)
                            || ke[0].matches_key(&KB::KEY_DOWN)
                            || ke[0].matches_key(&KB::KEY_J)
                            || ke[0].matches_key(&KB::KEY_UP)
                            || ke[0].matches_key(&KB::KEY_K)) =>
                    {
                        (true, self.perform_open(ctx).into())
                    }
                    _ if open && ke[0].matches_key(&KB::KEY_ENTER) => {
                        (true, self.perform_close(ctx, false))
                    }
                    _ if open && ke[0].matches_key(&KB::KEY_DOWN)
                        || ke[0].matches_key(&KB::KEY_J) =>
                    {
                        self.cursor_down(ctx);
                        (true, EventResponses::default())
                    }
                    _ if open && ke[0].matches_key(&KB::KEY_UP)
                        || ke[0].matches_key(&KB::KEY_K) =>
                    {
                        self.cursor_up(ctx);
                        (true, EventResponses::default())
                    }
                    _ if open && ke[0].matches_key(&KB::KEY_SPACE) => {
                        if self.scrollbar.get_selectability() != Selectability::Selected {
                            self.scrollbar
                                .set_selectability(ctx, Selectability::Selected);
                        }
                        self.scrollbar.receive_event(ctx, Event::KeyCombo(ke))
                    }
                    _ => (false, EventResponses::default()),
                };
            }
            Event::Mouse(me) => {
                let (mut clicked, mut dragging, mut scroll_up, mut scroll_down) =
                    (false, false, false, false);
                match me.kind {
                    MouseEventKind::Up(MouseButton::Left) => clicked = true,
                    MouseEventKind::Drag(MouseButton::Left) => dragging = true,
                    MouseEventKind::ScrollUp => scroll_up = true,
                    MouseEventKind::ScrollDown => scroll_down = true,
                    _ => {}
                }
                let open = *self.open.borrow();

                return match true {
                    _ if !open && clicked => (true, self.perform_open(ctx).into()),
                    _ if open && scroll_up => {
                        self.cursor_up(ctx);
                        (true, EventResponses::default())
                    }
                    _ if open && scroll_down => {
                        self.cursor_down(ctx);
                        (true, EventResponses::default())
                    }
                    _ if open && (!clicked || dragging) => {
                        let (x, y) = (me.column as usize, me.row as usize);

                        // change hovering location to the ev

                        // on arrow
                        if y == 0 && x == self.base.get_width(ctx).saturating_sub(1) {
                            return (true, EventResponses::default());

                        // on scrollbar
                        } else if y > 0
                            && x == self.base.get_width(ctx).saturating_sub(1)
                            && self.display_scrollbar()
                        {
                            if dragging {
                                if self.scrollbar.get_selectability() != Selectability::Selected {
                                    let _ = self
                                        .scrollbar
                                        .set_selectability(ctx, Selectability::Selected);
                                }
                                // send the the event to the scrollbar (x adjusted to 0)
                                let mut me_ = me;
                                me_.column = 0;
                                me_.row = y.saturating_sub(1) as u16;
                                return self.scrollbar.receive_event(ctx, Event::Mouse(me_));
                            }
                            return (true, EventResponses::default());
                        } else {
                            *self.cursor.borrow_mut() =
                                y + *self.base.pane.content_view_offset_y.borrow();
                        }
                        let _ = self.scrollbar.set_selectability(ctx, Selectability::Ready);
                        (true, EventResponses::default())
                    }
                    _ if open && clicked => {
                        let (x, y) = (me.column as usize, me.row as usize);
                        if y > 0
                            && x == self.base.get_width(ctx).saturating_sub(1)
                            && self.display_scrollbar()
                        {
                            if self.scrollbar.get_selectability() != Selectability::Selected {
                                let _ = self
                                    .scrollbar
                                    .set_selectability(ctx, Selectability::Selected);
                            }
                            // send the the event to the scrollbar (x adjusted to 0)
                            let mut me_ = me;
                            me_.column = 0;
                            me_.row = y.saturating_sub(1) as u16;
                            return self.scrollbar.receive_event(ctx, Event::Mouse(me_));
                        }

                        // on arrow close without change
                        if y == 0 && x == self.base.get_width(ctx).saturating_sub(1) {
                            return (true, self.perform_close(ctx, true));
                        }
                        let _ = self.scrollbar.set_selectability(ctx, Selectability::Ready);
                        *self.cursor.borrow_mut() =
                            y + *self.base.pane.content_view_offset_y.borrow();
                        (true, self.perform_close(ctx, false))
                    }
                    _ => (false, EventResponses::default()),
                };
            }
            _ => {}
        }
        (false, EventResponses::default())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.base.set_content_from_string(ctx, &self.text(ctx));

        let open = *self.open.borrow();

        // highlight the hovering entry
        if open {
            self.base
                .pane
                .content
                .borrow_mut()
                .change_style_along_y(*self.cursor.borrow(), *self.cursor_style.borrow());
        }

        let mut chs = self.base.drawing(ctx);

        // set the scrollbar on top of the content
        if open && self.display_scrollbar() {
            let mut sb_chs = self.scrollbar.drawing(ctx);
            // shift the scrollbar content to below the arrow
            for ch in sb_chs.iter_mut() {
                ch.x += self.base.get_width(ctx).saturating_sub(1) as u16;
                ch.y += 1;
            }
            chs.extend(sb_chs);
        }

        // set the arrow
        let arrow_ch = DrawChPos::new(
            *self.dropdown_arrow.borrow(),
            self.base.get_width(ctx).saturating_sub(1) as u16,
            0,
        );
        chs.push(arrow_ch);

        chs
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
    fn get_scl_location_set(&self) -> SclLocationSet {
        self.base.get_scl_location_set()
    }
    fn set_scl_location_set(&self, loc: SclLocationSet) {
        self.base.set_scl_location_set(loc)
    }
    fn visible(&self) -> bool {
        self.base.visible()
    }
    fn set_visible(&self, v: bool) {
        self.base.set_visible(v)
    }
}
