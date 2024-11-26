use {
    super::{VerticalSBPositions, VerticalScrollbar},
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct ListBox {
    pub pane: SelectablePane,
    /// the main listbox element (not the scrollbar)
    pub inner: Rc<RefCell<ListBoxInner>>,
    pub scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
}

#[derive(Clone)]
pub struct ListBoxInner {
    pub pane: Pane,
    pub current_sty: Rc<RefCell<Style>>,
    pub selectedness: Rc<RefCell<Selectability>>,
    pub entries: Rc<RefCell<Vec<String>>>,
    pub selected: Rc<RefCell<Vec<usize>>>,
    /// the entries which have been selected
    pub cursor: Rc<RefCell<Option<usize>>>,

    /// the last listbox position which was clicked, used for initialization
    /// of a new keyboard cursor if none exists and then it is initialized.
    pub last_clicked_position: Rc<RefCell<Option<usize>>>,

    /// position of a listbox cursor
    pub clicked_down: Rc<RefCell<bool>>,
    /// activated when mouse is clicked down while over object
    pub lines_per_item: Rc<RefCell<usize>>,
    /// how many lines each item is to take up
    pub selection_mode: Rc<RefCell<SelectionMode>>,

    #[allow(clippy::type_complexity)]
    /// function which executes when the selection changes. NOTE multiple items may be selected
    /// simultaniously if the ListBox is configured to allow it. If multiple items are selected,
    /// all the selected items will be passed to the function at every selection change.
    pub selection_made_fn: Rc<RefCell<Box<dyn FnMut(Context, Vec<String>) -> EventResponses>>>,

    pub item_selected_style: Rc<RefCell<Style>>,
    pub cursor_over_unselected_style: Rc<RefCell<Style>>,
    pub cursor_over_selected_style: Rc<RefCell<Style>>,
    pub scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
    pub is_dirty: Rc<RefCell<bool>>,
}

#[derive(Clone)]
pub enum SelectionMode {
    Single,
    /// only one item is selectable at a time, each selection will deselect the previous selected
    NoLimit,
    /// all items are selectable

    /// n items are selectable at a time, once n items are selected, no more items can
    /// be selected until one of the selected items is deselected
    UpTo(usize),
}

impl ListBox {
    const KIND: &'static str = "listbox";

    pub fn new(
        ctx: &Context, entries: Vec<String>,
        selection_made_fn: Box<dyn FnMut(Context, Vec<String>) -> EventResponses>,
    ) -> Self {
        let max_entry_width = entries
            .iter()
            .map(|r| r.lines().map(|l| l.chars().count()).max().unwrap_or(0))
            .max()
            .unwrap_or(0);
        let line_count = entries.iter().map(|r| r.lines().count()).sum::<usize>() as i32;
        let init_ctx = ctx.child_init_context();
        let inner = ListBoxInner::new(&init_ctx, entries, selection_made_fn);

        let pane = SelectablePane::new(&init_ctx, Self::KIND)
            .with_styles(ListBoxInner::STYLE)
            .with_dyn_width(DynVal::new_fixed(max_entry_width as i32))
            .with_dyn_height(DynVal::new_fixed(line_count));
        pane.pane.add_element(Box::new(inner.clone()));
        let lb = ListBox {
            pane,
            inner: Rc::new(RefCell::new(inner)),
            scrollbar: Rc::new(RefCell::new(None)),
        };
        let lb_ = lb.clone();
        lb.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                let sel = lb_.pane.get_selectability();
                if sel != Selectability::Selected {
                    *lb_.inner.borrow().cursor.borrow_mut() = None;
                }
                *lb_.inner.borrow().selectedness.borrow_mut() = sel;
                *lb_.inner.borrow().current_sty.borrow_mut() = lb_.pane.get_current_style();
                *lb_.inner.borrow().is_dirty.borrow_mut() = true;
            }));
        *lb.inner.borrow().current_sty.borrow_mut() = lb.pane.get_current_style();
        lb
    }

    pub fn with_styles(self, init_ctx: &Context, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self.inner.borrow().update_content(init_ctx);
        self
    }

    // ----------------------------------------------
    // decorators

    pub fn with_left_scrollbar(self, init_ctx: &Context) -> Self {
        self.with_scrollbar_inner(init_ctx, VerticalSBPositions::ToTheLeft)
    }

    pub fn with_right_scrollbar(self, init_ctx: &Context) -> Self {
        self.with_scrollbar_inner(init_ctx, VerticalSBPositions::ToTheRight)
    }

    pub fn with_scrollbar(self, init_ctx: &Context) -> Self {
        self.with_scrollbar_inner(init_ctx, VerticalSBPositions::ToTheRight)
    }

    fn with_scrollbar_inner(self, init_ctx: &Context, pos: VerticalSBPositions) -> Self {
        let height = DynVal::FULL;
        let content_height = self.inner.borrow().pane.content_height();

        let sb = VerticalScrollbar::new(init_ctx, height, init_ctx.s, content_height)
            .without_keyboard_events();
        match pos {
            VerticalSBPositions::ToTheLeft => {
                sb.set_at(0.into(), 0.into());
                self.inner.borrow().pane.set_start_x(1.into());
            }
            VerticalSBPositions::ToTheRight => {
                sb.set_at(DynVal::FULL.minus_fixed(1), 0.into());
                self.inner
                    .borrow()
                    .pane
                    .set_end_x(DynVal::FULL.minus_fixed(1));
            }
            VerticalSBPositions::None => {
                return self;
            }
        }
        sb.set_scrollable_view_size(init_ctx.child_context(&self.pane.get_dyn_location()).s);

        // wire the scrollbar to the listbox
        let pane_ = self.inner.borrow().pane.clone();
        let hook = Box::new(move |ctx, y| pane_.set_content_y_offset(&ctx, y));
        *sb.position_changed_hook.borrow_mut() = Some(hook);
        *self.scrollbar.borrow_mut() = Some(sb.clone());
        self.pane.pane.add_element(Box::new(sb.clone()));
        self.inner.borrow().scrollbar.replace(Some(sb));
        self
    }

    pub fn with_lines_per_item(self, init_ctx: &Context, lines: usize) -> Self {
        *self.inner.borrow().lines_per_item.borrow_mut() = lines;
        self.pane.set_dyn_height(DynVal::new_fixed(
            self.inner.borrow().entries.borrow().len() as i32 * lines as i32,
        ));
        self.inner.borrow().update_content(init_ctx);
        self
    }

    pub fn with_selection_mode(self, init_ctx: &Context, mode: SelectionMode) -> Self {
        *self.inner.borrow().selection_mode.borrow_mut() = mode;
        self.inner.borrow().update_content(init_ctx);
        self
    }

    pub fn with_width(self, init_ctx: &Context, width: DynVal) -> Self {
        self.pane.set_dyn_width(width);
        self.inner.borrow().update_content(init_ctx);
        self
    }
    pub fn with_height(self, init_ctx: &Context, height: DynVal) -> Self {
        self.pane.set_dyn_height(height);
        self.inner.borrow().update_content(init_ctx);
        self
    }
    pub fn with_size(self, init_ctx: &Context, width: DynVal, height: DynVal) -> Self {
        self.pane.set_dyn_width(width);
        self.pane.set_dyn_height(height);
        self.inner.borrow().update_content(init_ctx);
        self
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.pane.set_at(loc_x, loc_y);
        self
    }
}

impl ListBoxInner {
    const KIND: &'static str = "listbox_inner";

    const STYLE: SelStyles = SelStyles {
        selected_style: Style::new_const(Color::BLACK, Color::YELLOW),
        ready_style: Style::new_const(Color::BLACK, Color::GREY20),
        unselectable_style: Style::new_const(Color::BLACK, Color::GREY13),
    };

    const STYLE_ITEM_SELECTED: Style = Style::new_const(Color::WHITE, Color::NAVY);
    const STYLE_CURSOR_OVER_UNSELECTED: Style = Style::new_const(Color::BLACK, Color::LIGHT_BLUE);
    const STYLE_CURSOR_OVER_SELECTED: Style = Style::new_const(Color::WHITE, Color::BLUE);

    pub fn default_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KB::KEY_ENTER.into(), Priority::Focused),
            (KB::KEY_DOWN.into(), Priority::Focused),
            (KB::KEY_UP.into(), Priority::Focused),
            (KB::KEY_K.into(), Priority::Focused),
            (KB::KEY_J.into(), Priority::Focused),
            (KB::KEY_SPACE.into(), Priority::Focused),
        ])
    }

    pub fn new(
        init_ctx: &Context, entries: Vec<String>,
        selection_made_fn: Box<dyn FnMut(Context, Vec<String>) -> EventResponses>,
    ) -> Self {
        let max_lines_per_entry = entries.iter().map(|r| r.lines().count()).max().unwrap_or(0);

        let pane = Pane::new(init_ctx, Self::KIND)
            .with_self_receivable_events(Self::default_receivable_events())
            .with_dyn_width(DynVal::FULL)
            .with_dyn_height(DynVal::FULL);

        let lb = ListBoxInner {
            pane,
            current_sty: Rc::new(RefCell::new(Style::default())),
            selectedness: Rc::new(RefCell::new(Selectability::Ready)),
            entries: Rc::new(RefCell::new(entries)),
            lines_per_item: Rc::new(RefCell::new(max_lines_per_entry)),
            selected: Rc::new(RefCell::new(Vec::new())),
            cursor: Rc::new(RefCell::new(None)),
            last_clicked_position: Rc::new(RefCell::new(None)),
            clicked_down: Rc::new(RefCell::new(false)),
            selection_mode: Rc::new(RefCell::new(SelectionMode::NoLimit)),
            item_selected_style: Rc::new(RefCell::new(Self::STYLE_ITEM_SELECTED)),
            cursor_over_unselected_style: Rc::new(RefCell::new(Self::STYLE_CURSOR_OVER_UNSELECTED)),
            cursor_over_selected_style: Rc::new(RefCell::new(Self::STYLE_CURSOR_OVER_SELECTED)),
            selection_made_fn: Rc::new(RefCell::new(selection_made_fn)),
            scrollbar: Rc::new(RefCell::new(None)),
            is_dirty: Rc::new(RefCell::new(true)),
        };
        lb.update_content(init_ctx);
        lb
    }

    // ----------------------------------------------

    pub fn get_text_for_entry(&self, entry_i: usize, width: usize, entry_height: usize) -> String {
        let entry = self.entries.borrow()[entry_i].clone();

        // pad the text to the width and height
        let mut text: Vec<String> = entry.lines().map(|r| r.to_string()).collect();
        let text_len = text.len();
        #[allow(clippy::comparison_chain)]
        if text_len > entry_height {
            text.truncate(entry_height);
        } else if text_len < entry_height {
            for _ in text_len..entry_height {
                text.push("".to_string());
            }
        }

        // pad the text to the width of the listbox
        for line in text.iter_mut() {
            // cut off the text if it is too long
            if line.chars().count() > width {
                *line = format!("{}…", &line[..width.saturating_sub(1)]);
            }
            if line.chars().count() < width {
                *line = format!("{}{}", line, " ".repeat(width - line.chars().count()));
            }
        }
        text.join("\n")
    }

    pub fn correct_offsets(&self, ctx: &Context) {
        let Some(cursor) = *self.cursor.borrow() else {
            return;
        };
        let (start_y, end_y) = self.get_content_y_range_for_item_index(cursor);
        let y_offset = self.pane.get_content_y_offset();
        let height = self.pane.get_height(ctx);

        if end_y >= y_offset + height {
            self.pane.correct_offsets_to_view_position(ctx, 0, end_y);
        } else if start_y < y_offset {
            self.pane.correct_offsets_to_view_position(ctx, 0, start_y);
        }

        let y_offset = self.pane.get_content_y_offset();

        // call the scrollbar external change hook if it exists
        if let Some(sb) = self.scrollbar.borrow().as_ref() {
            sb.external_change(y_offset, self.pane.content_height(), ctx.s);
        }
        self.is_dirty.replace(true);
    }

    pub fn get_item_index_for_view_y(&self, y: usize) -> usize {
        let y_offset = self.pane.get_content_y_offset();
        let offset = y + y_offset;
        offset / *self.lines_per_item.borrow()
    }

    pub fn get_content_y_range_for_item_index(&self, index: usize) -> (usize, usize) {
        let start_y = index * *self.lines_per_item.borrow();
        let end_y = (start_y + *self.lines_per_item.borrow()).saturating_sub(1);
        (start_y, end_y)
    }

    pub fn set_entries(&self, ctx: &Context, entries: Vec<String>) {
        *self.entries.borrow_mut() = entries;
        self.update_content(ctx);
    }

    pub fn update_content(&self, ctx: &Context) {
        let mut content = String::new();
        let entries_len = self.entries.borrow().len();
        for i in 0..entries_len {
            content +=
                &self.get_text_for_entry(i, ctx.s.width.into(), *self.lines_per_item.borrow());
            if i < entries_len - 1 {
                content += "\n";
            }
        }
        self.pane.set_content_from_string(&content);
        self.update_highlighting(ctx);
        self.is_dirty.replace(true);
    }

    /// need to reset the content in order to reflect active style
    pub fn update_highlighting(&self, ctx: &Context) {
        // change the style for selection and the cursor
        for i in 0..self.entries.borrow().len() {
            let cursor = *self.cursor.borrow();
            let item_selected = self.selected.borrow().contains(&i);
            let selectedness = self.selectedness.borrow();

            let sty = match true {
                _ if item_selected
                    && cursor == Some(i)
                    && *selectedness == Selectability::Selected =>
                {
                    self.cursor_over_selected_style.borrow().clone()
                }
                _ if !item_selected
                    && cursor == Some(i)
                    && *selectedness == Selectability::Selected =>
                {
                    self.cursor_over_unselected_style.borrow().clone()
                }
                _ if item_selected => self.item_selected_style.borrow().clone(),
                _ => self.current_sty.borrow().clone(),
            };

            let (y_start, y_end) = self.get_content_y_range_for_item_index(i);
            for y in y_start..=y_end {
                self.pane
                    .content
                    .borrow_mut()
                    .change_style_along_y(y, sty.clone());
            }

            // update the rest of the lines
            let entries_len = self.entries.borrow().len();
            for i in entries_len * *self.lines_per_item.borrow()..self.pane.get_height(ctx) {
                let sty = self.current_sty.borrow().clone();
                self.pane.content.borrow_mut().change_style_along_y(i, sty);
            }
        }
    }

    pub fn cursor_up(&self, ctx: &Context) {
        let cursor = *self.cursor.borrow();
        match cursor {
            Some(cursor) if cursor > 0 => {
                *self.cursor.borrow_mut() = Some(cursor - 1);
            }
            None => {
                if let Some(lcp) = *self.last_clicked_position.borrow() {
                    *self.cursor.borrow_mut() = Some(lcp);
                    self.cursor_up(ctx);
                } else {
                    *self.cursor.borrow_mut() = Some(self.entries.borrow().len() - 1);
                }
            }
            _ => {}
        }
        self.correct_offsets(ctx);
    }

    pub fn cursor_down(&self, ctx: &Context) {
        let cursor = *self.cursor.borrow();
        match cursor {
            Some(cursor) if cursor < self.entries.borrow().len() - 1 => {
                *self.cursor.borrow_mut() = Some(cursor + 1);
            }
            None => {
                if let Some(lcp) = *self.last_clicked_position.borrow() {
                    *self.cursor.borrow_mut() = Some(lcp);
                    self.cursor_down(ctx);
                } else {
                    *self.cursor.borrow_mut() = Some(0);
                }
            }
            _ => {}
        }
        self.correct_offsets(ctx);
    }

    pub fn toggle_entry_selected_at_i(&self, ctx: &Context, i: usize) -> EventResponses {
        self.is_dirty.replace(true);
        let already_selected = self.selected.borrow().contains(&i);

        match *self.selection_mode.borrow() {
            SelectionMode::Single => {
                if already_selected {
                    self.selected.borrow_mut().retain(|&r| r != i);
                } else {
                    self.selected.borrow_mut().clear();
                    self.selected.borrow_mut().push(i);
                }
            }
            SelectionMode::NoLimit => {
                if already_selected {
                    self.selected.borrow_mut().retain(|&r| r != i);
                } else {
                    self.selected.borrow_mut().push(i);
                }
            }
            SelectionMode::UpTo(n) => {
                if already_selected {
                    self.selected.borrow_mut().retain(|&r| r != i);
                } else if self.selected.borrow().len() < n {
                    self.selected.borrow_mut().push(i);
                }
            }
        }

        let entries = self.entries.borrow().clone();
        let selected_entries = self
            .selected
            .borrow()
            .iter()
            .map(|i| entries[*i].clone())
            .collect();

        (self.selection_made_fn.borrow_mut())(ctx.clone(), selected_entries)
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for ListBox {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        if self.pane.get_selectability() == Selectability::Unselectable {
            return (false, EventResponses::default());
        }
        self.pane.receive_event(ctx, ev)
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for ListBoxInner {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        if captured {
            return (true, resps);
        }
        match ev {
            Event::KeyCombo(ke) => {
                if ke.is_empty() {
                    return (false, resps);
                }
                match true {
                    _ if ke[0] == KB::KEY_SPACE => {
                        if let Some(sb) = self.scrollbar.borrow().as_ref() {
                            let (captured, resps_) = sb.receive_event(ctx, Event::KeyCombo(ke));
                            resps.extend(resps_);
                            self.is_dirty.replace(true);
                            return (captured, resps);
                        } else {
                            return (true, resps);
                        }
                    }
                    _ if ke[0] == KB::KEY_DOWN || ke[0] == KB::KEY_J => {
                        self.cursor_down(ctx);
                        return (true, resps);
                    }
                    _ if ke[0] == KB::KEY_UP || ke[0] == KB::KEY_K => {
                        self.cursor_up(ctx);
                        return (true, resps);
                    }
                    _ if ke[0] == KB::KEY_ENTER => {
                        let Some(cursor) = *self.cursor.borrow() else {
                            return (true, resps);
                        };
                        let entries_len = self.entries.borrow().len();
                        if cursor >= entries_len {
                            return (true, resps);
                        }
                        let resps_ = self.toggle_entry_selected_at_i(ctx, cursor);
                        resps.extend(resps_);
                        return (true, resps);
                    }

                    _ => return (false, resps),
                };
            }
            Event::Mouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                let (mut clicked, mut dragging, mut scroll_up, mut scroll_down) =
                    (false, false, false, false);
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, resps);
                    }
                    MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }
                match me.kind {
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => clicked = true,
                    MouseEventKind::Drag(MouseButton::Left) => dragging = true,
                    MouseEventKind::ScrollUp => scroll_up = true,
                    MouseEventKind::ScrollDown => scroll_down = true,
                    _ => {}
                }

                match true {
                    _ if scroll_up => {
                        self.cursor_up(ctx);
                        return (true, resps);
                    }
                    _ if scroll_down => {
                        self.cursor_down(ctx);
                        return (true, resps);
                    }
                    _ if clicked => {
                        let (x, y) = (me.column as usize, me.row as usize);

                        // check if this should be a scrollbar event
                        if let Some(sb) = self.scrollbar.borrow().as_ref() {
                            if y > 0 && x == self.pane.get_width(ctx).saturating_sub(1) {
                                if dragging {
                                    // send the the event to the scrollbar (x adjusted to 0)
                                    let mut me_ = me;
                                    me_.column = 0;
                                    me_.row = y.saturating_sub(1) as u16;
                                    let (captured, resps_) =
                                        sb.receive_event(ctx, Event::Mouse(me_));
                                    self.is_dirty.replace(true);
                                    resps.extend(resps_);
                                    return (captured, resps);
                                }
                                return (true, resps);
                            }
                        }

                        // get item index at click position
                        let item_i = self.get_item_index_for_view_y(y);
                        if item_i >= self.entries.borrow().len() {
                            return (false, resps);
                        }

                        *self.last_clicked_position.borrow_mut() = Some(item_i);

                        // toggle selection
                        let resps_ = self.toggle_entry_selected_at_i(ctx, item_i);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ => return (false, resps),
                };
            }
            _ => {}
        }
        (false, resps)
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        if self.is_dirty.replace(false) {
            self.update_highlighting(ctx);
            self.update_content(ctx);
        }
        self.pane.drawing(ctx)
    }
}
