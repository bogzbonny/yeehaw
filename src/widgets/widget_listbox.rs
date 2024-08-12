/*
package widgets

import (
	"strings"

	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
	els "keybase.io/nwmod/nwmod/yeehaw/elements"
)

// TODO figure out how to simulate being pressed with time based events

type ListBox struct {
	*WidgetBase
	Items              []ListBoxItem
	LinesPerItem       int // how many lines each item is to take up
	Cursor             int // postion of a listbox cursor
	SelectionChangedFn func([]ListBoxItem) yh.EventResponse

	ItemSelectedStyle         tcell.Style
	CursorOverUnselectedStyle tcell.Style
	CursorOverSelectedStyle   tcell.Style

	YScrollbar ScrollbarOption

	RightClickMenu *els.RightClickMenuTemplate

	// for the potential scrollbar
	YChangedHook func(newYPosition, newHeight int)
}

// holds data relevant to a specific item in a listbox
type ListBoxItem struct {
	Name       string
	IsSelected bool
}

// NewListBoxItem creates a new ListBoxItem object
func NewListBoxItem(name string) ListBoxItem {
	return ListBoxItem{
		Name:       name,
		IsSelected: false,
	}
}

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

// when "active" hitting enter will click the button
var (
	ListBoxEvCombos = []yh.PrioritizableEv{
		yh.EnterEKC, yh.UpEKC, yh.DownEKC, yh.JLowerEKC, yh.KLowerEKC}

	ListBoxStyle = WBStyles{
		SelectedStyle:     tcell.StyleDefault.Background(tcell.ColorLightYellow).Foreground(tcell.ColorBlack),
		ReadyStyle:        tcell.StyleDefault.Background(tcell.ColorWhite).Foreground(tcell.ColorBlack),
		UnselectableStyle: tcell.StyleDefault.Background(tcell.ColorLightSlateGrey).Foreground(tcell.ColorBlack),
	}

	// for when the cursor is over a selected item in the list box
	ListBoxItemSelectedStyle         = tcell.StyleDefault.Background(tcell.ColorNavy).Foreground(tcell.ColorWhite)
	ListBoxCursorOverUnselectedStyle = tcell.StyleDefault.Background(tcell.ColorLightBlue).Foreground(tcell.ColorBlack)
	// for when the cursor is over an unselected item
	ListBoxCursorOverSelectedStyle = tcell.StyleDefault.Background(tcell.ColorBlue).Foreground(tcell.ColorWhite)
)

func NewListBox(pCtx yh.Context, items []ListBoxItem, SelectionChangedFn func([]ListBoxItem) yh.EventResponse) *ListBox {

	maxWidth := 0
	for _, item := range items {
		split := strings.Split(item.Name, "\n")
		for _, line := range split {
			if len(line) > maxWidth {
				maxWidth = len(line)
			}
		}
	}

	wb := NewWidgetBase(pCtx, NewStatic(maxWidth), NewStatic(len(items)), ListBoxStyle, ListBoxEvCombos)
	lb := &ListBox{
		WidgetBase:                wb,
		Items:                     items,
		LinesPerItem:              1,
		Cursor:                    0,
		ItemSelectedStyle:         ListBoxItemSelectedStyle,
		CursorOverUnselectedStyle: ListBoxCursorOverUnselectedStyle,
		CursorOverSelectedStyle:   ListBoxCursorOverSelectedStyle,

		YScrollbar:         NoScrollbar,
		SelectionChangedFn: SelectionChangedFn,
	}
	lb.UpdateContentText()
	return lb
}

func (lb *ListBox) WithSize(width, height SclVal) *ListBox {
	lb.Width, lb.Height = width, height
	lb.UpdateContentText()
	return lb
}

func (lb *ListBox) WithStyles(styles WBStyles) *ListBox {
	lb.Styles = styles
	lb.UpdateContentText()
	return lb
}

func (lb *ListBox) WithLinesPerItem(lines int) *ListBox {
	lb.LinesPerItem = lines
	lb.Height = NewStatic(len(lb.Items) * lines)
	lb.UpdateContentText()
	return lb
}

func (lb *ListBox) WithLeftScrollbar() *ListBox {
	lb.YScrollbar = LeftScrollbar
	return lb
}

func (lb *ListBox) WithRightScrollbar() *ListBox {
	lb.YScrollbar = RightScrollbar
	return lb
}

func (lb *ListBox) WithRightClickMenu(rcm *els.RightClickMenuTemplate) *ListBox {
	lb.RightClickMenu = rcm
	return lb
}

func (lb *ListBox) At(locX, locY SclVal) *ListBox {
	lb.WidgetBase.At(locX, locY)
	return lb
}

func (lb *ListBox) ToWidgets() Widgets {
	out := Widgets{lb}
	lb.AppendScrollbarWidget(&out)
	return out
}

func (lb *ListBox) AppendScrollbarWidget(ws *Widgets) {
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
}

//-------------------------------------------------------------

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
