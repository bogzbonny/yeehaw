use {
    super::{widget::RESP_DEACTIVATE, Selectability, Widget, Widgets},
    crate::{
        Color, Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponse, EventResponses, KeyPossibility, Keyboard as KB, ParentPane, Priority,
        ReceivableEventChanges, SortingHat, Parent,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct WidgetPane {
    pub pane: ParentPane,
    #[allow(clippy::type_complexity)]
    widgets: Rc<RefCell<Vec<(Box<dyn Widget>, Rc<RefCell<DynLocationSet>>)>>>,
    active_widget_index: Rc<RefCell<Option<usize>>>, // None means no widget active
}

impl WidgetPane {
    pub const KIND: &'static str = "widget_pane";

    pub fn default_receivable_events() -> Vec<Event> {
        vec![
            KB::KEY_ESC.into(),
            KB::KEY_TAB.into(),
            KB::KEY_BACKTAB.into(),
        ]
    }

    pub fn new(hat: &SortingHat) -> Self {
        let wp = WidgetPane {
            pane: ParentPane::new(hat, Self::KIND),
            widgets: Rc::new(RefCell::new(Vec::new())),
            active_widget_index: Rc::new(RefCell::new(None)),
        };
        wp.pane
            .pane
            .self_evs
            .borrow_mut()
            .push_many_at_priority(Self::default_receivable_events(), Priority::FOCUSED);
        wp
    }

    pub fn with_height(self, h: DynVal) -> Self {
        self.pane.pane.set_dyn_height(h);
        self
    }

    pub fn with_width(self, w: DynVal) -> Self {
        self.pane.pane.set_dyn_width(w);
        self
    }

    pub fn with_bg_color(self, c: Color) -> Self {
        self.pane.pane.default_ch.borrow_mut().style.bg = Some(c);
        self
    }

    pub fn add_widget(&self, w: Box<dyn Widget>) {
        self.pane.pane.self_evs.borrow_mut().extend(w.receivable());
        w.get_dyn_location_set().borrow_mut().set_z(w.get_z_index());
        let loc = w.get_dyn_location_set();
        //self.pane.add_element(Rc::new(RefCell::new(w))); // TODO add to the element organizer
        self.widgets.borrow_mut().push((w, loc));
    }

    pub fn add_widgets(&self, ws: Widgets) {
        for w in ws.0 {
            self.add_widget(w);
        }
    }

    pub fn remove_widget(&self, w: Box<dyn Widget>) {
        for i in 0..self.widgets.borrow().len() {
            if self.widgets.borrow()[i].0.id() == w.id() {
                self.widgets.borrow_mut().remove(i);
                return;
            }
        }
    }

    pub fn clear_widgets(&self) {
        self.widgets.borrow_mut().clear();
        *self.active_widget_index.borrow_mut() = None;
    }

    // deactivate all the Widgets
    pub fn unselect_selected_widget(&self, ctx: &Context) -> EventResponses {
        let active_index = *self.active_widget_index.borrow();
        if let Some(i) = active_index {
            let resps = self.widgets.borrow()[i]
                .0
                .set_selectability(ctx, Selectability::Ready);
            *self.active_widget_index.borrow_mut() = None;
            resps
        } else {
            EventResponses::default()
        }
    }

    // deactivate all the Widgets and process the responses returning only the receivable event
    // changes. we need this for change_priority
    pub fn unselect_selected_widget_process_resp(&self, ctx: &Context) -> ReceivableEventChanges {
        let active_index = *self.active_widget_index.borrow();
        if let Some(i) = active_index {
            let mut resps = self.widgets.borrow()[i]
                .0
                .set_selectability(ctx, Selectability::Ready);
            *self.active_widget_index.borrow_mut() = None;
            self.process_widget_resps(ctx, &mut resps, i);
            // ignore all unprocessed responses besides receivable event changes
            resps.get_receivable_event_changes()
        } else {
            ReceivableEventChanges::default()
        }
    }

    pub fn process_widget_resps(
        &self, ctx: &Context, resps: &mut EventResponses, widget_index: usize,
    ) {
        let mut extend_resps = vec![];
        for resp in resps.0.iter_mut() {
            let mut modified_resp = None;
            match resp {
                EventResponse::NewElement(new_el) => {
                    // adjust right click menu location to the widget
                    // location which made the request
                    let loc = self.widgets.borrow()[widget_index].1.borrow().clone();
                    new_el
                        .borrow()
                        .get_dyn_location_set()
                        .borrow_mut()
                        .adjust_locations_by(loc.l.start_x.clone(), loc.l.start_y.clone());
                }
                EventResponse::Metadata((k, _)) => {
                    if k == RESP_DEACTIVATE {
                        let resps_ = self.unselect_selected_widget(ctx);
                        extend_resps.extend(resps_.0);
                        modified_resp = Some(EventResponse::None);
                    }
                }

                _ => {}
            }
            if let Some(mr) = modified_resp {
                *resp = mr;
            }
        }
        resps.0.extend(extend_resps);
    }

    pub fn switch_between_widgets(
        &self, ctx: &Context, old_index: Option<usize>, new_index: Option<usize>,
    ) -> EventResponses {
        if old_index == new_index {
            return EventResponses::default();
        }

        if let Some(new_i) = new_index {
            if self.widgets.borrow()[new_i].0.get_selectability() == Selectability::Unselectable {
                return EventResponses::default();
            }
        }

        let mut resps = EventResponses::default();
        if let Some(old_i) = old_index {
            let mut resps_ = self.widgets.borrow()[old_i]
                .0
                .set_selectability(ctx, Selectability::Ready);

            self.process_widget_resps(ctx, &mut resps_, old_i);
            resps.extend(resps_.0);
        }

        if let Some(new_i) = new_index {
            let mut resps_ = self.widgets.borrow()[new_i]
                .0
                .set_selectability(ctx, Selectability::Selected);

            self.process_widget_resps(ctx, &mut resps_, new_i);
            *self.active_widget_index.borrow_mut() = Some(new_i);
            resps.extend(resps_.0);
        }
        resps
    }

    // gets the next ready widget index starting from the startingIndex provided
    pub fn next_ready_widget_index(&self, starting_index: Option<usize>) -> Option<usize> {
        let starting_index = starting_index.unwrap_or(0);
        let mut working_index = starting_index;
        for _ in 0..self.widgets.borrow().len() + 1 {
            working_index = (working_index + 1) % self.widgets.borrow().len();
            if self.widgets.borrow()[working_index].0.get_selectability()
                != Selectability::Unselectable
            {
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
        if self.widgets.borrow().is_empty() {
            return None;
        }
        let starting_index = starting_index.unwrap_or(self.widgets.borrow().len() - 1);
        let mut working_index = starting_index;
        for _ in 0..self.widgets.borrow().len() + 1 {
            working_index =
                (working_index + self.widgets.borrow().len() - 1) % self.widgets.borrow().len();
            if self.widgets.borrow()[working_index].0.get_selectability()
                != Selectability::Unselectable
            {
                return Some(working_index);
            }
            if working_index == starting_index {
                // we've come full circle just return the same index
                return Some(starting_index);
            }
        }
        None
    }

    pub fn switch_to_next_widget(&self, ctx: &Context) -> EventResponses {
        let i = *self.active_widget_index.borrow();
        self.switch_between_widgets(ctx, i, self.next_ready_widget_index(i))
    }

    pub fn switch_to_prev_widget(&self, ctx: &Context) -> EventResponses {
        let i = *self.active_widget_index.borrow();
        self.switch_between_widgets(ctx, i, self.prev_ready_widget_index(i))
    }

    // Returns true if one of the Widgets captures the events
    pub fn capture_key_event(
        &self, ctx: &Context, ev: Vec<KeyPossibility>,
        /*(captured, resp    )*/
    ) -> (bool, EventResponses) {
        if ev.is_empty() {
            return (false, EventResponses::default());
        }
        match true {
            _ if ev[0].matches_key(&KB::KEY_ESC) => {
                let resps = self.unselect_selected_widget(ctx);
                return (true, resps);
            }
            _ if ev[0].matches_key(&KB::KEY_TAB) => {
                let resps = self.switch_to_next_widget(ctx);
                return (true, resps);
            }
            _ if ev[0].matches_key(&KB::KEY_BACKTAB) => {
                let resps = self.switch_to_prev_widget(ctx);
                return (true, resps);
            }
            _ => {}
        }

        let active_index = *self.active_widget_index.borrow();
        if let Some(i) = active_index {
            let (captured, mut resps) = self.widgets.borrow()[i]
                .0
                .receive_event(ctx, Event::KeyCombo(ev));
            self.process_widget_resps(ctx, &mut resps, i);
            return (captured, resps);
        }
        (false, EventResponse::default().into())
    }

    pub fn capture_mouse_event(
        &self,
        ctx: &Context,
        ev: crossterm::event::MouseEvent,
        //(captured, resp    )
    ) -> (bool, EventResponses) {
        let mut clicked = false;
        if let MouseEventKind::Up(MouseButton::Left) = ev.kind {
            clicked = true;
        }

        let mut most_front_z_index = 0; // highest value is the most front
        let mut widget_index_loc = None; // index of widget with most front z index

        // find the widget with the most front z index
        for (i, (_, loc)) in self.widgets.borrow().iter().enumerate() {
            let loc = loc.borrow();
            if loc.contains(ctx, ev.column.into(), ev.row.into()) && loc.z > most_front_z_index {
                most_front_z_index = loc.z;
                widget_index_loc = Some((i, loc.clone()));
            }
        }

        let Some((widget_index, widget_loc)) = widget_index_loc else {
            let mut resps = if clicked {
                self.unselect_selected_widget(ctx)
            } else {
                EventResponses::default()
            };
            resps.extend(self.send_external_mouse_event(ctx, ev, None).0);
            return (false, resps);
        };

        let active_index = *self.active_widget_index.borrow();
        let resps = if clicked {
            self.switch_between_widgets(ctx, active_index, Some(widget_index))
        } else {
            EventResponses::default()
        };
        let ev_adj = widget_loc.l.adjust_mouse_event(ctx, &ev);
        let (captured, mut resps2) = self.widgets.borrow()[widget_index]
            .0
            .receive_event(ctx, Event::Mouse(ev_adj));

        self.process_widget_resps(ctx, &mut resps2, widget_index);
        resps2.extend(resps.0);

        resps2.extend(
            self.send_external_mouse_event(ctx, ev, Some(widget_index))
                .0,
        );
        (captured, resps2)
    }

    pub fn send_external_mouse_event(
        &self, ctx: &Context, ev: crossterm::event::MouseEvent, excluding_index: Option<usize>,
    ) -> EventResponses {
        let mut resps = EventResponses::default();
        for (i, (w, loc)) in self.widgets.borrow().iter().enumerate() {
            if let Some(i_) = excluding_index {
                if i == i_ {
                    continue;
                }
            }
            let ev_adj = loc.borrow().l.adjust_mouse_event(ctx, &ev);
            let (_, mut resps_) = w.receive_event(ctx, Event::ExternalMouse(ev_adj));
            self.process_widget_resps(ctx, &mut resps_, i);
            resps.extend(resps_.0);
        }
        resps
    }

    pub fn resize_event(&self, ctx: &Context) {
        for (w, _) in self.widgets.borrow().iter() {
            w.receive_event(ctx, Event::Resize);
        }
    }
}

impl Element for WidgetPane {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn id(&self) -> ElementID {
        self.pane.id()
    }

    // Returns the widget organizer's receivable events along
    // with the standard pane's self events.
    fn receivable(&self) -> Vec<(Event, Priority)> {
        // all of the events returned by the widget organizer are set to
        // focused because WO.Receivable only returns the events associated with
        // widget that is currently active.

        let wpes = match *self.active_widget_index.borrow() {
            Some(i) => self.widgets.borrow()[i].0.receivable(),
            None => Vec::new(),
        };

        // Add the widget pane's self events. These are default receivable events of the widget
        // organizer
        let mut rec = self.pane.receivable();
        rec.extend(wpes);
        rec
    }

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::Mouse(me) => {
                return self.capture_mouse_event(ctx, me);
            }
            Event::KeyCombo(ke) => {
                return self.capture_key_event(ctx, ke);
            }
            Event::ExternalMouse(me) => {
                return (false, self.send_external_mouse_event(ctx, me, None));
            }
            Event::Resize => {
                self.resize_event(ctx);
            }
            Event::Refresh => {}
            _ => {}
        }
        self.pane.receive_event(ctx, ev)
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        let mut rec = if p == Priority::UNFOCUSED {
            self.unselect_selected_widget_process_resp(ctx)
        } else {
            ReceivableEventChanges::default()
        };
        rec.concat(self.pane.change_priority(ctx, p));
        rec
    }

    // TODO can integrate in once trait upcasting is available
    // https://github.com/rust-lang/rust/issues/65991
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut chs = self.pane.drawing(ctx);

        let active_index = *self.active_widget_index.borrow();

        for (i, (w, loc)) in self.widgets.borrow().iter().enumerate() {
            // skip the active widget the will be drawn on the top after
            if Some(i) == active_index {
                continue;
            }

            if let Some(vis_loc) = ctx.visible_region {
                if !vis_loc.intersects_dyn_location_set(ctx, &loc.borrow()) {
                    continue;
                }
            }

            let mut ds = w.drawing(ctx);
            for d in &mut ds {
                d.update_colors_for_time_and_pos(ctx);
            }
            for mut d in ds {
                // adjust the location of the drawChPos relative to the WidgetPane
                d.adjust_by_dyn_location(ctx, &loc.borrow().l);
                // filter out chs that are outside of the WidgetPane bounds
                if d.y < ctx.s.height && d.x < ctx.s.width {
                    chs.push(d);
                }
            }
        }

        // lastly draw the active widget on top
        if let Some(i) = active_index {
            let ds = self.widgets.borrow()[i].0.drawing(ctx);
            let locs = self.widgets.borrow()[i].1.clone();

            for mut d in ds {
                // adjust the location of the drawChPos relative to the WidgetPane
                d.adjust_by_dyn_location(ctx, &locs.borrow().l);

                // filter out chs that are outside of the WidgetPane bounds
                if d.y < ctx.s.height && d.x < ctx.s.width {
                    chs.push(d);
                }
            }
        }

        chs
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn Parent>) {
        self.pane.set_upward_propagator(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.pane.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.pane.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.pane.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.pane.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.pane.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.pane.get_visible()
    }
}
