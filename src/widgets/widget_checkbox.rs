/*

package widgets

import (
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

type Checkbox struct {
	*WidgetBase
	Checked bool // whether the checkbox is checked or not

	// rune to use for the checkmark
	// recommended:  √ X x ✖
	Checkmark rune
	ClickedFn func(checked bool) yh.EventResponse // function which executes when checkbox is checked or unchecked
}

// when "active" hitting enter will click the button
var CheckboxEvCombos = []yh.PrioritizableEv{yh.EnterEKC}

var CheckboxStyle = WBStyles{
	SelectedStyle:     tcell.StyleDefault.Background(tcell.ColorLightYellow).Foreground(tcell.ColorBlack).Bold(true),
	ReadyStyle:        tcell.StyleDefault.Background(tcell.ColorWhite).Foreground(tcell.ColorBlack).Bold(true),
	UnselectableStyle: tcell.StyleDefault.Background(tcell.ColorLightSlateGrey).Foreground(tcell.ColorBlack).Bold(true),
}

func NewCheckbox(pCtx yh.Context) *Checkbox {
	wb := NewWidgetBase(pCtx, NewStatic(1), NewStatic(1), CheckboxStyle, CheckboxEvCombos)

	cb := &Checkbox{
		WidgetBase: wb,
		Checked:    false,
		Checkmark:  '√',
		ClickedFn:  nil,
	}
	wb.SetContentFromString(cb.Text())
	return cb
}

func (cb *Checkbox) At(locX, locY SclVal) *Checkbox {
	cb.WidgetBase.At(locX, locY)
	return cb
}

func (cb *Checkbox) ToWidgets() Widgets {
	return Widgets{cb}
}

// ---------------------------------------------------------

func (cb *Checkbox) Text() string {
	if cb.Checked {
		return string(cb.Checkmark)
	}
	return " "
}

func (cb *Checkbox) SetClickedFn(fn func(checked bool) yh.EventResponse) {
	cb.ClickedFn = fn
}

func (cb *Checkbox) Click() yh.EventResponse {
	cb.Checked = !cb.Checked
	resp := yh.NewEventResponse()
	if cb.ClickedFn != nil {
		resp = cb.ClickedFn(cb.Checked)
	}
	cb.SetContentFromString(cb.Text())
	return resp
}

// need to re set the content in order to reflect active style
func (cb *Checkbox) Drawing() (chs []yh.DrawChPos) {
	cb.SetContentFromString(cb.Text())
	return cb.WidgetBase.Drawing()
}

func (cb *Checkbox) ReceiveKeyEventCombo(
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

func (cb *Checkbox) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	if ev.Buttons() == tcell.Button1 {
		return true, cb.Click()
	}
	return false, yh.NewEventResponse()
}
*/
