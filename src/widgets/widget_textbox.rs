/*
package widgets

import (
	"fmt"
	"math"

	"github.com/atotto/clipboard"
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
	els "keybase.io/nwmod/nwmod/yeehaw/elements"
)

// TODO better multiline cursor movement
// retain greater cursor position between lines, ex:
//    123456789<cursor, starting position>
//    1234<cursor after moving down>
//    123456789<cursor, after moving down again>

type TextBox struct {
	*WidgetBase
	text         []rune
	chCursor     bool // whether or not this tb has a ch cursor
	editable     bool // whether or not this tb can be edited
	wordwrap     bool // whether or not to wrap text
	lineNumbered bool // whether or not there are lefthand line numbers
	cursorPos    int  // cursor absolute position in the text
	cursorStyle  tcell.Style

	visualMode         bool // whether or not the cursor is visual selecting
	mouseDragging      bool // if the mouse is currently dragging
	visualModeStartPos int  // the start position of the visual select

	TextChangedHook func(newText string) yh.EventResponse

	// when this hook is non-nil each characters style will be determined
	// via this hook.
	PositionStyleHook func(absPos int, existingSty tcell.Style) tcell.Style

	// This Hook is called each time the cursor moves
	CursorChangedHook func(absPos int)

	XChangedHook  func(newXPosition, newWidth int)
	YChangedHook  func(newYPosition, newHeight int)
	YChangedHook2 func(newYPosition, newHeight int, resp *yh.EventResponse) // a second hook for the line numbers textbox
	// TODO make an array of hooks

	XScrollbar  ScrollbarOption // either "", "top", or "bottom"
	YScrollbar  ScrollbarOption // either "", "left", or "right"
	CornerDecor yh.DrawCh       // for when there are two scrollbars

	RightClickMenu *els.RightClickMenuTemplate
}

var _ Widget = &TextBox{}

var TextboxEditableEvCombos = []yh.PrioritizableEv{yh.RunesEKC, yh.EnterEKC, yh.ShiftEnterEKC,
	yh.BackspaceEKC, yh.Backspace2EKC, //yh.MetaCLowerEKC, yh.MetaVLowerEKC,
	yh.LeftEKC, yh.RightEKC, yh.UpEKC, yh.DownEKC,
	yh.ShiftLeftEKC, yh.ShiftRightEKC, yh.ShiftUpEKC, yh.ShiftDownEKC}

// non-editable textboxes can still scroll with the arrow keys
var TextboxNonEditableEvCombos = []yh.PrioritizableEv{
	yh.LeftEKC, yh.RightEKC, yh.UpEKC, yh.DownEKC,
	yh.JLowerEKC, yh.KLowerEKC, yh.HLowerEKC, yh.LLowerEKC,
}

var TextboxStyle = WBStyles{
	SelectedStyle: tcell.StyleDefault.
		Background(tcell.ColorWhite).Foreground(tcell.ColorBlack),
	ReadyStyle: tcell.StyleDefault.
		Background(tcell.ColorLightGray).Foreground(tcell.ColorBlack),
	UnselectableStyle: tcell.StyleDefault.
		Background(tcell.ColorLightSlateGrey).Foreground(tcell.ColorBlack),
}

var DefaultCursorStyle = tcell.StyleDefault.Background(tcell.ColorBlue)
var DefaultRightClickMenuStyle = tcell.StyleDefault.Background(tcell.ColorLime)

func NewTextBox(pCtx yh.Context, text string) *TextBox {

	width, height := TextSize(text)
	wb := NewWidgetBase(pCtx, NewStatic(width), NewStatic(height), TextboxStyle, TextboxEditableEvCombos)

	tb := &TextBox{
		WidgetBase:         wb,
		text:               []rune(text),
		wordwrap:           true,
		chCursor:           true,
		editable:           true,
		cursorPos:          0,
		cursorStyle:        DefaultCursorStyle,
		visualMode:         false,
		mouseDragging:      false,
		visualModeStartPos: 0,
		TextChangedHook:    nil,
		XChangedHook:       nil,
		YChangedHook:       nil,
		XScrollbar:         NoScrollbar,
		YScrollbar:         NoScrollbar,
		CornerDecor:        yh.NewDrawCh('⁙', false, tcell.StyleDefault),
		RightClickMenu:     nil,
	}

	rcm := els.NewRightClickMenuTemplate(DefaultRightClickMenuStyle)
	rcm.SetMenuItems([]*els.MenuItem{
		els.NewMenuItem("Cut", true,
			func(_ yh.Context) yh.EventResponse {
				resp, _ := tb.CutToClipboard()
				return resp
			},
		),
		els.NewMenuItem("Copy", true,
			func(_ yh.Context) yh.EventResponse {
				_ = tb.CopyToClipboard()
				return yh.NewEventResponse()
			},
		),
		els.NewMenuItem("Paste", true,
			func(_ yh.Context) yh.EventResponse {
				resp, _ := tb.PasteFromClipboard()
				return resp
			},
		),
	})

	tb.RightClickMenu = rcm

	_ = tb.Drawing() // to set the base content
	return tb
}

// ---------------------------------------------------------
// Decorators

func (tb *TextBox) WithSize(width, height SclVal) *TextBox {
	tb.Width, tb.Height = width, height
	return tb
}

func (tb *TextBox) At(locX, locY SclVal) *TextBox {
	tb.WidgetBase.At(locX, locY)
	return tb
}

func (tb *TextBox) WithChCursor() *TextBox {
	tb.chCursor = true
	return tb
}

func (tb *TextBox) NoChCursor() *TextBox {
	tb.chCursor = false
	return tb
}

func (tb *TextBox) Editable() *TextBox {
	tb.editable = true
	return tb
}

func (tb *TextBox) NonEditable() *TextBox {
	tb.editable = false
	tb.ReceivableEvs = TextboxNonEditableEvCombos
	return tb
}

func (tb *TextBox) WordWrap() *TextBox {
	tb.wordwrap = true
	return tb
}

func (tb *TextBox) NoWordWrap() *TextBox {
	tb.wordwrap = false
	return tb
}

func (tb *TextBox) LineNumbered() *TextBox {
	tb.lineNumbered = true
	return tb
}

func (tb *TextBox) NoLineNumbered() *TextBox {
	tb.lineNumbered = false
	return tb
}

func (tb *TextBox) WithPositionStyleHook(hook func(absPos int, existingSty tcell.Style) tcell.Style) *TextBox {
	tb.PositionStyleHook = hook
	return tb
}

func (tb *TextBox) SetPositionStyleHook(hook func(absPos int, existingSty tcell.Style) tcell.Style) {
	tb.PositionStyleHook = hook
}

func (tb *TextBox) WithCursorChangedHook(hook func(absPos int)) *TextBox {
	tb.CursorChangedHook = hook
	return tb
}

func (tb *TextBox) SetCursorChangedHook(hook func(absPos int)) {
	tb.CursorChangedHook = hook
}

func (tb *TextBox) WithTextChangedHook(hook func(newText string) yh.EventResponse) *TextBox {
	tb.TextChangedHook = hook
	return tb
}

func (tb *TextBox) WithCursorStyle(style tcell.Style) *TextBox {
	tb.cursorStyle = style
	return tb
}

func (tb *TextBox) WithStyles(styles WBStyles) *TextBox {
	tb.Styles = styles
	return tb
}

func (tb *TextBox) WithLeftScrollbar() *TextBox {
	tb.YScrollbar = LeftScrollbar
	return tb
}

func (tb *TextBox) WithRightScrollbar() *TextBox {
	tb.YScrollbar = RightScrollbar
	return tb
}

func (tb *TextBox) WithTopScrollbar() *TextBox {
	tb.XScrollbar = TopScrollbar
	return tb
}

func (tb *TextBox) WithBottomScrollbar() *TextBox {
	tb.XScrollbar = BottomScrollbar
	return tb
}

func (tb *TextBox) WithCornerDecor(decor yh.DrawCh) *TextBox {
	tb.CornerDecor = decor
	return tb
}

func (tb *TextBox) ToWidgets() Widgets {

	x, y := tb.LocX, tb.LocY

	var out []Widget
	h, w := tb.Height, tb.Width

	out = []Widget{}

	var lnTB *TextBox = nil

	if tb.lineNumbered {
		// determine the width of the line numbers textbox

		// create the line numbers textbox
		lns, lnw := tb.GetLineNumbers()
		lnTB = NewTextBox(tb.GetParentCtx(), lns)
		lnTB.At(x, y)
		lnTB.Width = NewStatic(lnw)
		lnTB.Height = h
		lnTB.NoWordWrap()
		lnTB.NonEditable()
		lnTB.SetSelectability(Unselectable)
		out = append(out, lnTB)

		// reduce the width of the main textbox
		tb.Width = tb.Width.MinusStatic(lnw)
		tb.LocX = tb.LocX.PlusStatic(lnw)

		lastLnw := lnw

		// wire the line numbers textbox to the main textbox
		tb.YChangedHook2 = func(newYPosition, newHeight int, resp *yh.EventResponse) {
			lns, lnw := tb.GetLineNumbers()
			if lnw != lastLnw {
				diffLnw := lnw - lastLnw
				tb.Width = tb.Width.MinusStatic(diffLnw)
				resp.SetRelocation(yh.NewRelocationRequestLeft(diffLnw))
				lastLnw = lnw
			}
			lnTB.SetText(lns)
			lnTB.Width = NewStatic(lnw)
			lnTB.SetContentYOffset(newYPosition)
		}
	}

	out = append(out, tb)

	// add corner decor
	if tb.YScrollbar != NoScrollbar && tb.XScrollbar != NoScrollbar {
		cd := NewLabel(tb.GetParentCtx(), string(tb.CornerDecor.Ch)).WithStyle(tb.CornerDecor.Style)
		var cdX, cdY SclVal
		switch {
		case tb.YScrollbar == LeftScrollbar && tb.XScrollbar == TopScrollbar:
			cdX, cdY = x.MinusStatic(1), y.MinusStatic(1)
		case tb.YScrollbar == LeftScrollbar && tb.XScrollbar == BottomScrollbar:
			cdX, cdY = x.MinusStatic(1), y.Plus(h)
		case tb.YScrollbar == RightScrollbar && tb.XScrollbar == TopScrollbar:
			cdX, cdY = x.Plus(w), y.MinusStatic(1)
		case tb.YScrollbar == RightScrollbar && tb.XScrollbar == BottomScrollbar:
			cdX, cdY = x.Plus(w), y.Plus(h)
		}
		cd.At(cdX, cdY)
		out = append(out, cd)
	}

	if tb.YScrollbar != NoScrollbar {
		var x2 SclVal
		if tb.YScrollbar == LeftScrollbar {
			x2 = x.MinusStatic(1)
		} else { // right scrollbar
			x2 = x.Plus(w)
		}

		vsb := NewVerticalScrollbar(tb.GetParentCtx(), h, tb.ContentHeight())
		vsb.At(x2, y)

		// wire the scrollbar to the text box
		if tb.lineNumbered {
			vsb.PositionChangedHook = func(newYPosition int) {
				tb.SetContentYOffset(newYPosition)
				lnTB.SetContentYOffset(newYPosition)
			}
		} else {
			vsb.PositionChangedHook = tb.SetContentYOffset
		}
		tb.YChangedHook = vsb.ExternalChange

		out = append(out, vsb)
	}

	if tb.XScrollbar != NoScrollbar {
		var y2 SclVal
		if tb.XScrollbar == TopScrollbar {
			y2 = y.MinusStatic(1)
		} else { // bottom scrollbar
			y2 = y.Plus(h)
		}

		hsb := NewHorizontalScrollbar(tb.GetParentCtx(), w, tb.ContentWidth())
		hsb.At(x, y2)

		// wire the scrollbar to the text box
		hsb.PositionChangedHook = tb.SetContentXOffset
		tb.XChangedHook = hsb.ExternalChange

		out = append(out, hsb)
	}

	_ = tb.Drawing() // to set the base content
	return out
}

// ---------------------------------------------------------

func (tb *TextBox) GetCursorPos() int {
	return tb.cursorPos
}

func (tb *TextBox) SetCursorPos(newAbsPos int) {
	tb.cursorPos = newAbsPos
	if tb.CursorChangedHook != nil {
		tb.CursorChangedHook(tb.cursorPos)
	}
}

func (tb *TextBox) IncrCursorPos(posChange int) {
	tb.cursorPos += posChange
	if tb.CursorChangedHook != nil {
		tb.CursorChangedHook(tb.cursorPos)
	}
}

// ---------------------------------------------------------

func (tb *TextBox) SetSelectability(s Selectability) yh.EventResponse {
	if tb.Selectedness == Selected && s != Selected {
		tb.visualMode = false
	}
	return tb.WidgetBase.SetSelectability(s)
}

func (tb *TextBox) Drawing() (chs []yh.DrawChPos) {

	w := tb.GetWrapped()
	wrapped := w.WrappedStr()
	tb.SetContentFromString(wrapped)

	if tb.PositionStyleHook != nil {
		for _, wrCh := range w.chs {
			existingsty := tb.GetCurrentStyle()
			sty := tb.PositionStyleHook(wrCh.absPos, existingsty)
			tb.Content.ChangeStyleAtXY(wrCh.xPos, wrCh.yPos, sty)
		}
	}

	if tb.Selectedness == Selected && tb.chCursor {
		curX, curY := w.CursorXAndY(tb.cursorPos)
		tb.Content[curY][curX].Style = tb.cursorStyle
	}

	if tb.visualMode {
		if tb.visualModeStartPos < tb.cursorPos {
			for i := tb.visualModeStartPos; i <= tb.cursorPos; i++ {
				curX, curY := w.CursorXAndY(i)
				tb.Content[curY][curX].Style = tb.cursorStyle
			}
		}
		if tb.visualModeStartPos > tb.cursorPos {
			for i := tb.cursorPos; i <= tb.visualModeStartPos; i++ {
				curX, curY := w.CursorXAndY(i)
				tb.Content[curY][curX].Style = tb.cursorStyle
			}
		}
	}

	return tb.WidgetBase.Drawing()
}

// NOTE the resp is sent in to potentially modify the offsets from numbers tb
func (tb *TextBox) CorrectOffsets(w wrChs, resp *yh.EventResponse) {
	x, y := w.CursorXAndY(tb.cursorPos)
	yh.Debug("CorrectOffsets: pos: %v, x: %v, y: %v\n", tb.cursorPos, x, y)
	tb.CorrectOffsetsToViewPosition(x, y)

	// call the changed hooks
	if tb.YChangedHook != nil {
		tb.YChangedHook(tb.ContentYOffset, tb.ContentHeight())
	}

	if tb.YChangedHook2 != nil {
		tb.YChangedHook2(tb.ContentYOffset, tb.ContentHeight(), resp)
	}

	if tb.XChangedHook != nil {
		tb.XChangedHook(tb.ContentXOffset, tb.ContentWidth())
	}
}

func (tb *TextBox) VisualSelectedText() string {
	if !tb.visualMode {
		return string(tb.text[tb.cursorPos])
	}
	if tb.visualModeStartPos < tb.cursorPos {
		return string(tb.text[tb.visualModeStartPos : tb.cursorPos+1])
	}
	return string(tb.text[tb.cursorPos : tb.visualModeStartPos+1])
}

func (tb *TextBox) DeleteVisualSelection() yh.EventResponse {
	resp := yh.NewEventResponse()
	if !tb.visualMode {
		return resp
	}

	// delete everything in the visual selection
	rs := tb.text
	if tb.visualModeStartPos < tb.cursorPos {
		rs = append(rs[:tb.visualModeStartPos], rs[tb.cursorPos+1:]...)
		tb.SetCursorPos(tb.visualModeStartPos)
	} else {
		rs = append(rs[:tb.cursorPos], rs[tb.visualModeStartPos+1:]...)
		// (leave the cursor at the start of the visual selection)
	}
	tb.text = rs
	tb.visualMode = false
	w := tb.GetWrapped()
	tb.SetContentFromString(w.WrappedStr()) // See NOTE-1
	tb.CorrectOffsets(w, &resp)
	if tb.TextChangedHook != nil {
		resp = tb.TextChangedHook(string(tb.text))
	}
	return resp
}

func (tb *TextBox) CopyToClipboard() error {
	return clipboard.WriteAll(tb.VisualSelectedText())
}

func (tb *TextBox) CutToClipboard() (yh.EventResponse, error) {
	resp := yh.NewEventResponse()
	err := tb.CopyToClipboard()
	if err != nil {
		return resp, err
	}
	resp = tb.DeleteVisualSelection()
	return resp, nil
}

func (tb *TextBox) PasteFromClipboard() (yh.EventResponse, error) {
	resp := tb.DeleteVisualSelection()

	// paste from the clipboard
	cliptext, err := clipboard.ReadAll()
	if err != nil {
		return resp, err
	}
	if len(cliptext) == 0 {
		return resp, nil
	}
	cliprunes := []rune(cliptext)
	rs := tb.text
	rs = append(rs[:tb.cursorPos], append(cliprunes, rs[tb.cursorPos:]...)...)
	tb.text = rs
	tb.IncrCursorPos(len(cliprunes))
	w := tb.GetWrapped()
	tb.SetContentFromString(w.WrappedStr()) // See NOTE-1
	tb.CorrectOffsets(w, &resp)
	if tb.TextChangedHook != nil {
		resp = tb.TextChangedHook(string(tb.text))
	}
	return resp, nil
}

func (tb *TextBox) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
	resp = yh.NewEventResponse()

	if tb.Selectedness != Selected {
		return false, yh.NewEventResponse()
	}

	if !tb.chCursor {
		switch {
		case (yh.LeftEKC.Matches(evs) || yh.HLowerEKC.Matches(evs)):
			tb.ScrollLeft()
		case (yh.RightEKC.Matches(evs) || yh.LLowerEKC.Matches(evs)):
			tb.ScrollRight()
		case (yh.DownEKC.Matches(evs) || yh.JLowerEKC.Matches(evs)):
			tb.ScrollDown()
		case (yh.UpEKC.Matches(evs) || yh.KLowerEKC.Matches(evs)):
			tb.ScrollUp()
		}

		// call the offset hook
		if tb.YChangedHook != nil {
			tb.YChangedHook(tb.ContentYOffset, tb.ContentHeight())
		}

		return true, resp
	}

	visualModeEvent := false
	switch {
	case yh.ShiftLeftEKC.Matches(evs):
		visualModeEvent = true
		if !tb.visualMode {
			tb.visualMode = true
			tb.visualModeStartPos = tb.cursorPos
		}
		if tb.cursorPos > 0 {
			tb.IncrCursorPos(-1)
			w := tb.GetWrapped()
			tb.CorrectOffsets(w, &resp)
		}
	case yh.ShiftRightEKC.Matches(evs):
		visualModeEvent = true
		if !tb.visualMode {
			tb.visualMode = true
			tb.visualModeStartPos = tb.cursorPos
		}
		if tb.cursorPos < len(tb.text) {
			tb.IncrCursorPos(1)

			w := tb.GetWrapped()
			tb.CorrectOffsets(w, &resp)
		}

	case yh.ShiftUpEKC.Matches(evs):
		visualModeEvent = true
		if !tb.visualMode {
			tb.visualMode = true
			tb.visualModeStartPos = tb.cursorPos
		}
		w := tb.GetWrapped()
		tb.SetCursorPos(w.GetCursorAbovePosition(tb.cursorPos))
		tb.CorrectOffsets(w, &resp)

	case yh.ShiftDownEKC.Matches(evs):
		visualModeEvent = true
		if !tb.visualMode {
			tb.visualMode = true
			tb.visualModeStartPos = tb.cursorPos
		}
		w := tb.GetWrapped()
		tb.SetCursorPos(w.GetCursorBelowPosition(tb.cursorPos))

		tb.CorrectOffsets(w, &resp)

	case yh.LeftEKC.Matches(evs):

		if tb.cursorPos > 0 {
			// do not move left if at the beginning of a line
			if tb.text[tb.cursorPos-1] != '\n' {
				tb.IncrCursorPos(-1)

				w := tb.GetWrapped()
				tb.CorrectOffsets(w, &resp)
			}
		}
	case yh.RightEKC.Matches(evs) && tb.wordwrap:
		if tb.cursorPos < len(tb.text) {
			tb.IncrCursorPos(1)

			w := tb.GetWrapped()
			tb.CorrectOffsets(w, &resp)
		}

	// if wordwrap is disable do not move to the next line
	// when at the end of the line
	case yh.RightEKC.Matches(evs) && !tb.wordwrap:
		w := tb.GetWrapped()
		curX, curY := w.CursorXAndY(tb.cursorPos)
		l := w.GetLine(curY)
		if curX < len(l)-2 {
			tb.IncrCursorPos(1)
			tb.CorrectOffsets(w, &resp)
		}

	case yh.UpEKC.Matches(evs):
		w := tb.GetWrapped()
		tb.SetCursorPos(w.GetCursorAbovePosition(tb.cursorPos))
		tb.CorrectOffsets(w, &resp)
	case yh.DownEKC.Matches(evs):
		w := tb.GetWrapped()
		tb.SetCursorPos(w.GetCursorBelowPosition(tb.cursorPos))
		tb.CorrectOffsets(w, &resp)

	case tb.editable && yh.RunesEKC.Matches(evs):
		if len(evs) > 0 {
			rs := tb.text
			rs = append(rs[:tb.cursorPos], append([]rune{evs[0].Rune()},
				rs[tb.cursorPos:]...)...)
			tb.text = rs
			tb.IncrCursorPos(1)
			w := tb.GetWrapped()

			// NOTE-1: must call SetContentFromString to update the content
			// before updating the offset or else the offset amount may not
			// exist in the content and the widget base will reject the new
			// offset
			tb.SetContentFromString(w.WrappedStr())
			tb.CorrectOffsets(w, &resp)

			if tb.TextChangedHook != nil {
				resp = tb.TextChangedHook(string(tb.text))
			}
		}

	case yh.MetaCLowerEKC.Matches(evs):
		_ = tb.CopyToClipboard() // ignore err

	case tb.editable && yh.MetaVLowerEKC.Matches(evs): // TODO fix
		resp, _ = tb.PasteFromClipboard() // ignore err

	case tb.editable && (yh.BackspaceEKC.Matches(evs) || yh.Backspace2EKC.Matches(evs)):
		if tb.visualMode {
			resp = tb.DeleteVisualSelection()
		} else if tb.cursorPos > 0 {
			rs := tb.text
			rs = append(rs[:tb.cursorPos-1], rs[tb.cursorPos:]...)
			tb.text = rs
			tb.IncrCursorPos(-1)
			w := tb.GetWrapped()
			tb.SetContentFromString(w.WrappedStr()) // See NOTE-1
			tb.CorrectOffsets(w, &resp)
			if tb.TextChangedHook != nil {
				resp = tb.TextChangedHook(string(tb.text))
			}
		}
	case tb.editable && yh.EnterEKC.Matches(evs):
		rs := tb.text
		rs = append(rs[:tb.cursorPos], append([]rune{'\n'}, rs[tb.cursorPos:]...)...)
		tb.text = rs
		tb.IncrCursorPos(1)
		w := tb.GetWrapped()
		tb.SetContentFromString(w.WrappedStr()) // See NOTE-1
		tb.CorrectOffsets(w, &resp)
	}

	if tb.visualMode && !visualModeEvent {
		tb.visualMode = false
	}

	return true, resp
}

func (tb *TextBox) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	resp = yh.NewEventResponse()

	// handle secondary click
	if tb.RightClickMenu != nil {
		// send the event to the right click menu to check for right click
		createRCM := tb.RightClickMenu.CreateMenuIfRightClick(ev)
		if createRCM.HasWindow() {
			return true, yh.NewEventResponse().WithWindow(createRCM)
		}
	}

	if ev.Buttons() == tcell.WheelDown && ev.Modifiers() == tcell.ModNone && tb.Selectedness == Selected {
		w := tb.GetWrapped()
		tb.SetCursorPos(w.GetCursorBelowPosition(tb.cursorPos))
		tb.CorrectOffsets(w, &resp)
		return true, yh.NewEventResponse()
	}

	if ev.Buttons() == tcell.WheelUp && ev.Modifiers() == tcell.ModNone && tb.Selectedness == Selected {
		w := tb.GetWrapped()
		tb.SetCursorPos(w.GetCursorAbovePosition(tb.cursorPos))
		tb.CorrectOffsets(w, &resp)
		return true, yh.NewEventResponse()
	}
	if ev.Buttons() == tcell.WheelDown && ev.Modifiers() == tcell.ModShift && tb.Selectedness == Selected {
		w := tb.GetWrapped()
		tb.SetCursorPos(w.GetCursorLeftPosition(tb.cursorPos))
		tb.CorrectOffsets(w, &resp)
		return true, yh.NewEventResponse()
	}

	if ev.Buttons() == tcell.WheelUp && ev.Modifiers() == tcell.ModShift && tb.Selectedness == Selected {
		w := tb.GetWrapped()
		tb.SetCursorPos(w.GetCursorRightPosition(tb.cursorPos))
		tb.CorrectOffsets(w, &resp)
		return true, yh.NewEventResponse()
	}

	if ev.Buttons() == tcell.ButtonNone && tb.Selectedness == Selected {
		tb.mouseDragging = false
	}

	// set the cursor to the mouse position on primary click
	if ev.Buttons() == tcell.Button1 && tb.Selectedness == Selected { //left click
		x, y := ev.Position()
		x += tb.ContentXOffset
		y += tb.ContentYOffset
		w := tb.GetWrapped()

		if tb.mouseDragging && !tb.visualMode {
			tb.visualMode = true
			tb.visualModeStartPos = tb.cursorPos
		}
		if !tb.mouseDragging && tb.visualMode {
			tb.visualMode = false
		}
		tb.mouseDragging = true

		tb.SetCursorPos(w.GetNearestValidCursorFromPosition(x, y))
		tb.CorrectOffsets(tb.GetWrapped(), &resp)
		return true, yh.NewEventResponse()
	}
	return tb.WidgetBase.ReceiveMouseEvent(ev)
}

func (tb *TextBox) GetText() string {
	return string(tb.text)
}

func (tb *TextBox) SetText(text string) {
	tb.text = []rune(text)
}

// returns the wrapped characters of the text
func (tb *TextBox) GetWrapped() wrChs {
	rs := append(tb.text, ' ') // add the space for the final possible position
	chs := []wrCh{}
	maxX := 0
	x, y := 0, 0 // working x and y position in the textbox
	for absPos, r := range rs {
		if tb.wordwrap && x == tb.GetWidth() {
			y++
			x = 0
			if x > maxX {
				maxX = x
			}
			chs = append(chs, newWrCh('\n', -1, x, y))
		}
		if r == '\n' {

			// If chCursor without wordwrap, add an extra space to the end of
			// the line so that the cursor can be placed there. Without this
			// extra space, placing the cursor at the end of the largest line
			// will panic.
			if tb.chCursor && !tb.wordwrap {
				if x > maxX {
					maxX = x
				}
				chs = append(chs, newWrCh(' ', -1, x, y))
			}

			// the "newline character" exists as an extra space at
			// the end of the line
			if x > maxX {
				maxX = x
			}
			chs = append(chs, newWrCh('\n', absPos, x, y))

			// move the working position to the beginning of the next line
			y++
			x = 0
		} else {
			if x > maxX {
				maxX = x
			}
			chs = append(chs, newWrCh(r, absPos, x, y))
			x++
		}
	}
	return wrChs{
		chs:  chs,
		maxX: maxX,
	}
}

// ------------------------------------------------

// returns the formatted line numbers of the textbox
// line numbers are right justified
func (tb *TextBox) GetLineNumbers() (content string, contentWidth int) {
	wrChs := tb.GetWrapped()

	// get the max line number
	maxLineNum := 0
	for i, wrCh := range wrChs.chs {
		if wrCh.ch == '\n' || i == 0 {
			if wrCh.absPos != -1 || i == 0 {
				maxLineNum++
			}
		}
	}

	// get the largest amount of digits in the line numbers from the string
	lineNumWidth := len(fmt.Sprintf("%d", maxLineNum))

	s := ""
	trueLineNum := 1
	for i, wrCh := range wrChs.chs {
		if wrCh.ch == '\n' || i == 0 {
			if wrCh.absPos != -1 || i == 0 {
				s += fmt.Sprintf("%*d ", lineNumWidth, trueLineNum)
				trueLineNum++
			}
			s += "\n"
		}
	}

	return s, lineNumWidth + 1 // +1 for the extra space after the digits
}

// ------------------------------------------------

// wrapped character
type wrCh struct {
	ch rune // the character

	// absolute position in the text
	// If this character is a NOT a part of the text and only introduced
	// due to line wrapping, the absPos will be -1 (and ch='\n')
	absPos int

	xPos int // x position in the line
	yPos int // y position of the line
}

func newWrCh(ch rune, absPos, xPos, yPos int) wrCh {
	return wrCh{ch: ch, absPos: absPos, xPos: xPos, yPos: yPos}
}

// wrapped characters
type wrChs struct {
	chs  []wrCh
	maxX int // the maximum x position within the wrapped characters
}

func (w wrChs) WrappedStr() string {
	s := ""
	for _, wrCh := range w.chs {
		s += string(wrCh.ch)
	}
	return s
}

// gets the cursor x and y position in the wrapped text
// from the absolute cursor position provided.
func (w wrChs) CursorXAndY(curAbs int) (x int, y int) {
	for _, wrCh := range w.chs {
		if wrCh.absPos == curAbs {
			return wrCh.xPos, wrCh.yPos
		}
	}
	return -1, -1
}

// gets the line at the given y position
func (w wrChs) GetLine(y int) []rune {
	s := []rune{}
	for _, wrCh := range w.chs {
		if wrCh.yPos == y {
			s = append(s, wrCh.ch)
		}
	}
	return s
}

// maximum y position in the wrapped text
func (w wrChs) MaxY() int {
	return w.chs[len(w.chs)-1].yPos
}

func (w wrChs) MaxX() int {
	return w.maxX
}

// Determine the cursor position above the current cursor position.
func (w wrChs) GetCursorAbovePosition(curAbs int) (newCurAbs int) {

	// first get the current cursor position and the index of the current
	// cursor position in the wrapped text
	curX, curY := -1, -1
	cursorIndex := -1
	for i, wrCh := range w.chs {
		if wrCh.absPos == curAbs {
			curX, curY = wrCh.xPos, wrCh.yPos
			cursorIndex = i
		}
	}
	if curY > 0 {
		// move backwards in the wrapped text until we find the first
		// character with the same x position as the current cursor position
		for i := cursorIndex - 1; i >= 0; i-- {
			if w.chs[i].yPos == curY-1 && w.chs[i].xPos <= curX {
				return w.chs[i].absPos
			}
		}
	}
	return curAbs // no change
}

// Determine the cursor position below the current cursor position.
func (w wrChs) GetCursorBelowPosition(curAbs int) (newCurAbs int) {

	// first get the current cursor position and the index of the current
	// cursor position in the wrapped text
	curX, curY := -1, -1
	cursorIndex := -1
	for i, wrCh := range w.chs {
		if wrCh.absPos == curAbs {
			curX, curY = wrCh.xPos, wrCh.yPos
			cursorIndex = i
		}
	}
	if curY < w.MaxY() {
		// move backwards in the wrapped text until we find the first
		// character with a y position one greater than the current cursor
		// position and with an x position less than or equal to the current
		// cursor position.
		for i := len(w.chs) - 1; i > cursorIndex; i-- {
			if w.chs[i].yPos == curY+1 && w.chs[i].xPos <= curX {
				return w.chs[i].absPos
			}
		}
	}
	return curAbs // no change
}

func (w wrChs) GetCursorLeftPosition(curAbs int) (newCurAbs int) {

	// first get the current cursor position
	curX := -1
	for _, wrCh := range w.chs {
		if wrCh.absPos == curAbs {
			curX = wrCh.xPos
		}
	}
	if curX > 0 {
		return curAbs - 1
	}
	return curAbs // no change
}

func (w wrChs) GetCursorRightPosition(curAbs int) (newCurAbs int) {

	// first get the current cursor position
	curX, curY := -1, -1
	for _, wrCh := range w.chs {
		if wrCh.absPos == curAbs {
			curX, curY = wrCh.xPos, wrCh.yPos
		}
	}
	l := w.GetLine(curY)
	if curX < len(l)-2 {
		return curAbs + 1
	}
	return curAbs // no change
}

func (w wrChs) GetNearestValidCursorFromPosition(x, y int) (newCurAbs int) {

	nearestAbs := -1     // nearest absolute position with the same y position
	nearestAbsYPos := -1 // Y position of the nearest absolute position
	nearestAbsXPos := -1 // X position of the nearest absolute position
	for _, wrCh := range w.chs {
		if wrCh.absPos == -1 {
			continue
		}
		if wrCh.yPos == y && wrCh.xPos == x {
			return wrCh.absPos
		}

		// TODO make my own abs function to avoid float casting
		if math.Abs(float64(wrCh.yPos-y)) < math.Abs(float64(nearestAbsYPos-y)) {
			nearestAbsYPos = wrCh.yPos
			nearestAbsXPos = wrCh.xPos
			nearestAbs = wrCh.absPos
		} else if math.Abs(float64(wrCh.yPos-y)) == math.Abs(float64(nearestAbsYPos-y)) &&
			math.Abs(float64(wrCh.xPos-x)) < math.Abs(float64(nearestAbsXPos-x)) {
			nearestAbsYPos = wrCh.yPos
			nearestAbsXPos = wrCh.xPos
			nearestAbs = wrCh.absPos
		}
	}

	return nearestAbs
}
*/
