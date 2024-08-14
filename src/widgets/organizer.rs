use {
    super::{Selectability, Widget},
    crate::{
        Context, Event, EventResponse, Keyboard as KB, Location, Priority, ReceivableEventChanges,
    },
};

#[derive(Default)]
pub struct WidgetOrganizer {
    pub widgets: Vec<(Box<dyn Widget>, Location)>,
    active_widget_index: Option<usize>, // None means no widget active
}

impl WidgetOrganizer {
    pub fn default_receivable_events() -> Vec<Event> {
        vec![
            KB::KEY_ESC.into(),
            KB::KEY_TAB.into(),
            KB::KEY_BACKTAB.into(),
        ]
    }

    pub fn refresh(&mut self) {
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

    pub fn add_widget(&mut self, w: Box<dyn Widget>, loc: Location) {
        self.widgets.push((w, loc));
    }

    pub fn remove_widget(&mut self, w: Box<dyn Widget>) {
        for i in 0..self.widgets.len() {
            if self.widgets[i].0.id() == w.id() {
                self.widgets.remove(i);
                return;
            }
        }
    }

    pub fn clear_widgets(&mut self) {
        self.widgets.clear();
        self.active_widget_index = None;
    }

    // deactivate all the Widgets
    pub fn unselect_selected_widget(&mut self) -> ReceivableEventChanges {
        if let Some(i) = self.active_widget_index {
            let resp = self.widgets[i].0.set_selectability(Selectability::Ready);
            // ignore all responses besides receiving event changes
            let resp = resp.get_receivable_event_changes().unwrap_or_default();
            self.active_widget_index = None;
            resp
        } else {
            ReceivableEventChanges::default()
        }
    }

    pub fn process_widget_resp(&mut self, resp: &mut EventResponse, widget_index: usize) {
        // adjust right click menu location to the widget
        // location which made the request
        if let Some(mut win) = resp.window.clone() {
            let loc = self.widgets[widget_index].1.clone();
            win.loc.adjust_locations_by(loc.start_x, loc.start_y);
            resp.window = Some(win);
        }

        // resize the widget
        if let Some(reloc) = resp.relocation.clone() {
            self.widgets[widget_index].1.relocate(reloc);
            resp.relocation = None;
        }

        if resp.deactivate {
            let rec = self.unselect_selected_widget();
            resp.concat_receivable_event_changes(rec);
            resp.deactivate = false
        }
    }

    pub fn switch_between_widgets(
        &mut self, old_index: Option<usize>, new_index: Option<usize>,
    ) -> ReceivableEventChanges {
        if old_index == new_index {
            return ReceivableEventChanges::default();
        }

        if let Some(new_i) = new_index {
            if self.widgets[new_i].0.get_selectability() == Selectability::Unselectable {
                return ReceivableEventChanges::default();
            }
        }

        let mut rec = ReceivableEventChanges::default();
        if let Some(old_i) = old_index {
            let mut resp = self.widgets[old_i]
                .0
                .set_selectability(Selectability::Ready);
            // ignore all responses besides receiving event changes
            self.process_widget_resp(&mut resp, old_i);
            let resp = resp.get_receivable_event_changes().unwrap_or_default();
            rec = resp;
        }

        if let Some(new_i) = new_index {
            let mut resp = self.widgets[new_i]
                .0
                .set_selectability(Selectability::Selected);
            // ignore all responses besides receiving event changes
            self.process_widget_resp(&mut resp, new_i);
            let resp = resp.get_receivable_event_changes().unwrap_or_default();
            rec.concat(resp);
            self.active_widget_index = Some(new_i);
        }
        rec
    }

    // gets the next ready widget index starting from the startingIndex provided
    pub fn next_ready_widget_index(&self, starting_index: Option<usize>) -> Option<usize> {
        let starting_index = starting_index.unwrap_or(0);
        let mut working_index = starting_index;
        for _ in 0..self.widgets.len() + 1 {
            working_index = (working_index + 1) % self.widgets.len();
            if self.widgets[working_index].0.get_selectability() != Selectability::Unselectable {
                return Some(working_index);
            }
            if working_index == starting_index {
                // we've come full circle just return the same index
                return Some(starting_index);
            }
        }
        None
    }

    // gets the previous ready widget index starting from the startingIndex provided
    pub fn prev_ready_widget_index(&self, starting_index: Option<usize>) -> Option<usize> {
        if self.widgets.is_empty() {
            return None;
        }
        let starting_index = starting_index.unwrap_or(self.widgets.len() - 1);
        let mut working_index = starting_index;
        for _ in 0..self.widgets.len() + 1 {
            working_index = (working_index + self.widgets.len() - 1) % self.widgets.len();
            if self.widgets[working_index].0.get_selectability() != Selectability::Unselectable {
                return Some(working_index);
            }
            if working_index == starting_index {
                // we've come full circle just return the same index
                return Some(starting_index);
            }
        }
        None
    }

    pub fn switch_to_next_widget(&mut self) -> ReceivableEventChanges {
        self.switch_between_widgets(
            self.active_widget_index,
            self.next_ready_widget_index(self.active_widget_index),
        )
    }

    pub fn switch_to_prev_widget(&mut self) -> ReceivableEventChanges {
        self.switch_between_widgets(
            self.active_widget_index,
            self.prev_ready_widget_index(self.active_widget_index),
        )
    }

    pub fn receivable(&self) -> Vec<(Event, Priority)> {
        match self.active_widget_index {
            Some(i) => self.widgets[i].0.receivable(),
            None => Vec::new(),
        }
    }

    // Returns true if one of the Widgets captures the events
}

/*

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
