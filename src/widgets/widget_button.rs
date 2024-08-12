/*
package widgets

import (
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

// TODO figure out how to simulate being pressed with time based events

type Button struct {
	*WidgetBase
	Text      string
	HasSides  bool // if false no depression runes will be displayed
	Sides     ButtonSides
	ClickedFn func() yh.EventResponse // function which executes when button moves from pressed -> unpressed
}

type ButtonSides struct {
	Left  rune
	Right rune
}

// Button Sides Inspiration:
//
//	]button[  ⡇button⢸
//	]button[  ⢸button⡇
//	⎤button⎣  ❳button❲ ⎣⦘button⦗⎤
var ButtonSides1 = ButtonSides{
	Left:  ']',
	Right: '[',
}

func (bs ButtonSides) ButtonText(text string) string {
	return string(bs.Left) + text + string(bs.Right)
}

// when "active" hitting enter will click the button
var ButtonEvCombos = []yh.PrioritizableEv{yh.EnterEKC}

var ButtonStyle = WBStyles{
	SelectedStyle:     tcell.StyleDefault.Background(tcell.ColorLightYellow).Foreground(tcell.ColorBlack),
	ReadyStyle:        tcell.StyleDefault.Background(tcell.ColorWhite).Foreground(tcell.ColorBlack),
	UnselectableStyle: tcell.StyleDefault.Background(tcell.ColorLightSlateGrey).Foreground(tcell.ColorBlack),
}

func NewButton(pCtx yh.Context, text string, clickedFn func() yh.EventResponse) *Button {
	sides := ButtonSides1
	//r := yh.NewPriorityEvs(yh.Focused, ButtonEvCombos)
	//wb := NewWidgetBase(pCtx, len(text)+2, 1, ButtonStyle, r)
	wb := NewWidgetBase(pCtx, NewStatic(len(text)+2), NewStatic(1), ButtonStyle, ButtonEvCombos)
	wb.SetContentFromString(sides.ButtonText(text))

	return &Button{
		WidgetBase: wb,
		Text:       text,
		HasSides:   true,
		Sides:      sides,
		ClickedFn:  clickedFn,
	}
}

// ----------------------------------------------
// decorators

func (b *Button) WithoutSides() *Button {
	b.HasSides = false
	b.SetContentFromString(b.Text)
	b.WidgetBase.Width = NewStatic(len([]rune(b.Text)))
	return b
}

func (b *Button) ToWidgets() Widgets {
	return Widgets{b}
}

func (b *Button) WithStyles(styles WBStyles) *Button {
	b.Styles = styles
	return b
}

func (b *Button) At(locX, locY SclVal) *Button {
	b.WidgetBase.At(locX, locY)
	return b
}

// ----------------------------------------------

func (b *Button) Click() yh.EventResponse {
	resp := b.ClickedFn()
	return resp.WithDeactivate()
}

// need to re set the content in order to reflect active style
func (b *Button) Drawing() (chs []yh.DrawChPos) {
	if b.HasSides {
		b.SetContentFromString(b.Sides.ButtonText(b.Text))
	} else {
		b.SetContentFromString(b.Text)
	}
	return b.WidgetBase.Drawing()
}

func (b *Button) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
	if b.Selectedness != Selected {
		return false, yh.NewEventResponse()
	}

	switch {
	case yh.EnterEKC.Matches(evs):
		return true, b.Click()
	}
	return false, yh.NewEventResponse()
}

func (b *Button) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	if ev.Buttons() == tcell.Button1 { //left click
		return true, b.Click()
	}
	return false, yh.NewEventResponse()
}
*/
