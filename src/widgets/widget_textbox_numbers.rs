/*
package widgets

import (
	"fmt"
	"strconv"
	"unicode"

	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

type NumbersTextBox struct {
	*TextBox
	Value            int64
	HasUpDownButtons bool // if true, adds up/down buttons to the right of the text box
}

var _ Widget = &NumbersTextBox{}

var NumbersTextboxEditableEvCombos = []yh.PrioritizableEv{yh.RunesEKC, yh.EnterEKC, yh.ShiftEnterEKC,
	yh.BackspaceEKC, yh.Backspace2EKC, yh.LeftEKC, yh.RightEKC, yh.UpEKC, yh.DownEKC}

func NewNumbersTextBox(pCtx yh.Context, startingValue int64) *NumbersTextBox {
	tb := NewTextBox(pCtx, fmt.Sprintf("%d", startingValue))
	return &NumbersTextBox{
		TextBox:          tb,
		Value:            startingValue,
		HasUpDownButtons: true,
	}
}

// ---------------------------------------------------------
// Decorators

func (tb *NumbersTextBox) WithUpDownButtons() *NumbersTextBox {
	tb.HasUpDownButtons = true
	return tb
}

func (tb *NumbersTextBox) WithoutUpDownButtons() *NumbersTextBox {
	tb.HasUpDownButtons = false
	return tb
}

func (tb *NumbersTextBox) WithSize(width, height SclVal) *NumbersTextBox {
	tb.Width, tb.Height = width, height
	return tb
}

func (tb *NumbersTextBox) WithTextChangedHook(hook func(newText string) yh.EventResponse) *NumbersTextBox {
	tb.TextChangedHook = hook
	return tb
}

func (tb *NumbersTextBox) WithCursorStyle(style tcell.Style) *NumbersTextBox {
	tb.cursorStyle = style
	return tb
}

func (tb *NumbersTextBox) WithStyles(styles WBStyles) *NumbersTextBox {
	tb.Styles = styles
	return tb
}

func (tb *NumbersTextBox) At(locX, locY SclVal) *NumbersTextBox {
	tb.WidgetBase.At(locX, locY)
	return tb
}

func (tb *NumbersTextBox) ToWidgets() Widgets {

	x, y := tb.LocX, tb.LocY

	//width, height := tb.GetWidth(pCtx), tb.GetHeight(pCtx)
	//tbX2, tbY2 := x+width-1, y+height-1
	//locTB := yh.NewLocation(x, tbX2, y, tbY2, WidgetZIndex)
	out := []Widget{tb}

	if tb.HasUpDownButtons {
		upBtn := NewButton(tb.GetParentCtx(), "▲", func() yh.EventResponse {
			tb.ChangeValue(tb.Value + 1)
			return yh.NewEventResponse()
		}).WithoutSides()
		downBtn := NewButton(tb.GetParentCtx(), "▼", func() yh.EventResponse {
			tb.ChangeValue(tb.Value - 1)
			return yh.NewEventResponse()
		}).WithoutSides()

		UpBtnX := x.Plus(tb.Width)
		DownBtnX := UpBtnX.PlusStatic(1)
		upBtn.At(UpBtnX, y)
		downBtn.At(DownBtnX, y)

		out = append(out, upBtn)
		out = append(out, downBtn)
	}
	return out
}

// ---------------------------------------------------------

func (tb *NumbersTextBox) ChangeValue(newValue int64) {
	tb.Value = newValue
	tb.SetText(fmt.Sprintf("%d", newValue))
	if tb.TextChangedHook != nil {
		tb.TextChangedHook(string(tb.text))
	}
}

func (tb *NumbersTextBox) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
	yh.Debug("NumbersTextBox.ReceiveKeyEventCombo()")
	if tb.Selectedness != Selected {
		return false, yh.NewEventResponse()
	}

	switch {
	case yh.UpEKC.Matches(evs):
		tb.ChangeValue(tb.Value + 1)
	case yh.DownEKC.Matches(evs):
		tb.ChangeValue(tb.Value - 1)

	case yh.RunesEKC.Matches(evs):
		if len(evs) > 0 {
			r := evs[0].Rune()
			if !unicode.IsDigit(r) {
				return true, yh.NewEventResponse()
			}
			tb.TextBox.ReceiveKeyEventCombo(evs)
			value_str := tb.GetText()
			value, err := strconv.ParseInt(value_str, 10, 64)
			if err != nil {
				tb.ChangeValue(tb.Value) // reset to previous value
				return true, yh.NewEventResponse()
			}
			tb.ChangeValue(value)
		}

	case yh.BackspaceEKC.Matches(evs) || yh.Backspace2EKC.Matches(evs):
		if tb.cursorPos > 0 {
			tb.TextBox.ReceiveKeyEventCombo(evs)
			value_str := tb.GetText()
			value, err := strconv.ParseInt(value_str, 10, 64)
			if err != nil {
				tb.ChangeValue(tb.Value) // reset to previous value
				return true, yh.NewEventResponse()
			}
			tb.ChangeValue(value)
		}
	case yh.EnterEKC.Matches(evs):
		// don't allow
	default:
		tb.TextBox.ReceiveKeyEventCombo(evs)
	}

	return true, yh.NewEventResponse()
}
*/
