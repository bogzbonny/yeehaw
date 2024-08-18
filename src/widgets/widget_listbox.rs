use {
    super::{
        SclVal, ScrollbarPositions, Selectability, VerticalScrollbar, WBStyles, Widget, WidgetBase,
        Widgets,
    },
    crate::{
        element::RelocationRequest, Context, DrawCh, DrawChPos, Element, ElementID, Event,
        EventResponse, EventResponses, Keyboard as KB, Priority, ReceivableEventChanges, RgbColour,
        SortingHat, Style, UpwardPropagator, ZIndex,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct ListBox {
    pub base: WidgetBase,
    pub entries: Rc<RefCell<Vec<String>>>,
    pub selected: Rc<RefCell<usize>>, // the entry which has been selected
    pub cursor: Rc<RefCell<usize>>,   // position of a listbox cursor

    pub lines_per_item: Rc<RefCell<usize>>, // how many lines each item is to take up

    pub selection_mode: SelectionMode,

    #[allow(clippy::type_complexity)]
    // function which executes when the selection changes. NOTE multiple items may be selected
    // simultaniously if the ListBox is configured to allow it. If multiple items are selected,
    // all the selected items will be passed to the function at every selection change.
    pub selection_made_fn: Rc<RefCell<Box<dyn FnMut(Context, Vec<String>) -> EventResponses>>>,

    pub item_selected_style: Rc<RefCell<Style>>,
    pub cursor_over_unselected_style: Rc<RefCell<Style>>,
    pub cursor_over_selected_style: Rc<RefCell<Style>>,

    pub scrollbar_options: Rc<RefCell<ScrollbarPositions>>,
    pub scrollbar: Option<VerticalScrollbar>,
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
            KB::KEY_SPACE.into(), // XXX ensure accounted for
        ]
    }

    pub fn new(
        hat: &SortingHat, _ctx: &Context, entries: Vec<String>,
        selection_made_fn: Box<dyn FnMut(Context, Vec<String>) -> EventResponses>,
    ) -> Self {
        let max_entry_width = entries
            .iter()
            .map(|r| r.lines().map(|l| l.chars().count()).max().unwrap_or(0))
            .max()
            .unwrap_or(0);
        let line_count = entries.iter().map(|r| r.lines().count()).sum();
        let max_lines_per_entry = entries.iter().map(|r| r.lines().count()).max().unwrap_or(0);

        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            SclVal::new_fixed(max_entry_width),
            SclVal::new_fixed(line_count),
            Self::STYLE,
            Self::default_receivable_events(),
        );

        let lb = ListBox {
            base: wb,
            entries: Rc::new(RefCell::new(entries)),
            lines_per_item: Rc::new(RefCell::new(max_lines_per_entry)),
            selected: Rc::new(RefCell::new(0)),
            cursor: Rc::new(RefCell::new(0)),
            selection_mode: SelectionMode::NoLimit,

            item_selected_style: Rc::new(RefCell::new(Self::STYLE_ITEM_SELECTED)),
            cursor_over_unselected_style: Rc::new(RefCell::new(Self::STYLE_CURSOR_OVER_UNSELECTED)),
            cursor_over_selected_style: Rc::new(RefCell::new(Self::STYLE_CURSOR_OVER_SELECTED)),

            scrollbar_options: Rc::new(RefCell::new(ScrollbarPositions::None)),
            scrollbar: None,
            selection_made_fn: Rc::new(RefCell::new(selection_made_fn)),
        };
        lb.update_content_text();
        lb
    }

    // ----------------------------------------------
    // decorators

    pub fn with_left_scrollbar(mut self) -> Self {
        *self.scrollbar_options.borrow_mut() = ScrollbarPositions::Left;
        self
    }

    pub fn with_right_scrollbar(mut self) -> Self {
        *self.scrollbar_options.borrow_mut() = ScrollbarPositions::Right;
        self
    }

    // XXX TODO
    //pub fn with_right_click_menu(mut self) -> Self {
    //self.right_click_menu = rcm;
    //    self
    //}

    pub fn with_lines_per_item(mut self, lines: usize) -> Self {
        *self.lines_per_item.borrow_mut() = lines;
        self.base
            .set_attr_scl_height(SclVal::new_fixed(self.entries.borrow().len() * lines));
        self.update_content_text();
        self
    }

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self.update_content_text();
        self
    }

    pub fn with_width(self, width: SclVal) -> Self {
        self.base.set_attr_scl_width(width);
        self.update_content_text();
        self
    }
    pub fn with_height(self, height: SclVal) -> Self {
        self.base.set_attr_scl_height(height);
        self.update_content_text();
        self
    }
    pub fn with_size(self, width: SclVal, height: SclVal) -> Self {
        self.base.set_attr_scl_width(width);
        self.base.set_attr_scl_height(height);
        self.update_content_text();
        self
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self, hat: &SortingHat, ctx: &Context) -> Widgets {
        let mut out = Widgets(vec![Box::new(self)]);
        /*
            if lb.YScrollbar != NoScrollbar {
                itemsHeight := len(lb.Items) * lb.LinesPerItem
                vsb := NewVerticalScrollbar(lb.GetParentCtx(), lb.Height, itemsHeight)
                if lb.YScrollbar == LeftScrollbar {
                    vsb.At(lb.LocX.MinusStatic(1), lb.LocY)
                } else if lb.YScrollbar == RightScrollbar { // right scrollbar
                    vsb.At(lb.LocX.Plus(lb.Width), lb.LocY)
                }

                // wire the scrollbar to the text box
                vsb.PositionChangedHook = lb.SetContentYOffset
                lb.YChangedHook = vsb.ExternalChange
                *ws = append(*ws, vsb)
            }
        */
        if let ScrollbarPositions::None = *self.scrollbar_options.borrow() {
            return out;
        }
        let height = self.base.get_attr_scl_height().clone();
        let content_height = self.base.content_height();
        let vsb = VerticalScrollbar::new(hat, height, content_height);
        if let ScrollbarPositions::Left = *self.scrollbar_options.borrow() {
            vsb.at(
                self.base.get_attr_scl_loc_x().minus_fixed(1),
                self.base.get_attr_scl_loc_y().clone(),
            );
        } else if let ScrollbarPositions::Right = *self.scrollbar_options.borrow() {
            vsb.at(
                self.base
                    .get_attr_scl_loc_x()
                    .plus(self.base.get_attr_scl_width()),
                self.base.get_attr_scl_loc_y().clone(),
            );
        }
        out.0.push(Box::new(vsb));

        out
    }

    // ----------------------------------------------

    // TRANSLATED TO HERE XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

    pub fn correct_offsets(&self, ctx: &Context) {
        let cursor_pos = *self.cursor.borrow();
        self.base
            .correct_offsets_to_view_position(ctx, 0, cursor_pos);
        self.scrollbar.external_change(
            ctx,
            *self.base.sp.content_view_offset_y.borrow(),
            self.base.content_height(),
        );
    }

    pub fn padded_entry_text(&self, ctx: &Context, i: usize) -> String {
        let entry = self.entries.borrow()[i].clone();
        let entry_len = entry.chars().count();
        //let width = self.base.get_width(ctx);
        let width = self.get_scl_width().get_val(ctx.get_width().into());
        let left_padding = *self.left_padding.borrow();
        let right_padding = width.saturating_sub(entry_len + left_padding);
        let pad_left = " ".repeat(left_padding);
        let pad_right = " ".repeat(right_padding);
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
        let h = self.expanded_height();
        self.base.set_attr_scl_height(SclVal::new_fixed(h));

        // must set the content for the offsets to be correct
        self.base.set_content_from_string(ctx, &self.text(ctx));
        self.correct_offsets(ctx);

        let rr = RelocationRequest::new_down(h as i32 - 1);
        EventResponse::default().with_relocation(rr)
    }

    pub fn perform_close(&self, ctx: &Context, escaped: bool) -> EventResponses {
        *self.open.borrow_mut() = false;
        *self.base.sp.content_view_offset_y.borrow_mut() = 0;
        self.scrollbar
            .external_change(ctx, 0, self.base.content_height());
        self.base.set_attr_scl_height(SclVal::new_fixed(1));
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

/*

//-------------------------------------------------------------

func (lbi ListBoxItem) GetText(width, height int) string {

    // pad the text to the width and height
    text := strings.Split(lbi.Name, "\n")
    if len(text) > height {
        text = text[:height]
    } else if len(text) < height {
        for i := len(text); i < height; i++ {
            text = append(text, "")
        }
    }

    // pad the text to the width of the listbox
    for i := range text {

        // cut off the text if it is too long
        if len(text[i]) > width && width >= 3 {
            text[i] = text[i][:width-3] + "..."
        }

        if len(text[i]) < width {
            text[i] = text[i] + strings.Repeat(" ", width-len(text[i]))
        }
    }
    return strings.Join(text, "\n")
}

func (lb *ListBox) CorrectOffsets() {
    startY, endY := lb.GetContentYRangeForItemIndex(lb.Cursor)
    _, _ = startY, endY

    if endY >= lb.ContentYOffset+lb.GetHeight() {
        lb.CorrectOffsetsToViewPosition(0, endY)
    } else if startY < lb.ContentYOffset {
        lb.CorrectOffsetsToViewPosition(0, startY)
    }

    // call the scrollbar hook if it exists
    if lb.YChangedHook != nil {
        lb.YChangedHook(lb.ContentYOffset, lb.ContentHeight())
    }
}

func (lb *ListBox) GetItemIndexForViewY(y int) int {
    offset := y + lb.ContentYOffset
    if offset < 0 {
        return -1
    }
    return offset / lb.LinesPerItem
}

func (lb *ListBox) GetContentYRangeForItemIndex(index int) (startY int, endY int) {
    startY = index * lb.LinesPerItem
    endY = startY + lb.LinesPerItem - 1
    return startY, endY
}

func (lb *ListBox) SetSelectability(s Selectability) yh.EventResponse {
    if lb.Selectedness == Selected && s != Selected {
        lb.Cursor = -1
    }
    return lb.WidgetBase.SetSelectability(s)
}

func (lb *ListBox) SetItems(items []ListBoxItem) {
    lb.Items = items
}

func (lb *ListBox) SetItemsFromStrings(items []string) {

    lbItems := make([]ListBoxItem, len(items))
    maxWidth := 0
    for i, item := range items {
        split := strings.Split(item, "\n")
        for _, line := range split {
            if len(line) > maxWidth {
                maxWidth = len(line)
            }
        }
        lbItems[i] = NewListBoxItem(item)
    }

    lb.Items = lbItems
    lb.UpdateContentText()
}

// XXX seems like UpdateContentText should be called something else if it
// is also needed to update the styles
func (lb *ListBox) UpdateContentText() {
    str := ""
    for i, item := range lb.Items {
        str += item.GetText(lb.GetWidth(), lb.LinesPerItem)
        if i < len(lb.Items)-1 {
            str += "\n"
        }
    }
    lb.SetContentFromString(str)
    lb.UpdateHighlighting()
}

func (lb *ListBox) Drawing() (chs []yh.DrawChPos) {
    lb.UpdateHighlighting() // XXX this can probably happen in a more targeted way
    return lb.WidgetBase.Drawing()
}

// need to re set the content in order to reflect active style
func (lb *ListBox) UpdateHighlighting() {

    // change the style for selection and the cursor
    for i, item := range lb.Items {

        sty := lb.GetCurrentStyle() // from the widget base
        switch {
        case item.IsSelected && lb.Cursor == i && lb.Selectedness == Selected:
            sty = lb.CursorOverSelectedStyle
        case !item.IsSelected && lb.Cursor == i && lb.Selectedness == Selected:
            sty = lb.CursorOverUnselectedStyle
        case item.IsSelected:
            sty = lb.ItemSelectedStyle
        }
        yStart, yEnd := lb.GetContentYRangeForItemIndex(i)
        for y := yStart; y <= yEnd; y++ {
            lb.Content = lb.Content.ChangeStyleAlongY(y, sty)
        }
    }

    // update the rest of the lines
    for i := len(lb.Items) * lb.LinesPerItem; i < lb.GetHeight(); i++ {
        sty := lb.GetCurrentStyle() // from the widget base
        lb.Content = lb.Content.ChangeStyleAlongY(i, sty)
    }
}

func (lb *ListBox) CursorUp() {
    if lb.Cursor > 0 {
        lb.Cursor--
    }
    if lb.Cursor == -1 {
        lb.Cursor = len(lb.Items) - 1
    }
    lb.CorrectOffsets()
}

func (lb *ListBox) CursorDown() {
    if lb.Cursor < len(lb.Items)-1 {
        lb.Cursor++
    }
    if lb.Cursor == -1 {
        lb.Cursor = 0
    }
    lb.CorrectOffsets()
}

func (lb *ListBox) ReceiveKeyEventCombo(evs []*tcell.EventKey) (
    captured bool, resp yh.EventResponse) {

    resp = yh.NewEventResponse()

    if lb.Selectedness != Selected {
        return false, yh.NewEventResponse()
    }

    captured = false
    switch {
    case yh.DownEKC.Matches(evs) || yh.JLowerEKC.Matches(evs):
        lb.CursorDown()
        captured = true

    case yh.UpEKC.Matches(evs) || yh.KLowerEKC.Matches(evs):
        lb.CursorUp()
        captured = true

    case yh.EnterEKC.Matches(evs):
        if lb.Cursor >= 0 && lb.Cursor < len(lb.Items) {
            lb.Items[lb.Cursor].IsSelected = !lb.Items[lb.Cursor].IsSelected
            captured = true
            if lb.SelectionChangedFn != nil {
                resp = lb.SelectionChangedFn(lb.Items)
            }
        }
    }
    return captured, resp
}

func (lb *ListBox) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {

    resp = yh.NewEventResponse()

    if ev.Buttons() == tcell.WheelDown {
        lb.CursorDown()
        return true, resp
    }
    if ev.Buttons() == tcell.WheelUp {
        lb.CursorUp()
        return true, resp
    }

    // if not primary/secondary click, do nothing
    if ev.Buttons() != tcell.Button1 && ev.Buttons() != tcell.Button2 {
        return false, resp
    }

    _, y := ev.Position()                   // get click position
    itemIndex := lb.GetItemIndexForViewY(y) // get item index at click position

    if itemIndex < 0 || itemIndex >= len(lb.Items) { // invalid item index
        return false, resp
    }

    // toggle selection
    if ev.Buttons() == tcell.Button1 {
        lb.Items[itemIndex].IsSelected = !lb.Items[itemIndex].IsSelected
        if lb.SelectionChangedFn != nil {
            resp = lb.SelectionChangedFn(lb.Items)
        }
        return true, resp
    }

    // handle secondary click
    if lb.RightClickMenu != nil {
        // send the event to the right click menu to check for right click
        createRCM := lb.RightClickMenu.CreateMenuIfRightClick(ev)
        if createRCM.HasWindow() {
            return true, yh.NewEventResponse().WithWindow(createRCM)
        }
    }

    return false, resp
}
*/
