/*
import (
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

type ClrSelector struct {
	*WidgetBase
	Pallet     *Pallet
	Clr        tcell.Color
	SelectedFn func(newClr tcell.Color) yh.EventResponse // function which executes when checkbox is checked or unchecked
}

// basic pallet of colors
// vs    H >
//
// R>  <colours>
//
//
// R    G    B
// H    S    V
type Pallet struct {
	*WidgetBase
}

// when "active" hitting enter will click the button
var ClrSelectorEvCombos = []yh.PrioritizableEv{yh.EnterEKC}

var ClrSelectorStyle = WBStyles{
	SelectedStyle:     tcell.StyleDefault.Background(tcell.ColorLightYellow).Foreground(tcell.ColorBlack).Bold(true),
	ReadyStyle:        tcell.StyleDefault.Background(tcell.ColorWhite).Foreground(tcell.ColorBlack).Bold(true),
	UnselectableStyle: tcell.StyleDefault.Background(tcell.ColorLightSlateGrey).Foreground(tcell.ColorBlack).Bold(true),
}

func NewClrSelector(pCtx yh.Context) *ClrSelector {
	wb := NewWidgetBase(pCtx, NewStatic(1), NewStatic(1), ClrSelectorStyle, ClrSelectorEvCombos)

	cs := &ClrSelector{
		WidgetBase: wb,
		Checked:    false,
		Checkmark:  '√',
		ClickedFn:  nil,
	}
	wb.SetContentFromString(cs.Text())
	return cs
}

func (cs *ClrSelector) At(locX, locY SclVal) *ClrSelector {
	cs.WidgetBase.At(locX, locY)
	return cs
}

func (cs *ClrSelector) ToWidgets() Widgets {
	return Widgets{cs}
}

// ---------------------------------------------------------

func (cs *ClrSelector) Text() string {
	if cs.Checked {
		return string(cs.Checkmark)
	}
	return " "
}

func (cs *ClrSelector) SetClickedFn(fn func(checked bool) yh.EventResponse) {
	cs.ClickedFn = fn
}

func (cs *ClrSelector) Click() yh.EventResponse {
	cs.Checked = !cs.Checked
	resp := yh.NewEventResponse()
	if cs.ClickedFn != nil {
		resp = cs.ClickedFn(cs.Checked)
	}
	cs.SetContentFromString(cs.Text())
	return resp
}

// need to re set the content in order to reflect active style
func (cs *ClrSelector) Drawing() (chs []yh.DrawChPos) {
	cs.SetContentFromString(cs.Text())
	return cs.WidgetBase.Drawing()
}

func (cs *ClrSelector) ReceiveKeyEventCombo(
	evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {

	if cs.Selectedness != Selected {
		return false, yh.NewEventResponse()
	}

	switch {
	case yh.EnterEKC.Matches(evs):
		return true, cs.Click()
	}
	return true, yh.NewEventResponse()
}

func (cs *ClrSelector) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	if ev.Buttons() == tcell.Button1 {
		return true, cs.Click()
	}
	return false, yh.NewEventResponse()
}
*/
