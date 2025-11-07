use {
    crate::{
        Context, DrawAction, DrawCh, DrawRegion, DrawUpdate, DynLocation, DynLocationSet, Element,
        ElementID, Event, EventResponse, EventResponses, Keyboard, MouseEvent, Parent,
        ReceivableEvent, ReceivableEvents, ZIndex,
    },
    rayon::prelude::*,
    std::collections::HashMap,
    std::{cell::RefCell, rc::Rc},
};

/// TODO description
#[derive(Clone, Default)]
pub struct ElementOrganizer {
    pub els: Rc<RefCell<HashMap<ElementID, ElDetails>>>,

    #[allow(clippy::type_complexity)]
    /// the last draw details for each element
    ///                                              (location, visibility, overflow)
    last_draw_details: Rc<RefCell<Vec<(ElementID, (DynLocationSet, bool, bool))>>>,

    /// the queue of elements to be visually removed on the next draw
    /// NOTE the elements will already be removed from the els and prioritizer, however
    /// we wait until next draw to propogate messages to clear those elements from the screen
    removed_element_queue: Rc<RefCell<Vec<ElementID>>>,
}

/// element details
#[derive(Clone)]
pub struct ElDetails {
    pub el: Box<dyn Element>,

    /// NOTE we keep references to the location and visibility within the element
    /// rather than just calling into tht element each time to reduce locking.
    /// LocationSet of the element
    pub loc: Rc<RefCell<DynLocationSet>>,
    /// whether the element is set to display
    pub vis: Rc<RefCell<bool>>,

    /// whether the element is allowed to overflow its the context
    pub overflow: Rc<RefCell<bool>>,
}

impl ElDetails {
    pub fn new(el: Box<dyn Element>) -> Self {
        let loc = el.get_ref_cell_dyn_location_set().clone();
        let vis = el.get_ref_cell_visible();
        let overflow = el.get_ref_cell_overflow();
        Self {
            el,
            loc,
            vis,
            overflow,
        }
    }

    pub fn set_visibility(&self, vis: bool) {
        *self.vis.borrow_mut() = vis;
    }

    pub fn set_location_set(&self, loc: DynLocationSet) {
        *self.loc.borrow_mut() = loc;
    }
}

impl ElementOrganizer {
    pub fn add_element(&self, el: Box<dyn Element>, parent: Option<Box<dyn Parent>>) {
        // assign the new element id
        let el_id = el.id().clone();

        let z = el.get_dyn_location_set().z;

        // put it at the top of the z-dim (pushing everything else down))
        self.update_el_z_index(&el_id, z);

        // give the child element a reference to the parent (the up passed in as an
        // input)
        // NOTE: this is used in upwards propagation of changes to inputability
        // initiated by an element other than the parent (via this element organizer)
        // (ex: a sibling initiating a change to inputability, as opposed to this eo
        // passing an event to the child through ReceiveEventKeys)
        if let Some(parent) = parent {
            el.set_parent(parent);
        }

        let el_details = ElDetails::new(el);
        self.els.borrow_mut().insert(el_id.clone(), el_details);
    }

    pub fn remove_element(&self, el_id: &ElementID) {
        self.last_draw_details
            .borrow_mut()
            .retain(|(id, _)| id != el_id);
        self.removed_element_queue.borrow_mut().push(el_id.clone());
        self.els.borrow_mut().remove(el_id);
    }

    /// removes all elements from the element organizer
    pub fn clear_elements(&self) {
        self.removed_element_queue.borrow_mut().extend(
            self.last_draw_details
                .borrow_mut()
                .drain(..)
                .map(|(id, _)| id),
        );
        self.els.borrow_mut().clear();
    }

    /// hide_element is similar to remove_element, but does not remove the element from the element
    /// organizer, thus the hidden element is still able to receive certain events such as exit or
    /// resize. The visibility of the element is set to false.
    pub fn hide_element(&self, el_id: &ElementID) {
        let Some(details) = self.get_element_details(el_id) else {
            return;
        };
        details.set_visibility(false);
        details.el.set_focused(false);
    }

    /// unhide_element is used to unhide an element that was previously hidden. It sets the
    /// visibility of the element to true
    pub fn unhide_element(&self, el_id: &ElementID) {
        let Some(details) = self.get_element_details(el_id) else {
            return;
        };
        details.set_visibility(true);
        details.el.set_focused(true);
    }

    /// get_element_by_id returns the element registered under the given id in the eo
    pub fn get_element_details(&self, el_id: &ElementID) -> Option<ElDetails> {
        self.els.borrow().get(el_id).cloned()
    }

    /// get_element_by_id returns the element registered under the given id in the eo
    pub fn get_element(&self, el_id: &ElementID) -> Option<Box<dyn Element>> {
        self.els.borrow().get(el_id).map(|ed| ed.el.clone())
    }

    /// get_element_by_id returns the element registered under the given id in the eo
    pub fn get_element_attribute(&self, el_id: &ElementID, key: &str) -> Option<Vec<u8>> {
        self.els
            .borrow()
            .get(el_id)
            .map(|ed| ed.el.get_attribute(key))?
    }

    pub fn get_location(&self, el_id: &ElementID) -> Option<DynLocationSet> {
        self.els
            .borrow()
            .get(el_id)
            .map(|ed| ed.loc.borrow().clone())
    }

    /// get_el_at_pos returns the element at the given position
    pub fn get_element_details_at_pos(&self, dr: &DrawRegion, x: i32, y: i32) -> Option<ElDetails> {
        for (_, details) in self.els.borrow().iter() {
            if details.loc.borrow().contains(dr, x, y) {
                return Some(details.clone());
            }
        }
        None
    }

    /// get_el_id_at_pos returns the element id at the given position
    pub fn get_el_id_at_pos(&self, dr: &DrawRegion, x: i32, y: i32) -> Option<ElementID> {
        for (el_id, details) in self.els.borrow().iter() {
            if details.loc.borrow().contains(dr, x, y) {
                return Some(el_id.clone());
            }
        }
        None
    }

    pub fn set_all_visibility(&self, vis: bool) {
        for details in self.els.borrow().values() {
            details.set_visibility(vis);
        }
    }

    /// update_el_primary_location updates the primary location of the element with the given id
    pub fn update_el_location_set(&self, el_id: ElementID, loc: DynLocationSet) {
        self.els
            .borrow_mut()
            .entry(el_id)
            .and_modify(|ed| *ed.loc.borrow_mut() = loc);
    }

    /// update_el_primary_location updates the primary location of the element with the given id
    pub fn update_el_primary_location(&self, el_id: ElementID, loc: DynLocation) {
        self.els
            .borrow_mut()
            .entry(el_id)
            .and_modify(|ed| ed.loc.borrow_mut().l = loc);
    }

    /// updates the extra locations for the given element
    ///pub fn update_extra_locations_for_el(
    pub fn update_el_extra_locations(&self, el_id: ElementID, extra_locations: Vec<DynLocation>) {
        self.els
            .borrow_mut()
            .entry(el_id)
            .and_modify(|ed| ed.loc.borrow_mut().extra = extra_locations);
    }

    /// update_el_z_index updates the z-index of the element with the given id
    ///
    /// NOTE: if the given index is taken, the element currently filling that index
    /// will be pushed further back in the z-dimension (i.e. its z-index will be
    /// incremented)
    pub fn update_el_z_index(&self, el_id: &ElementID, z: ZIndex) {
        if let Some(details) = self.get_el_at_z_index(z) {
            self.increment_z_index_for_el(details);
        }
        self.els
            .borrow_mut()
            .entry(el_id.clone())
            .and_modify(|ed| ed.loc.borrow_mut().z = z);
    }

    pub fn get_greatest_z_index(&self) -> (ElementID, ZIndex) {
        let mut max_z = (ElementID::default(), 0);
        for (el_id, details) in self.els.borrow().iter() {
            if details.loc.borrow().z > max_z.1 {
                max_z = (el_id.clone(), details.loc.borrow().z);
            }
        }
        max_z
    }

    /// brings the element at the provided id to the top of the z-index stack
    pub fn set_el_to_top(&self, el_id: &ElementID) {
        let (el_id_, z) = self.get_greatest_z_index();
        if el_id == &el_id_ {
            // already at the top!
            return;
        }
        self.update_el_z_index(el_id, z + 1);
    }

    /// AllDrawing executes Drawing functions on all elements in the element
    /// organizer.
    /// A DrawChPos slice is returned and passed up the chain to the top of the TUI
    /// element hierarchy.
    /// NOTE: the elements are sorted by z-index, from lowest to highest (furthest
    /// back to furthest forward) and then drawn in that order, such that the element
    /// with the highest z-index is drawn last and thus is on top of all others in the
    /// DrawChPos slice
    /// if force update is set to true then all elements will be drawn, regardless of
    /// if they have changed since the last draw
    pub fn all_drawing_updates(
        &self, ctx: &Context, dr: &DrawRegion, force_update: bool,
    ) -> Vec<DrawUpdate> {
        let mut eoz: Vec<(ElementID, ElDetails)> = Vec::new();

        for (el_id, details) in self.els.borrow().iter() {
            eoz.push((el_id.clone(), details.clone()));
        }

        // sort z index from low to high
        eoz.sort_by(|a, b| a.1.loc.borrow().z.cmp(&b.1.loc.borrow().z));

        let mut updates = self
            .removed_element_queue
            .borrow_mut()
            .drain(..)
            .map(|el_id| DrawUpdate::clear_all_at_sub_id(vec![el_id]))
            .collect::<Vec<DrawUpdate>>();

        // draw elements in order from highest z-index to lowest
        for el_id_z in eoz {
            let details = self.get_element_details(&el_id_z.0).expect("impossible");

            // set force update if vis or location has changed since last draw
            let mut force_update = force_update;
            let mut push_update = false;
            if let Some((_, (ref mut last_loc, ref mut last_vis, ref mut last_overflow))) = self
                .last_draw_details
                .borrow_mut()
                .iter_mut()
                .find(|(el_id, _)| el_id == &el_id_z.0)
            {
                if last_loc != &*details.loc.borrow()
                    || last_vis != &*details.vis.borrow()
                    || last_overflow != &*details.overflow.borrow()
                {
                    force_update = true;
                }

                // update the last draw details
                *last_loc = details.loc.borrow().clone();
                *last_vis = *details.vis.borrow();
                *last_overflow = *details.overflow.borrow();
            } else {
                push_update = true
            }
            if push_update {
                // update the last draw details
                self.last_draw_details.borrow_mut().push((
                    el_id_z.0.clone(),
                    (
                        details.loc.borrow().clone(),
                        *details.vis.borrow(),
                        *details.overflow.borrow(),
                    ),
                ));
            }

            let mut vis = *details.vis.borrow();
            if vis {
                if let Some(vis_loc) = dr.visible_region {
                    vis = vis_loc.intersects_dyn_location_set(dr, &details.loc.borrow());
                }
            }
            if !vis {
                updates.push(DrawUpdate::clear_all_at_sub_id(vec![el_id_z.0]));
                continue;
            }

            let child_dr = dr.child_region(&el_id_z.1.loc.borrow().l);
            let mut el_upds = details.el.drawing(ctx, &child_dr, force_update);

            for mut el_upd in el_upds.drain(..) {
                // prepend the element_id to the DrawUpdate
                el_upd.prepend_id(el_id_z.0.clone(), el_id_z.1.loc.borrow().z);

                match el_upd.action {
                    DrawAction::ClearAll => {}
                    DrawAction::Remove => {}
                    DrawAction::Update(ref mut dcps) | DrawAction::Extend(ref mut dcps) => {
                        //let mut dcps = details.el.drawing(&child_ctx);
                        let l = details.loc.borrow().l.clone();
                        let s = dr.size;
                        let child_s = child_dr.size;

                        let mut start_x = l.get_start_x_from_size(s);
                        let mut start_y = l.get_start_y_from_size(s);
                        // check for overflow
                        if start_x < 0 {
                            start_x = 0;
                        }
                        if start_y < 0 {
                            start_y = 0;
                        }

                        // NOTE this is a computational bottleneck
                        // currently using rayon for parallelization
                        if *details.overflow.borrow() {
                            dcps.par_iter_mut().for_each(|dcp| {
                                dcp.set_draw_size_offset_colors(child_s, start_x, start_y);
                                dcp.x += start_x as u16;
                                dcp.y += start_y as u16;
                            });
                        } else {
                            dcps.par_iter_mut().for_each(|dcp| {
                                dcp.set_draw_size_offset_colors(child_s, start_x, start_y);
                                if dcp.x >= child_s.width || dcp.y >= child_s.height {
                                    // it'd be better to delete, but we can't delete from a parallel iterator
                                    // also using a filter here its slower that this
                                    dcp.ch = DrawCh::skip();
                                    (dcp.x, dcp.y) = (0, 0);
                                } else {
                                    dcp.x += start_x as u16;
                                    dcp.y += start_y as u16;
                                }
                            });
                        }
                    }
                }
                updates.push(el_upd);
            }
        }
        updates
    }

    /// Partially process the event response for whatever is possible to be processed
    /// in the element organizer. Further processing may be required by the element
    /// which owns this element organizer.
    ///
    /// NOTE this function modifies the event responses in place
    #[allow(clippy::borrowed_box)]
    pub fn partially_process_ev_resps(
        &self, ctx: &Context, el_id: &ElementID, resps: &mut EventResponses,
        parent: &Box<dyn Parent>,
    ) {
        let Some(details) = self.get_element_details(el_id) else {
            // NOTE this can happen due to timing issues if an event is sent to an element
            // that is then removed before the event is processed
            return;
        };

        let mut extend_resps = EventResponses::default();
        for r in resps.0.iter_mut() {
            match r {
                EventResponse::None => {}
                EventResponse::Quit => {}
                EventResponse::Custom(_, _) => {}
                EventResponse::Move(_) => {}
                EventResponse::Resize(_) => {}
                EventResponse::Destruct => {
                    // send down an exit event to the element about to be destroyed
                    let _ = details.el.receive_event(ctx, Event::Exit);
                    self.remove_element(el_id);
                    *r = EventResponse::None;
                }
                EventResponse::BringToFront => {
                    self.set_el_to_top(el_id);
                    *r = EventResponse::None;
                }
                EventResponse::UnfocusOthers => {
                    for (el_id_, _) in self.els.borrow().iter() {
                        if el_id_ == el_id {
                            continue;
                        }
                        self.change_focused_for_el(el_id_, false);
                    }
                    // NOTE continue to propogate the focus event upwards
                }
                EventResponse::Focus => {
                    self.change_focused_for_el(el_id, true);
                    // NOTE continue to propogate the focus event upwards
                }
                EventResponse::NewElement(new_el, ref mut new_el_resps) => {
                    // adjust the location of the window to be relative to the given element and adds the element
                    // to the element organizer
                    let mut ls = new_el.get_dyn_location_set().clone();
                    ls.adjust_locations_by(
                        details.loc.borrow().l.start_x.clone(),
                        details.loc.borrow().l.start_y.clone(),
                    );
                    new_el.set_dyn_location_set(ls);
                    self.add_element(new_el.clone(), Some(parent.clone()));
                    if let Some(new_el_resps) = new_el_resps {
                        self.partially_process_ev_resps(ctx, &new_el.id(), new_el_resps, parent);
                        extend_resps.0.extend(new_el_resps.drain(..));
                    }
                    *r = EventResponse::None;
                }
            }
        }
        resps.0.extend(extend_resps.drain(..));
    }

    /// can the element receive the event provided
    pub fn can_receive(&self, ev: &Event) -> bool {
        for details in self.els.borrow().values() {
            if details.el.can_receive(ev) {
                return true;
            }
        }
        false
    }

    pub fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>> {
        let mut rec = Vec::new();
        for (_, details) in self.els.borrow().iter() {
            let rec_ = details.el.receivable();
            rec.extend(rec_);
        }
        rec
    }

    pub fn event_process(
        &self, ctx: &Context, ev: Event, parent: Box<dyn Parent>,
    ) -> (bool, EventResponses) {
        let (captured, resps) = match ev {
            Event::KeyCombo(_) | Event::Custom(_, _) => {
                let (el_id, resps) = self.routed_event_process(ctx, ev, parent);
                (el_id.is_some(), resps)
            }
            Event::Mouse(me) => {
                let (el_id, resps) = self.mouse_event_process(ctx, &me, parent);
                (el_id.is_some(), resps)
            }
            Event::ExternalMouse(me) => {
                // send the mouse event to all the children
                let resp = self.external_mouse_event_process(ctx, &me, parent);
                (false, resp) // never capture
            }
            Event::Initialize => {
                let resps = self.initialize(ctx, parent);
                (false, resps) // never capture
            }
            Event::Exit | Event::Resize => self.propogate_event_to_all(ctx, ev, parent),
        };

        // TODO uncomment/fix
        // ensure_no_duplicate_priorities(parent.get_id);

        (captured, resps)
    }

    //// check for priority overloading.
    //// Panic if two children have registered the same ev/cmd at the same priority
    //// (excluding Unfocused). If false the event will be sent to the ev/cmd to which
    //// happens to be first in the prioritizer
    //pub fn ensure_no_duplicate_priorities(&self, _parent: &ElementID) {
    //    #[cfg(debug_assertions)]
    //    for pe in self.0.iter() {
    //        if pe.priority != Priority::Unfocused {
    //            if let Some(existing_id) = self.get_priority_ev_id(&(pe.event.clone(), pe.priority))
    //            {
    //                if existing_id != *pe.id {
    //                    panic!(
    //                        "EvPrioritizer found at least 2 events registered to different \
    //                         elements with the same priority for parent {_parent}. \
    //                         \n\tid1: {existing_id} \
    //                         \n\tid2: {}\n\tpr: {}\n\tev: {:?}",
    //                        pe.id, pe.priority, pe.event
    //                    )
    //                }
    //            }
    //        }
    //    }
    //}

    /// routed_event_process:
    /// - determines the appropriate element to send the event to then sends the event
    ///    - if the event isn't captured then send it to the next element able to receive this event
    /// - partially processes changes to the elements receivable events
    ///
    /// NOTE elements may choose to not capture events in order to continue sending the
    ///      event to the next element in the chain
    pub fn routed_event_process(
        &self, ctx: &Context, ev: Event, parent: Box<dyn Parent>,
    ) -> (Option<ElementID>, EventResponses) {
        // determine element_id to send events to
        let el_ids = self.get_destination_el(&ev);

        let mut resps = EventResponses::default();
        let mut capturing_el_id = None;
        for el_id in el_ids {
            let el_details = self
                .get_element_details(&el_id)
                .expect("no element for destination id in routed_event_process");

            let (captured, mut resps_) = el_details.el.receive_event(ctx, ev.clone());
            self.partially_process_ev_resps(ctx, &el_id, &mut resps_, &parent);
            resps.0.extend(resps_.drain(..));

            if captured {
                capturing_el_id = Some(el_id);
                break;
            }
        }
        (capturing_el_id, resps)
    }

    /// GetDestinationEl returns the id of the element that should
    /// receive the given event.
    pub fn get_destination_el(&self, input_ev: &Event) -> Vec<ElementID> {
        let mut dests = vec![];
        for (el_id, el_det) in self.els.borrow().iter() {
            if el_det.el.can_receive(input_ev) {
                dests.push(el_id.clone());
            }
        }
        dests
    }

    pub fn get_destination_el_from_kb(
        &self, kb: &mut Keyboard,
    ) -> Option<(ElementID, Vec<crossterm::event::KeyEvent>)> {
        for (el_id, el_det) in self.els.borrow().iter() {
            let recs = el_det.el.receivable();
            for rec in recs {
                for rec in rec.borrow().0.iter() {
                    let ReceivableEvent::KeyCombo(ref ekc) = rec else {
                        continue;
                    };
                    if let Some(eks) = kb.matches(ekc, true) {
                        return Some((el_id.clone(), eks));
                    }
                }
            }
        }
        None
    }

    pub fn propogate_event_to_all(
        &self, ctx: &Context, ev: Event, parent: Box<dyn Parent>,
    ) -> (bool, EventResponses) {
        let mut resps = EventResponses::default();
        for (el_id, details) in self.els.borrow().iter() {
            let (_, mut resps_) = details.el.receive_event(ctx, ev.clone());
            self.partially_process_ev_resps(ctx, el_id, &mut resps_, &parent);
            resps.0.extend(resps_.drain(..));
        }
        (false, resps)
    }

    /// sends an event to a specific element
    pub fn send_event_to_el(
        &self, ctx: &Context, el_id: &ElementID, ev: Event, parent: Box<dyn Parent>,
    ) -> EventResponses {
        let details = self
            .get_element_details(el_id)
            .expect("no element for destination id in send_event_to_el");

        let (_, mut resps) = details.el.receive_event(ctx, ev);
        self.partially_process_ev_resps(ctx, el_id, &mut resps, &parent);
        resps
    }

    /// initialize essentially refreshes the state of the element organizer.
    ///
    /// NOTE: the refresh allows for less meticulous construction of the main.go file. Elements can
    /// be added in whatever order, so long as your_main_el.refresh() is called after all elements
    /// are added.
    pub fn initialize(&self, ctx: &Context, parent: Box<dyn Parent>) -> EventResponses {
        // initialize all children
        let mut resps = EventResponses::default();
        for (_, details) in self.els.borrow().iter() {
            let (_, mut resp_) = details.el.receive_event(ctx, Event::Initialize);
            self.partially_process_ev_resps(ctx, &details.el.id(), &mut resp_, &parent);
            resps.0.extend(resp_.drain(..));
        }
        resps
    }

    /// change_focused_for_el updates a child element to a new focus.
    pub fn change_focused_for_el(&self, el_id: &ElementID, focused: bool) {
        let details = self
            .get_element_details(el_id)
            .expect("no element for destination id"); // TODO log

        // NOTE these changes are the changes for
        // THIS element organizer (not the child element)
        details.el.set_focused(focused);
    }

    /// get_el_id_z_order_under_mouse returns a list of all Elements whose locations
    /// include the position of the mouse event
    pub fn get_el_id_z_order_under_mouse(&self, ev: &MouseEvent) -> Vec<(ElementID, ZIndex)> {
        let mut ezo: Vec<(ElementID, ZIndex)> = Vec::new();

        for (el_id, details) in self.els.borrow().iter() {
            if !*details.vis.borrow() {
                continue;
            }
            if details.loc.borrow().contains(&ev.dr, ev.column, ev.row) {
                ezo.push((el_id.clone(), details.loc.borrow().z));
            }
        }

        // reverse sort the elements by z-index (highest-z to lowest-z)
        ezo.sort_by(|a, b| b.1.cmp(&a.1));
        ezo
    }

    /// mouse_event_process :
    /// - determines the appropriate element to send mouse events to
    /// - sends the event to the element
    /// - processes changes to the element's receivable events
    pub fn mouse_event_process(
        &self, ctx: &Context, ev: &MouseEvent, parent: Box<dyn Parent>,
    ) -> (Option<ElementID>, EventResponses) {
        //debug!("mouse_event_process: ev: {ev:?}");
        let eoz = self.get_el_id_z_order_under_mouse(ev);

        let mut resps = EventResponses::default();
        let mut capturing_el_id = None;
        let mut i = 0;
        loop {
            let Some((el_id, _)) = eoz.get(i) else {
                break; // past the end of the list
            };

            let details = self
                .get_element_details(el_id)
                .expect("no element for destination id");

            // adjust event to the relative position of the element
            let ev_adj = details.loc.borrow().l.adjusted_mouse_event(ev);

            // send mouse event to the element
            //let (captured, mut resps_) = details.el.receive_event(&child_ctx, Event::Mouse(ev_adj));
            let (captured, mut resps_) = details.el.receive_event(ctx, Event::Mouse(ev_adj));
            self.partially_process_ev_resps(ctx, el_id, &mut resps_, &parent);
            resps.0.extend(resps_.drain(..));

            if !captured {
                // proceed to the next element
                i += 1;
                continue;
            }

            capturing_el_id = Some(el_id.clone());
            break;
        }

        // send the mouse event as an external event to all other elements
        // capture the responses
        for (el_id2, details2) in self.els.borrow().iter() {
            if let Some(ref capturing_el_id) = capturing_el_id {
                if capturing_el_id == el_id2 {
                    continue;
                }
            }
            //let child_ctx = ctx.child_context(&details2.loc.borrow().l);
            let ev_adj = details2.loc.borrow().l.adjusted_mouse_event(ev);
            let (_, mut resps_) = details2.el.receive_event(ctx, Event::ExternalMouse(ev_adj));
            //.receive_event(&child_ctx, Event::ExternalMouse(ev_adj));
            //debug!("about to process external mouse resp: id:{el_id2:?} resps: {resps_:?}");
            self.partially_process_ev_resps(ctx, el_id2, &mut resps_, &parent);
            resps.0.append(&mut resps_.0);
        }

        (capturing_el_id, resps)
    }

    /// sends the external mouse command to all elements in the organizer
    pub fn external_mouse_event_process(
        &self, ctx: &Context, ev: &MouseEvent, parent: Box<dyn Parent>,
    ) -> EventResponses {
        let mut resps = EventResponses::default();
        for (el_id, details) in self.els.borrow().iter() {
            //let child_ctx = ctx.child_context(&details.loc.borrow().l);
            let ev_adj = details.loc.borrow().l.adjusted_mouse_event(ev);
            let (_, mut resps_) = details
                .el
                //.receive_event(&child_ctx, Event::ExternalMouse(ev_adj));
                .receive_event(ctx, Event::ExternalMouse(ev_adj));
            self.partially_process_ev_resps(ctx, el_id, &mut resps_, &parent);
            resps.extend(resps_);
        }
        resps
    }

    /// get_el_id_at_z_index returns the element-id at the given z index, or None if
    /// no element exists at the given z index
    pub fn get_el_at_z_index(&self, z: ZIndex) -> Option<ElDetails> {
        for (_, details) in self.els.borrow().iter() {
            if details.loc.borrow().z == z {
                return Some(details.clone());
            }
        }
        None
    }

    /// increment_z_index_for_el_id increments the z-index of the element with the given id,
    /// pushing it further back in the visual stack.
    ///
    /// NOTE: If an element already occupies the index that the given element is
    /// attempting to occupy, the element occupying the index will be pushed back as
    /// well.
    ///
    /// To move an element in the z-dimension, relative to other elements, use
    /// update_z_index_for_el_id
    pub fn increment_z_index_for_el(&self, el_details: ElDetails) {
        let z = el_details.loc.borrow().z;
        // current z of element
        // check if element exists at next z-index
        if let Some(details2) = self.get_el_at_z_index(z + 1) {
            // recursively increment z-index of element at next z-index
            self.increment_z_index_for_el(details2);
        }

        // increment z-index of the element
        self.els
            .borrow_mut()
            .entry(el_details.el.id().clone())
            .and_modify(|ed| ed.loc.borrow_mut().z = z + 1);
    }

    /// is_z_index_occupied returns true if an element exists at the given z-index
    pub fn is_z_index_occupied(&self, z: ZIndex) -> bool {
        self.els
            .borrow()
            .values()
            .any(|details| details.loc.borrow().z == z)
    }

    /// set_visibility_for_el sets the Visibility of the given element ID
    pub fn set_visibility_for_el(&self, el_id: ElementID, visible: bool) {
        self.els
            .borrow_mut()
            .entry(el_id)
            .and_modify(|ed| *ed.vis.borrow_mut() = visible);
    }
}
