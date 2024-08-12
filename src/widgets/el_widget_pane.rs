/*
   package widgets

import (
    yh "keybase.io/nwmod/nwmod/yeehaw"
    els "keybase.io/nwmod/nwmod/yeehaw/elements"
)

type WidgetPane struct {
    els.StandardPane
    WO *WidgetOrganizer
}

func NewWidgetPane(defaultCh yh.DrawCh) *WidgetPane {
    wp := &WidgetPane{
        StandardPane: els.NewEmptyStandardPane(defaultCh),
        WO:           NewWidgetOrganizer(),
    }
    wp.SelfEvs = yh.NewPriorityEvs(yh.Focused, WOSelfReceivable)
    return wp
}

var _ yh.Element = &WidgetPane{}

// fulfills the yh.Element interface
func (wp *WidgetPane) ReceiveEvent(ctx yh.Context, ev interface{}) (
    captured bool, resp yh.EventResponse) {

    switch ev := ev.(type) {
    case yh.KeysEvent:
        return wp.WO.CaptureKeyEvents(ev)
    case yh.MouseEvent:
        return wp.WO.CaptureMouseEvent(ev)
    case yh.ResizeEvent:
        wp.WO.ResizeEvent(ctx)
    case yh.RefreshEvent:
        wp.WO.Refresh()
    }

    return wp.StandardPane.ReceiveEvent(ctx, ev)
}

func (wp *WidgetPane) AddWidget(w Widget) {
    a := yh.NewPriorityEvs(yh.Focused, w.Receivable())
    wp.SelfEvs = append(wp.SelfEvs, a...)
    wp.WO.AddWidget(w, w.GetLocation())
}

func (wp *WidgetPane) AddWidgets(ws []Widget) {
    for _, w := range ws {
        wp.AddWidget(w)
    }
}

func (wp *WidgetPane) RemoveWidget(w Widget) {
    wp.WO.RemoveWidget(w)
}

func (wp *WidgetPane) ClearWidgets() {
    wp.WO.ClearWidgets()
}

// fulfills the yh.Element interface
func (wp *WidgetPane) Drawing(ctx yh.Context) []yh.DrawChPos {
    chs := wp.StandardPane.Drawing(ctx)
    chs = append(chs, wp.WO.Drawing(ctx)...)
    return chs
}

// fulfills the yh.Element interface
func (wp *WidgetPane) ChangePriority(pCtx yh.Context, p yh.Priority) yh.InputabilityChanges {

    var ic yh.InputabilityChanges
    if p == yh.Unfocused {
        ic = wp.WO.UnselectSelectedWidget()
    }
    ic.Concat(wp.StandardPane.ChangePriority(pCtx, p))
    return ic
}

// Returns the widget organizer's receivable events along
// with the standard pane's self events.
func (wp *WidgetPane) Receivable() []yh.PriorityEv {

    // all of the events returned by the widget organizer are set to
    // focused because WO.Receivable only returns the events associated with
    // widget that is currently active.
    wpes := yh.NewPriorityEvs(yh.Focused, wp.WO.Receivable())

    // Add the widget pane's self events. These are WOSelfReceivable
    return append(wp.StandardPane.Receivable(), wpes...)
}
*/
