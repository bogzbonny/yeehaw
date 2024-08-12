/*
package widgets

import (
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
	mf "keybase.io/nwmod/nwmod/yeehaw/widgets/megafonts"
)

type Title struct {
	*WidgetBase
}

func NewTitle(pCtx yh.Context, text string, font mf.Megafont) *Title {
	megaText := font.GetMegaText(text)
	w, h := megaText.Size()
	yh.Debug("NewTitle: w=%d, h=%d", w, h)
	wb := NewWidgetBase(pCtx, NewStatic(w), NewStatic(h), DefaultWBStyles, []yh.PrioritizableEv{})
	_ = wb.SetSelectability(Unselectable)
	wb.SetContent(megaText)
	//yh.Debug("NewTitle: drawing=%v\n", wb.Drawing())

	return &Title{
		WidgetBase: wb,
	}
}

func (t *Title) At(locX, locY SclVal) *Title {
	t.WidgetBase.At(locX, locY)
	return t
}

func (t *Title) ToWidgets() Widgets {
	return Widgets{t}
}

func (t *Title) ReceiveKeyEventCombo(_ []*tcell.EventKey) (bool, yh.EventResponse) {
	return false, yh.NewEventResponse()
}

func (t *Title) ReceiveMouseEvent(_ *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
	return false, yh.NewEventResponse()
}
*/
