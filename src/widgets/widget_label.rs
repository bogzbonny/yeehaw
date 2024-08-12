/*
package widgets

import (
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

type Label struct {
	*WidgetBase
	Justification LabelJustification
	text          string
}

type LabelJustification string

const (
	JustifyLeft  LabelJustification = "left"
	JustifyRight LabelJustification = "right"
	JustifyDown  LabelJustification = "down" // some wacky shit
	JustifyUp    LabelJustification = "up"
)

// when "active" hitting enter will click the button
var LabelEvCombos = []yh.PrioritizableEv{}

var LabelStyle = WBStyles{
	SelectedStyle:     tcell.StyleDefault, // unused
	ReadyStyle:        tcell.StyleDefault, // unused
	UnselectableStyle: tcell.StyleDefault.Foreground(tcell.ColorWhite),
}

func NewLabel(pCtx yh.Context, text string) *Label {
	w, h := TextSize(text)
	wb := NewWidgetBase(pCtx, NewStatic(w), NewStatic(h), LabelStyle, LabelEvCombos)
	_ = wb.SetSelectability(Unselectable)
	wb.SetContentFromString(text)
	return &Label{
		WidgetBase:    wb,
		Justification: JustifyLeft,
		text:          text,
	}
}

func (l *Label) WithLeftJustification() *Label {
	l.Justification = JustifyLeft
	return l
}

func (l *Label) WithRightJustification() *Label {
	l.Justification = JustifyRight
	return l
}

func (l *Label) WithDownJustification() *Label {
	l.Justification = JustifyDown
	return l
}

func (l *Label) WithUpJustification() *Label {
	l.Justification = JustifyUp
	return l
}

// Rotate the text by 90 degrees
// intended to be used with WithDownJustification or WithUpJustification
func (l *Label) WithRotatedText() *Label {
	l.Content = l.Content.Rotate90Deg()
	l.Width, l.Height = l.Height, l.Width
	return l
}

func (l *Label) WithStyle(sty tcell.Style) *Label {
	l.Styles.UnselectableStyle = sty

	// this is necessary to actually update the content of the label w/
	// the new style
	// TODO: consider moving this somewhere else if it needs to be called in
	// many places
	l.SetContentFromString(l.text)
	return l
}

// Updates the content and size of the label
func (l *Label) SetText(text string) {
	l.SetContentFromString(text)
	w, h := TextSize(text)
	l.Width, l.Height = NewStatic(w), NewStatic(h)
}

func (l *Label) At(locX, locY SclVal) *Label {
	l.WidgetBase.At(locX, locY)
	return l
}

func (l *Label) ToWidgets() Widgets {
	x, y := l.LocX, l.LocY
	switch l.Justification {
	case JustifyLeft: // do nothing
	case JustifyRight:
		x = x.Minus(l.Width.MinusStatic(1))
	case JustifyDown: // do nothing
	case JustifyUp:
		y = y.Minus(l.Height.MinusStatic(1))
	}
	l.At(x, y)
	return Widgets{l}
}

func (l *Label) ReceiveKeyEventCombo(_ []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
	return false, yh.NewEventResponse()
}

func (l *Label) ReceiveMouseEvent(_ *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	return false, yh.NewEventResponse()
}
*/
