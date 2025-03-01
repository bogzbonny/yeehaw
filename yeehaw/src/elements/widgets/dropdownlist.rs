use {
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
};

// TODO multiline dropdown entry support
// TODO allow for scrollbar mouse events

#[derive(Clone)]
pub struct DropdownList {
    pub pane: SelectablePane,
    pub entries: Rc<RefCell<Vec<String>>>,
    pub left_padding: Rc<RefCell<usize>>,
    /// width explicitly set by the caller
    pub specified_width: Rc<RefCell<Option<DynVal>>>,
    /// the entry which has been selected
    pub selected: Rc<RefCell<usize>>,
    /// the entry that is currently hovered while open
    pub cursor: Rc<RefCell<usize>>,
    /// if the list is open
    pub open: Rc<RefCell<bool>>,
    /// the max height of the entire dropdown list when expanded
    pub max_expanded_height: Rc<RefCell<usize>>,
    /// ▼
    pub dropdown_arrow: Rc<RefCell<DrawCh>>,
    /// style for the selected entry
    pub cursor_style: Rc<RefCell<Style>>,
    /// activated when mouse is clicked down while over object
    pub clicked_down: Rc<RefCell<bool>>,
    #[allow(clippy::type_complexity)]
    /// function which executes when button moves from pressed -> unpressed
    pub selection_made_fn: Rc<RefCell<DropdownListFn>>,
    /// embedded scrollbar in dropdown list
    pub scrollbar: VerticalScrollbar,

    /// if true, then the content should be updated on next drawing
    pub dirty: Rc<RefCell<bool>>,
}

pub type DropdownListFn = Box<dyn FnMut(Context, String) -> EventResponses>;

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl DropdownList {
    const KIND: &'static str = "dropdownlist";

    const STYLE: SelStyles = SelStyles {
        selected_style: Style::new_const(Color::BLACK, Color::LIGHT_YELLOW3),
        ready_style: Style::new_const(Color::BLACK, Color::WHITE),
        unselectable_style: Style::new_const(Color::BLACK, Color::GREY13),
    };

    const STYLE_DD_CURSOR: Style = Style::new_const(Color::WHITE, Color::BLUE);

    const DEFAULT_DROPDOWN_ARROW: DrawCh =
        DrawCh::const_new('▼', Style::new_const(Color::BLACK, Color::GREY13));

    /// needs to be slightly above other elements to select properly
    /// if widgets overlap
    const Z_INDEX: ZIndex = 101;

    pub fn default_receivable_events() -> ReceivableEvents {
        ReceivableEvents(vec![
            (KB::KEY_ENTER.into()),
            (KB::KEY_DOWN.into()),
            (KB::KEY_UP.into()),
            (KB::KEY_K.into()),
            (KB::KEY_J.into()),
            (KB::KEY_SPACE.into()),
        ])
    }

    pub fn new<S: Into<String>>(
        ctx: &Context, entries: Vec<S>,
        selection_made_fn: Box<dyn FnMut(Context, String) -> EventResponses>,
    ) -> Self {
        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_focused_receivable_events(Self::default_receivable_events())
            .with_styles(Self::STYLE)
            .with_dyn_height(DynVal::new_fixed(1))
            .with_z(Self::Z_INDEX);

        let sb =
            VerticalScrollbar::new(ctx, DynVal::new_fixed(0), Size::default(), 0).without_arrows();

        //wire the scrollbar to the dropdown list
        let pane_ = pane.clone();
        let hook = Box::new(move |_, y| pane_.set_content_y_offset(None, y));
        *sb.position_changed_hook.borrow_mut() = Some(hook);

        let entries = entries.into_iter().map(|s| s.into()).collect();

        let d = DropdownList {
            pane,
            entries: Rc::new(RefCell::new(entries)),
            left_padding: Rc::new(RefCell::new(1)),
            specified_width: Rc::new(RefCell::new(None)),
            selected: Rc::new(RefCell::new(0)),
            cursor: Rc::new(RefCell::new(0)),
            open: Rc::new(RefCell::new(false)),
            max_expanded_height: Rc::new(RefCell::new(10)),
            dropdown_arrow: Rc::new(RefCell::new(Self::DEFAULT_DROPDOWN_ARROW)),
            cursor_style: Rc::new(RefCell::new(Self::STYLE_DD_CURSOR)),
            clicked_down: Rc::new(RefCell::new(false)),
            selection_made_fn: Rc::new(RefCell::new(selection_made_fn)),
            scrollbar: sb,
            dirty: Rc::new(RefCell::new(true)),
        };
        d.pane.set_dyn_width(d.calculate_dyn_width());

        let d_ = d.clone();
        d.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                if d_.pane.get_selectability() != Selectability::Selected && *d_.open.borrow() {
                    d_.perform_close_escape();
                }
                d_.dirty.replace(true);
            }));

        d
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self
    }

    pub fn with_arrow(self, ch: DrawCh) -> Self {
        *self.dropdown_arrow.borrow_mut() = ch;
        self
    }

    pub fn with_dyn_width(self, width: DynVal) -> Self {
        *self.specified_width.borrow_mut() = Some(width);
        self.pane.set_dyn_width(self.calculate_dyn_width());
        self
    }

    pub fn with_left_padding(self, padding: usize) -> Self {
        *self.left_padding.borrow_mut() = padding;
        self.pane.set_dyn_width(self.calculate_dyn_width());
        self
    }

    pub fn with_max_expanded_height(self, height: usize) -> Self {
        *self.max_expanded_height.borrow_mut() = height;
        self.scrollbar.set_dyn_height(
            DynVal::new_fixed(height as i32), // view height (same as the dropdown list height)
            DynVal::new_fixed(height.saturating_sub(1) as i32), // scrollbar height (1 less, b/c scrollbar's below the drop-arrow)
            Some(self.entries.borrow().len()),                  // scrollable domain
        );
        self
    }

    pub fn with_fn(self, selection_made_fn: DropdownListFn) -> Self {
        self.set_fn(selection_made_fn);
        self
    }

    pub fn set_fn(&self, selection_made_fn: DropdownListFn) {
        *self.selection_made_fn.borrow_mut() = selection_made_fn;
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    #[must_use]
    pub fn set_selected(&self, ctx: &Context, idx: usize) -> EventResponses {
        self.dirty.replace(true);
        *self.selected.borrow_mut() = idx;
        (self.selection_made_fn.borrow_mut())(
            ctx.clone(),
            self.entries.borrow()[*self.selected.borrow()].clone(),
        )
    }

    #[must_use]
    pub fn set_selected_str(&self, ctx: &Context, s: &str) -> EventResponses {
        self.dirty.replace(true);
        let idx = self.entries.borrow().iter().position(|r| r == s);
        if let Some(idx) = idx {
            *self.selected.borrow_mut() = idx;
            (self.selection_made_fn.borrow_mut())(
                ctx.clone(),
                self.entries.borrow()[*self.selected.borrow()].clone(),
            )
        } else {
            EventResponses::default()
        }
    }

    pub fn get_selected(&self) -> usize {
        *self.selected.borrow()
    }

    pub fn get_selected_str(&self) -> String {
        self.entries.borrow()[*self.selected.borrow()].clone()
    }

    pub fn set_entries<S: Into<String>>(&self, entries: Vec<S>) {
        *self.entries.borrow_mut() = entries.into_iter().map(|s| s.into()).collect();
        self.dirty.replace(true);
    }

    // ----------------------------------------------

    pub fn correct_offsets(&self, dr: &DrawRegion) {
        let cursor_pos = *self.cursor.borrow();
        self.pane
            .correct_offsets_to_view_position(dr, 0, cursor_pos);
        self.scrollbar.external_change(
            self.pane.get_content_y_offset(),
            self.pane.content_height(),
            dr.size,
        );
    }

    pub fn calculate_dyn_width(&self) -> DynVal {
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
        DynVal::new_fixed(left_padding + max_entry_width + arrow_width)
    }

    pub fn padded_entry_text(&self, dr: &DrawRegion, i: usize) -> String {
        let entry = self.entries.borrow()[i].clone();
        let entry_len = entry.chars().count();
        let width = dr.get_width() as usize; // NOTE use dr width which is already the width of the element
        let left_padding = *self.left_padding.borrow();
        let right_padding = width.saturating_sub(entry_len + left_padding);
        let pad_left = " ".repeat(left_padding);
        let pad_right = " ".repeat(right_padding);
        format!("{}{}{}", pad_left, entry, pad_right)
    }

    /// doesn't include the arrow text
    pub fn text(&self, dr: &DrawRegion) -> String {
        if !*self.open.borrow() {
            return self.padded_entry_text(dr, *self.selected.borrow());
        }
        let mut out = String::new();
        let entries_len = self.entries.borrow().len();
        for i in 0..entries_len {
            out += &self.padded_entry_text(dr, i);
            if i != entries_len.saturating_sub(1) {
                out += "\n";
            }
        }
        out
    }

    /// the height of the dropdown list while expanded
    pub fn expanded_height(&self) -> usize {
        let max_height = *self.max_expanded_height.borrow();
        if self.entries.borrow().len() > max_height {
            return max_height;
        }
        self.entries.borrow().len()
    }

    /// whether or not the dropdown list should display a scrollbar
    pub fn display_scrollbar(&self) -> bool {
        self.entries.borrow().len() > self.expanded_height()
    }

    pub fn perform_open(&self) {
        self.dirty.replace(true);
        *self.open.borrow_mut() = true;
        *self.cursor.borrow_mut() = *self.selected.borrow();
        let h = self.expanded_height();
        self.pane.set_dyn_height(DynVal::new_fixed(h as i32));

        let mut dr = DrawRegion::default().with_size(*self.pane.get_last_size());
        dr.size.height = h as u16;
        self.pane.set_content_from_string(self.text(&dr));
    }

    pub fn perform_close_escape(&self) {
        self.dirty.replace(true);
        *self.open.borrow_mut() = false;
        // NOTE we are using the default context here, as we know
        // that a content_y_offset of 0 is safe. a lil' hacky
        self.pane.set_content_y_offset(None, 0);
        self.scrollbar
            .external_change(0, self.pane.content_height(), self.pane.content_size());
        self.pane.set_dyn_height(DynVal::new_fixed(1));
    }

    pub fn perform_close(&self, ctx: &Context, escaped: bool) -> EventResponses {
        self.dirty.replace(true);
        *self.open.borrow_mut() = false;
        self.pane.set_content_y_offset(None, 0);
        self.scrollbar
            .external_change(0, self.pane.content_height(), self.pane.content_size());
        self.pane.set_dyn_height(DynVal::new_fixed(1));
        if !escaped && *self.selected.borrow() != *self.cursor.borrow() {
            *self.selected.borrow_mut() = *self.cursor.borrow();
            (self.selection_made_fn.borrow_mut())(
                ctx.clone(),
                self.entries.borrow()[*self.selected.borrow()].clone(),
            )
        } else {
            EventResponses::default()
        }
    }

    pub fn cursor_up(&self) {
        self.dirty.replace(true);
        if *self.cursor.borrow() > 0 {
            *self.cursor.borrow_mut() -= 1;
        }
    }

    pub fn cursor_down(&self) {
        self.dirty.replace(true);
        if *self.cursor.borrow() < self.entries.borrow().len().saturating_sub(1) {
            *self.cursor.borrow_mut() += 1;
        }
    }

    pub fn update_content(&self, dr: &DrawRegion) {
        let open = *self.open.borrow();
        if open {
            self.correct_offsets(dr);
        }

        let sty = self.pane.get_current_style();
        let mut content = DrawChs2D::from_string(self.text(dr), sty);

        // NOTE use the ctx width as the width as the context has already been shrunk to 100% of the
        // element size
        let width = dr.get_width();

        // highlight the hovering entry
        if open {
            content.change_style_along_y(*self.cursor.borrow(), self.cursor_style.borrow().clone());
        }

        let offset = self.pane.get_content_y_offset() as u16;

        // set the scrollbar on top of the content
        if open && self.display_scrollbar() {
            let sb_dr = dr.child_region(&self.scrollbar.get_dyn_location_set().l);
            let sb_chs = self
                .scrollbar
                .get_content(&sb_dr)
                // shift the scrollbar content to below the arrow
                .to_draw_ch_pos(width.saturating_sub(1), 1 + offset);
            content.apply_vec_draw_ch_pos(sb_chs);
        }

        // set the arrow
        let arrow_ch = DrawChPos::new(
            self.dropdown_arrow.borrow().clone(),
            width.saturating_sub(1),
            offset,
        );
        content.apply_draw_ch_pos(arrow_ch);
        self.pane.set_content(content);
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for DropdownList {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        if captured {
            return (true, resps);
        }
        if self.pane.get_selectability() == Selectability::Unselectable {
            return (false, resps);
        }
        match ev {
            Event::KeyCombo(ke) => {
                if self.pane.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, resps);
                }
                let open = *self.open.borrow();
                match true {
                    _ if !open
                        && (ke[0] == KB::KEY_ENTER
                            || ke[0] == KB::KEY_DOWN
                            || ke[0] == KB::KEY_J
                            || ke[0] == KB::KEY_UP
                            || ke[0] == KB::KEY_K) =>
                    {
                        self.perform_open();
                        return (true, resps);
                    }
                    _ if open && ke[0] == KB::KEY_ENTER => (true, self.perform_close(ctx, false)),
                    _ if open && ke[0] == KB::KEY_DOWN || ke[0] == KB::KEY_J => {
                        self.cursor_down();
                        return (true, resps);
                    }
                    _ if open && ke[0] == KB::KEY_UP || ke[0] == KB::KEY_K => {
                        self.cursor_up();
                        return (true, resps);
                    }
                    _ if open && ke[0] == KB::KEY_SPACE => {
                        let (captured, resps_) =
                            self.scrollbar.receive_event(ctx, Event::KeyCombo(ke));
                        resps.extend(resps_);
                        self.dirty.replace(true);
                        return (captured, resps);
                    }
                    _ => return (false, resps),
                };
            }
            Event::Mouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                let open = *self.open.borrow();
                let (mut clicked, mut dragging, mut scroll_up, mut scroll_down) =
                    (false, false, false, false);
                if !open {
                    match me.kind {
                        MouseEventKind::Down(MouseButton::Left) if !open => {
                            *self.clicked_down.borrow_mut() = true;
                            return (true, resps);
                        }
                        MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                        _ => {
                            *self.clicked_down.borrow_mut() = false;
                        }
                    }
                }
                match me.kind {
                    MouseEventKind::Up(MouseButton::Left) if clicked_down || open => clicked = true,
                    MouseEventKind::Drag(MouseButton::Left) => dragging = true,
                    MouseEventKind::ScrollUp => scroll_up = true,
                    MouseEventKind::ScrollDown => scroll_down = true,
                    _ => {}
                }

                match true {
                    _ if !open && clicked => {
                        self.perform_open();
                        return (true, resps);
                    }
                    _ if open && scroll_up => {
                        self.cursor_up();
                        return (true, resps);
                    }
                    _ if open && scroll_down => {
                        self.cursor_down();
                        return (true, resps);
                    }
                    _ if open && (!clicked || dragging) => {
                        let (x, y) = (me.column as usize, me.row as usize);

                        // change hovering location to the ev

                        // on arrow
                        if y == 0 && x == self.pane.get_width(&me.dr).saturating_sub(1) {
                            self.dirty.replace(true);
                            return (true, resps);

                        // on scrollbar
                        } else if y > 0
                            && x == self.pane.get_width(&me.dr).saturating_sub(1)
                            && self.display_scrollbar()
                        {
                            if dragging {
                                // send the the event to the scrollbar (x adjusted to 0)
                                let mut me_ = me;
                                me_.column = 0;
                                me_.row = y as i32 - 1;
                                let (captured, resps_) =
                                    self.scrollbar.receive_event(ctx, Event::Mouse(me_));
                                resps.extend(resps_);
                                return (captured, resps);
                            }
                            self.dirty.replace(true);
                            return (true, resps);
                        } else {
                            *self.cursor.borrow_mut() = y + self.pane.get_content_y_offset();
                        }
                        self.dirty.replace(true);
                        return (true, resps);
                    }
                    _ if open && clicked => {
                        let (x, y) = (me.column as usize, me.row as usize);
                        if y > 0
                            && x == self.pane.get_width(&me.dr).saturating_sub(1)
                            && self.display_scrollbar()
                        {
                            // send the the event to the scrollbar (x adjusted to 0)
                            let mut me_ = me;
                            me_.column = 0;
                            me_.row = y as i32 - 1;
                            let (captured, resps_) =
                                self.scrollbar.receive_event(ctx, Event::Mouse(me_));
                            resps.extend(resps_);
                            self.dirty.replace(true);
                            return (captured, resps);
                        }

                        // on arrow close without change
                        if y == 0 && x == self.pane.get_width(&me.dr).saturating_sub(1) {
                            let resps_ = self.perform_close(ctx, true);
                            resps.extend(resps_);
                            return (true, resps);
                        }
                        *self.cursor.borrow_mut() = y + self.pane.get_content_y_offset();

                        let resps_ = self.perform_close(ctx, false);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ => return (false, resps),
                };
            }
            Event::Resize => {
                self.dirty.replace(true);
                return (true, resps);
            }
            _ => {}
        }
        (false, resps)
    }

    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        if *self.dirty.borrow() || force_update {
            self.update_content(dr);
            *self.dirty.borrow_mut() = false;
        }
        self.pane.drawing(ctx, dr, force_update)
    }
}
