/*
package widgets

import (
	"strings"

	tcell "github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

//  WIDGET FARMER       ✲
//                         /|\      *
//  ⌂  ⌂  ⌂         ✲      \|/   /  *  \
//                 ✲            * time  *
//  water      ~  _|_  ~         \  *  /      ⌃
//  light        /   \              *       \   /
//  nutrience   / o o \   hi,             discovery
//  eneergy    /  ._   \  dont u d4re       /   \
//  darkness        \       munch my crops    ⌄
//                   -<<<-
//     |    |    |    |    |    |    |    |     f
//    \|/  \|/  \|/  \|/  \|/  \|/  \|/  \|/  \ o /
//    \|/  \|/  \:)  \|/  \|\  \|/  \|/  \|/  \ c /
//    \|/  \|/  \|/  \|/  \|/  \|/  \|/  \|/  \ u /
//     |    |    |    | oo |    |    |    |     s

// widgets are a basically really simple elements
// besides that a widget is aware of its location
type Widget interface {

	// widgets can only receive events when they are active
	Receivable() []yh.PrioritizableEv

	GetParentCtx() yh.Context
	SetParentCtx(parentCtx yh.Context)

	// Draw the widget to the screen
	Drawing() (chs []yh.DrawChPos)

	SetStyles(styles WBStyles)

	ResizeEvent(parentCtx yh.Context)

	GetLocation() yh.Location
	GetSclLocation() SclLocation

	ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse)

	// NOTE the mouse event input is adjusted for the widgets location
	// (aka if you click on the top-left corner of the widget ev.Position()
	// will be 0, 0)
	ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse)

	GetSelectability() Selectability

	// NOTE window creation in response to SetSelectability
	// is currently not supported
	SetSelectability(s Selectability) yh.EventResponse

	ToWidgets() Widgets
}

type Selectability string

const (
	Selected     Selectability = "selected"     // currently selected
	Ready        Selectability = "ready"        // not selected but able to be selected
	Unselectable Selectability = "unselectable" // unselectable
)

//------------------------------------------------

const WidgetZIndex = 10

type Widgets []Widget

func NewWidgets(ws ...Widget) Widgets {
	return ws
}

// returns the smallest location which encompasses all
// the sub-locations for all the contained widgets
func (ws Widgets) OverallLoc() (l SclLocation) {
	if len(ws) == 0 {
		return l
	}

	for _, wl := range ws {
		wlLoc := wl.GetSclLocation()
		l.StartX = l.StartX.PlusMinOf(wlLoc.StartX)
		l.EndX = l.EndX.PlusMaxOf(wlLoc.EndX)
		l.StartY = l.StartY.PlusMinOf(wlLoc.StartY)
		l.EndY = l.EndY.PlusMaxOf(wlLoc.EndY)
	}
	return l
}

type LabelPosition string

const (
	// label positions
	//
	//      1  2
	//     5████7
	//      ████
	//     6████8
	//      3  4

	AboveThenLeft   LabelPosition = "above-then-left"  // 1
	AboveThenRight  LabelPosition = "above-then-right" // 2
	BelowThenLeft   LabelPosition = "below-then-left"  // 3
	BelowThenRight  LabelPosition = "below-then-right" // 4
	LeftThenTop     LabelPosition = "left-then-above"  // 5
	LeftThenBottom  LabelPosition = "left-then-below"  // 6
	RightThenTop    LabelPosition = "right-then-above" // 7
	RightThenBottom LabelPosition = "right-then-below" // 8
)

// get the label location from the label position
func (ws *Widgets) LabelPositionToXY(p LabelPosition, labelWidth, labelHeight int) (x, y SclVal) {
	l := ws.OverallLoc()
	switch p {
	case AboveThenLeft:
		x = l.StartX
		y = l.StartY.MinusStatic(labelHeight)
	case AboveThenRight:
		x = l.EndX
		y = l.StartY.MinusStatic(labelHeight)
	case BelowThenLeft:
		x = l.StartX
		y = l.EndY.PlusStatic(1)
	case BelowThenRight:
		x = l.EndX
		y = l.EndY.PlusStatic(1)
	case LeftThenTop:
		x = l.StartX.MinusStatic(labelWidth)
		y = l.StartY
	case LeftThenBottom:
		x = l.StartX.MinusStatic(labelWidth)
		y = l.EndY
	case RightThenTop:
		x = l.EndX.PlusStatic(1)
		y = l.StartY
	case RightThenBottom:
		x = l.EndX.PlusStatic(1)
		y = l.EndY
	default:
		panic("unknown label position")
	}
	return x, y
}

// adds the label at the position provided
func (ws *Widgets) AddLabel(l *Label, p LabelPosition) {
	x, y := ws.LabelPositionToXY(p, l.GetWidth(), l.GetHeight())
	l.At(x, y)
	*ws = append(*ws, l.ToWidgets()...)
}

func (ws Widgets) WithLabel(label string) Widgets {
	// label to the right if a width of 1 otherwise label the top left
	if ws.OverallLoc().Width(ws.GetParentCtx()) == 1 {
		return ws.WithRightTopLabel(label)
	}
	return ws.WithAboveLeftLabel(label)
}

func (ws Widgets) GetParentCtx() yh.Context {
	if len(ws) == 0 {
		return yh.Context{}
	}
	return ws[0].GetParentCtx()
}

func (ws Widgets) WithAboveLeftLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label), AboveThenLeft)
	return ws
}

func (ws Widgets) WithAboveRightLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithRightJustification(), AboveThenRight)
	return ws
}

func (ws Widgets) WithBelowLeftLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label), BelowThenLeft)
	return ws
}

func (ws Widgets) WithBelowRightLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithRightJustification(), BelowThenRight)
	return ws
}

func (ws Widgets) WithLeftTopLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithRightJustification(), LeftThenTop)
	return ws
}

func (ws Widgets) WithLeftBottomLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithRightJustification(), LeftThenBottom)
	return ws
}

func (ws Widgets) WithRightTopLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithLeftJustification(), RightThenTop)
	return ws
}

func (ws Widgets) WithRightBottomLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithLeftJustification(), RightThenBottom)
	return ws
}

// ---------------
// vertical labels

func (ws Widgets) WithLeftTopVerticalLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithRotatedText().WithDownJustification(), LeftThenTop)
	return ws
}

func (ws Widgets) WithLeftBottomVerticalLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithRotatedText().WithUpJustification(), LeftThenBottom)
	return ws
}

func (ws Widgets) WithRightTopVerticalLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithRotatedText().WithDownJustification(), RightThenTop)
	return ws
}

func (ws Widgets) WithRightBottomVerticalLabel(label string) Widgets {
	ws.AddLabel(NewLabel(ws.GetParentCtx(), label).WithRotatedText().WithUpJustification(), RightThenBottom)
	return ws
}

//------------------------------------------------

type WidgetBase struct {
	pCtx yh.Context // last parent context

	Selectedness Selectability // lol

	// function called when a mouse event is successfully received
	//ReceiveMouseEventFn func(ev *tcell.EventMouse)

	// the receivableEvents when this widget is active
	//ReceivableEvs []yh.PriorityEv
	ReceivableEvs []yh.PrioritizableEv

	// size of the widget (NOT the content space)
	Width, Height SclVal
	LocX, LocY    SclVal // top left location of the widget relative to parent

	Content         yh.DrawChs2D // [Y][X]DrawCh
	ContentMaxWidth int          // max width of the content
	ContentXOffset  int
	ContentYOffset  int
	Styles          WBStyles
}

// func NewWidgetBase(pCtx, width, height int, sty WBStyles, receivableEvs []yh.PriorityEv) *WidgetBase {
func NewWidgetBase(pCtx yh.Context, width, height SclVal, sty WBStyles, receivableEvs []yh.PrioritizableEv) *WidgetBase {
	return &WidgetBase{
		pCtx:         pCtx,
		Selectedness: Ready,
		//ReceiveMouseEventFn: nil,
		ReceivableEvs:  receivableEvs,
		Width:          width,
		Height:         height,
		LocX:           NewStatic(0),
		LocY:           NewStatic(0),
		Content:        [][]yh.DrawCh{},
		ContentYOffset: 0,
		ContentXOffset: 0,
		Styles:         sty,
	}
}

type WBStyles struct {
	SelectedStyle     tcell.Style
	ReadyStyle        tcell.Style
	UnselectableStyle tcell.Style
}

var (
	// default styles
	DefaultWBStyles = WBStyles{
		SelectedStyle:     tcell.StyleDefault,
		ReadyStyle:        tcell.StyleDefault,
		UnselectableStyle: tcell.StyleDefault,
	}
)

func NewWBStyles(selectedStyle, readyStyle, unselectableStyle tcell.Style) WBStyles {
	return WBStyles{
		SelectedStyle:     selectedStyle,
		ReadyStyle:        readyStyle,
		UnselectableStyle: unselectableStyle,
	}
}

//-------------------------

func (wb *WidgetBase) GetWidth() int {
	return wb.Width.GetVal(wb.pCtx.GetWidth())
}

func (wb *WidgetBase) GetHeight() int {
	return wb.Height.GetVal(wb.pCtx.GetHeight())
}

func (wb *WidgetBase) At(locX, locY SclVal) {
	wb.LocX, wb.LocY = locX, locY
}

func (wb *WidgetBase) GetParentCtx() yh.Context {
	return wb.pCtx
}

func (wb *WidgetBase) SetParentCtx(pCtx yh.Context) {
	wb.pCtx = pCtx
}

func (wb *WidgetBase) ResizeEvent(pCtx yh.Context) {
	wb.SetParentCtx(pCtx)
}

func (wb *WidgetBase) GetLocation() yh.Location {
	w, h := wb.GetWidth(), wb.GetHeight()
	x1, y1 := wb.LocX.GetVal(wb.pCtx.GetWidth()), wb.LocY.GetVal(wb.pCtx.GetHeight())
	x2, y2 := x1+w-1, y1+h-1
	return yh.NewLocation(x1, x2, y1, y2, WidgetZIndex)
}

func (wb *WidgetBase) GetSclLocation() SclLocation {
	x1, y1 := wb.LocX, wb.LocY
	x2, y2 := x1.Plus(wb.Width).MinusStatic(1), y1.Plus(wb.Height).MinusStatic(1)
	return NewSclLocation(x1, x2, y1, y2)
}

func (wb *WidgetBase) ScrollUp() {
	wb.SetContentYOffset(wb.ContentYOffset - 1)
}

func (wb *WidgetBase) ScrollDown() {
	wb.SetContentYOffset(wb.ContentYOffset + 1)
}

func (wb *WidgetBase) ScrollLeft() {
	wb.SetContentXOffset(wb.ContentXOffset - 1)
}

func (wb *WidgetBase) ScrollRight() {
	wb.SetContentXOffset(wb.ContentXOffset + 1)
}

func (wb *WidgetBase) SetContentXOffset(x int) {
	if x < 0 {
		x = 0
	} else if x > wb.ContentWidth()-wb.GetWidth() {
		x = wb.ContentWidth() - wb.GetWidth()
	}
	wb.ContentXOffset = x
}

func (wb *WidgetBase) SetContentYOffset(y int) {
	if y < 0 {
		y = 0
	} else if y > wb.ContentHeight()-wb.GetHeight() {
		y = wb.ContentHeight() - wb.GetHeight()
	}
	wb.ContentYOffset = y
}

func (wb *WidgetBase) ContentWidth() int {
	return wb.ContentMaxWidth
}

func (wb *WidgetBase) ContentHeight() int {
	return len(wb.Content)
}

// sets content from string
func (wb *WidgetBase) SetContentFromString(s string) {

	lines := strings.Split(string(s), "\n")
	rs := [][]rune{}
	sty := wb.GetCurrentStyle()

	width, height := wb.GetWidth(), wb.GetHeight() // width and height of the content area
	for _, line := range lines {
		if width < len(line) {
			width = len(line)
		}
		rs = append(rs, []rune(line))
	}
	wb.ContentMaxWidth = width
	if height < len(rs) {
		height = len(rs)
	}

	// initialize the content with blank characters
	// of the height and width of the widget
	wb.Content = [][]yh.DrawCh{}
	for y := 0; y < height; y++ {
		wb.Content = append(wb.Content, []yh.DrawCh{})
		for x := 0; x < width; x++ {
			wb.Content[y] = append(wb.Content[y], yh.NewDrawCh(' ', false, sty))
		}
	}

	// now fill in with actual content
	for y := 0; y < height; y++ {
		for x := 0; x < width; x++ {
			var r rune
			if y < len(rs) && x < len(rs[y]) {
				r = rs[y][x]
			} else {
				continue
			}
			dch := yh.NewDrawCh(r, false, sty)
			wb.Content[y][x] = dch
		}
	}
}

func (wb *WidgetBase) SetContent(content yh.DrawChs2D) []yh.PrioritizableEv {
	wb.Content = content
	wb.ContentMaxWidth = content.Width()
	return nil
}

// CorrectOffsetsToViewPosition changes the content offsets within the
// WidgetBase in order to bring the given view position into view.
func (wb *WidgetBase) CorrectOffsetsToViewPosition(x, y int) {

	// set y offset if cursor out of bounds
	if y >= wb.ContentYOffset+wb.GetHeight() {
		yh.Debug("cor1, %d, %d, %d", y, wb.ContentYOffset, wb.GetHeight())
		wb.SetContentYOffset(y - wb.GetHeight() + 1)
	} else if y < wb.ContentYOffset {
		yh.Debug("cor2")
		wb.SetContentYOffset(y)
	}

	// correct the offset if the offset is now showing lines that don't exist in
	// the content
	if wb.ContentYOffset+wb.GetHeight() > wb.ContentHeight()-1 {
		yh.Debug("cor3")
		wb.SetContentYOffset(wb.ContentHeight() - 1)
	}

	// set x offset if cursor out of bounds
	if x >= wb.ContentXOffset+wb.GetWidth() {
		yh.Debug("cor4")
		wb.SetContentXOffset(x - wb.GetWidth() + 1)
	} else if x < wb.ContentXOffset {
		yh.Debug("cor5")
		wb.SetContentXOffset(x)
	}

	// correct the offset if the offset is now showing characters to the right
	// which don't exist in the content.
	if wb.ContentXOffset+wb.GetWidth() > wb.ContentWidth()-1 {
		yh.Debug("cor6")
		wb.SetContentXOffset(wb.ContentWidth() - 1)
	}
}

// default implementation of Receivable, only receive when widget is active
func (wb *WidgetBase) Receivable() []yh.PrioritizableEv {
	if wb.Selectedness == Selected {
		//return yh.ToPrioritizableEvs(wb.ReceivableEvs)
		return wb.ReceivableEvs
	}
	return []yh.PrioritizableEv{}
}

func (wb *WidgetBase) GetSelectability() Selectability {
	return wb.Selectedness
}

func (wb *WidgetBase) SetSelectability(s Selectability) yh.EventResponse {
	if wb.Selectedness == s {
		return yh.NewEventResponse()
	}

	var ic yh.InputabilityChanges
	switch s {
	case Selected:
		pes := yh.NewPriorityEvs(yh.Focused, wb.ReceivableEvs)
		ic.AddEvs(pes)
	case Ready, Unselectable:
		if wb.Selectedness == Selected {
			ic.RemoveEvs(wb.ReceivableEvs)
		}
	}
	wb.Selectedness = s
	return yh.NewEventResponse().WithInputabilityChanges(ic)
}

func (wb *WidgetBase) Disable() yh.EventResponse {
	return wb.SetSelectability(Unselectable)
}

func (wb *WidgetBase) Enable() yh.EventResponse {
	return wb.SetSelectability(Ready)
}

func (wb *WidgetBase) GetCurrentStyle() tcell.Style {
	switch wb.Selectedness {
	case Selected:
		return wb.Styles.SelectedStyle
	case Ready:
		return wb.Styles.ReadyStyle
	case Unselectable:
		return wb.Styles.UnselectableStyle
	}
	panic("unreachable style")
}

func (wb *WidgetBase) Drawing() (chs []yh.DrawChPos) {
	sty := wb.GetCurrentStyle()
	h, w := wb.GetHeight(), wb.GetWidth()

	for y := wb.ContentYOffset; y < wb.ContentYOffset+h; y++ {
		for x := wb.ContentXOffset; x < wb.ContentXOffset+w; x++ {
			var ch yh.DrawCh
			if y < len(wb.Content) && x < len(wb.Content[y]) {
				ch = wb.Content[y][x]
			} else {
				ch = yh.NewDrawCh(' ', false, sty)
			}
			chs = append(chs, yh.NewDrawChPos(ch, x-wb.ContentXOffset, y-wb.ContentYOffset))
		}
	}
	return chs
}

func (wb *WidgetBase) SetStyles(styles WBStyles) {
	wb.Styles = styles
}

func (wb *WidgetBase) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	//if wb.ReceiveMouseEventFn != nil {
	//    wb.ReceiveMouseEventFn(ev)
	//}
	return false, yh.NewEventResponse()
}
*/
