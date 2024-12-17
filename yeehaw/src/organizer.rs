use {
    crate::{
        prioritizer::EventPrioritizer, Context, DrawAction, DrawCh, DrawUpdate, DynLocation,
        DynLocationSet, Element, ElementID, Event, EventResponse, EventResponses, Parent, Priority,
        ReceivableEventChanges, RelMouseEvent, SelfReceivableEvents, ZIndex,
    },
    rayon::prelude::*,
    std::collections::HashMap,
    std::{cell::RefCell, rc::Rc},
};

/// ElementOrganizer prioritizes and organizes all the elements contained
/// within it
#[derive(Clone, Default)]
pub struct ElementOrganizer {
    pub els: Rc<RefCell<HashMap<ElementID, ElDetails>>>,
    pub prioritizer: Rc<RefCell<EventPrioritizer>>,

    #[allow(clippy::type_complexity)]
    /// the last draw details for each element
    ///                                              (location, visibility, overflow)
    last_draw_details: Rc<RefCell<Vec<(ElementID, (DynLocationSet, bool, bool))>>>,

    /// the queue of elements to be removed on the next draw
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
    #[must_use]
    pub fn add_element(
        &self, el: Box<dyn Element>, parent: Option<Box<dyn Parent>>,
    ) -> EventResponse {
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

        // add the elements recievable events and commands to the prioritizer
        let receivable_evs = el.receivable();
        self.prioritizer
            .borrow_mut()
            .include(&el_id, &receivable_evs);

        let el_details = ElDetails::new(el);
        self.els.borrow_mut().insert(el_id.clone(), el_details);

        let rec = ReceivableEventChanges::default().with_add_evs(receivable_evs.0);
        EventResponse::ReceivableEventChanges(rec)
    }

    #[must_use]
    pub fn remove_element(&self, el_id: &ElementID) -> EventResponse {
        self.last_draw_details
            .borrow_mut()
            .retain(|(id, _)| id != el_id);
        self.removed_element_queue.borrow_mut().push(el_id.clone());
        self.els.borrow_mut().remove(el_id);
        let rm_evs = self.prioritizer.borrow_mut().remove_entire_element(el_id);
        let rec = ReceivableEventChanges::default().with_remove_evs(rm_evs);
        EventResponse::ReceivableEventChanges(rec)
    }

    /// removes all elements from the element organizer
    #[must_use]
    pub fn clear_elements(&self) -> EventResponse {
        self.removed_element_queue.borrow_mut().extend(
            self.last_draw_details
                .borrow_mut()
                .drain(..)
                .map(|(id, _)| id),
        );
        let pes = self.receivable().drain(..).map(|(e, _)| e).collect();
        *self.prioritizer.borrow_mut() = EventPrioritizer::default();
        let rec = ReceivableEventChanges::default().with_remove_evs(pes);
        EventResponse::ReceivableEventChanges(rec)
    }

    #[must_use]
    /// hide_element is similar to remove_element, it removees its receivable events from the
    /// prioritizer, but does not remove the element from the element organizer, thus the hidden
    /// element is still able to receive certain events such as exit or resize. The visibility of
    /// the element is set to false.
    pub fn hide_element(&self, el_id: &ElementID) -> EventResponse {
        let Some(details) = self.get_element_details(el_id) else {
            return EventResponse::None;
        };
        details.set_visibility(false);
        let _ = details.el.change_priority(Priority::Unfocused);

        let rm_evs = self.prioritizer.borrow_mut().remove_entire_element(el_id);
        let add_evs = details.el.receivable(); // must also add evs again for evs that are not removed
        self.prioritizer.borrow_mut().include(el_id, &add_evs);
        let rec = ReceivableEventChanges::default()
            .with_remove_evs(rm_evs)
            .with_add_evs(add_evs.0);
        EventResponse::ReceivableEventChanges(rec)
    }

    #[must_use]
    /// unhide_element is used to unhide an element that was previously hidden. It sets the
    /// visibility of the element to true and adds the elements receivable events to the
    /// prioritizer.
    pub fn unhide_element(&self, el_id: &ElementID) -> EventResponse {
        let Some(details) = self.get_element_details(el_id) else {
            return EventResponse::None;
        };
        details.set_visibility(true);
        let _ = details.el.change_priority(Priority::Focused);

        let rm_evs = self.prioritizer.borrow_mut().remove_entire_element(el_id);
        let add_evs = details.el.receivable(); // must also add evs again for evs that are not removed
        self.prioritizer.borrow_mut().include(el_id, &add_evs);
        let rec = ReceivableEventChanges::default()
            .with_remove_evs(rm_evs)
            .with_add_evs(add_evs.0);
        EventResponse::ReceivableEventChanges(rec)
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
    pub fn get_element_details_at_pos(&self, ctx: &Context, x: i32, y: i32) -> Option<ElDetails> {
        for (_, details) in self.els.borrow().iter() {
            if details.loc.borrow().contains(ctx, x, y) {
                return Some(details.clone());
            }
        }
        None
    }

    /// get_el_id_at_pos returns the element id at the given position
    pub fn get_el_id_at_pos(&self, ctx: &Context, x: i32, y: i32) -> Option<ElementID> {
        for (el_id, details) in self.els.borrow().iter() {
            if details.loc.borrow().contains(ctx, x, y) {
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
        //self.locations.entry(el_id).and_modify(|l| (*l) = loc);
        self.els
            .borrow_mut()
            .entry(el_id)
            .and_modify(|ed| *ed.loc.borrow_mut() = loc);
    }

    /// update_el_primary_location updates the primary location of the element with the given id
    pub fn update_el_primary_location(&self, el_id: ElementID, loc: DynLocation) {
        //self.locations.entry(el_id).and_modify(|l| l.l = loc);
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

    /// Receivable returns all of the key combos and commands registered to this
    /// element organizer, along with their priorities
    pub fn receivable(&self) -> SelfReceivableEvents {
        let mut out = SelfReceivableEvents::default();
        for details in self.els.borrow().values() {
            let pr_evs = details.el.receivable();
            out.extend(pr_evs.0);
        }
        out
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
    pub fn all_drawing_updates(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
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

            if !*details.vis.borrow() {
                updates.push(DrawUpdate::clear_all_at_sub_id(vec![el_id_z.0]));
                continue;
            }
            if let Some(vis_loc) = ctx.visible_region {
                if !vis_loc.intersects_dyn_location_set(ctx, &details.loc.borrow()) {
                    continue;
                }
            }

            let child_ctx = ctx.child_context(&el_id_z.1.loc.borrow().l);
            let mut el_upds = details.el.drawing(&child_ctx, force_update);

            for mut el_upd in el_upds.drain(..) {
                // prepend the element_id to the DrawUpdate
                el_upd.prepend_id(el_id_z.0.clone());

                match el_upd.action {
                    DrawAction::ClearAll => {}
                    DrawAction::Remove => {}
                    DrawAction::Update(ref mut dcps) | DrawAction::Extend(ref mut dcps) => {
                        //let mut dcps = details.el.drawing(&child_ctx);
                        let l = details.loc.borrow().l.clone();
                        let s = ctx.size;
                        let child_s = child_ctx.size;

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
                                dcp.set_draw_size_offset_colors(
                                    child_s,
                                    start_x as u16,
                                    start_y as u16,
                                );
                                dcp.x += start_x as u16;
                                dcp.y += start_y as u16;
                            });
                        } else {
                            dcps.par_iter_mut().for_each(|dcp| {
                                dcp.set_draw_size_offset_colors(
                                    child_s,
                                    start_x as u16,
                                    start_y as u16,
                                );
                                if dcp.x >= child_s.width || dcp.y >= child_s.height {
                                    // it'd be better to delete, but we can't delete from a parallel iterator
                                    // also using a filter here its slower that this
                                    dcp.ch = DrawCh::transparent();
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

    /// write func to remove/add evCombos and commands from EvPrioritizer and
    /// CommandPrioritizer, using the ReceivableEventChanges struct
    pub fn process_receivable_event_changes(
        &self, el_id: &ElementID, rec: &ReceivableEventChanges,
    ) {
        self.prioritizer
            .borrow_mut()
            .process_receivable_event_changes(el_id, rec);
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
            log_err!("no element for id in partially_process_ev_resps");
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

                    let resp_ = self.remove_element(el_id);
                    // NOTE no need to process the receivable event changes here,
                    // they've already been removed in the above call
                    *r = resp_;
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
                        let rec = self.change_priority_for_el(el_id_, Priority::Unfocused);
                        let add_ = Self::generate_perceived_priorities(
                            parent.get_parent_priority(),
                            rec.add.clone().into(),
                        );
                        let remove_ = add_.iter().map(|a| a.0.clone()).collect();
                        let rec_for_higher = ReceivableEventChanges::new(add_.0, remove_);
                        extend_resps.push(EventResponse::ReceivableEventChanges(rec_for_higher));
                    }
                }
                EventResponse::Focus => {
                    let rec = self.change_priority_for_el(el_id, Priority::Focused);
                    let add_ = Self::generate_perceived_priorities(
                        parent.get_parent_priority(),
                        rec.add.clone().into(),
                    );
                    let remove_ = add_.iter().map(|a| a.0.clone()).collect();
                    let rec_for_higher = ReceivableEventChanges::new(add_.0, remove_);

                    // NOTE this needs to be added to extend_resps instead of just to *r as
                    // if UnfocusOthers is called then it is placed in extend_resps and
                    // will be processed after this *r (AND if this rec_for_higher contains
                    // duplicate events, they will be removed by the UnfocusOthers call)
                    extend_resps.push(EventResponse::ReceivableEventChanges(rec_for_higher));
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
                    let resp_ = self.add_element(new_el.clone(), Some(parent.clone()));
                    if let Some(new_el_resps) = new_el_resps {
                        self.partially_process_ev_resps(ctx, &new_el.id(), new_el_resps, parent);
                        extend_resps.0.extend(new_el_resps.drain(..));
                    }
                    *r = resp_;
                }
                EventResponse::ReceivableEventChanges(ref mut rec) => {
                    self.process_receivable_event_changes(el_id, rec);

                    // Modify the ReceivableEventChanges to reflect the perceived priorities
                    // of the parent element. Required as this EventResponse is being passed
                    // up the chain further to the next parent element.
                    // TODO could remove clones and drain each vec.
                    let add_ = Self::generate_perceived_priorities(
                        parent.get_parent_priority(),
                        rec.add.clone().into(),
                    );
                    let remove_ = add_.iter().map(|a| a.0.clone()).collect();
                    let rec_for_higher = ReceivableEventChanges::new(add_.0, remove_);
                    *r = EventResponse::ReceivableEventChanges(rec_for_higher);
                }
            }
        }
        resps.0.extend(extend_resps.drain(..));
    }

    /// generate_perceived_priorities generates the "perceived priorities" of the
    /// provided events. It receives a function which can then use each perceived
    /// priority however it needs to.
    ///
    /// **IMPORTANT NOTE**
    ///
    /// The "perceived priorities" are the effective priorities of an element FROM
    /// the perspective of an element two or more levels ABOVE the element in the tree.
    ///
    /// Relative priorities between the children elements of a parent element
    /// should be perserved. To ensure this, the priorities of children should
    /// never be modified but instead interpreted as "perceived priorities".
    ///
    ///```text
    ///    EXAMPLE:        Element 0 (ABOVE_FOCUSED)
    ///                        evA (ABOVE_FOCUSED)     ┐
    ///                      evB (ABOVE_FOCUSED)     ├─perceived-priorities
    ///                      evC (ABOVE_FOCUSED)     │
    ///                      evD (HIGHEST_FOCUS)     ┘
    ///                             │
    ///                     Element 1
    ///                        evA (ABOVE_FOCUSED)
    ///                        evB (FOCUSED)
    ///                      evC (UNFOCUSED)
    ///                      evD (HIGHEST_FOCUS)
    ///                ┌────────────┴───────────┐
    ///               Element 2                Element 3
    ///                evA (ABOVE_FOCUSED)      evC (UNFOCUSED)
    ///                evB (FOCUSED)            evD (HIGHEST_FOCUS)
    ///```
    ///
    /// This function does not modify the priorities of any child element, but
    /// instead generates the "perceived priorities" in the following way:
    ///  1. If the input priority (pr) is UNFOCUSED:
    ///     - simply interpret all the childrens' priorities as unfocused.
    ///       (everything set in the ic will be unfocused).
    ///  2. if the input priority (pr) is FOCUSED or greater:
    ///     - individually interpret each child's Receivable Event priority as
    ///       the greatest of either the input priority to this function (pr),
    ///       or the child event's current priority.
    ///
    /// INPUTS
    ///   - The real_pes is the real priority events of the child element.
    ///   - The parent_pr is the priority of the parent element
    ///   - The perceived_pes is the perceived priority events of a child element for
    ///     this element for this element's parent (the grandparent of the child).
    pub fn generate_perceived_priorities(
        parent_pr: Priority, real_pes: SelfReceivableEvents,
    ) -> SelfReceivableEvents {
        let mut perceived_pes = vec![];
        #[allow(clippy::comparison_chain)]
        if parent_pr == Priority::Unfocused {
            for child in real_pes.0 {
                perceived_pes.push((child.0, Priority::Unfocused));
            }
            // leave the children alone! they're fine
        } else {
            for child in real_pes.0 {
                let pr = match true {
                    _ if child.1 == Priority::Unfocused => Priority::Unfocused,
                    _ if child.1 < parent_pr => child.1,
                    _ => parent_pr,
                };
                perceived_pes.push((child.0, pr));
            }
        }
        perceived_pes.into()
    }

    /// Replaces the element at the given ID with a new element
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
    //    ic.set_add_evs(new_evs);
    //    ic
    //}

    pub fn event_process(
        &self, ctx: &Context, ev: Event, parent: Box<dyn Parent>,
    ) -> (bool, EventResponses) {
        let p_id = parent.get_id();
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
        self.prioritizer
            .borrow()
            .ensure_no_duplicate_priorities(&p_id);

        (captured, resps)
    }

    /// routed_event_process:
    /// - determines the appropriate element to send the event to then sends the event
    ///    - if the event isn't captured then send it to the next element able to receive this event
    ///      (ordered by priority)
    /// - partially processes changes to the elements receivable events
    ///
    /// NOTE elements may choose to not capture events in order to continue sending the
    ///      event to the next element in the chain
    pub fn routed_event_process(
        &self, ctx: &Context, ev: Event, parent: Box<dyn Parent>,
    ) -> (Option<ElementID>, EventResponses) {
        // determine element_id to send events to
        let el_ids = self.prioritizer.borrow().get_destination_el(&ev);

        let mut resps = EventResponses::default();
        let mut capturing_el_id = None;
        for el_id in el_ids {
            let el_details = self
                .get_element_details(&el_id)
                .expect("no element for destination id in routed_event_process");

            let child_ctx = ctx.child_context(&el_details.loc.borrow().l);
            let (captured, mut resps_) = el_details.el.receive_event(&child_ctx, ev.clone());
            self.partially_process_ev_resps(ctx, &el_id, &mut resps_, &parent);
            resps.0.extend(resps_.drain(..));

            if captured {
                capturing_el_id = Some(el_id);
                break;
            }
        }
        (capturing_el_id, resps)
    }

    pub fn propogate_event_to_all(
        &self, ctx: &Context, ev: Event, parent: Box<dyn Parent>,
    ) -> (bool, EventResponses) {
        let mut resps = EventResponses::default();
        for (el_id, details) in self.els.borrow().iter() {
            let el_ctx = ctx.child_context(&details.loc.borrow().l);
            let (_, mut resps_) = details.el.receive_event(&el_ctx, ev.clone());
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

        let child_ctx = ctx.child_context(&details.loc.borrow().l);
        let (_, mut resps) = details.el.receive_event(&child_ctx, ev);
        self.partially_process_ev_resps(ctx, el_id, &mut resps, &parent);
        resps
    }

    /// initialize updates the prioritizers essentially refreshing the state of the element organizer.
    ///
    /// NOTE: the refresh allows for less meticulous construction of the main.go file. Elements can
    /// be added in whatever order, so long as your_main_el.refresh() is called after all elements
    /// are added.
    pub fn initialize(&self, ctx: &Context, parent: Box<dyn Parent>) -> EventResponses {
        // reset prioritizers
        *self.prioritizer.borrow_mut() = EventPrioritizer::default();

        // initialize all children
        let mut resps = EventResponses::default();
        for (_, details) in self.els.borrow().iter() {
            let el_ctx = ctx.child_context(&details.loc.borrow().l);
            let (_, mut resp_) = details.el.receive_event(&el_ctx, Event::Initialize);
            self.partially_process_ev_resps(ctx, &details.el.id(), &mut resp_, &parent);
            resps.0.extend(resp_.drain(..));

            let rec = details.el.receivable().to_receivable_event_changes();
            self.process_receivable_event_changes(&details.el.id(), &rec);
        }
        resps
    }

    /// change_priority_for_el updates a child element to a new priority. It does
    /// this by asking the child element to return its registered events w/
    /// priorities updated to a given priority.
    pub fn change_priority_for_el(
        &self, el_id: &ElementID, pr: Priority,
    ) -> ReceivableEventChanges {
        let details = self
            .get_element_details(el_id)
            .expect("no element for destination id"); // TODO log

        //let child_ctx = self.get_context_for_el(higher_ctx, &details);
        //let rec = details.el.change_priority(&child_ctx, pr);

        // NOTE these changes are the changes for
        // THIS element organizer (not the child element)
        let rec = details.el.change_priority(pr);
        self.process_receivable_event_changes(el_id, &rec);
        rec
    }

    /// get_el_id_z_order_under_mouse returns a list of all Elements whose locations
    /// include the position of the mouse event
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

        // reverse sort the elements by z-index (highest-z to lowest-z)
        ezo.sort_by(|a, b| b.1.cmp(&a.1));
        ezo
    }

    /// mouse_event_process :
    /// - determines the appropriate element to send mouse events to
    /// - sends the event to the element
    /// - processes changes to the element's receivable events
    pub fn mouse_event_process(
        &self, ctx: &Context, ev: &crossterm::event::MouseEvent, parent: Box<dyn Parent>,
    ) -> (Option<ElementID>, EventResponses) {
        //debug!("mouse_event_process: ev: {ev:?}");
        let eoz = self.get_el_id_z_order_under_mouse(ctx, ev);

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
            let child_ctx = ctx.child_context(&details.loc.borrow().l);

            // adjust event to the relative position of the element
            let ev_adj = details.loc.borrow().l.adjust_mouse_event(ctx, ev);

            // send mouse event to the element
            let (captured, mut resps_) = details.el.receive_event(&child_ctx, Event::Mouse(ev_adj));
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
            let child_ctx = ctx.child_context(&details2.loc.borrow().l);
            let ev_adj = details2.loc.borrow().l.adjust_mouse_event_external(ctx, ev);
            let (_, mut resps_) = details2
                .el
                .receive_event(&child_ctx, Event::ExternalMouse(ev_adj));
            //debug!("about to process external mouse resp: id:{el_id2:?} resps: {resps_:?}");
            self.partially_process_ev_resps(ctx, el_id2, &mut resps_, &parent);
            resps.0.append(&mut resps_.0);
        }

        (capturing_el_id, resps)
    }

    /// sends the external mouse command to all elements in the organizer
    pub fn external_mouse_event_process(
        &self, ctx: &Context, ev: &RelMouseEvent, parent: Box<dyn Parent>,
    ) -> EventResponses {
        let mut resps = EventResponses::default();
        for (el_id, details) in self.els.borrow().iter() {
            let child_ctx = ctx.child_context(&details.loc.borrow().l);
            let ev_adj = details
                .loc
                .borrow()
                .l
                .adjust_mouse_event_external2(ctx, ev.clone());
            let (_, mut resps_) = details
                .el
                .receive_event(&child_ctx, Event::ExternalMouse(ev_adj));
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
    /// UpdateZIndexForElID
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
