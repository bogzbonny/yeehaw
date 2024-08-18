use {
    super::{SclVal, Selectability, VerticalScrollbar, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        element::RelocationRequest, Context, DrawCh, DrawChPos, Element, ElementID, Event,
        EventResponse, EventResponses, Keyboard as KB, Priority, ReceivableEventChanges, RgbColour,
        SortingHat, Style, UpwardPropagator, YHAttributes, ZIndex,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

//TODO add scrollbar support
//TODO multiline dropdown entry support

//type DropdownList struct {
//    *WidgetBase
//    Entries           []string
//    LeftPadding       int
//    Selected          int                           // the entry which has been selected
//    Cursor            int                           // the entry that is currently hovered while open
//    Open              bool                          // if the list is open
//    MaxExpandedHeight int                           // the max height of the entire dropdown list when expanded, -1 = no max
//    DropdownArrow     yh.DrawCh                     // ▼
//    CursorStyle       tcell.Style                   // style for the selected entry
//    SelectionMadeFn   func(string) yh.EventResponse // function which executes when button moves from pressed -> unpressed

//    Scrollbar: VerticalScrollbar
//}

#[derive(Clone)]
pub struct DropdownList {
    pub base: WidgetBase,
    pub entries: Rc<RefCell<Vec<String>>>,
    pub left_padding: Rc<RefCell<usize>>,
    pub selected: Rc<RefCell<usize>>, // the entry which has been selected
    pub cursor: Rc<RefCell<usize>>,   // the entry that is currently hovered while open
    pub open: Rc<RefCell<bool>>,      // if the list is open
    pub max_expanded_height: Rc<RefCell<Option<usize>>>, // the max height of the entire dropdown list when expanded, None = no max
    pub dropdown_arrow: Rc<RefCell<char>>,               // ▼
    pub cursor_style: Rc<RefCell<Style>>,                // style for the selected entry
    #[allow(clippy::type_complexity)]
    pub selection_made_fn: Rc<RefCell<Box<dyn FnMut(Context, String) -> EventResponse>>>, // function which executes when button moves from pressed -> unpressed
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
        hat: &SortingHat, entries: Vec<String>,
        selection_made_fn: Box<dyn FnMut(Context, String) -> EventResponse>,
    ) -> Self {
        let max_width = entries.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            SclVal::new_fixed(max_width),
            SclVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        let sb = VerticalScrollbar::new(hat, SclVal::new_fixed(0), 0)
            .without_arrows()
            .with_styles(Self::STYLE_SCROLLBAR);

        //wire the scrollbar to the dropdown list
        let wb_ = wb.clone();
        let hook = Rc::new(RefCell::new(move |ctx, y| {
            wb_.set_content_y_offset(&ctx, y)
        }));
        *sb.position_changed_hook.borrow_mut() = Some(hook);

        // TRANSLATION NOTE there used to be a drawing() call before returning the list

        DropdownList {
            base: wb,
            entries: Rc::new(RefCell::new(entries)),
            left_padding: Rc::new(RefCell::new(1)),
            selected: Rc::new(RefCell::new(0)),
            cursor: Rc::new(RefCell::new(0)),
            open: Rc::new(RefCell::new(false)),
            max_expanded_height: Rc::new(RefCell::new(None)),
            dropdown_arrow: Rc::new(RefCell::new('▼')),
            cursor_style: Rc::new(RefCell::new(Self::STYLE_DD_CURSOR)),
            selection_made_fn: Rc::new(RefCell::new(selection_made_fn)),
            scrollbar: sb,
        }
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn with_width(self, width: usize) -> Self {
        self.base.set_attr_scl_width(SclVal::new_fixed(width));
        self
    }

    pub fn with_max_expanded_height(self, height: usize) -> Self {
        *self.max_expanded_height.borrow_mut() = Some(height);
        self.scrollbar.set_height(
            SclVal::new_fixed(height), // view height (same as the dropdown list height)
            SclVal::new_fixed(height.saturating_sub(1)), // scrollbar height (1 less, b/c scrollbar's below the drop-arrow)
            self.entries.borrow().len(),                 // scrollable domain
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
        self.base
            .correct_offsets_to_view_position(ctx, 0, *self.cursor.borrow());
        self.scrollbar.external_change(
            ctx,
            *self.base.sp.content_view_offset_y.borrow(),
            self.base.content_height(),
        );
    }

    pub fn padded_entry_text(&self, ctx: &Context, i: usize) -> String {
        let entry = self.entries.borrow()[i].clone();
        let entry_len = entry.chars().count();
        let width = self.base.get_width(ctx);
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
            if i != entries_len - 1 {
                out += "\n";
            }
        }
        out
    }

    // the height of the dropdown list while expanded
    pub fn expanded_height(&self) -> usize {
        if let Some(max_height) = *self.max_expanded_height.borrow() {
            if self.entries.borrow().len() > max_height {
                return max_height;
            }
        }
        self.entries.borrow().len()
    }

    // whether or not the dropdown list should display a scrollbar
    pub fn display_scrollbar(&self) -> bool {
        self.max_expanded_height.borrow().is_some()
            && self.entries.borrow().len() > self.expanded_height()
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
        let resp = if !escaped && *self.selected.borrow() != *self.cursor.borrow() {
            *self.selected.borrow_mut() = *self.cursor.borrow();
            (self.selection_made_fn.borrow_mut())(
                ctx.clone(),
                self.entries.borrow()[*self.selected.borrow()].clone(),
            )
        } else {
            EventResponse::default()
        };
        let rr = RelocationRequest::new_down(-(self.expanded_height() as i32 - 1));
        resp.with_relocation(rr).into()
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
// XXX must override the widget function here ... maybe through the use of a hook which should
// be called in the widget function?? ... or can maybe override in the impl Widget for Dropdownlist {} block
// SEE https://www.reddit.com/r/learnrust/comments/15wftju/referencing_a_traits_default_implementation_of_a/
//   - can use the pre-processing example by minno
func (d *DropdownList) SetSelectability(s Selectability) yh.EventResponse {
    if d.Selectedness == Selected && s != Selected {
        if d.Open {
            return d.PerformClose(true)
        }
    }
    return d.WidgetBase.SetSelectability(s)
}
*/

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
                //if ke[0].matches(&KB::KEY_ENTER) {
                //    return (true, self.click());
                //}
            }
            Event::Mouse(_me) => {
                //if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                //    return (true, self.click());
                //}
            }
            _ => {}
        }
        (false, EventResponses::default())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        // need to re set the content in order to reflect active style
        //self.base.set_content_from_string(&self.text());
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

/*


// need to reset the content in order to reflect active style
func (d *DropdownList) Drawing() []yh.DrawChPos {

    d.SetContentFromString(d.Text())

    // highlight the hovering entry
    if d.Open {
        d.Content = d.Content.ChangeStyleAlongY(d.Cursor, d.CursorStyle)
    }

    chs := d.WidgetBase.Drawing()

    // set the scrollbar on top of the content
    if d.Open && d.Scrollbar != nil && d.DisplayScrollbar() {
        sbchs := d.Scrollbar.Drawing()
        // shift the scrollbar content to below the arrow
        for i := range sbchs {
            sbchs[i].X += d.GetWidth() - 1
            sbchs[i].Y += 1
        }
        chs = append(chs, sbchs...)
    }

    // set the arrow
    chs = append(chs, yh.NewDrawChPos(d.DropdownArrow, d.GetWidth()-1, 0))

    return chs
}

func (d *DropdownList) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {

    if d.Selectedness != Selected {
        return false, yh.NewEventResponse()
    }

    switch {
    case !d.Open && (yh.EnterEKC.Matches(evs) ||
        yh.DownEKC.Matches(evs) || yh.JLowerEKC.Matches(evs) ||
        yh.UpEKC.Matches(evs) || yh.KLowerEKC.Matches(evs)):
        return true, d.PerformOpen()
    case d.Open && yh.EnterEKC.Matches(evs):
        return true, d.PerformClose(false)
    case d.Open && (yh.DownEKC.Matches(evs) || yh.JLowerEKC.Matches(evs)):
        d.CursorDown()
    case d.Open && (yh.UpEKC.Matches(evs) || yh.KLowerEKC.Matches(evs)):
        d.CursorUp()
    case d.Open && yh.SpaceEKC.Matches(evs):
        if d.Scrollbar.Selectedness != Selected {
            _ = d.Scrollbar.SetSelectability(Selected)
        }
        return d.Scrollbar.ReceiveKeyEventCombo(evs)
    }

    return false, yh.NewEventResponse()
}

func (d *DropdownList) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {

    clicked := ev.Buttons() == tcell.Button1

    switch {
    case !d.Open && clicked:
        return true, d.PerformOpen()

    case d.Open && ev.Buttons() == tcell.WheelUp:
        d.CursorUp()
    case d.Open && ev.Buttons() == tcell.WheelDown:
        d.CursorDown()

    case d.Open && !clicked:
        // change hovering location to the ev
        x, y := ev.Position()
        if y == 0 && x == d.GetWidth()-1 { // on arrow
            break
        } else if y > 0 && x == d.GetWidth()-1 && d.DisplayScrollbar() { // on scrollbar
            break
        } else {
            d.Cursor = y + d.ContentYOffset
        }
        _ = d.Scrollbar.SetSelectability(Ready)

    case d.Open && clicked:
        x, y := ev.Position()
        if y > 0 && x == d.GetWidth()-1 && d.DisplayScrollbar() {
            if d.Scrollbar.Selectedness != Selected {
                _ = d.Scrollbar.SetSelectability(Selected)
            }
            // send the the event to the scrollbar (x adjusted to 0)
            ev2 := tcell.NewEventMouse(0, y-1, ev.Buttons(), ev.Modifiers())
            return d.Scrollbar.ReceiveMouseEvent(ev2)
        }
        if y == 0 && x == d.GetWidth()-1 { // on arrow close without change
            return true, d.PerformClose(true)
        }
        _ = d.Scrollbar.SetSelectability(Ready)
        d.Cursor = y + d.ContentYOffset
        return true, d.PerformClose(false)
    }
    return false, yh.NewEventResponse()
}
*/
