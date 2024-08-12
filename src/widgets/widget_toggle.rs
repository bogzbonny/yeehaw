/*
package widgets

import (
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

// TODO figure out how to simulate being pressed with time based events

type Toggle struct {
	*WidgetBase
	Left         string
	Right        string
	LeftSelected bool //otherwise right is selected
	SelectedSty  tcell.Style
	ToggledFn    func(selected string) yh.EventResponse // function which executes when button moves from pressed -> unpressed
}

// when "active" hitting enter will click the button
var (
	ToggleEvCombos = []yh.PrioritizableEv{
		yh.EnterEKC, yh.LeftEKC, yh.RightEKC, yh.HLowerEKC, yh.LLowerEKC}

	ToggleStyle = WBStyles{
		SelectedStyle:     tcell.StyleDefault.Background(tcell.ColorLightYellow).Foreground(tcell.ColorBlack),
		ReadyStyle:        tcell.StyleDefault.Background(tcell.ColorWhite).Foreground(tcell.ColorBlack),
		UnselectableStyle: tcell.StyleDefault.Background(tcell.ColorLightSlateGrey).Foreground(tcell.ColorBlack),
	}

	DefaultSelectedSty = tcell.StyleDefault.Background(tcell.ColorBlue).Foreground(tcell.ColorBlack)
)

func NewToggle(pCtx yh.Context, left, right string, toggledFn func(selected string) yh.EventResponse) *Toggle {
	wb := NewWidgetBase(pCtx, NewStatic(len(left)+len(right)), NewStatic(1), ToggleStyle, ToggleEvCombos)
	wb.SetContentFromString(left + right)
	t := &Toggle{
		WidgetBase:   wb,
		Left:         left,
		Right:        right,
		LeftSelected: true,
		SelectedSty:  DefaultSelectedSty,
		ToggledFn:    toggledFn,
	}

	_ = t.Drawing()
	return t
}

func (t *Toggle) At(locX, locY SclVal) *Toggle {
	t.WidgetBase.At(locX, locY)
	return t
}

// returns Widgets for ease of labeling
func (t *Toggle) ToWidgets() Widgets {
	return Widgets{t}
}

func (t *Toggle) PerformToggle() yh.EventResponse {
	t.LeftSelected = !t.LeftSelected
	if t.ToggledFn != nil {
		return t.ToggledFn(t.Selected())
	}
	return yh.NewEventResponse()
}

func (t *Toggle) Selected() string {
	if t.LeftSelected {
		return t.Left
	}
	return t.Right
}

// need to re set the content in order to reflect active style
func (t *Toggle) Drawing() (chs []yh.DrawChPos) {
	t.SetContentFromString(t.Left + t.Right)
	if t.LeftSelected {
		for i := 0; i < len(t.Left); i++ {
			t.Content[0][i].Style = t.SelectedSty
		}
	} else {
		for i := len(t.Left); i < len(t.Left)+len(t.Right); i++ {
			t.Content[0][i].Style = t.SelectedSty
		}
	}
	return t.WidgetBase.Drawing()
}

func (t *Toggle) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
	resp = yh.NewEventResponse()
	if t.Selectedness != Selected {
		return false, resp
	}

	captured = false
	switch {
	case yh.EnterEKC.Matches(evs):
		captured = true
		resp = t.PerformToggle()
	case yh.LeftEKC.Matches(evs) || yh.HLowerEKC.Matches(evs):
		if !t.LeftSelected {
			captured = true
			resp = t.PerformToggle()
		}
	case yh.RightEKC.Matches(evs) || yh.LLowerEKC.Matches(evs):
		if t.LeftSelected {
			captured = true
			resp = t.PerformToggle()
		}
	}
	return captured, resp
}

func (t *Toggle) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	resp = yh.NewEventResponse()
	if ev.Buttons() != tcell.Button1 { //left click
		return false, yh.NewEventResponse()
	}

	x, _ := ev.Position()
	switch {
	case !t.LeftSelected && x < len(t.Left):
		resp = t.PerformToggle()
	case t.LeftSelected && x >= len(t.Left):
		resp = t.PerformToggle()
	}
	return true, resp
}
*/
