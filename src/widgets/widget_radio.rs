/*
package widgets

import (
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

// TODO figure out how to simulate being pressed with time based events

type RadioBtns struct {
	*WidgetBase
	Radios          []RadioBtn
	Selected        int
	RadioSelectedFn func(i int, selected string) yh.EventResponse // function which executes when button moves from pressed -> unpressed
}

// radio button
type RadioBtn struct {
	Name  string
	OnCh  yh.DrawCh
	OffCh yh.DrawCh
}

// inspiration:
// ◯ ◉ ◯ ○
// ◯ ◯ ⦿ ●
// ⍟ ◉ ◯ ○
func NewBasicRadioBtn(name string) RadioBtn {
	return RadioBtn{
		OnCh:  yh.DrawCh{Ch: '⍟', Style: tcell.StyleDefault.Foreground(tcell.ColorWhite)},
		OffCh: yh.DrawCh{Ch: '◯', Style: tcell.StyleDefault.Foreground(tcell.ColorWhite)},
		Name:  name,
	}
}

// when "active" hitting enter will click the button
var (
	RadioEvCombos = []yh.PrioritizableEv{
		yh.UpEKC, yh.DownEKC, yh.JLowerEKC, yh.KLowerEKC}

	RadioStyle = WBStyles{
		SelectedStyle:     tcell.StyleDefault.Foreground(tcell.ColorLightYellow),
		ReadyStyle:        tcell.StyleDefault.Foreground(tcell.ColorWhite),
		UnselectableStyle: tcell.StyleDefault.Foreground(tcell.ColorLightSlateGrey),
	}
)

func NewRadioBtns(pCtx yh.Context, radios []string, radioSelectedFn func(i int, selected string) yh.EventResponse) *RadioBtns {

	maxWidth := 0
	for _, radio := range radios {
		if len(radio) > maxWidth {
			maxWidth = len(radio)
		}
	}
	maxWidth++ // add one for the radio button

	btns := make([]RadioBtn, len(radios))
	for i, radio := range radios {
		btns[i] = NewBasicRadioBtn(radio)
	}

	wb := NewWidgetBase(pCtx, NewStatic(maxWidth), NewStatic(len(radios)), RadioStyle, RadioEvCombos)
	radioBtns := &RadioBtns{
		WidgetBase:      wb,
		Radios:          btns,
		RadioSelectedFn: radioSelectedFn,
		Selected:        0,
	}

	_ = radioBtns.Drawing()
	return radioBtns
}

func (r *RadioBtns) At(locX, locY SclVal) *RadioBtns {
	r.WidgetBase.At(locX, locY)
	return r
}

func (r *RadioBtns) ToWidgets() Widgets {
	return Widgets{r}
}

// need to re set the content in order to reflect active style
func (r *RadioBtns) Drawing() (chs []yh.DrawChPos) {

	str := ""
	for i, radio := range r.Radios {
		if i == r.Selected {
			str += string(radio.OnCh.Ch)
		} else {
			str += string(radio.OffCh.Ch)
		}
		str += radio.Name
		if i != len(r.Radios)-1 {
			str += "\n"
		}
	}
	r.SetContentFromString(str)

	for i := range r.Radios {
		if i == r.Selected {
			r.Content[r.Selected][0].Style = r.Radios[r.Selected].OnCh.Style
		} else {
			r.Content[i][0].Style = r.Radios[i].OffCh.Style
		}
	}
	return r.WidgetBase.Drawing()
}

func (r *RadioBtns) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
	resp = yh.NewEventResponse()

	if r.Selectedness != Selected {
		return false, resp
	}

	captured = false
	switch {
	case yh.DownEKC.Matches(evs) || yh.JLowerEKC.Matches(evs):
		if r.Selected < len(r.Radios)-1 {
			r.Selected++
			captured = true
			if r.RadioSelectedFn != nil {
				resp = r.RadioSelectedFn(r.Selected, r.Radios[r.Selected].Name)
			}
		}
	case yh.UpEKC.Matches(evs) || yh.KLowerEKC.Matches(evs):
		if r.Selected > 0 {
			r.Selected--
			captured = true
			if r.RadioSelectedFn != nil {
				resp = r.RadioSelectedFn(r.Selected, r.Radios[r.Selected].Name)
			}
		}
	}

	return captured, resp
}

func (r *RadioBtns) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	resp = yh.NewEventResponse()
	if ev.Buttons() != tcell.Button1 { //left click
		return false, resp
	}

	_, y := ev.Position()
	if y >= 0 && y < len(r.Radios) {
		r.Selected = y
		if r.RadioSelectedFn != nil {
			resp = r.RadioSelectedFn(r.Selected, r.Radios[r.Selected].Name)
		}
	}

	return true, resp
}
*/
