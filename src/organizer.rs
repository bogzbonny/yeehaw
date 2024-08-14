use {
    crate::{
        element::ReceivableEventChanges, prioritizer::EventPrioritizer, CommandEvent, Context,
        CreateWindow, DrawChPos, Element, ElementID, Event, EventResponse, Location, LocationSet,
        Priority, UpwardPropagator, ZIndex,
    },
    std::collections::HashMap,
    std::{cell::RefCell, rc::Rc},
};

// ElementOrganizer prioritizes and organizes all the elements contained
// within it
pub struct ElementOrganizer {
    // XXX TODO combine into one hashmap
    elements: HashMap<ElementID, Rc<RefCell<dyn Element>>>,
    locations: HashMap<ElementID, LocationSet>, // LocationSet of all of the elements contained
    visibility: HashMap<ElementID, bool>,       // whether the element is set to display

    pub prioritizer: EventPrioritizer,

    // TODO turn to debug assert statement?
    //
    // Panic if two children have registered the same ev/cmd at the same
    // priority. If false the event will be sent to the ev/cmd to which happens
    // to be first in the prioritizer
    panic_on_overload: bool,
}

impl Default for ElementOrganizer {
    fn default() -> Self {
        ElementOrganizer {
            elements: HashMap::new(),
            locations: HashMap::new(),
            visibility: HashMap::new(),
            prioritizer: EventPrioritizer::default(),
            panic_on_overload: true,
        }
    }
}

impl ElementOrganizer {
    pub fn add_element(
        &mut self, el: Rc<RefCell<dyn Element>>, up: Option<Rc<RefCell<dyn UpwardPropagator>>>,
        loc: LocationSet, vis: bool,
    ) {
        // assign the new element id
        let el_id = el.borrow().id().clone();

        // put it at the top of the z-dim (pushing everything else down))
        self.update_el_z_index(&el_id, 0);

        self.locations.insert(el_id.clone(), loc);
        self.elements.insert(el_id.clone(), el.clone());
        self.visibility.insert(el_id.clone(), vis);

        // add the elements recievable events and commands to the prioritizer
        let receivable_evs = el.borrow().receivable();
        self.prioritizer
            .include(&el_id, &receivable_evs, self.panic_on_overload);

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

    pub fn remove_element(&mut self, el_id: &ElementID) -> ReceivableEventChanges {
        self.elements.remove(el_id);
        self.locations.remove(el_id);
        self.visibility.remove(el_id);
        let rm_evs = self.prioritizer.remove_entire_element(el_id);
        ReceivableEventChanges::default().with_remove_evs(rm_evs)
    }

    // remove_all_elements removes all elements from the element organizer
    pub fn remove_all_elements(&mut self) -> ReceivableEventChanges {
        self.elements.clear();
        self.locations.clear();
        self.visibility.clear();
        let pes = self.receivable().drain(..).map(|(e, _)| e).collect();
        self.prioritizer = EventPrioritizer::default();
        ReceivableEventChanges::default().with_remove_evs(pes)
    }

    // get_element_by_id returns the element registered under the given id in the eo
    pub fn get_element_by_id(&self, el_id: &ElementID) -> Option<Rc<RefCell<dyn Element>>> {
        self.elements.get(el_id).cloned()
    }

    // must_get_locations returns the context for the element registered under the given id
    pub fn must_get_locations(&self, el_id: ElementID) -> &LocationSet {
        self.locations
            .get(&el_id)
            .expect("must get locations which doesn't exist")
    }

    // get_el_at_pos returns the element at the given position
    pub fn get_el_at_pos(&self, x: i32, y: i32) -> Option<Rc<RefCell<dyn Element>>> {
        for (id, locs) in &self.locations {
            if locs.contains(x, y) {
                return self.elements.get(id).cloned();
            }
        }
        None
    }

    // get_el_id_at_pos returns the element id at the given position
    pub fn get_el_id_at_pos(&self, x: i32, y: i32) -> Option<ElementID> {
        for (el_id, locs) in &self.locations {
            if locs.contains(x, y) {
                return Some(el_id.clone());
            }
        }
        None
    }

    // update_el_locations_by_id updates the locations of the element with the given id
    // to the given locations
    pub fn update_el_locations_by_id(&mut self, el_id: ElementID, locs: LocationSet) {
        self.locations.insert(el_id, locs);
    }

    // update_el_primary_location updates the primary location of the element with the given id
    pub fn update_el_primary_location(&mut self, el_id: ElementID, loc: Location) {
        self.locations.entry(el_id).and_modify(|l| l.l = loc);
    }

    // update_el_primary_location updates the primary location of the element with the given id
    pub fn update_el_location(&mut self, el_id: ElementID, loc: LocationSet) {
        self.locations.entry(el_id).and_modify(|l| (*l) = loc);
    }

    // TODO rename to consisten with above
    // updates the extra locations for the given element
    pub fn update_extra_locations_for_el(
        &mut self, el_id: &ElementID, extra_locations: Vec<Location>,
    ) {
        self.locations
            .entry(el_id.clone())
            .and_modify(|l| l.extra = extra_locations);
    }

    // update_el_z_index updates the z-index of the element with the given id
    //
    // NOTE: if the given index is taken, the element currently filling that index
    // will be pushed further back in the z-dimension (i.e. its z-index will be
    // incremented)
    //
    // TRANSLATION: SetZIndexForElement set_z_index_for_element
    pub fn update_el_z_index(&mut self, el_id: &ElementID, z: i32) {
        if self.is_z_index_occupied(z) {
            let id = self.get_el_id_at_z_index(z).unwrap(); // XXX shouldn't panic
            self.increment_z_index_for_el_id(id);
        }
        self.locations
            .entry(el_id.clone())
            .and_modify(|l| (l.z) = z);
    }

    // get_context_for_el_id returns the context for the element registered under the given id
    pub fn get_context_for_el_id(&self, el_id: &ElementID) -> Context {
        let size = self.locations[el_id].l.get_size();
        Context::new(size, self.visibility[el_id])
    }

    // GetHighestZIndex returns the highest z-index of all elements
    // NOTE: the highest z-index is the furthest back visually
    pub fn get_highest_z_index(&self) -> i32 {
        let mut highest = 0;
        for locs in self.locations.values() {
            if locs.z > highest {
                highest = locs.z;
            }
        }
        highest
    }

    // get_lowest_z_index returns the lowest z-index of all elements
    // NOTE: the lowest z-index is the furthest forward visually
    pub fn get_lowest_z_index(&self) -> i32 {
        let mut lowest = None;
        for locs in self.locations.values() {
            if let Some(l) = lowest {
                if locs.z < l {
                    lowest = Some(locs.z);
                }
            } else {
                lowest = Some(locs.z);
            }
        }
        lowest.unwrap_or(0)
    }

    // Receivable returns all of the key combos and commands registered to this
    // element organizer, along with their priorities
    pub fn receivable(&self) -> Vec<(Event, Priority)> {
        let mut out = Vec::new();
        for el in self.elements.values() {
            let pr_evs = el.borrow().receivable();
            out.extend(pr_evs);
        }
        out
    }

    // AllDrawing executes Drawing functions on all elements in the element
    // organizer.
    // A DrawChPos slice is returned and passed up the chain to the top of the CUI
    // element hierarchy.
    // NOTE: the elements are sorted by z-index, from highest to lowest (furthest
    // back to furthest forward) and then drawn in that order, such that the element
    // with the lowest z-index is drawn last and thus is on top of all others in the
    // DrawChPos slice
    pub fn all_drawing(&self) -> Vec<DrawChPos> {
        let mut out = Vec::new();
        let mut eoz: Vec<(ElementID, ZIndex)> = Vec::new();

        for el_id in self.elements.keys() {
            let z = self.locations[el_id].z;
            eoz.push((el_id.clone(), z));
        }

        // sort z index from high to low
        eoz.sort_by(|a, b| b.1.cmp(&a.1));

        // draw elements in order from highest z-index to lowest
        for el_id_z in eoz {
            let ctx = self.get_context_for_el_id(&el_id_z.0);
            let el = self.get_element_by_id(&el_id_z.0).unwrap();
            let dcps = el.borrow().drawing(&ctx);

            let locs = self.must_get_locations(el_id_z.0);

            for mut dcp in dcps {
                dcp.adjust_by_location(&locs.l);
                out.push(dcp);
            }
        }

        out
    }

    // write func to remove/add evCombos and commands from EvPrioritizer and
    // CommandPrioritizer, using the ReceivableEventChanges struct
    /*pub fn process_changes_to_inputability(*/
    pub fn process_receivable_event_changes(
        &mut self, el_id: &ElementID, ic: &ReceivableEventChanges,
    ) {
        self.prioritizer.remove(el_id, &ic.remove);
        self.prioritizer
            .include(el_id, &ic.add, self.panic_on_overload);
    }

    // Partially process the event response for whatever is possible to be processed
    // in the element organizer. Further processing may be required by the element
    // which owns this element organizer.
    pub fn partially_process_ev_resp(
        &mut self, el_id: &ElementID, mut r: EventResponse,
    ) -> EventResponse {
        // replace this entire element
        if let Some(repl) = r.replacement {
            let ctx = self.get_context_for_el_id(el_id);
            self.replace_el(el_id, repl);
            r.replacement = None;

            // resize replacement
            // TODO may not be neccessary. Explore further w/ fixes to resizing
            let el = self.get_element_by_id(el_id).unwrap(); // XXX remove unwrap?? use expect here??
            el.borrow_mut().receive_event(&ctx, Event::Resize);
        }

        if let Some(ref elr) = r.extra_locations {
            // adjust extra locations to be relative to the given element
            let loc = self.locations[el_id].clone();
            let mut adj_extra_locs = Vec::new();
            for mut l in elr.extra_locs.clone() {
                l.adjust_location_by(loc.l.start_x, loc.l.start_y);
                adj_extra_locs.push(l.clone());
            }

            // update extra locations
            self.update_extra_locations_for_el(el_id, adj_extra_locs);
        }

        let window = r.window.take();
        if let Some(window) = window {
            self.process_create_window(el_id, window);
        }

        if r.destruct {
            let ic = self.remove_element(el_id);

            r.concat_receivable_event_changes(ic);
            r.destruct = false;
        }

        r
    }

    // ProcessCreateWindow adjusts the location of the window to be relative to the
    // given element and adds the element to the element organizer
    pub fn process_create_window(&mut self, el_id: &ElementID, mut cw: CreateWindow) {
        // adjust location of window to be relative to the given element
        let loc = self.locations[el_id].clone(); // location of element
        cw.loc.l.adjust_location_by(loc.l.start_x, loc.l.start_y);

        self.add_element(cw.el, None, cw.loc, true);
    }

    // Replaces the element at the given ID with a new element
    pub fn replace_el(
        &mut self, el_id: &ElementID, new_el: Rc<RefCell<dyn Element>>,
    ) -> ReceivableEventChanges {
        let mut ic = ReceivableEventChanges::default();

        // register new element to organizer under ID of old element
        if let Some(old_el) = self.elements.insert(el_id.clone(), new_el.clone()) {
            let evs: Vec<Event> = old_el
                .borrow()
                .receivable()
                .drain(..)
                .map(|(e, _)| e)
                .collect();
            self.prioritizer.remove(el_id, &evs);
            ic = ic.with_remove_evs(evs);
        }

        // register all events of new element to the prioritizers
        let new_evs = new_el.borrow().receivable();
        self.prioritizer
            .include(el_id, &new_evs, self.panic_on_overload);
        ic.add_evs(new_evs);

        ic
    }

    // key_events_process:
    // - determines the appropriate element to send key events to
    // - sends the event combo to the element
    // - processes changes to the elements receivable events
    pub fn key_events_process(
        &mut self, evs: Vec<crossterm::event::KeyEvent>,
    ) -> Option<(ElementID, EventResponse)> {
        // determine elementID to send events to
        let evs: Event = evs.into();
        let el_id = self.prioritizer.get_destination_el(&evs)?;

        // get element
        let el = self
            .get_element_by_id(&el_id)
            .expect("no element for destination id");

        // send EventKeys to element w/ context
        let ctx = self.get_context_for_el_id(&el_id);
        let (_, r) = el.borrow_mut().receive_event(&ctx, evs);
        let r = self.partially_process_ev_resp(&el_id, r);

        if let Some(changes) = r.get_receivable_event_changes() {
            self.process_receivable_event_changes(&el_id, &changes);
        }

        Some((el_id, r))
    }

    // refresh does the following:
    // - updates prioritizers
    // - triggers a resize event in all children.
    // This essentially refreshes the state of the element organizer.
    //
    // NOTE: the refresh allows for less meticulous construction of the
    // main.go file. Elements can be added in whatever order, so long as
    // your_main_el.refresh() is called after all elements are added.
    pub fn refresh(&mut self) {
        // reset prioritizers
        self.prioritizer = EventPrioritizer::default();

        // refresh all children
        for (el_id, el) in self.elements.iter() {
            let el_ctx = self.get_context_for_el_id(el_id);
            el.borrow_mut().receive_event(&el_ctx, Event::Refresh);
            el.borrow_mut().receive_event(&el_ctx, Event::Resize);
        }
    }

    // change_priority_for_el updates a child element (el_id) to a new priority. It does
    // this by asking the child element to return its registered events w/
    // priorities updated to a given priority.
    pub fn change_priority_for_el(
        &mut self, el_id: ElementID, pr: Priority,
    ) -> ReceivableEventChanges {
        let el = self
            .get_element_by_id(&el_id)
            .expect("no element for destination id"); // XXX something else

        let ctx = self.get_context_for_el_id(&el_id);

        // NOTE these changes are the changes for
        // THIS element organizer (not the child element)
        let changes = el.borrow_mut().change_priority(&ctx, pr);
        self.process_receivable_event_changes(&el_id, &changes);
        changes
    }

    // get_el_id_z_order_under_mouse returns a list of all Elements whose locations
    // include the position of the mouse event
    pub fn get_el_id_z_order_under_mouse(
        &self, ev: &crossterm::event::MouseEvent,
    ) -> Vec<(ElementID, ZIndex)> {
        let mut ezo: Vec<(ElementID, ZIndex)> = Vec::new();

        for (el_id, _) in self.elements.iter() {
            let ctx = self.get_context_for_el_id(el_id);
            let locs = &self.locations[el_id];
            if !ctx.visible {
                continue;
            }

            let Some(z) = locs.get_z_index_for_point(ev.column.into(), ev.row.into()) else {
                continue;
            };

            ezo.push((el_id.clone(), z));
        }
        ezo
    }

    // mouse_event_process :
    // - determines the appropriate element to send mouse events to
    // - sends the event to the element
    // - processes changes to the element's receivable events
    pub fn mouse_event_process(
        &mut self, ev: &crossterm::event::MouseEvent,
    ) -> Option<(ElementID, EventResponse)> {
        let mut eoz = self.get_el_id_z_order_under_mouse(ev);
        if eoz.is_empty() {
            return None;
        }

        // get highest z element that falls under mouse event

        // sort z index from high to low
        eoz.sort_by(|a, b| b.1.cmp(&a.1));
        let el_id = eoz[0].0.clone();
        let el = self
            .get_element_by_id(&el_id)
            .expect("no element for destination id");
        let locs = &self.locations[&el_id];
        let ctx = self.get_context_for_el_id(&el_id);

        // adjust event to the relative position of the element
        let ev_adj = locs.l.adjust_mouse_event(ev);

        // send mouse event to element
        let (_, ev_resp) = el.borrow_mut().receive_event(&ctx, Event::Mouse(ev_adj));
        if let Some(changes) = ev_resp.get_receivable_event_changes() {
            self.process_receivable_event_changes(&el_id, &changes);
        }
        let mut ev_resp = self.partially_process_ev_resp(&el_id, ev_resp);

        // move element to top of z-dim if primary click
        // TODO why... why do we do this here?!
        if let crossterm::event::MouseEventKind::Up(button) = ev_adj.kind {
            if button == crossterm::event::MouseButton::Left {
                self.update_el_z_index(&el_id, 0);
            }
        }

        // send the mouse event as an external event to all other elements
        // capture the responses
        let mut resps = Vec::new();
        for (el_id2, el) in self.elements.iter() {
            if *el_id2 == el_id {
                continue;
            }
            let (_, r) = el.borrow_mut().receive_event(&ctx, Event::Mouse(*ev));
            resps.push((el_id2.clone(), r));
        }

        // combine the event responses from the elements that receive the event
        // and all the elements that receive an external event
        for (el_id2, r) in resps {
            if let Some(changes) = r.get_receivable_event_changes() {
                self.process_receivable_event_changes(&el_id2, &changes);
            }
            let r = self.partially_process_ev_resp(&el_id2, r);

            // combine the receivable-event changes, all other external responses are
            // ignored. TODO explain why.

            if let Some(re) = r.get_receivable_event_changes() {
                ev_resp.concat_receivable_event_changes(re);
            }
        }

        Some((el_id, ev_resp))
    }

    // get_el_id_at_z_index returns the element-id at the given z index, or None if
    // no element exists at the given z index
    pub fn get_el_id_at_z_index(&self, z: ZIndex) -> Option<ElementID> {
        for (el_id, loc) in self.locations.iter() {
            if loc.z == z {
                return Some(el_id.clone());
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
    pub fn increment_z_index_for_el_id(&mut self, el_id: ElementID) {
        let z = self.locations[&el_id].z; // current z of element

        // check if element exists at next z-index
        if self.is_z_index_occupied(z + 1) {
            // recursively increment z-index of element at next z-index
            let id = self.get_el_id_at_z_index(z + 1).unwrap();
            self.increment_z_index_for_el_id(id);
        }

        // increment z-index of the element
        self.locations.entry(el_id).and_modify(|l| (l.z) = z + 1);
    }

    // is_z_index_occupied returns true if an element exists at the given z-index
    pub fn is_z_index_occupied(&self, z: ZIndex) -> bool {
        self.locations.values().any(|locs| locs.z == z)
    }

    // set_visibility_for_el sets the Visibility of the given element ID
    pub fn set_visibility_for_el(&mut self, el_id: ElementID, visible: bool) {
        self.visibility.insert(el_id, visible);
    }

    // receive_command_event attempts to execute the given command
    //                                                       (captured, resp         )
    pub fn receive_command_event(&mut self, ev: CommandEvent) -> (bool, EventResponse) {
        let ev = Event::Command(ev);
        let Some(el_id) = self.prioritizer.get_destination_el(&ev) else {
            return (false, EventResponse::default());
        };

        let Some(el) = self.get_element_by_id(&el_id) else {
            // XXX TODO return error
            return (false, EventResponse::default());
        };
        let ctx = self.get_context_for_el_id(&el_id);

        let (captured, resp) = el.borrow_mut().receive_event(&ctx, ev);
        if let Some(changes) = resp.get_receivable_event_changes() {
            self.process_receivable_event_changes(&el_id, &changes);
        }
        let resp = self.partially_process_ev_resp(&el_id, resp);

        (captured, resp)
    }
}
