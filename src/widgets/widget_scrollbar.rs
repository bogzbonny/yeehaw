/*
package widgets

import (
	"math"

	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

// TODO Better handling of dragging if the mouse leaves the scrollbar
// location but is still dragging

// up is backwards, down is forwards
type VerticalScrollbar struct {
	Scrollbar
}

// left is backwards, right is forwards
type HorizontalScrollbar struct {
	Scrollbar
}

// Scrollbar Positions
type ScrollbarOption string

const (
	NoScrollbar     ScrollbarOption = ""
	LeftScrollbar   ScrollbarOption = "left"
	RightScrollbar  ScrollbarOption = "right"
	TopScrollbar    ScrollbarOption = "top"
	BottomScrollbar ScrollbarOption = "bottom"
)

var VScrollbarEvCombos = []yh.PrioritizableEv{yh.UpEKC, yh.DownEKC, yh.SpaceEKC}
var HScrollbarEvCombos = []yh.PrioritizableEv{yh.LeftEKC, yh.RightEKC}

var ScrollbarStyle = WBStyles{
	SelectedStyle:     tcell.StyleDefault.Background(tcell.ColorLightYellow).Foreground(tcell.ColorBlack),
	ReadyStyle:        tcell.StyleDefault.Background(tcell.ColorDarkSlateGrey).Foreground(tcell.ColorWhite),
	UnselectableStyle: tcell.StyleDefault.Background(tcell.ColorDarkSlateGrey).Foreground(tcell.ColorWhite),
}

func NewVerticalScrollbar(pCtx yh.Context, scrollableViewHeight SclVal, scrollableHeight int) *VerticalScrollbar {
	wb := NewWidgetBase(pCtx, NewStatic(1), scrollableViewHeight, ScrollbarStyle, VScrollbarEvCombos)
	return &VerticalScrollbar{
		Scrollbar{
			WidgetBase:          wb,
			ScrollableDomainChs: scrollableHeight,
			ScrollableViewChs:   scrollableViewHeight,
			ScrollbarLengthChs:  scrollableViewHeight,
			ScrollablePosition:  0,
			HasArrows:           true,
			BackwardsArrow:      '▲',
			ForwardsArrow:       '▼',
			EmptyBlock:          ' ',
			FullBlock:           '█',
			ForwardsHalfBlock:   '▄',
			BackwardsHalfBlock:  '▀',
			Unnessecary:         '░',
			PositionChangedHook: nil,
			CurrentlyDragging:   false,
			StartDragPosition:   0,
			JumpScrollPercent:   10,
			JumpScrollMinAmount: 3,
		},
	}
}

func NewHorizontalScrollbar(pCtx yh.Context, scrollableViewWidth SclVal, scrollableWidth int) *HorizontalScrollbar {
	wb := NewWidgetBase(pCtx, scrollableViewWidth, NewStatic(1), ScrollbarStyle, HScrollbarEvCombos)
	return &HorizontalScrollbar{
		Scrollbar{
			WidgetBase:          wb,
			ScrollableDomainChs: scrollableWidth,
			ScrollableViewChs:   scrollableViewWidth,
			ScrollbarLengthChs:  scrollableViewWidth,
			ScrollablePosition:  0,
			HasArrows:           true,
			BackwardsArrow:      '◀',
			ForwardsArrow:       '▶',
			EmptyBlock:          ' ',
			FullBlock:           '█',
			ForwardsHalfBlock:   '▐',
			BackwardsHalfBlock:  '▌',
			Unnessecary:         '░',
			PositionChangedHook: nil,
			CurrentlyDragging:   false,
			StartDragPosition:   0,
			JumpScrollPercent:   10,
			JumpScrollMinAmount: 3,
		},
	}
}

func (sb *VerticalScrollbar) SetHeight(viewHeight, scrollbarLength SclVal, scrollableHeight int) {
	sb.ScrollableViewChs = viewHeight
	sb.WidgetBase.Height = scrollbarLength
	sb.ScrollbarLengthChs = scrollbarLength
	sb.ScrollableDomainChs = scrollableHeight
}

func (sb *HorizontalScrollbar) SetWidth(viewWidth, scrollbarLength SclVal, scrollableWidth int) {
	sb.ScrollableViewChs = viewWidth
	sb.WidgetBase.Width = scrollbarLength
	sb.ScrollbarLengthChs = scrollbarLength
	sb.ScrollableDomainChs = scrollableWidth
}

func (sb *VerticalScrollbar) WithStyle(sty WBStyles) *VerticalScrollbar {
	sb.WidgetBase.Styles = sty
	return sb
}

func (sb *HorizontalScrollbar) WithStyle(sty WBStyles) *HorizontalScrollbar {
	sb.WidgetBase.Styles = sty
	return sb
}

func (sb *VerticalScrollbar) WithoutArrows() *VerticalScrollbar {
	sb.HasArrows = false
	return sb
}

func (sb *HorizontalScrollbar) WithoutArrows() *HorizontalScrollbar {
	sb.HasArrows = false
	return sb
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
type Scrollbar struct {
	*WidgetBase

	// The ScrollableDomainChs is the scrollable dimension in true characters.
	// It is AFFECTED by the scrollbar and NOT the literal area of the scrollbar
	// itself.
	ScrollableDomainChs int // how large the area is that can be scrolled

	// how much of the scrollable area is visible in true chars.
	ScrollableViewChs SclVal

	// Length of the actual scrollbar (and arrows) in true characters.
	// Typically this is the same as ScrollableViewChs, however some situations
	// call for a different size scrollbar than the scrollable view, such as the
	// dropdown menu with a scrollbar below the dropdown-arrow.
	ScrollbarLengthChs SclVal

	// how far down the area is scrolled from the top in true chars.
	// The ScrollablePosition will be the first line of the area scrolled to.
	ScrollablePosition int

	HasArrows bool // if the scrollbar has arrows

	BackwardsArrow     rune
	ForwardsArrow      rune
	EmptyBlock         rune
	FullBlock          rune
	ForwardsHalfBlock  rune
	BackwardsHalfBlock rune
	Unnessecary        rune // for when the scrollbar ought not to exist

	// function the scrollbar will call everytime there is a position change
	PositionChangedHook func(newPosition int)

	// is the scrollbar currently being dragged?
	CurrentlyDragging bool
	StartDragPosition int // in true characters

	// The percent (0-100) of the total scrollable domain
	// to scroll when a click in the scrollbar whitespace is made.
	JumpScrollPercent int

	// minimum amount to scroll during a jump scroll
	JumpScrollMinAmount int
}

// if the Scrollbar currently cannot be used due to insufficient domain.
func (sb *Scrollbar) IsCurrentlyUnnecessary(pSize int) bool {
	return sb.ScrollableDomainChs <= sb.ScrollableViewChs.GetVal(pSize)
}

func (sb *Scrollbar) JumpScrollAmount() int {
	js := sb.ScrollableDomainChs * sb.JumpScrollPercent / 100
	if js < sb.JumpScrollMinAmount {
		js = sb.JumpScrollMinAmount
	}
	return js
}

func (sb *Scrollbar) JumpScrollBackwards(pSize int) {
	sb.ScrollToPosition(pSize, sb.ScrollablePosition-sb.JumpScrollAmount())
}

func (sb *Scrollbar) JumpScrollForwards(pSize int) {
	sb.ScrollToPosition(pSize, sb.ScrollablePosition+sb.JumpScrollAmount())
}

func (sb *Scrollbar) CanScrollBackwards() bool {
	return sb.ScrollablePosition > 0
}

func (sb *Scrollbar) ScrollBackwards() {
	if !sb.CanScrollBackwards() {
		return
	}
	sb.ScrollablePosition--
	if sb.PositionChangedHook != nil {
		sb.PositionChangedHook(sb.ScrollablePosition)
	}
}

func (sb *Scrollbar) CanScrollForwards(pSize int) bool {
	return sb.ScrollablePosition < sb.ScrollableDomainChs-sb.ScrollableViewChs.GetVal(pSize)
}

func (sb *Scrollbar) ScrollForwards(pSize int) {
	if !sb.CanScrollForwards(pSize) {
		return
	}
	sb.ScrollablePosition++
	if sb.PositionChangedHook != nil {
		sb.PositionChangedHook(sb.ScrollablePosition)
	}
}

// scroll to the position within the scrollable domain.
func (sb *Scrollbar) ScrollToPosition(pSize, position int) {
	if position < 0 {
		position = 0
	}
	if position > sb.ScrollableDomainChs-sb.ScrollableViewChs.GetVal(pSize) {
		position = sb.ScrollableDomainChs - sb.ScrollableViewChs.GetVal(pSize)
	}
	sb.ScrollablePosition = position
	if sb.PositionChangedHook != nil {
		sb.PositionChangedHook(sb.ScrollablePosition)
	}
}

// the scrollbar domain is the total space which the scroll bar may occupy (both
// the bar and the empty space above and below it) measured in half-increments.
// Each half-increment represents half a character, as the scrollbar may use
// half characters to represent its position.
func (sb *Scrollbar) ScrollBarDomainInHalfIncrements(pSize int) int {
	// minus 2 for the backwards and forwards arrows
	arrows := 2
	if !sb.HasArrows {
		arrows = 0
	}
	// times 2 for half characters
	//return 2 * (sb.ScrollableViewChs - arrows)
	out := 2 * (sb.ScrollbarLengthChs.GetVal(pSize) - arrows)
	if out < 0 {
		return 0
	}
	return out
}

func (sb *Scrollbar) ScrollBarSizeInHalfIncrements(pSize int) int {
	domainIncr := sb.ScrollBarDomainInHalfIncrements(pSize)
	percentViewable := float64(sb.ScrollableViewChs.GetVal(pSize)) / float64(sb.ScrollableDomainChs)
	// scrollbar size in half increments
	scrollbarIncr := int(math.Round(
		percentViewable * float64(domainIncr),
	))
	if scrollbarIncr < 1 { // minimum size of 1 half-increment
		scrollbarIncr = 1
	}
	if scrollbarIncr > domainIncr { // safeguard
		scrollbarIncr = domainIncr
	}
	return scrollbarIncr
}

// the number of true view characters per full scrollbar character (aka 2
// half-increments)
func (sb *Scrollbar) TrueChsPerScrollbarCharacter(pSize int) int {
	scrollbarIncr := sb.ScrollBarSizeInHalfIncrements(pSize)
	return int(float64(sb.ScrollbarLengthChs.GetVal(pSize)) / (float64(scrollbarIncr) / 2))
	//return sb.ScrollableDomainChs / (scrollbarIncr / 2)
}

func (sb *Scrollbar) SetSelectability(s Selectability) yh.EventResponse {
	sb.CurrentlyDragging = false
	return sb.WidgetBase.SetSelectability(s)
}

// Get an array of half-increments of the scrollbar domain area
func (sb *Scrollbar) ScrollBarDomainArrayOfHalfIncrements(pSize int) (incrementIsFilled []bool) {
	domainIncr := sb.ScrollBarDomainInHalfIncrements(pSize)
	scrollbarIncr := sb.ScrollBarSizeInHalfIncrements(pSize)

	// total increments within the scrollbar domain for space above and below the bar
	totalSpacerIncr := domainIncr - scrollbarIncr

	trueChsAbove := sb.ScrollablePosition
	incrAbove := int(math.Round(
		float64(trueChsAbove) /
			float64(sb.ScrollableDomainChs-sb.ScrollableViewChs.GetVal(pSize)) * float64(totalSpacerIncr),
	))

	// correct incase the rounding gives an extra increment
	if incrAbove+scrollbarIncr > domainIncr {
		incrAbove = domainIncr - scrollbarIncr
	}

	// -----------------------------------------------
	// determine whether each increment is a filled.
	if domainIncr <= 0 {
		//yh.Debug("----------------------------\n")
		//yh.Debug("incrAbove: %v, scrollbarIncr: %v, domainIncr: %v\n", incrAbove, scrollbarIncr, domainIncr)
		//yh.Debug("totalSpacerIncr: %v, trueChsAbove: %v, sb.ScrollableDomainChs: %v\n", totalSpacerIncr, trueChsAbove, sb.ScrollableDomainChs)
		//yh.Debug("pSize: %v, sb.ScrollableViewChs.GetVal(pSize)): %v\n", pSize, sb.ScrollableViewChs.GetVal(pSize))
		return []bool{}
	}
	incrFilled := make([]bool, domainIncr)
	for i := incrAbove; i < incrAbove+scrollbarIncr; i++ {
		incrFilled[i] = true
	}
	return incrFilled
}

func lastIncrFilled(incrFilled []bool) int {
	for i := len(incrFilled) - 1; i >= 0; i-- {
		if incrFilled[i] {
			return i
		}
	}
	return -1
}

func firstIncrFilled(incrFilled []bool) int {
	for i := 0; i < len(incrFilled); i++ {
		if incrFilled[i] {
			return i
		}
	}
	return -1
}

// used for mouse dragging the scrollbar. What the incrementIsFilled should look
// like if it dragged down by one rune (aka 2 half increments)
func (sb *Scrollbar) DragForwardsBy1Ch(pSize int) {
	yh.Debug("dragging forwards by 1 ch\n")
	startIncrs := sb.ScrollBarDomainArrayOfHalfIncrements(pSize)
	yh.Debug("startIncrs: %v\n", startIncrs)
	lastFilled := lastIncrFilled(startIncrs)
	yh.Debug("lastFilled: %v\n", lastFilled)
	if lastFilled == -1 {
		return
	}
	goalLastFilled := lastFilled + 2
	if goalLastFilled >= len(startIncrs) {
		goalLastFilled = len(startIncrs) - 1
	}
	yh.Debug("goalLastFilled: %v\n", goalLastFilled)

	for {
		// safegaurd against infinite loop
		if !sb.CanScrollForwards(pSize) {
			break
		}
		sb.ScrollForwards(pSize)
		currentIncr := sb.ScrollBarDomainArrayOfHalfIncrements(pSize)
		currLastFilled := lastIncrFilled(currentIncr)
		if currLastFilled == goalLastFilled {
			break
		}
	}
}

// Same as DragForwardsBy1Ch but in the backwards direction
func (sb *Scrollbar) DragBackwardsBy1Ch(pSize int) {
	startIncrs := sb.ScrollBarDomainArrayOfHalfIncrements(pSize)
	firstFilled := firstIncrFilled(startIncrs)
	if firstFilled == -1 {
		return
	}
	goalFirstFilled := firstFilled - 2
	if goalFirstFilled < 0 {
		goalFirstFilled = 0
	}

	for {
		// safegaurd against infinite loop
		if !sb.CanScrollBackwards() {
			break
		}
		sb.ScrollBackwards()
		currentIncr := sb.ScrollBarDomainArrayOfHalfIncrements(pSize)
		currFirstFilled := firstIncrFilled(currentIncr)
		if currFirstFilled == goalFirstFilled {
			break
		}
	}
}

func (sb *Scrollbar) ScrollBarDomainArrayOfRunes(pSize int) []rune {
	incrFilled := sb.ScrollBarDomainArrayOfHalfIncrements(pSize)
	rs := []rune{}
	// determine the characters based on the filled increments
	for i := range incrFilled {
		if i%2 == 1 {
			switch {
			case incrFilled[i-1] && incrFilled[i]:
				rs = append(rs, sb.FullBlock)
			case incrFilled[i-1] && !incrFilled[i]:
				rs = append(rs, sb.BackwardsHalfBlock)
			case !incrFilled[i-1] && incrFilled[i]:
				rs = append(rs, sb.ForwardsHalfBlock)
			case !incrFilled[i-1] && !incrFilled[i]:
				rs = append(rs, sb.EmptyBlock)
			}
		}
	}
	return rs

}

func (sb *Scrollbar) DrawingRunes(pSize int) (chs []rune) {
	if sb.IsCurrentlyUnnecessary(pSize) {
		for i := 0; i < sb.ScrollbarLengthChs.GetVal(pSize); i++ {
			chs = append(chs, sb.Unnessecary)
		}
	} else {
		if sb.HasArrows {
			chs = []rune{sb.BackwardsArrow}
		}
		chs = append(chs, sb.ScrollBarDomainArrayOfRunes(pSize)...)
		if sb.HasArrows {
			chs = append(chs, sb.ForwardsArrow)
		}
	}
	return chs
}

// Call this when the position has been changed external to the scrollbar
// newViewOffset is the new position of the view in full characters
// newViewDomain is the number of full characters of the full scrollable domain
func (sb *Scrollbar) ExternalChange(pSize, newViewOffset, newDomainChs int) {
	sb.ScrollablePosition = newViewOffset
	sb.ScrollableDomainChs = newDomainChs
	sb.UpdateSelectibility(pSize)
}

// process for the selectibility of the scrollbar
func (sb *Scrollbar) UpdateSelectibility(pSize int) {
	if sb.IsCurrentlyUnnecessary(pSize) {
		sb.CurrentlyDragging = false
		_ = sb.SetSelectability(Unselectable)
	} else {
		_ = sb.SetSelectability(Ready)
	}
}

type sbRelPosition byte

const (
	none   sbRelPosition = 0x00
	before sbRelPosition = 0x01
	on     sbRelPosition = 0x02
	after  sbRelPosition = 0x03
)

// is the provided position before, on, or after the scrollbar?
func (sb *Scrollbar) PositionRelativeToScrollbar(pSize, pos int) sbRelPosition {
	lastScrollbarPos := sb.ScrollbarLengthChs.GetVal(pSize) - 1 // last pos the actual scrollbar may be in
	if sb.HasArrows {
		if pos == 0 || pos == sb.ScrollbarLengthChs.GetVal(pSize)-1 {
			return none
		}
		pos -= 1                                                   // account for the backwards arrow
		lastScrollbarPos = sb.ScrollbarLengthChs.GetVal(pSize) - 3 // account for the both arrow
	}

	rs := sb.ScrollBarDomainArrayOfRunes(pSize)
	if pos >= len(rs) {
		return after
	}

	firstFull, lastFull := -1, -1
	backwardsHalfPos, forwardsHalfPos := -1, -1
	for i, r := range rs {
		if r == sb.FullBlock {
			if firstFull == -1 {
				firstFull = i
			}
			lastFull = i
		}
		if r == sb.BackwardsHalfBlock {
			backwardsHalfPos = i
		}
		if r == sb.ForwardsHalfBlock {
			forwardsHalfPos = i
		}
	}

	// edge cases for when very near the end
	if pos == 0 && forwardsHalfPos == 0 {
		return before
	}
	if pos == lastScrollbarPos && backwardsHalfPos == lastScrollbarPos {
		return after
	}

	if firstFull == -1 {
		switch {
		case backwardsHalfPos == pos || forwardsHalfPos == pos:
			return on
		case pos < forwardsHalfPos || pos < backwardsHalfPos:
			return before
		case pos > forwardsHalfPos || pos > backwardsHalfPos:
			return after
		default:
			return none
		}
	}

	switch {
	case pos < firstFull:
		return before
	case pos > lastFull:
		return after
	default:
		return on
	}
}

// -------------------------------------------------------------------
// Specific implementations for the vertical and horizontal scrollbars

func (vsb *VerticalScrollbar) ExternalChange(newViewOffset, newDomainChs int) {
	vsb.ScrollablePosition = newViewOffset
	vsb.ScrollableDomainChs = newDomainChs
	ctx := vsb.WidgetBase.GetParentCtx()
	vsb.UpdateSelectibility(ctx.GetHeight())
}

func (hsb *HorizontalScrollbar) ExternalChange(newViewOffset, newDomainChs int) {
	hsb.ScrollablePosition = newViewOffset
	hsb.ScrollableDomainChs = newDomainChs
	ctx := hsb.WidgetBase.GetParentCtx()
	hsb.UpdateSelectibility(ctx.GetWidth())
}

func (vsb *VerticalScrollbar) ResizeEvent(pCtx yh.Context) {
	vsb.UpdateSelectibility(pCtx.GetHeight())
	vsb.WidgetBase.ResizeEvent(pCtx)
}

func (hsb *HorizontalScrollbar) ResizeEvent(pCtx yh.Context) {
	hsb.UpdateSelectibility(pCtx.GetWidth())
	hsb.WidgetBase.ResizeEvent(pCtx)
}

func (vsb *VerticalScrollbar) Drawing() []yh.DrawChPos {
	ctx := vsb.WidgetBase.GetParentCtx()
	chs := vsb.DrawingRunes(ctx.GetHeight())

	// compile the runes into a horizontal string
	str := ""
	for i, ch := range chs {
		str += string(ch)
		if i != len(chs)-1 {
			str += "\n"
		}
	}

	vsb.SetContentFromString(str)
	return vsb.WidgetBase.Drawing()
}
func (hsb *HorizontalScrollbar) Drawing() []yh.DrawChPos {
	ctx := hsb.WidgetBase.GetParentCtx()
	str := string(hsb.DrawingRunes(ctx.GetWidth()))
	hsb.SetContentFromString(str)
	return hsb.WidgetBase.Drawing()
}

func (vsb *VerticalScrollbar) ToWidgets() Widgets {
	return Widgets{vsb}
}

func (hsb *HorizontalScrollbar) ToWidgets() Widgets {
	return Widgets{hsb}
}

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
