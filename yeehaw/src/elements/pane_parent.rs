use {
    crate::{
        Color, Context, DrawCh, DrawChPos, DrawChs2D, DynLocation, DynLocationSet, DynVal, Element,
        ElementID, ElementOrganizer, Event, EventResponse, EventResponses, Pane, Parent, Priority,
        ReceivableEventChanges, SelfReceivableEvents, Size, Style, ZIndex,
    },
    std::collections::HashMap,
    std::{
        ops::Deref,
        {
            cell::{Ref, RefCell, RefMut},
            rc::Rc,
        },
    },
};

/// ParentPane is a pane element which other objects can embed and build off
/// of. It is a pane which can have children panes.
///
/// NOTE the ParentPane does not itself fulfill the Element trait however
/// it provides much of the boilerplate required to do so.
///
/// the element store (el_store) is a store for sub-elements. Any of the sub-elements can be
/// accessed any of the contents and share it with other elements of this parent pane.
#[derive(Clone)]
pub struct ParentPane {
    pub pane: Pane,
    pub eo: ElementOrganizer,
    pub el_store: Rc<RefCell<HashMap<String, Vec<u8>>>>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl ParentPane {
    pub fn new(ctx: &Context, kind: &'static str) -> Self {
        let pane = Pane::new(ctx, kind).with_focused(ctx);
        ParentPane {
            pane,
            eo: ElementOrganizer::default(),
            el_store: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn with_z(self, z: ZIndex) -> Self {
        self.pane.set_z(z);
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, x: D, y: D2) -> Self {
        self.set_at(x.into(), y.into());
        self
    }

    pub fn with_kind(self, kind: &'static str) -> Self {
        self.pane.set_kind(kind);
        self
    }

    pub fn with_bg_color(self, c: Color) -> Self {
        self.pane.default_ch.borrow_mut().style.set_bg(c);
        self
    }

    pub fn with_style(self, sty: Style) -> Self {
        let ch = DrawCh::new(' ', sty);
        self.pane.set_default_ch(ch);
        self
    }

    pub fn with_transparent(self) -> Self {
        let ch = DrawCh::transparent();
        self.pane.set_default_ch(ch);
        self
    }

    pub fn focused(self) -> Self {
        *self.pane.element_priority.borrow_mut() = Priority::Focused;
        self
    }

    pub fn unfocused(self) -> Self {
        *self.pane.element_priority.borrow_mut() = Priority::Unfocused;
        self
    }

    pub fn with_element(self, el: Box<dyn Element>) -> Self {
        let _ = self.add_element(el); // ignore the response as this is used during initialization
        self
    }

    #[must_use]
    pub fn add_element(&self, el: Box<dyn Element>) -> EventResponse {
        self.eo.add_element(el, Some(Box::new(self.clone())))
    }

    #[must_use]
    pub fn remove_element(&self, el_id: &ElementID) -> EventResponse {
        self.eo.remove_element(el_id)
    }

    #[must_use]
    pub fn clear_elements(&self) -> EventResponse {
        self.eo.clear_elements()
    }

    pub fn has_elements(&self) -> bool {
        !self.eo.els.borrow().is_empty()
    }

    pub fn perceived_priorities_of_eo(&self) -> SelfReceivableEvents {
        let pr = self.pane.get_element_priority();
        let pes = self.eo.receivable(); // registered receivable events
        ElementOrganizer::generate_perceived_priorities(pr, pes)
    }

    //pub fn change_priority_for_el(
    //    &self, ctx: &Context, el_id: ElementID, p: Priority,
    //) -> ReceivableEventChanges {
    //    let mut ic = self.eo.change_priority_for_el(ctx, el_id, p);
    //    let mut to_add = vec![];
    //    // Check if any of the ic.remove match pane.self_evs. If so, add those events to
    //    // the ic.add.
    //    //
    //    // NOTE: this is necessary because:
    //    // 1. An event passed in the ic.remove will remove ALL instances of an
    //    // event registered to the ElementOrganizer (eo) of the parent of this
    //    // element. This is true because all events in the parent of this element
    //    // are registered with the ID of THIS element.
    //    //    e.g. if EventX is being passed in the ic.remove and EventX occurs
    //    //    twice in the prioritizer of the EO of the parent of this element, BOTH
    //    //    instances of EventX will be removed when the EO processes the
    //    //    ReceivableEventChanges.
    //    // 2. If this element has registered EventX as a SelfEv and EventX is also
    //    // passed up in the ic.remove, then EventX will be removed from the parent
    //    // organizer and this element will no longer be able to recieve EventX even
    //    // though it still wants to.
    //    //
    //    // NOTE: Leaving the remove event in the ic.remove removes artifacts further
    //    // up the tree. i.e, if we simply remove the event from the ic.remove, then
    //    // the parent of this element will have an artifact registration for an
    //    // event that serves no purpose.
    //    //
    //    // NOTE: If there are duplicate events in the ic.remove, then the
    //    // following code will add duplicate events to the ic.add. This will
    //    // result in duplicate events registered with the same priority and ID in
    //    // this element's parent. This seems harmless and is probably more efficient
    //    // than checking for duplicates.
    //    for rm in ic.remove.iter() {
    //        for self_ev in self.pane.self_evs.borrow().0.iter() {
    //            if *rm == self_ev.0 {
    //                to_add.push((self_ev.0.clone(), self_ev.1));
    //            }
    //        }
    //    }
    //    ic.push_add_evs(to_add);
    //    ic
    //}

    // -------------------------------------
    // Element functions

    //pub fn get_element_by_id(&self, el_id: &ElementID) -> Option<Rc<RefCell<dyn Element>>> {
    pub fn get_element(&self, el_id: &ElementID) -> Option<Box<dyn Element>> {
        self.eo.get_element(el_id)
    }

    pub fn get_element_attribute(&self, el_id: &ElementID, key: &str) -> Option<Vec<u8>> {
        self.eo.get_element_attribute(el_id, key)
    }

    pub fn update_el_z_index(&self, el_id: &ElementID, z: ZIndex) {
        self.eo.update_el_z_index(el_id, z);
    }

    /// NOTE this name was chosen to distinguish itself from propagate_responses_upward
    pub fn send_responses_upward(&self, ctx: &Context, resps: EventResponses) {
        self.pane.send_responses_upward(ctx, resps);
    }

    pub fn focus(&self, ctx: &Context) {
        let rec = self.change_priority(Priority::Focused);
        if self.pane.has_parent() {
            let resps = EventResponse::ReceivableEventChanges(rec);
            self.send_responses_upward(ctx, resps.into());
        }
    }

    pub fn unfocus(&self, ctx: &Context) {
        let rec = self.change_priority(Priority::Unfocused);
        if self.pane.has_parent() {
            let resps = EventResponse::ReceivableEventChanges(rec);
            self.send_responses_upward(ctx, resps.into());
        }
    }

    /// sends an event to a specific element
    #[must_use]
    pub fn send_event_to_el(&self, ctx: &Context, el_id: &ElementID, ev: Event) -> EventResponses {
        self.eo
            .send_event_to_el(ctx, el_id, ev, Box::new(self.clone()))
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for ParentPane {
    fn receivable(&self) -> SelfReceivableEvents {
        let mut pes = self.perceived_priorities_of_eo();
        pes.extend(self.pane.receivable().0);
        pes
    }

    /// primarily a placeholder function. An element using the parent pane should
    /// write their own receive_event function.
    /// TODO verify that this code is or isn't used anywhere
    ///                                               (captured, resp         )
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        //debug!(
        //    "ParentPane({})::receive_event_inner: ev={:?}",
        //    self.id(),
        //    ev,
        //);

        self.eo.event_process(ctx, ev, Box::new(self.clone()))
    }

    /// ChangePriority returns a priority change (InputabilityChanges) to its
    /// parent organizer so as to update the priority of all events registered to
    /// this element.
    ///
    /// NOTE: The receivable event changes (rec) that this parent pane sends up is the
    /// combination of:
    ///   - this element's priority changes (the SelfEvs, aka the
    ///     Self Receivable Events)
    ///   - the "perceived priorities" of the childens' Receivable Events
    ///     (aka the results of the child's Receivable() function) The "perceived
    ///     priorities" are the effective priority FROM the perspective of the
    ///     element ABOVE this element in the tree.
    fn change_priority(&self, pr: Priority) -> ReceivableEventChanges {
        // first change the priority of the self evs. These are "this elements
        // priority changes". NO changes should be made to the childen,
        // the perceived priorities of the children should be interpreted.
        let mut rec = self.pane.change_priority(pr);

        // update the perceived priorities of the children and update the prioritizer
        for (_, el_details) in self.eo.els.borrow().iter() {
            let pes = el_details.el.receivable(); // self evs (and child eo's evs)
            for pe in ElementOrganizer::generate_perceived_priorities(pr, pes).0 {
                rec.update_priority_for_ev(pe.0, pe.1);
            }
        }

        rec
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        if !self.get_visible() {
            return Vec::with_capacity(0);
        }
        let mut out = self.pane.drawing(ctx);
        out.extend(self.eo.all_drawing(ctx));
        out
    }
}

impl Parent for ParentPane {
    /// DO NOT CALL THIS FUNCTION DIRECTLY
    /// This function is intended for internal propogation ONLY if you need to propogate changes
    /// use the function: send_responses_upward
    ///
    /// Passes changes to inputability to this element's parent element. If
    /// updateThisElementsPrioritizers is TRUE then this element's prioritizers should be updated
    /// using the given IC. This should be set to false when an upwards propagation is being
    /// initiated as all of the changes to the prioritzers should have already been handled. The
    /// boolean should be set to true on all further calls as the changes are propagated upstream so
    /// as to update the ancestors' prioritizers.
    ///
    /// childEl is the element which is invoking the propagation from BELOW this parent pane. This
    /// is used by the parent to determine which events/cmds to update the prioritizers for.
    ///
    /// The propagateEl is the element to send further upward propagation to. Typically this means
    /// the Element which is inheriting THIS parent pane.
    ///
    /// NOTE: propagateEl is necessary as the parent pane will usually have registered an element
    /// that extends ParentPane. If this ParentPane sent itself, it would not match the child
    /// registered in the parent's EO.
    ///
    /// NOTE this function should be extended from if the parent pane is used as a base for a more
    /// complex element. As the developer you should be fulfilling the
    /// propagate_responses_upward function directly.
    ///
    /// NOTE the parent_ctx is the correct context for THIS parent pane.
    fn propagate_responses_upward(
        &self, parent_ctx: &Context, child_el_id: &ElementID, mut resps: EventResponses,
    ) {
        let b: Box<dyn Parent> = Box::new(self.clone());
        self.eo
            .partially_process_ev_resps(parent_ctx, child_el_id, &mut resps, &b);
        if let Some(parent) = self.pane.parent.borrow_mut().deref() {
            let next_parent_ctx = parent_ctx.must_get_parent_context();
            parent.propagate_responses_upward(next_parent_ctx, &self.id(), resps);
        }
    }

    fn get_store_item(&self, key: &str) -> Option<Vec<u8>> {
        self.el_store.borrow().get(key).cloned()
    }

    fn set_store_item(&self, key: &str, value: Vec<u8>) {
        self.el_store.borrow_mut().insert(key.to_string(), value);
    }

    fn get_priority(&self) -> Priority {
        self.pane.get_element_priority()
    }

    fn get_id(&self) -> ElementID {
        self.pane.id()
    }
}
