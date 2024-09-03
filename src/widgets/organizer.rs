use {
    super::{widget::RESP_DEACTIVATE, Selectability, Widget},
    crate::{
        Context, DrawChPos, Event, EventResponse, EventResponses, KeyPossibility, Keyboard as KB,
        Priority, ReceivableEventChanges, SclLocationSet,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Default)]
pub struct WidgetOrganizer {
    #[allow(clippy::type_complexity)]
    pub widgets: Vec<(Box<dyn Widget>, Rc<RefCell<SclLocationSet>>)>,
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

    pub fn add_widget(&mut self, w: Box<dyn Widget>) {
        let loc = w.get_scl_location_set();
        self.widgets.push((w, loc));
    }

    pub fn get_widget(&self, index: usize) -> Option<&Box<dyn Widget>> {
        self.widgets.get(index).map(|(w, _)| w)
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
    pub fn unselect_selected_widget(&mut self, ctx: &Context) -> EventResponses {
        if let Some(i) = self.active_widget_index {
            let resps = self.widgets[i]
                .0
                .set_selectability(ctx, Selectability::Ready);
            self.active_widget_index = None;
            resps
        } else {
            EventResponses::default()
        }
    }

    // deactivate all the Widgets and process the responses returning only the receivable event
    // changes. we need this for change_priority
    pub fn unselect_selected_widget_process_resp(
        &mut self, ctx: &Context,
    ) -> ReceivableEventChanges {
        if let Some(i) = self.active_widget_index {
            let mut resps = self.widgets[i]
                .0
                .set_selectability(ctx, Selectability::Ready);
            self.active_widget_index = None;
            self.process_widget_resps(ctx, &mut resps, i);
            // ignore all unprocessed responses besides receivable event changes
            resps.get_receivable_event_changes()
        } else {
            ReceivableEventChanges::default()
        }
    }

    pub fn process_widget_resps(
        &mut self, ctx: &Context, resps: &mut EventResponses, widget_index: usize,
    ) {
        let mut extend_resps = vec![];
        for resp in resps.0.iter_mut() {
            let mut modified_resp = None;
            match resp {
                //EventResponse::ExtraLocations(extra) => {
                //    // adjust extra locations to be relative to the given element
                //    let mut adj_extra_locs = Vec::new();
                //    let loc = self.widgets[widget_index].1.borrow();
                //    for mut l in extra.clone() {
                //        l.adjust_location_by(loc.l.start_x.clone(), loc.l.start_y.clone());
                //        adj_extra_locs.push(l.clone());
                //    }
                //    self.widgets[widget_index].1.borrow_mut().extra = adj_extra_locs;
                //}
                EventResponse::NewElement(new_el) => {
                    // adjust right click menu location to the widget
                    // location which made the request
                    let loc = self.widgets[widget_index].1.borrow();
                    let og_loc = new_el.borrow().get_scl_location_set();

                    debug!("og x: {:?}", og_loc.borrow().get_start_x(ctx));
                    debug!("og y: {:?}", og_loc.borrow().get_start_y(ctx));
                    debug!("adj x: {:?}", loc.get_start_x(ctx));
                    debug!("adj y: {:?}", loc.get_start_y(ctx));

                    new_el
                        .borrow()
                        .get_scl_location_set()
                        .borrow_mut()
                        .adjust_locations_by(loc.l.start_x.clone(), loc.l.start_y.clone());

                    let final_loc = new_el.borrow().get_scl_location_set();
                    debug!("final x: {:?}", final_loc.borrow().get_start_x(ctx));
                    debug!("final y: {:?}", final_loc.borrow().get_start_y(ctx));
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
        &mut self, ctx: &Context, old_index: Option<usize>, new_index: Option<usize>,
    ) -> EventResponses {
        if old_index == new_index {
            return EventResponses::default();
        }

        if let Some(new_i) = new_index {
            if self.widgets[new_i].0.get_selectability() == Selectability::Unselectable {
                return EventResponses::default();
            }
        }

        let mut resps = EventResponses::default();
        if let Some(old_i) = old_index {
            let mut resps_ = self.widgets[old_i]
                .0
                .set_selectability(ctx, Selectability::Ready);

            self.process_widget_resps(ctx, &mut resps_, old_i);
            resps.extend(resps_.0);
        }

        if let Some(new_i) = new_index {
            let mut resps_ = self.widgets[new_i]
                .0
                .set_selectability(ctx, Selectability::Selected);

            self.process_widget_resps(ctx, &mut resps_, new_i);
            self.active_widget_index = Some(new_i);
            resps.extend(resps_.0);
        }
        resps
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

    pub fn switch_to_next_widget(&mut self, ctx: &Context) -> EventResponses {
        self.switch_between_widgets(
            ctx,
            self.active_widget_index,
            self.next_ready_widget_index(self.active_widget_index),
        )
    }

    pub fn switch_to_prev_widget(&mut self, ctx: &Context) -> EventResponses {
        self.switch_between_widgets(
            ctx,
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
    pub fn capture_key_event(
        &mut self, ctx: &Context, ev: Vec<KeyPossibility>,
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

        if let Some(i) = self.active_widget_index {
            let (captured, mut resps) = self.widgets[i].0.receive_event(ctx, Event::KeyCombo(ev));
            self.process_widget_resps(ctx, &mut resps, i);
            return (captured, resps);
        }
        (false, EventResponse::default().into())
    }

    pub fn capture_mouse_event(
        &mut self,
        ctx: &Context,
        ev: crossterm::event::MouseEvent,
        //(captured, resp    )
    ) -> (bool, EventResponses) {
        let mut clicked = false;
        if let MouseEventKind::Up(MouseButton::Left) = ev.kind {
            clicked = true;
        }

        let mut most_front_z_index = i32::MAX; // lowest value is the most front
        let mut widget_index = None; // index of widget with most front z index
        let mut widget_loc = SclLocationSet::default();

        // find the widget with the most front z index
        for (i, (_, loc)) in self.widgets.iter().enumerate() {
            let loc = loc.borrow();
            if loc.contains(ctx, ev.column.into(), ev.row.into()) && loc.z < most_front_z_index {
                most_front_z_index = loc.z;
                widget_index = Some(i);
                widget_loc = loc.clone();
            }
        }

        let Some(widget_index) = widget_index else {
            if clicked {
                let resps = self.unselect_selected_widget(ctx);
                return (false, resps);
            }
            return (false, EventResponses::default());
        };

        let resps = if clicked {
            self.switch_between_widgets(ctx, self.active_widget_index, Some(widget_index))
        } else {
            EventResponses::default()
        };
        let ev_adj = widget_loc.l.adjust_mouse_event(ctx, &ev);
        let (captured, mut resps2) = self.widgets[widget_index]
            .0
            .receive_event(ctx, Event::Mouse(ev_adj));

        self.process_widget_resps(ctx, &mut resps2, widget_index);
        resps2.extend(resps.0);
        (captured, resps2)
    }

    pub fn resize_event(&mut self, ctx: &Context) {
        for (w, _loc) in &mut self.widgets {
            w.receive_event(ctx, Event::Resize);
            //*loc.borrow_mut() = w.get_scl_location_set().borrow().clone();
        }
    }

    // draws all the Widgets
    pub fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut out = Vec::new();

        for (i, (w, loc)) in self.widgets.iter().enumerate() {
            // skip the active widget the will be drawn on the top after
            if Some(i) == self.active_widget_index {
                continue;
            }

            let ds = w.drawing(ctx);
            for mut d in ds {
                // adjust the location of the drawChPos relative to the WidgetPane
                d.adjust_by_scl_location(ctx, &loc.borrow().l);
                // filter out chs that are outside of the WidgetPane bounds
                if d.y < ctx.s.height && d.x < ctx.s.width {
                    out.push(d);
                }
            }
        }

        // lastly draw the active widget on top
        if let Some(i) = self.active_widget_index {
            let ds = self.widgets[i].0.drawing(ctx);
            let locs = self.widgets[i].1.clone();

            for mut d in ds {
                // adjust the location of the drawChPos relative to the WidgetPane
                d.adjust_by_scl_location(ctx, &locs.borrow().l);

                // filter out chs that are outside of the WidgetPane bounds
                if d.y < ctx.s.height && d.x < ctx.s.width {
                    out.push(d);
                }
            }
        }

        out
    }
}
