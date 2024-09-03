use {
    super::{
        Selectability, VerticalSBPositions, VerticalScrollbar, WBStyles, Widget, WidgetBase,
        Widgets,
    },
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponses, Keyboard as KB, Priority,
        ReceivableEventChanges, RgbColour, SclLocationSet, SclVal, SortingHat, Style,
        UpwardPropagator,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct ListBox {
    pub base: WidgetBase,
    pub entries: Rc<RefCell<Vec<String>>>,
    pub selected: Rc<RefCell<Vec<usize>>>, // the entries which have been selected
    pub cursor: Rc<RefCell<Option<usize>>>, // position of a listbox cursor

    pub lines_per_item: Rc<RefCell<usize>>, // how many lines each item is to take up

    pub selection_mode: Rc<RefCell<SelectionMode>>,

    #[allow(clippy::type_complexity)]
    // function which executes when the selection changes. NOTE multiple items may be selected
    // simultaniously if the ListBox is configured to allow it. If multiple items are selected,
    // all the selected items will be passed to the function at every selection change.
    pub selection_made_fn: Rc<RefCell<Box<dyn FnMut(Context, Vec<String>) -> EventResponses>>>,

    pub item_selected_style: Rc<RefCell<Style>>,
    pub cursor_over_unselected_style: Rc<RefCell<Style>>,
    pub cursor_over_selected_style: Rc<RefCell<Style>>,

    pub scrollbar_options: Rc<RefCell<VerticalSBPositions>>,
    pub scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
    // XXX TODO
    //pub right_click_menu: Option<RightClickMenuTemplate>, // right click menu for the list
}

#[derive(Clone)]
pub enum SelectionMode {
    Single, // only one item is selectable at a time, each selection will deselect the previous selected
    NoLimit, // all items are selectable

    // n items are selectable at a time, once n items are selected, no more items can
    // be selected until one of the selected items is deselected
    UpTo(usize),
}

impl ListBox {
    const KIND: &'static str = "widget_listbox";

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

    const STYLE_ITEM_SELECTED: Style = Style::new()
        .with_bg(RgbColour::NAVY)
        .with_fg(RgbColour::WHITE);
    const STYLE_CURSOR_OVER_UNSELECTED: Style = Style::new()
        .with_bg(RgbColour::LIGHT_BLUE)
        .with_fg(RgbColour::BLACK);
    const STYLE_CURSOR_OVER_SELECTED: Style = Style::new()
        .with_bg(RgbColour::BLUE)
        .with_fg(RgbColour::WHITE);

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
        hat: &SortingHat, ctx: &Context, entries: Vec<String>,
        selection_made_fn: Box<dyn FnMut(Context, Vec<String>) -> EventResponses>,
    ) -> Self {
        let max_entry_width = entries
            .iter()
            .map(|r| r.lines().map(|l| l.chars().count()).max().unwrap_or(0))
            .max()
            .unwrap_or(0);
        let line_count = entries.iter().map(|r| r.lines().count()).sum::<usize>() as i32;
        let max_lines_per_entry = entries.iter().map(|r| r.lines().count()).max().unwrap_or(0);

        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            SclVal::new_fixed(max_entry_width as i32),
            SclVal::new_fixed(line_count),
            Self::STYLE,
            Self::default_receivable_events(),
        );

        let lb = ListBox {
            base: wb,
            entries: Rc::new(RefCell::new(entries)),
            lines_per_item: Rc::new(RefCell::new(max_lines_per_entry)),
            selected: Rc::new(RefCell::new(Vec::new())),
            cursor: Rc::new(RefCell::new(None)),
            selection_mode: Rc::new(RefCell::new(SelectionMode::NoLimit)),

            item_selected_style: Rc::new(RefCell::new(Self::STYLE_ITEM_SELECTED)),
            cursor_over_unselected_style: Rc::new(RefCell::new(Self::STYLE_CURSOR_OVER_UNSELECTED)),
            cursor_over_selected_style: Rc::new(RefCell::new(Self::STYLE_CURSOR_OVER_SELECTED)),

            scrollbar_options: Rc::new(RefCell::new(VerticalSBPositions::None)),
            scrollbar: Rc::new(RefCell::new(None)),
            selection_made_fn: Rc::new(RefCell::new(selection_made_fn)),
        };
        lb.update_content(ctx);
        lb
    }

    // ----------------------------------------------
    // decorators

    pub fn with_left_scrollbar(self) -> Self {
        *self.scrollbar_options.borrow_mut() = VerticalSBPositions::ToTheLeft;
        self
    }

    pub fn with_right_scrollbar(self) -> Self {
        *self.scrollbar_options.borrow_mut() = VerticalSBPositions::ToTheRight;
        self
    }

    pub fn with_scrollbar(self) -> Self {
        *self.scrollbar_options.borrow_mut() = VerticalSBPositions::ToTheRight;
        self
    }

    // XXX TODO
    //pub fn with_right_click_menu(mut self) -> Self {
    //self.right_click_menu = rcm;
    //    self
    //}

    pub fn with_lines_per_item(self, ctx: &Context, lines: usize) -> Self {
        *self.lines_per_item.borrow_mut() = lines;
        self.base.set_scl_height(SclVal::new_fixed(
            self.entries.borrow().len() as i32 * lines as i32,
        ));
        self.update_content(ctx);
        self
    }

    pub fn with_selection_mode(self, ctx: &Context, mode: SelectionMode) -> Self {
        *self.selection_mode.borrow_mut() = mode;
        self.update_content(ctx);
        self
    }

    pub fn with_styles(self, ctx: &Context, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self.update_content(ctx);
        self
    }

    pub fn with_width(self, ctx: &Context, width: SclVal) -> Self {
        self.base.set_scl_width(width);
        self.update_content(ctx);
        self
    }
    pub fn with_height(self, ctx: &Context, height: SclVal) -> Self {
        self.base.set_scl_height(height);
        self.update_content(ctx);
        self
    }
    pub fn with_size(self, ctx: &Context, width: SclVal, height: SclVal) -> Self {
        self.base.set_scl_width(width);
        self.base.set_scl_height(height);
        self.update_content(ctx);
        self
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self, hat: &SortingHat) -> Widgets {
        let position = *self.scrollbar_options.borrow();
        if let VerticalSBPositions::None = position {
            return Widgets(vec![Box::new(self)]);
        }
        let height = self.base.get_scl_height();
        let content_height = self.base.content_height();
        let mut sb =
            VerticalScrollbar::new(hat, height, content_height).with_styles(Self::STYLE_SCROLLBAR);
        if let VerticalSBPositions::ToTheLeft = position {
            sb = sb.at(
                self.base.get_scl_start_x().minus_fixed(1),
                self.base.get_scl_start_y().clone(),
            );
        } else if let VerticalSBPositions::ToTheRight = position {
            sb = sb.at(
                self.base.get_scl_start_x().plus(self.base.get_scl_width()),
                self.base.get_scl_start_y(),
            );
        }

        // wire the scrollbar to the text box
        let wb_ = self.base.clone();
        let hook = Box::new(move |ctx, y| wb_.set_content_y_offset(&ctx, y));
        *sb.position_changed_hook.borrow_mut() = Some(hook);
        *self.scrollbar.borrow_mut() = Some(sb.clone());

        Widgets(vec![Box::new(self), Box::new(sb)])
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
                //line = line[..width.saturating_sub(1)].to_string() + "…";
                *line = format!("{}…", &line[..width.saturating_sub(1)]);
            }
            if line.chars().count() < width {
                *line = format!("{}{}", line, " ".repeat(width - line.chars().count()));
            }
        }
        text.join("\n")
    }

    //pub fn correct_offsets(&self, ctx: &Context) {
    //    let cursor_pos = *self.cursor.borrow();
    //    self.base
    //        .correct_offsets_to_view_position(ctx, 0, cursor_pos);
    //    self.scrollbar.external_change(
    //        ctx,
    //        *self.base.pane.content_view_offset_y.borrow(),
    //        self.base.content_height(),
    //    );
    //}

    pub fn correct_offsets(&self, ctx: &Context) {
        let Some(cursor) = *self.cursor.borrow() else {
            return;
        };
        let (start_y, end_y) = self.get_content_y_range_for_item_index(cursor);
        let y_offset = *self.base.pane.content_view_offset_y.borrow();
        let height = self.base.get_height(ctx);

        if end_y >= y_offset + height {
            self.base.correct_offsets_to_view_position(ctx, 0, end_y);
        } else if start_y < y_offset {
            self.base.correct_offsets_to_view_position(ctx, 0, start_y);
        }

        let y_offset = *self.base.pane.content_view_offset_y.borrow();

        // call the scrollbar external change hook if it exists
        if let Some(sb) = self.scrollbar.borrow().as_ref() {
            sb.external_change(ctx, y_offset, self.base.content_height());
        }
    }

    pub fn get_item_index_for_view_y(&self, y: usize) -> usize {
        let y_offset = *self.base.pane.content_view_offset_y.borrow();
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
            content += &self.get_text_for_entry(
                i,
                self.base.get_width(ctx),
                *self.lines_per_item.borrow(),
            );
            if i < entries_len - 1 {
                content += "\n";
            }
        }
        self.base.set_content_from_string(ctx, &content);
        self.update_highlighting(ctx);
    }

    // need to reset the content in order to reflect active style
    pub fn update_highlighting(&self, ctx: &Context) {
        // change the style for selection and the cursor
        for i in 0..self.entries.borrow().len() {
            let cursor = *self.cursor.borrow();
            let item_selected = self.selected.borrow().contains(&i);
            let selectedness = self.base.get_selectability();

            let sty = match true {
                _ if item_selected
                    && cursor == Some(i)
                    && selectedness == Selectability::Selected =>
                {
                    *self.cursor_over_selected_style.borrow()
                }
                _ if !item_selected
                    && cursor == Some(i)
                    && selectedness == Selectability::Selected =>
                {
                    *self.cursor_over_unselected_style.borrow()
                }
                _ if item_selected => *self.item_selected_style.borrow(),
                _ => self.base.get_current_style(),
            };

            let (y_start, y_end) = self.get_content_y_range_for_item_index(i);
            for y in y_start..=y_end {
                self.base
                    .pane
                    .content
                    .borrow_mut()
                    .change_style_along_y(y, sty);
            }

            // update the rest of the lines
            let entries_len = self.entries.borrow().len();
            for i in entries_len * *self.lines_per_item.borrow()..self.base.get_height(ctx) {
                let sty = self.base.get_current_style();
                self.base
                    .pane
                    .content
                    .borrow_mut()
                    .change_style_along_y(i, sty);
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
                *self.cursor.borrow_mut() = Some(self.entries.borrow().len() - 1);
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
                *self.cursor.borrow_mut() = Some(0);
            }
            _ => {}
        }
        self.correct_offsets(ctx);
    }

    pub fn toggle_entry_selected_at_i(&self, ctx: &Context, i: usize) -> EventResponses {
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

impl Widget for ListBox {
    fn set_selectability_pre_hook(&self, _: &Context, s: Selectability) -> EventResponses {
        if self.base.get_selectability() == Selectability::Selected && s != Selectability::Selected
        {
            *self.cursor.borrow_mut() = None;
        }
        EventResponses::default()
    }
}

impl Element for ListBox {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }
                return match true {
                    _ if ke[0].matches_key(&KB::KEY_SPACE) => {
                        if let Some(sb) = self.scrollbar.borrow().as_ref() {
                            if sb.get_selectability() != Selectability::Selected {
                                sb.set_selectability(ctx, Selectability::Selected);
                            }
                            sb.receive_event(ctx, Event::KeyCombo(ke))
                        } else {
                            (true, EventResponses::default())
                        }
                    }
                    _ if ke[0].matches_key(&KB::KEY_DOWN) || ke[0].matches_key(&KB::KEY_J) => {
                        self.cursor_down(ctx);
                        (true, EventResponses::default())
                    }
                    _ if ke[0].matches_key(&KB::KEY_UP) || ke[0].matches_key(&KB::KEY_K) => {
                        self.cursor_up(ctx);
                        (true, EventResponses::default())
                    }
                    _ if ke[0].matches_key(&KB::KEY_ENTER) => {
                        let Some(cursor) = *self.cursor.borrow() else {
                            return (true, EventResponses::default());
                        };
                        let entries_len = self.entries.borrow().len();
                        if cursor >= entries_len {
                            return (true, EventResponses::default());
                        }
                        return (true, self.toggle_entry_selected_at_i(ctx, cursor));
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

                return match true {
                    _ if scroll_up => {
                        self.cursor_up(ctx);
                        (true, EventResponses::default())
                    }
                    _ if scroll_down => {
                        self.cursor_down(ctx);
                        (true, EventResponses::default())
                    }
                    // XXX TODO
                    //// handle right click
                    //if lb.RightClickMenu != nil {
                    //    // send the event to the right click menu to check for right click
                    //    createRCM := lb.RightClickMenu.CreateMenuIfRightClick(ev)
                    //    if createRCM.HasWindow() {
                    //        return true, yh.NewEventResponse().WithWindow(createRCM)
                    //    }
                    //}
                    _ if (clicked || dragging) => {
                        let (x, y) = (me.column as usize, me.row as usize);

                        // check if this should be a scrollbar event
                        if let Some(sb) = self.scrollbar.borrow().as_ref() {
                            if y > 0 && x == self.base.get_width(ctx).saturating_sub(1) {
                                if dragging {
                                    if sb.get_selectability() != Selectability::Selected {
                                        let _ = sb.set_selectability(ctx, Selectability::Selected);
                                    }
                                    // send the the event to the scrollbar (x adjusted to 0)
                                    let mut me_ = me;
                                    me_.column = 0;
                                    me_.row = y.saturating_sub(1) as u16;
                                    return sb.receive_event(ctx, Event::Mouse(me_));
                                }
                                return (true, EventResponses::default());
                            }
                        }

                        // get item index at click position
                        let item_i = self.get_item_index_for_view_y(y);
                        if item_i >= self.entries.borrow().len() {
                            return (false, EventResponses::default());
                        }

                        // toggle selection
                        (true, self.toggle_entry_selected_at_i(ctx, item_i))
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
        self.update_highlighting(ctx); // this can probably happen in a more targeted way
        self.base.drawing(ctx)
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

    fn get_scl_location_set(&self) -> Rc<RefCell<SclLocationSet>> {
        self.base.get_scl_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.base.get_visible()
    }
}
