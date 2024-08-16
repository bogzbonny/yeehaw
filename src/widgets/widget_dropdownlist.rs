//use {
//    super::{SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
//    crate::{
//        YHAttributes, Context, DrawChPos, Element, ElementID, Event, EventResponse,
//        Keyboard as KB, Priority, ReceivableEventChanges, RgbColour, SortingHat, Style,
//        UpwardPropagator,
//    },
//    crossterm::event::{MouseButton, MouseEventKind},
//    std::{cell::RefCell, rc::Rc},
//};

// TODO add scrollbar support
// TODO multiline dropdown entry support

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

//    Scrollbar *VerticalScrollbar
//}

//#[derive(Clone)]
//pub struct DropdownList {
//    pub base: WidgetBase,
//    pub entries: Rc<RefCell<Vec<String>>>,
//    pub left_padding: Rc<RefCell<usize>>,
//    pub selected: Rc<RefCell<usize>>,
//    pub cursor: Rc<RefCell<usize>>,
//    pub open: Rc<RefCell<bool>>,
//    pub max_expanded_height: Rc<RefCell<i32>>,
//    pub dropdown_arrow: Rc<RefCell<char>>,
//    pub cursor_style: Rc<RefCell<Style>>,
//    pub selection_made_fn: Rc<RefCell<Box<dyn FnMut(String) -> EventResponse>>>,
//    // embedded scrollbar in dropdown list
//    //pub scrollbar: Rc<RefCell<VerticalScrollbar>>,
//}

//impl DropdownList {
//    const KIND: &'static str = "widget_checkbox";

//    const STYLE: WBStyles = WBStyles {
//        selected_style: Style::new()
//            .with_bg(RgbColour::YELLOW)
//            .with_fg(RgbColour::BLACK)
//            .with_attr(YHAttributes::new().with_bold()),
//        ready_style: Style::new()
//            .with_bg(RgbColour::WHITE)
//            .with_fg(RgbColour::BLACK)
//            .with_attr(YHAttributes::new().with_bold()),
//        unselectable_style: Style::new()
//            .with_bg(RgbColour::GREY13)
//            .with_fg(RgbColour::BLACK)
//            .with_attr(YHAttributes::new().with_bold()),
//    };

//    pub fn default_receivable_events() -> Vec<Event> {
//        vec![KB::KEY_ENTER.into()] // when "active" hitting enter will click the button
//    }

//    pub fn new(hat: &SortingHat, ctx: &Context) -> Self {
//        let wb = WidgetBase::new(
//            hat,
//            Self::KIND,
//            ctx.clone(),
//            SclVal::new_fixed(1),
//            SclVal::new_fixed(1),
//            Self::STYLE,
//            Self::default_receivable_events(),
//        );
//        DropdownList {
//            base: wb,
//            checked: Rc::new(RefCell::new(false)),
//            checkmark: Rc::new(RefCell::new('√')),
//            clicked_fn: Rc::new(RefCell::new(|_| EventResponse::default())),
//        }
//    }

//    // ----------------------------------------------
//    // decorators

//    pub fn with_styles(self, styles: WBStyles) -> Self {
//        self.base.set_styles(styles);
//        self
//    }

//    pub fn with_clicked_fn(mut self, clicked_fn: Box<dyn FnMut(bool) -> EventResponse>) -> Self {
//        self.clicked_fn = Rc::new(RefCell::new(clicked_fn));
//        self
//    }

//    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
//        self.base.at(loc_x, loc_y);
//        self
//    }

//    pub fn to_widgets(self) -> Widgets {
//        Widgets(vec![Box::new(self)])
//    }

//    // ----------------------------------------------

//    pub fn text(&self) -> String {
//        if *self.checked.borrow() {
//            return self.checkmark.borrow().to_string();
//        }
//        " ".to_string()
//    }

//    pub fn click(&self) -> EventResponse {
//        let checked = !*self.checked.borrow();
//        self.checked.replace(checked);
//        self.base.set_content_from_string(&self.text());
//        (self.clicked_fn.borrow_mut())(checked)
//    }
//}

//impl Widget for DropdownList {}

//impl Element for DropdownList {
//    fn kind(&self) -> &'static str {
//        self.base.kind()
//    }
//    fn id(&self) -> ElementID {
//        self.base.id()
//    }
//    fn receivable(&self) -> Vec<(Event, Priority)> {
//        self.base.receivable()
//    }

//    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponse) {
//        let _ = self.base.receive_event(ctx, ev.clone());
//        match ev {
//            Event::KeyCombo(ke) => {
//                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
//                    return (false, EventResponse::default());
//                }
//                if ke[0].matches(&KB::KEY_ENTER) {
//                    return (true, self.click());
//                }
//            }
//            Event::Mouse(me) => {
//                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
//                    return (true, self.click());
//                }
//            }
//            _ => {}
//        }
//        (false, EventResponse::default())
//    }

//    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
//        self.base.change_priority(ctx, p)
//    }
//    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
//        // need to re set the content in order to reflect active style
//        self.base.set_content_from_string(&self.text());
//        self.base.drawing(ctx)
//    }
//    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
//        self.base.get_attribute(key)
//    }
//    fn set_attribute(&self, key: &str, value: Vec<u8>) {
//        self.base.set_attribute(key, value)
//    }
//    fn set_upward_propagator(&self, up: Rc<RefCell<dyn UpwardPropagator>>) {
//        self.base.set_upward_propagator(up)
//    }
//}

/*

// when "active" hitting enter will click the button
var DropdownListEvCombos = []yh.PrioritizableEv{
    yh.EnterEKC, yh.DownEKC, yh.UpEKC, yh.KLowerEKC, yh.JLowerEKC, yh.SpaceEKC}

var DropdownListStyle = WBStyles{
    SelectedStyle:     tcell.StyleDefault.Background(tcell.ColorLightYellow).Foreground(tcell.ColorBlack),
    ReadyStyle:        tcell.StyleDefault.Background(tcell.ColorWhite).Foreground(tcell.ColorBlack),
    UnselectableStyle: tcell.StyleDefault.Background(tcell.ColorLightSlateGrey).Foreground(tcell.ColorBlack),
}

var DefaultDropdownArrow = yh.NewDrawCh('▼', false,
    tcell.StyleDefault.Background(tcell.ColorLightGrey).Foreground(tcell.ColorBlack))

var DefaultDDLCursorStyle = tcell.StyleDefault.Background(tcell.ColorBlue)
var DefaultDDLLeftPadding = 1

// needs to be slightly above other widgets to select properly
// if widgets overlap
const DropdownListZIndex = WidgetZIndex - 1

var DDScrollbarStyle = WBStyles{
    SelectedStyle:     tcell.StyleDefault.Background(tcell.ColorDarkSlateGrey).Foreground(tcell.ColorWhite),
    ReadyStyle:        tcell.StyleDefault.Background(tcell.ColorDarkSlateGrey).Foreground(tcell.ColorWhite),
    UnselectableStyle: tcell.StyleDefault.Background(tcell.ColorDarkSlateGrey).Foreground(tcell.ColorWhite),
}

func NewDropdownList(pCtx yh.Context, entries []string, selectionMadeFn func(string) yh.EventResponse) *DropdownList {

    maxWidth := 0
    for _, entry := range entries {
        if len(entry) > maxWidth {
            maxWidth = len(entry)
        }
    }

    wb := NewWidgetBase(pCtx, NewStatic(maxWidth), NewStatic(1), DropdownListStyle, DropdownListEvCombos)
    sb := NewVerticalScrollbar(pCtx, NewStatic(0), 0).WithoutArrows().WithStyle(DDScrollbarStyle)

    d := &DropdownList{
        WidgetBase:        wb,
        Entries:           entries,
        LeftPadding:       DefaultDDLLeftPadding,
        Selected:          0,
        Open:              false,
        MaxExpandedHeight: -1,
        DropdownArrow:     DefaultDropdownArrow,
        CursorStyle:       DefaultDDLCursorStyle,
        SelectionMadeFn:   selectionMadeFn,
        Scrollbar:         sb,
    }

    //wire the scrollbar to the dropdown list
    sb.PositionChangedHook = d.SetContentYOffset

    _ = d.Drawing()
    return d
}

func (d *DropdownList) WithWidth(width int) *DropdownList {
    d.Width = NewStatic(width)
    return d
}

func (d *DropdownList) WithMaxExpandedHeight(height int) *DropdownList {
    d.MaxExpandedHeight = height
    d.Scrollbar.SetHeight(
        NewStatic(height),   // view height (same as the dropdown list height)
        NewStatic(height-1), // scrollbar height (1 less, b/c scrollbar's below the drop-arrow)
        len(d.Entries))      // scrollable domain
    return d
}

func (d *DropdownList) At(locX, locY SclVal) *DropdownList {
    d.WidgetBase.At(locX, locY)
    return d
}

// returns Widgets for ease of labeling
func (d *DropdownList) ToWidgets() Widgets {
    return []Widget{d}
}

// ----------------------------------------------

func (d *DropdownList) GetLocation() yh.Location {
    loc := d.WidgetBase.GetLocation()
    loc.Z = DropdownListZIndex
    return loc
}

// ----------------------------------------------

func (d *DropdownList) CorrectOffsets() {
    d.CorrectOffsetsToViewPosition(0, d.Cursor)
    d.Scrollbar.ExternalChange(d.ContentYOffset, d.ContentHeight())
}

func (d *DropdownList) paddedEntryText(i int) string {
    entry := d.Entries[i]
    rightPadding := 0
    width := d.GetWidth()
    if width > len(entry)+d.LeftPadding {
        rightPadding = width - len(entry) - d.LeftPadding
    }
    padLeft := strings.Repeat(" ", d.LeftPadding)
    padRight := strings.Repeat(" ", rightPadding)
    return fmt.Sprintf("%s%s%s", padLeft, entry, padRight)
}

// doesn't include the arrow text
func (d *DropdownList) Text() string {

    if !d.Open {
        return d.paddedEntryText(d.Selected)
    }

    out := ""
    for i := range d.Entries {
        out += d.paddedEntryText(i)
        if i != len(d.Entries)-1 {
            out += "\n"
        }
    }
    return out
}

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

func (d *DropdownList) SetSelectability(s Selectability) yh.EventResponse {
    if d.Selectedness == Selected && s != Selected {
        if d.Open {
            return d.PerformClose(true)
        }
    }
    return d.WidgetBase.SetSelectability(s)
}

func (d *DropdownList) ExpandedHeight() int {
    if d.MaxExpandedHeight != -1 && len(d.Entries) > d.MaxExpandedHeight {
        return d.MaxExpandedHeight
    }
    return len(d.Entries)
}

// whether or not the dropdown list should display a scrollbar
func (d *DropdownList) DisplayScrollbar() bool {
    return d.MaxExpandedHeight != -1 && len(d.Entries) > d.MaxExpandedHeight
}

func (d *DropdownList) PerformOpen() yh.EventResponse {
    d.Open = true
    d.Cursor = d.Selected
    h := d.ExpandedHeight()
    d.WidgetBase.Height = NewStatic(h)

    // must set the content for the offsets to be correct
    d.SetContentFromString(d.Text())
    d.CorrectOffsets()

    rr := yh.NewRelocationRequestDown(h - 1)
    return yh.NewEventResponse().WithRelocation(rr)
}

func (d *DropdownList) PerformClose(escaped bool) yh.EventResponse {
    d.Open = false
    d.ContentYOffset = 0
    d.Scrollbar.ExternalChange(d.ContentYOffset, d.ContentHeight())
    d.WidgetBase.Height = NewStatic(1)
    resp := yh.NewEventResponse()
    if !escaped && d.Selected != d.Cursor {
        d.Selected = d.Cursor
        if d.SelectionMadeFn != nil {
            resp = d.SelectionMadeFn(d.Entries[d.Selected])
        }
    }
    rr := yh.NewRelocationRequestDown(-(d.ExpandedHeight() - 1))
    return resp.WithRelocation(rr)
}

func (d *DropdownList) CursorUp() {
    if d.Cursor > 0 {
        d.Cursor--
    }
    d.CorrectOffsets()
}

func (d *DropdownList) CursorDown() {
    if d.Cursor < len(d.Entries)-1 {
        d.Cursor++
    }
    d.CorrectOffsets()
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
