/*
package widgets

import (
	tcell "github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

type WidgetOrganizer struct {
	Widgets           []Widget
	locations         []yh.Location
	activeWidgetIndex int // -1 means no widget active
}

func NewWidgetOrganizer() *WidgetOrganizer {
	return &WidgetOrganizer{
		Widgets:           []Widget{},
		locations:         []yh.Location{},
		activeWidgetIndex: -1,
	}
}

func (wo *WidgetOrganizer) Refresh() {
	/*
		// get number of selectable Widgets
		readyWidgetCount := 0
		for _, w := range wo.Widgets {
			if w.GetSelectability() != Unselectable {
				readyWidgetCount++
			}
		}

		// TODO: this doesn't lock the widget as active. Pressing tab still deactivates it
		// if only one selectable widget, select it
		if readyWidgetCount == 1 {
			wo.switchToNextWidget()
		}
	*/
}

func (wo *WidgetOrganizer) AddWidget(w Widget, loc yh.Location) {
	wo.Widgets = append(wo.Widgets, w)
	wo.locations = append(wo.locations, loc)
}

func (wo *WidgetOrganizer) RemoveWidget(w Widget) {
	for i, widget := range wo.Widgets {
		if widget == w {
			wo.Widgets = append(wo.Widgets[:i], wo.Widgets[i+1:]...)
			wo.locations = append(wo.locations[:i], wo.locations[i+1:]...)
			return
		}
	}
}

func (wo *WidgetOrganizer) ClearWidgets() {
	wo.Widgets = []Widget{}
	wo.locations = []yh.Location{}
	wo.activeWidgetIndex = -1
}

// deactivate all the Widgets
func (wo *WidgetOrganizer) UnselectSelectedWidget() yh.InputabilityChanges {
	if wo.activeWidgetIndex != -1 {
		resp := wo.Widgets[wo.activeWidgetIndex].SetSelectability(Ready)
		// ignore all responses besides inputability
		wo.processWidgetResp(&resp, wo.activeWidgetIndex)
		ic, _ := resp.GetInputabilityChanges()
		wo.activeWidgetIndex = -1
		return ic
	}
	return yh.InputabilityChanges{}
}

func (wo *WidgetOrganizer) switchBetweenWidgets(oldIndex, newIndex int) yh.InputabilityChanges {

	ic := yh.InputabilityChanges{}
	if wo.Widgets[newIndex].GetSelectability() == Unselectable {
		return ic
	}
	if oldIndex == newIndex {
		return ic
	}
	if oldIndex != -1 {
		resp := wo.Widgets[oldIndex].SetSelectability(Ready)
		// ignore all responses besides inputability
		wo.processWidgetResp(&resp, oldIndex)
		ic, _ = resp.GetInputabilityChanges()
	}

	resp := wo.Widgets[newIndex].SetSelectability(Selected)

	// ignore all responses besides inputability
	wo.processWidgetResp(&resp, newIndex)
	ic2, _ := resp.GetInputabilityChanges()

	ic.Concat(ic2)
	wo.activeWidgetIndex = newIndex
	return ic
}

// gets the next ready widget index starting from the startingIndex provided
func (wo *WidgetOrganizer) nextReadyWidgetIndex(startingIndex int) int {
	if startingIndex == -1 {
		startingIndex = 0
	}

	workingIndex := startingIndex
	for {
		workingIndex = (workingIndex + 1) % len(wo.Widgets)
		if wo.Widgets[workingIndex].GetSelectability() != Unselectable {
			return workingIndex
		}
		if workingIndex == startingIndex {
			// come full circle return the same index
			return startingIndex
		}
	}
}

// gets the previous ready widget index starting from the startingIndex provided
func (wo *WidgetOrganizer) prevReadyWidgetIndex(startingIndex int) int {
	if wo.activeWidgetIndex == -1 {
		return len(wo.Widgets) - 1
	}

	workingIndex := startingIndex
	for {
		workingIndex = (workingIndex - 1 + len(wo.Widgets)) % len(wo.Widgets)
		if wo.Widgets[workingIndex].GetSelectability() != Unselectable {
			return workingIndex
		}
		if workingIndex == startingIndex {
			// come full circle return the same index
			return startingIndex
		}
	}
	//panic("can't get prev index")
}

func (wo *WidgetOrganizer) switchToNextWidget() yh.InputabilityChanges {
	return wo.switchBetweenWidgets(wo.activeWidgetIndex,
		wo.nextReadyWidgetIndex(wo.activeWidgetIndex))
}

func (wo *WidgetOrganizer) switchToPrevWidget() yh.InputabilityChanges {
	return wo.switchBetweenWidgets(wo.activeWidgetIndex,
		wo.prevReadyWidgetIndex(wo.activeWidgetIndex))
}

func (wo *WidgetOrganizer) Receivable() []yh.PrioritizableEv {
	if wo.activeWidgetIndex == -1 {
		return []yh.PrioritizableEv{}
	}
	return wo.Widgets[wo.activeWidgetIndex].Receivable()
}

func (wo *WidgetOrganizer) processWidgetResp(resp *yh.EventResponse, widgetIndex int) {

	// adjust right click menu location to the widget
	// location which made the request
	if win, found := resp.GetWindow(); found {
		loc := wo.locations[widgetIndex]
		win.Loc.AdjustLocationsBy(loc.StartX, loc.StartY)
		resp.SetWindow(win)
	}

	// resize the widget
	if reloc, found := resp.GetRelocation(); found {
		wo.locations[widgetIndex].Relocate(reloc)
		resp.RemoveRelocation()
	}

	if deactivate := resp.GetDeactivate(); deactivate {
		ic := wo.UnselectSelectedWidget()
		resp.ConcatInputabilityChanges(ic)
		resp.RemoveDeactivate()
	}
}

var WOSelfReceivable = []yh.PrioritizableEv{
	yh.EscEKC,
	yh.TabEKC,
	yh.BackTabEKC,
}

// Returns true if one of the Widgets captures the events
func (wo *WidgetOrganizer) CaptureKeyEvents(evs []*tcell.EventKey) (
	captured bool, resp yh.EventResponse) {

	if len(evs) == 0 {
		return false, yh.NewEventResponse()
	}

	switch {
	case yh.EscEKC.Matches(evs):
		ic := wo.UnselectSelectedWidget()
		return true, yh.NewEventResponse().WithInputabilityChanges(ic)
	case yh.TabEKC.Matches(evs):
		ic := wo.switchToNextWidget()
		return true, yh.NewEventResponse().WithInputabilityChanges(ic)
	case yh.BackTabEKC.Matches(evs):
		ic := wo.switchToPrevWidget()
		return true, yh.NewEventResponse().WithInputabilityChanges(ic)
	}

	if wo.activeWidgetIndex == -1 {
		return false, yh.NewEventResponse()
	}
	captured, resp = wo.Widgets[wo.activeWidgetIndex].ReceiveKeyEventCombo(evs)
	wo.processWidgetResp(&resp, wo.activeWidgetIndex)
	return captured, resp
}

func (wo *WidgetOrganizer) CaptureMouseEvent(ev *tcell.EventMouse) (
	captured bool, resp yh.EventResponse) {

	clicked := true
	if ev.Buttons() == tcell.ButtonNone {
		clicked = false
	}

	var mostFrontZIndex yh.ZIndex = 1000 // lowest value is the most front
	widgetIndex := -1                    // index of widget with most front z index
	widgetLoc := yh.Location{}

	// find the widget with the most front z index
	for i, loc := range wo.locations {
		//if locs.ContainsWithinPrimary(ev.Position()) {
		if loc.Contains(ev.Position()) {
			if loc.Z < mostFrontZIndex {
				mostFrontZIndex = loc.Z
				widgetIndex = i
				widgetLoc = loc
			}
		}
	}

	if widgetIndex == -1 {
		if clicked {
			ic := wo.UnselectSelectedWidget()
			return false, yh.NewEventResponse().WithInputabilityChanges(ic)
		}
		return false, yh.NewEventResponse()
	}

	ic := yh.NewInputabilityChanges()
	if clicked {
		ic = wo.switchBetweenWidgets(wo.activeWidgetIndex, widgetIndex)
	}
	evAdj := widgetLoc.AdjustMouseEvent(ev)
	captured, resp = wo.Widgets[widgetIndex].ReceiveMouseEvent(evAdj)
	wo.processWidgetResp(&resp, widgetIndex)
	resp.ConcatInputabilityChanges(ic)

	return captured, resp
}

func (wo *WidgetOrganizer) ResizeEvent(ctx yh.Context) {
	// resize and refresh all locations
	for i, w := range wo.Widgets {
		w.ResizeEvent(ctx)
		wo.locations[i] = w.GetLocation()
	}
}

// draws all the Widgets
func (wo *WidgetOrganizer) Drawing(ctx yh.Context) []yh.DrawChPos {
	out := []yh.DrawChPos{}
	for i, w := range wo.Widgets {

		// skip the active widget the will be drawn on the top after
		if i == wo.activeWidgetIndex {
			continue
		}

		ds := w.Drawing()
		loc := wo.locations[i]

		for _, d := range ds {

			// adjust the location of the drawChPos relative to the WidgetPane
			d.AdjustByLocation(loc)

			// filter out chs that are outside of the WidgetPane bounds
			if d.Y >= 0 && d.Y < ctx.S.Height && d.X >= 0 && d.X < ctx.S.Width {
				out = append(out, d)
				//yh.Debug("drawing: %v\n", d)
			}
		}
	}

	// lastly draw the active widget on top
	if wo.activeWidgetIndex != -1 {
		ds := wo.Widgets[wo.activeWidgetIndex].Drawing()
		locs := wo.locations[wo.activeWidgetIndex]

		for _, d := range ds {

			// adjust the location of the drawChPos relative to the WidgetPane
			d.AdjustByLocation(locs)

			// filter out chs that are outside of the WidgetPane bounds
			if d.Y >= 0 && d.Y < ctx.S.Height && d.X >= 0 && d.X < ctx.S.Width {
				out = append(out, d)
			}
		}
	}
	return out
}
*/
