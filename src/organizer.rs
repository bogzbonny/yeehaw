use {
    crate::{
        element::ReceivableEventChanges, prioritizer::EventPrioritizer, CommandEvent, Context,
        DrawChPos, DynLocation, DynLocationSet, Element, ElementID, Event, EventResponse,
        EventResponses, Priority, UpwardPropagator, ZIndex,
    },
    std::collections::HashMap,
    std::{cell::RefCell, rc::Rc},
};

// ElementOrganizer prioritizes and organizes all the elements contained
// within it
#[derive(Clone, Default)]
pub struct ElementOrganizer {
    pub els: Rc<RefCell<HashMap<ElementID, ElDetails>>>,
    pub prioritizer: Rc<RefCell<EventPrioritizer>>,
}

// element details
#[derive(Clone)]
pub struct ElDetails {
    pub el: Rc<RefCell<dyn Element>>,

    // NOTE we keep references to the location and visibility within the element
    // rather than just calling into tht element each time to reduce locking.
    pub loc: Rc<RefCell<DynLocationSet>>, // LocationSet of the element
    pub vis: Rc<RefCell<bool>>,           // whether the element is set to display
}

impl ElDetails {
    pub fn new(el: Rc<RefCell<dyn Element>>) -> Self {
        let loc = el.borrow().get_dyn_location_set().clone();
        let vis = el.borrow().get_visible().clone();
        Self { el, loc, vis }
    }

    pub fn set_visibility(&self, vis: bool) {
        *self.vis.borrow_mut() = vis;
    }

    pub fn set_location_set(&self, loc: DynLocationSet) {
        *self.loc.borrow_mut() = loc;
    }
}

impl ElementOrganizer {
    pub fn add_element(&self, el: Rc<RefCell<dyn Element>>, up: Option<Box<dyn UpwardPropagator>>) {
        // assign the new element id
        let el_id = el.borrow().id().clone();

        let z = el.borrow().get_dyn_location_set().borrow().z;

        // put it at the top of the z-dim (pushing everything else down))
        self.update_el_z_index(&el_id, z);

        let el_details = ElDetails::new(el.clone());
        self.els.borrow_mut().insert(el_id.clone(), el_details);

        // add the elements recievable events and commands to the prioritizer
        let receivable_evs = el.borrow().receivable();
        //debug!("add_element: receivable_evs: {:?}", receivable_evs);
        self.prioritizer
            .borrow_mut()
            .include(&el_id, &receivable_evs);

        // give the child element a reference to the parent (the up passed in as an
        // input)
        // NOTE: this is used in upwards propagation of changes to inputability
        // initiated by an element other than the parent (via this element organizer)
        // (ex: a sibling initiating a change to inputability, as opposed to this eo
        // passing an event to the child through ReceiveEventKeys)
        if let Some(up) = up {
            el.borrow_mut().set_upward_propagator(up);
        }
    }

    pub fn remove_element(&self, el_id: &ElementID) -> ReceivableEventChanges {
        self.els.borrow_mut().remove(el_id);
        let rm_evs = self.prioritizer.borrow_mut().remove_entire_element(el_id);
        ReceivableEventChanges::default().with_remove_evs(rm_evs)
    }

    // removes all elements from the element organizer
    pub fn clear_elements(&self) -> ReceivableEventChanges {
        self.els.borrow_mut().clear();
        let pes = self.receivable().drain(..).map(|(e, _)| e).collect();
        *self.prioritizer.borrow_mut() = EventPrioritizer::default();
        ReceivableEventChanges::default().with_remove_evs(pes)
    }

    // get_element_by_id returns the element registered under the given id in the eo
    pub fn get_element_details(&self, el_id: &ElementID) -> Option<ElDetails> {
        self.els.borrow().get(el_id).cloned()
    }

    // get_element_by_id returns the element registered under the given id in the eo
    //pub fn get_element_by_id(&self, el_id: &ElementID) -> Option<Rc<RefCell<dyn Element>>> {
    pub fn get_element(&self, el_id: &ElementID) -> Option<Rc<RefCell<dyn Element>>> {
        self.els.borrow().get(el_id).map(|ed| ed.el.clone())
    }

    pub fn get_location(&self, el_id: &ElementID) -> Option<DynLocationSet> {
        self.els
            .borrow()
            .get(el_id)
            .map(|ed| ed.loc.borrow().clone())
    }

    // get_el_at_pos returns the element at the given position
    pub fn get_element_details_at_pos(&self, ctx: &Context, x: i32, y: i32) -> Option<ElDetails> {
        for (_, details) in self.els.borrow().iter() {
            if details.loc.borrow().contains(ctx, x, y) {
                return Some(details.clone());
            }
        }
        None
    }

    // get_el_id_at_pos returns the element id at the given position
    pub fn get_el_id_at_pos(&self, ctx: &Context, x: i32, y: i32) -> Option<ElementID> {
        for (el_id, details) in self.els.borrow().iter() {
            if details.loc.borrow().contains(ctx, x, y) {
                return Some(el_id.clone());
            }
        }
        None
    }

    // update_el_primary_location updates the primary location of the element with the given id
    pub fn update_el_location_set(&self, el_id: ElementID, loc: DynLocationSet) {
        //self.locations.entry(el_id).and_modify(|l| (*l) = loc);
        self.els
            .borrow_mut()
            .entry(el_id)
            .and_modify(|ed| *ed.loc.borrow_mut() = loc);
    }

    // update_el_primary_location updates the primary location of the element with the given id
    pub fn update_el_primary_location(&self, el_id: ElementID, loc: DynLocation) {
        //self.locations.entry(el_id).and_modify(|l| l.l = loc);
        self.els
            .borrow_mut()
            .entry(el_id)
            .and_modify(|ed| ed.loc.borrow_mut().l = loc);
    }

    // updates the extra locations for the given element
    //pub fn update_extra_locations_for_el(
    pub fn update_el_extra_locations(&self, el_id: ElementID, extra_locations: Vec<DynLocation>) {
        self.els
            .borrow_mut()
            .entry(el_id)
            .and_modify(|ed| ed.loc.borrow_mut().extra = extra_locations);
    }

    // update_el_z_index updates the z-index of the element with the given id
    //
    // NOTE: if the given index is taken, the element currently filling that index
    // will be pushed further back in the z-dimension (i.e. its z-index will be
    // incremented)
    //
    // TRANSLATION: SetZIndexForElement set_z_index_for_element
    pub fn update_el_z_index(&self, el_id: &ElementID, z: ZIndex) {
        if let Some(details) = self.get_el_at_z_index(z) {
            self.increment_z_index_for_el(details);
        }
        self.els
            .borrow_mut()
            .entry(el_id.clone())
            .and_modify(|ed| ed.loc.borrow_mut().z = z);
    }

    // get_context_for_el_id returns the context for the element registered under the given id
    pub fn get_context_for_el(&self, higher_ctx: &Context, el_details: &ElDetails) -> Context {
        let size = el_details.loc.borrow().l.get_size(higher_ctx);
        let visible_region = if let Some(mut vr) = higher_ctx.visible_region {
            // make the visible region relative to the el
            let el_x = el_details.loc.borrow().l.get_start_x(higher_ctx) as u16;
            let el_y = el_details.loc.borrow().l.get_start_y(higher_ctx) as u16;
            vr.start_x = vr.start_x.saturating_sub(el_x);
            vr.end_x = vr.end_x.saturating_sub(el_x);
            vr.start_y = vr.start_y.saturating_sub(el_y);
            vr.end_y = vr.end_y.saturating_sub(el_y);
            Some(vr)
        } else {
            None
        };
        Context::new(size).with_visible_region(visible_region)
    }

    // Receivable returns all of the key combos and commands registered to this
    // element organizer, along with their priorities
    pub fn receivable(&self) -> Vec<(Event, Priority)> {
        let mut out = Vec::new();
        for details in self.els.borrow().values() {
            let pr_evs = details.el.borrow().receivable();
            out.extend(pr_evs);
        }
        out
    }

    // AllDrawing executes Drawing functions on all elements in the element
    // organizer.
    // A DrawChPos slice is returned and passed up the chain to the top of the CUI
    // element hierarchy.
    // NOTE: the elements are sorted by z-index, from lowest to highest (furthest
    // back to furthest forward) and then drawn in that order, such that the element
    // with the highest z-index is drawn last and thus is on top of all others in the
    // DrawChPos slice
    pub fn all_drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut out = Vec::new();
        let mut eoz: Vec<(ElementID, ElDetails)> = Vec::new();

        for (el_id, details) in self.els.borrow().iter() {
            eoz.push((el_id.clone(), details.clone()));
        }

        // sort z index from low to high
        eoz.sort_by(|a, b| a.1.loc.borrow().z.cmp(&b.1.loc.borrow().z));

        // draw elements in order from highest z-index to lowest
        for el_id_z in eoz {
            let details = self.get_element_details(&el_id_z.0).expect("impossible");
            if !*details.vis.borrow() {
                continue;
            }
            if let Some(vis_loc) = ctx.visible_region {
                if !vis_loc.intersects_dyn_location_set(ctx, &details.loc.borrow()) {
                    continue;
                }
            }

            let child_ctx = self.get_context_for_el(ctx, &el_id_z.1);
            let dcps = details.el.borrow().drawing(&child_ctx);
            for mut dcp in dcps {
                dcp.adjust_by_dyn_location(ctx, &details.loc.borrow().l);
                out.push(dcp);
            }
        }

        out
    }

    // write func to remove/add evCombos and commands from EvPrioritizer and
    // CommandPrioritizer, using the ReceivableEventChanges struct
    /*pub fn process_changes_to_inputability(*/
    pub fn process_receivable_event_changes(&self, el_id: &ElementID, ic: &ReceivableEventChanges) {
        self.prioritizer.borrow_mut().remove(el_id, &ic.remove);
        self.prioritizer.borrow_mut().include(el_id, &ic.add);
    }

    // Partially process the event response for whatever is possible to be processed
    // in the element organizer. Further processing may be required by the element
    // which owns this element organizer.
    pub fn partially_process_ev_resps(
        &self, _ctx: &Context, el_id: &ElementID, resps: &mut EventResponses,
    ) {
        let Some(details) = self.get_element_details(el_id) else {
            // TODO log error
            return;
        };

        for r in resps.0.iter_mut() {
            let mut modified_resp: Option<EventResponse> = None;
            match r {
                //EventResponse::ExtraLocations(extra) => {
                //    // adjust extra locations to be relative to the given element
                //    let mut adj_extra_locs = Vec::new();
                //    for mut l in extra.clone() {
                //        l.adjust_location_by(
                //            details.loc.borrow().l.start_x.clone(),
                //            details.loc.borrow().l.start_y.clone(),
                //        );
                //        adj_extra_locs.push(l.clone());
                //    }

                //    // update extra locations
                //    self.update_el_extra_locations(el_id.clone(), adj_extra_locs);
                //}
                EventResponse::NewElement(new_el) => {
                    // adjust the location of the window to be relative to the given element and adds the element
                    // to the element organizer
                    new_el
                        .borrow()
                        .get_dyn_location_set()
                        .borrow_mut()
                        .adjust_locations_by(
                            details.loc.borrow().l.start_x.clone(),
                            details.loc.borrow().l.start_y.clone(),
                        );
                    self.add_element(new_el.clone(), None);
                    modified_resp = Some(EventResponse::None);
                }
                EventResponse::Destruct => {
                    let ic = self.remove_element(el_id);
                    // NOTE no need to process the receivable event changes here,
                    // they've already been removed in the above call
                    modified_resp = Some(EventResponse::ReceivableEventChanges(ic));
                }
                EventResponse::ReceivableEventChanges(rec) => {
                    self.process_receivable_event_changes(el_id, rec);
                }
                _ => {}
            }
            if let Some(mr) = modified_resp {
                *r = mr;
            }
        }
    }

    // Replaces the element at the given ID with a new element
    //pub fn replace_el(
    //    &self, el_id: &ElementID, new_el: Rc<RefCell<dyn Element>>,
    //) -> ReceivableEventChanges {
    //    let mut ic = ReceivableEventChanges::default();

    //    let Some(old_details) = self.els.borrow_mut().remove(el_id) else {
    //        return ic;
    //    };
    //    let evs: Vec<Event> = old_details
    //        .el
    //        .borrow()
    //        .receivable()
    //        .drain(..)
    //        .map(|(e, _)| e)
    //        .collect();
    //    self.prioritizer.borrow_mut().remove(el_id, &evs);
    //    ic = ic.with_remove_evs(evs);

    //    let new_el_id = new_el.borrow().id().clone();
    //    let new_details = ElDetails::new(new_el.clone());
    //    new_details.set_location_set(old_details.loc.borrow().clone());
    //    new_details.set_visibility(*old_details.vis.borrow());
    //    self.els.borrow_mut().insert(new_el_id.clone(), new_details);

    //    // register all events of new element to the prioritizers
    //    let new_evs = new_el.borrow().receivable();
    //    self.prioritizer.borrow_mut().include(&new_el_id, &new_evs);
    //    ic.add_evs(new_evs);
    //    ic
    //}

    // key_events_process:
    // - determines the appropriate element to send key events to
    // - sends the event combo to the element
    // - processes changes to the elements receivable events
    pub fn key_events_process(
        &self, ctx: &Context, evs: Vec<crossterm::event::KeyEvent>,
    ) -> Option<(ElementID, EventResponses)> {
        // determine elementID to send events to
        let evs: Event = evs.into();
        let el_id = self.prioritizer.borrow().get_destination_el(&evs);

        let el_id = match el_id {
            Some(e) => e,
            None => {
                return None;
            }
        };

        // get element
        let el_details = self
            .get_element_details(&el_id)
            .expect("no element for destination id");

        // send EventKeys to element w/ context
        let child_ctx = self.get_context_for_el(ctx, &el_details);
        let (_, mut resps) = el_details.el.borrow_mut().receive_event(&child_ctx, evs);

        self.partially_process_ev_resps(ctx, &el_id, &mut resps);
        Some((el_id, resps))
    }

    // refresh does the following:
    // - updates prioritizers
    // - triggers a resize event in all children.
    // This essentially refreshes the state of the element organizer.
    //
    // NOTE: the refresh allows for less meticulous construction of the
    // main.go file. Elements can be added in whatever order, so long as
    // your_main_el.refresh() is called after all elements are added.
    pub fn refresh(&self, ctx: &Context) {
        // reset prioritizers
        *self.prioritizer.borrow_mut() = EventPrioritizer::default();

        // refresh all children
        for (_, details) in self.els.borrow().iter() {
            let el_ctx = self.get_context_for_el(ctx, details);
            let _ = details
                .el
                .borrow_mut()
                .receive_event(&el_ctx, Event::Refresh);
            let _ = details
                .el
                .borrow_mut()
                .receive_event(&el_ctx, Event::Resize);

            let pe = details.el.borrow().receivable();
            self.prioritizer
                .borrow_mut()
                .include(&details.el.borrow().id(), &pe)
        }

        //debug!("post-refresh prioritizer: {:?}", self.prioritizer.borrow());
    }

    // change_priority_for_el updates a child element (el_id) to a new priority. It does
    // this by asking the child element to return its registered events w/
    // priorities updated to a given priority.
    pub fn change_priority_for_el(
        &self, ctx: &Context, el_id: ElementID, pr: Priority,
    ) -> ReceivableEventChanges {
        let details = self
            .get_element_details(&el_id)
            .expect("no element for destination id"); // XXX something else

        let child_ctx = self.get_context_for_el(ctx, &details);

        // NOTE these changes are the changes for
        // THIS element organizer (not the child element)
        let changes = details.el.borrow_mut().change_priority(&child_ctx, pr);
        self.process_receivable_event_changes(&el_id, &changes);
        changes
    }

    // get_el_id_z_order_under_mouse returns a list of all Elements whose locations
    // include the position of the mouse event
    pub fn get_el_id_z_order_under_mouse(
        &self, ctx: &Context, ev: &crossterm::event::MouseEvent,
    ) -> Vec<(ElementID, ZIndex)> {
        let mut ezo: Vec<(ElementID, ZIndex)> = Vec::new();

        for (el_id, details) in self.els.borrow().iter() {
            if !*details.vis.borrow() {
                continue;
            }
            if details
                .loc
                .borrow()
                .contains(ctx, ev.column.into(), ev.row.into())
            {
                ezo.push((el_id.clone(), details.loc.borrow().z));
            }
        }
        ezo
    }

    // mouse_event_process :
    // - determines the appropriate element to send mouse events to
    // - sends the event to the element
    // - processes changes to the element's receivable events
    pub fn mouse_event_process(
        &self, ctx: &Context, ev: &crossterm::event::MouseEvent,
    ) -> Option<(ElementID, EventResponses)> {
        let eoz = self.get_el_id_z_order_under_mouse(ctx, ev);

        // get the highest-z element from the eoz list
        let max_z = eoz.iter().max_by_key(|(_, z)| *z)?;
        let el_id = max_z.0.clone();

        let details = self
            .get_element_details(&el_id)
            .expect("no element for destination id");
        let child_ctx = self.get_context_for_el(ctx, &details);

        // adjust event to the relative position of the element
        let ev_adj = details.loc.borrow().l.adjust_mouse_event(ctx, ev);

        // send mouse event to element
        let (_, mut ev_resps) = details
            .el
            .borrow_mut()
            .receive_event(&child_ctx, Event::Mouse(ev_adj));
        self.partially_process_ev_resps(ctx, &el_id, &mut ev_resps);

        // send the mouse event as an external event to all other elements
        // capture the responses
        let mut el_resps = Vec::new();
        for (el_id2, details2) in self.els.borrow().iter() {
            let child_ctx = self.get_context_for_el(ctx, details2); // XXX this is new make sure this doesn't mess stuff up (test menu)
            if *el_id2 == el_id {
                continue;
            }
            let (_, r) = details2
                .el
                .borrow_mut()
                .receive_event(&child_ctx, Event::ExternalMouse(*ev));
            el_resps.push((el_id2.clone(), r));
        }

        // combine the event responses from the elements that receive the event
        // and all the elements that receive an external event
        for (el_id2, mut resps) in el_resps {
            self.partially_process_ev_resps(ctx, &el_id2, &mut resps);
            ev_resps.extend(resps.0);
        }
        Some((el_id, ev_resps))
    }

    // sends the external mouse command to all elements in the organizer
    pub fn external_mouse_event_process(
        &self, ctx: &Context, ev: &crossterm::event::MouseEvent,
    ) -> EventResponses {
        let mut ev_resps = EventResponses::default();
        for (el_id, details) in self.els.borrow().iter() {
            let child_ctx = self.get_context_for_el(ctx, details);
            let (_, mut r) = details
                .el
                .borrow_mut()
                .receive_event(&child_ctx, Event::ExternalMouse(*ev));
            self.partially_process_ev_resps(ctx, el_id, &mut r);
            ev_resps.extend(r.0);
        }
        ev_resps
    }

    // get_el_id_at_z_index returns the element-id at the given z index, or None if
    // no element exists at the given z index
    pub fn get_el_at_z_index(&self, z: ZIndex) -> Option<ElDetails> {
        for (_, details) in self.els.borrow().iter() {
            if details.loc.borrow().z == z {
                return Some(details.clone());
            }
        }
        None
    }

    // increment_z_index_for_el_id increments the z-index of the element with the given id,
    // pushing it further back in the visual stack.
    //
    // NOTE: If an element already occupies the index that the given element is
    // attempting to occupy, the element occupying the index will be pushed back as
    // well.
    //
    // To move an element in the z-dimension, relative to other elements, use
    // UpdateZIndexForElID
    pub fn increment_z_index_for_el(&self, el_details: ElDetails) {
        let z = el_details.loc.borrow().z; // current z of element

        // check if element exists at next z-index
        if self.is_z_index_occupied(z + 1) {
            // recursively increment z-index of element at next z-index
            let details2 = self.get_el_at_z_index(z + 1).unwrap();
            self.increment_z_index_for_el(details2);
        }

        // increment z-index of the element
        self.els
            .borrow_mut()
            .entry(el_details.el.borrow().id().clone())
            .and_modify(|ed| ed.loc.borrow_mut().z = z + 1);
    }

    // is_z_index_occupied returns true if an element exists at the given z-index
    pub fn is_z_index_occupied(&self, z: ZIndex) -> bool {
        self.els
            .borrow()
            .values()
            .any(|details| details.loc.borrow().z == z)
    }

    // set_visibility_for_el sets the Visibility of the given element ID
    pub fn set_visibility_for_el(&self, el_id: ElementID, visible: bool) {
        self.els
            .borrow_mut()
            .entry(el_id)
            .and_modify(|ed| *ed.vis.borrow_mut() = visible);
    }

    // receive_command_event attempts to execute the given command
    //                                                       (captured, resp         )
    pub fn receive_command_event(&self, ctx: &Context, ev: CommandEvent) -> (bool, EventResponses) {
        let ev = Event::Command(ev);
        let Some(el_id) = self.prioritizer.borrow().get_destination_el(&ev) else {
            return (false, EventResponses::default());
        };

        let Some(details) = self.get_element_details(&el_id) else {
            // XXX TODO return error
            return (false, EventResponses::default());
        };
        let child_ctx = self.get_context_for_el(ctx, &details);
        let (captured, mut resps) = details.el.borrow_mut().receive_event(&child_ctx, ev);

        self.partially_process_ev_resps(ctx, &el_id, &mut resps);
        (captured, resps)
    }
}
