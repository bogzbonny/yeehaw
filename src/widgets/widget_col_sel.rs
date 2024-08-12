/*

package widgets

import (
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

type ClrSelector struct {
	*WidgetBase
	Checked bool // whether the checkbox is checked or not

	// rune to use for the checkmark
	// recommended:  √ X x ✖
	Checkmark rune
	ClickedFn func(checked bool) yh.EventResponse // function which executes when checkbox is checked or unchecked
}

func NewClrSelector(pCtx yh.Context) *ClrSelector {
	wb := NewWidgetBase(pCtx, NewStatic(1), NewStatic(1), CheckboxStyle, CheckboxEvCombos)

	cb := &ClrSelector{
		WidgetBase: wb,
		Checked:    false,
		Checkmark:  '√',
		ClickedFn:  nil,
	}
	wb.SetContentFromString(cb.Text())
	return cb
}

func (cb *ClrSelector) At(locX, locY SclVal) *ClrSelector {
	cb.WidgetBase.At(locX, locY)
	return cb
}

func (cb *ClrSelector) ToWidgets() Widgets {
	return Widgets{cb}
}

// ---------------------------------------------------------

func (cb *ClrSelector) Text() string {
	if cb.Checked {
		return string(cb.Checkmark)
	}
	return " "
}

func (cb *ClrSelector) SetClickedFn(fn func(checked bool) yh.EventResponse) {
	cb.ClickedFn = fn
}

func (cb *ClrSelector) Click() yh.EventResponse {
	cb.Checked = !cb.Checked
	resp := yh.NewEventResponse()
	if cb.ClickedFn != nil {
		resp = cb.ClickedFn(cb.Checked)
	}
	cb.SetContentFromString(cb.Text())
	return resp
}

// need to re set the content in order to reflect active style
func (cb *ClrSelector) Drawing() (chs []yh.DrawChPos) {
	cb.SetContentFromString(cb.Text())
	return cb.WidgetBase.Drawing()
}

func (cb *ClrSelector) ReceiveKeyEventCombo(
	evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {

	if cb.Selectedness != Selected {
		return false, yh.NewEventResponse()
	}

	switch {
	case yh.EnterEKC.Matches(evs):
		return true, cb.Click()
	}
	return true, yh.NewEventResponse()
}

func (cb *ClrSelector) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	if ev.Buttons() == tcell.Button1 {
		return true, cb.Click()
	}
	return false, yh.NewEventResponse()
}
*/
