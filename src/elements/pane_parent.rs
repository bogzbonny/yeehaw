use {
    crate::{
        element::ReceivableEventChanges, Context, DrawCh, DrawChPos, DynLocation, DynLocationSet,
        DynVal, Element, ElementID, ElementOrganizer, Event, EventResponses, Pane, Parent,
        Priority, SortingHat, Style, ZIndex,
    },
    std::collections::HashMap,
    std::{
        ops::Deref,
        {cell::RefCell, rc::Rc},
    },
};

// ParentPane is a pane element which other objects can embed and build off
// of. It is a pane which can have children panes.
//
// NOTE the ParentPane does not itself fulfill the Element trait however
// it provides much of the boilerplate required to do so.
//
// the element store (el_store) is a store for sub-elements. Any of the sub-elements can be
// accessed any of the contents and share it with other elements of this parent pane.
#[derive(Clone)]
pub struct ParentPane {
    pub pane: Pane,
    pub eo: ElementOrganizer,
    pub el_store: Rc<RefCell<HashMap<String, Vec<u8>>>>,
}

impl ParentPane {
    pub fn new(hat: &SortingHat, kind: &'static str) -> Self {
        let pane = Pane::new(hat, kind);
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

    pub fn at(self, x: DynVal, y: DynVal) -> Self {
        self.set_at(x, y);
        self
    }

    pub fn set_at(&self, x: DynVal, y: DynVal) {
        self.pane.set_at(x, y);
    }

    pub fn set_start_x(&self, x: DynVal) {
        self.pane.set_start_x(x);
    }

    pub fn set_start_y(&self, y: DynVal) {
        self.pane.set_start_y(y);
    }

    pub fn set_end_x(&self, x: DynVal) {
        self.pane.set_end_x(x);
    }

    pub fn set_end_y(&self, y: DynVal) {
        self.pane.set_end_y(y);
    }

    pub fn set_z(&self, z: ZIndex) {
        self.pane.set_z(z);
    }

    pub fn set_dyn_width(&self, w: DynVal) {
        self.pane.set_dyn_width(w);
    }

    pub fn set_dyn_height(&self, h: DynVal) {
        self.pane.set_dyn_height(h);
    }

    pub fn get_dyn_location(&self) -> DynLocation {
        self.pane.get_dyn_location()
    }

    pub fn set_dyn_location(&self, dl: DynLocation) {
        self.pane.set_dyn_location(dl);
    }

    pub fn with_kind(self, kind: &'static str) -> Self {
        self.pane.set_kind(kind);
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

    pub fn with_dyn_height(self, h: DynVal) -> Self {
        self.pane.set_dyn_height(h);
        self
    }

    pub fn with_dyn_width(self, w: DynVal) -> Self {
        self.pane.set_dyn_width(w);
        self
    }

    pub fn add_element(&self, el: Rc<RefCell<dyn Element>>) {
        self.eo
            .add_element(el.clone(), Some(Box::new(self.clone())));
    }

    pub fn remove_element(&self, el_id: &ElementID) {
        self.eo.remove_element(el_id);
    }

    pub fn clear_elements(&self) {
        self.eo.clear_elements();
    }

    // generate_perceived_priorities generates the "perceived priorities" of the
    // provided events. It receives a function which can then use each perceived
    // priority however it needs to.
    //
    // **IMPORTANT NOTE**
    //
    // The "perceived priorities" are the effective priorities of an element FROM
    // the perspective of an element two or more levels ABOVE the element in the tree.
    //
    // Relative priorities between the children elements of a parent element
    // should be perserved. To ensure this, the priorities of children should
    // never be modified but instead interpreted as "perceived priorities".
    //
    //	EXAMPLE:	  	Element 0 (ABOVE_FOCUSED)
    //				  	  evA (ABOVE_FOCUSED)     ┐
    //				      evB (ABOVE_FOCUSED)     ├─perceived-priorities
    //				      evC (ABOVE_FOCUSED)     │
    //		              evD (HIGHEST_FOCUS)     ┘
    //			                 │
    //				 	Element 1
    //				  	  evA (ABOVE_FOCUSED)
    //				  	  evB (FOCUSED)
    //				      evC (UNFOCUSED)
    //				      evD (HIGHEST_FOCUS)
    //	            ┌────────────┴───────────┐
    //		   	Element 2                Element 3
    //		   	 evA (ABOVE_FOCUSED)      evC (UNFOCUSED)
    //		   	 evB (FOCUSED)            evD (HIGHEST_FOCUS)
    //
    // This function does not modify the priorities of any child element, but
    // instead generates the "perceived priorities" in the following way:
    //  1. If the input priority (pr) is UNFOCUSED:
    //     - simply interpret all the childrens' priorities as unfocused.
    //     (everything set in the ic will be unfocused).
    //  2. if the input priority (pr) is FOCUSED or greater:
    //     - individually interpret each child's Receivable Event priority as
    //     the greatest of either the input priority to this function (pr),
    //     or the child event's current priority.
    //
    // INPUTS
    //   - The real_pes is the real priority events of the child element.
    //   - The parent_pr is the priority that the parent element is being changed to
    //   - The perceived_pes is the perceived priority events of a child element for
    //     this element for this element's parent (the grandparent of the child).
    pub fn generate_perceived_priorities(
        &self, parent_pr: Priority, real_pes: Vec<(Event, Priority)>,
    ) -> Vec<(Event, Priority)> {
        let mut perceived_pes = vec![];
        #[allow(clippy::comparison_chain)]
        if parent_pr == Priority::UNFOCUSED {
            for child in real_pes {
                perceived_pes.push((child.0, Priority::UNFOCUSED));
            }
            // leave the children alone! they're fine
        } else if parent_pr < Priority::UNFOCUSED {
            // "Focused or greater"
            for child in real_pes {
                let pr = match true {
                    _ if child.1 == Priority::UNFOCUSED => Priority::UNFOCUSED,
                    _ if child.1 < parent_pr => child.1,
                    _ => parent_pr,
                };
                perceived_pes.push((child.0, pr));
            }
        }
        perceived_pes
    }

    pub fn perceived_priorities_of_eo(&self) -> Vec<(Event, Priority)> {
        let pr = self.pane.get_element_priority();
        let pes = self.eo.receivable(); // registered receivable events
        self.generate_perceived_priorities(pr, pes)
    }

    pub fn change_priority_for_el(
        &self, ctx: &Context, el_id: ElementID, p: Priority,
    ) -> ReceivableEventChanges {
        let mut ic = self.eo.change_priority_for_el(ctx, el_id, p);
        let mut to_add = vec![];

        // Check if any of the ic.remove match pane.self_evs. If so, add those events to
        // the ic.add.
        //
        // NOTE: this is necessary because:
        // 1. An event passed in the ic.remove will remove ALL instances of an
        // event registered to the ElementOrganizer (eo) of the parent of this
        // element. This is true because all events in the parent of this element
        // are registered with the ID of THIS element.
        //    e.g. if EventX is being passed in the ic.remove and EventX occurs
        //    twice in the prioritizer of the EO of the parent of this element, BOTH
        //    instances of EventX will be removed when the EO processes the
        //    ReceivableEventChanges.
        // 2. If this element has registered EventX as a SelfEv and EventX is also
        // passed up in the ic.remove, then EventX will be removed from the parent
        // organizer and this element will no longer be able to recieve EventX even
        // though it still wants to.
        //
        // NOTE: Leaving the remove event in the ic.remove removes artifacts further
        // up the tree. i.e, if we simply remove the event from the ic.remove, then
        // the parent of this element will have an artifact registration for an
        // event that serves no purpose.
        //
        // NOTE: If there are duplicate events in the ic.remove, then the
        // following code will add duplicate events to the ic.add. This will
        // result in duplicate events registered with the same priority and ID in
        // this element's parent. This seems harmless and is probably more efficient
        // than checking for duplicates.
        for rm in ic.remove.iter() {
            for self_ev in self.pane.self_evs.borrow().0.iter() {
                if *rm == self_ev.0 {
                    to_add.push((self_ev.0.clone(), self_ev.1));
                }
            }
        }
        ic.set_add_evs(to_add);
        ic
    }

    // -------------------------------------
    // Element functions

    //pub fn get_element_by_id(&self, el_id: &ElementID) -> Option<Rc<RefCell<dyn Element>>> {
    pub fn get_element(&self, el_id: &ElementID) -> Option<Rc<RefCell<dyn Element>>> {
        self.eo.get_element(el_id)
    }

    // TRANSLATION: SetZIndexForElement set_z_index_for_element
    pub fn update_el_z_index(&self, el_id: &ElementID, z: ZIndex) {
        self.eo.update_el_z_index(el_id, z);
    }
}

impl Element for ParentPane {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }

    fn id(&self) -> ElementID {
        self.pane.id()
    }

    fn receivable(&self) -> Vec<(Event, Priority)> {
        let mut pes = self.perceived_priorities_of_eo();
        pes.extend(self.pane.receivable());
        pes
    }

    // primarily a placeholder function. An element using the parent pane should
    // write their own receive_event function.
    // TODO verify that this code is or isn't used anywhere
    //                                               (captured, resp         )
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        debug!("ParentPane({}) receive_event_inner: {:?}", self.id(), ev);
        debug!(
            "ParentPane({}) (self.priority: {}) receivable: {:?}",
            self.id(),
            self.pane.get_element_priority(),
            self.receivable()
        );

        match ev {
            Event::Refresh => {
                self.eo.refresh(ctx);
                (false, EventResponses::default())
            }
            Event::Mouse(me) => {
                let (_, resps) = self
                    .eo
                    .mouse_event_process(ctx, &me, Box::new(self.clone()));
                (true, resps)
            }
            Event::ExternalMouse(me) => {
                // send the mouse event to all the children
                (
                    false,
                    self.eo
                        .external_mouse_event_process(ctx, &me, Box::new(self.clone())),
                )
            }
            Event::KeyCombo(ke) => {
                //debug!("ParentPane::receive_key_event: {}, {:?}", self.id(), ke);
                // convert ke to Vec<crossterm::event::KeyEvent>
                let ke = ke.into_iter().filter_map(|kp| kp.get_key()).collect();
                let mep = self.eo.key_events_process(ctx, ke, Box::new(self.clone()));
                let Some((_, resps)) = mep else {
                    return (true, EventResponses::default());
                };
                (true, resps)
            }
            _ => self.pane.receive_event(ctx, ev),
        }
    }

    // ChangePriority returns a priority change (InputabilityChanges) to its
    // parent organizer so as to update the priority of all events registered to
    // this element.
    //
    // NOTE: The priority changes (ic) that this parent pane sends up is the
    // combination of:
    //   - this element's priority changes (the SelfEvs, aka the
    //     Self Receivable Events)
    //   - the "perceived priorities" of the childens' Receivable Events
    //     (aka the results of the child's Receivable() function) The "perceived
    //     priorities" are the effective priority FROM the perspective of the
    //     element ABOVE this element in the tree.

    fn change_priority(&self, ctx: &Context, pr: Priority) -> ReceivableEventChanges {
        // first change the priority of the self evs. These are "this elements
        // priority changes". NO changes should be made to the childen,
        // the perceived priorities of the children should be interpreted.
        let mut ic = self.pane.change_priority(ctx, pr);

        // update the perceived priorities of the children
        for (_, el_details) in self.eo.els.borrow().iter() {
            let pes = el_details.el.borrow().receivable(); // self evs (and child eo's evs)
            for pe in self.generate_perceived_priorities(pr, pes) {
                ic.update_priority_for_ev(pe.0, pe.1);
            }
        }

        ic
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        if !*self.get_visible().borrow() {
            return vec![];
        }
        let mut out = self.pane.drawing(ctx);
        out.extend(self.eo.all_drawing(ctx));
        out
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_parent(&self, up: Box<dyn Parent>) {
        self.pane.set_parent(up)
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

impl Parent for ParentPane {
    // Passes changes to inputability to this element's parent element. If
    // updateThisElementsPrioritizers is TRUE then this element's prioritizers should be updated
    // using the given IC. This should be set to false when an upwards propagation is being
    // initiated as all of the changes to the prioritzers should have already been handled. The
    // boolean should be set to true on all further calls as the changes are propagated upstream so
    // as to update the ancestors' prioritizers.
    //
    // childEl is the element which is invoking the propagation from BELOW this parent pane. This
    // is used by the parent to determine which events/cmds to update the prioritizers for.
    //
    // The propagateEl is the element to send further upward propagation to. Typically this means
    // the Element which is inheriting THIS parent pane.
    //
    // NOTE: propagateEl is necessary as the parent pane will usually have registered an element
    // that extends ParentPane. If this ParentPane sent itself, it would not match the child
    // registered in the parent's EO.
    //
    // NOTE this function should be extended from if the parent pane is used as a base for a more
    // complex element. As the developer you should be fulfilling the
    // propagate_responses_upward function directly.
    fn propagate_responses_upward(&self, child_el_id: &ElementID, mut resps: EventResponses) {
        self.eo
            .partially_process_ev_resps(child_el_id, &mut resps, Box::new(self.clone()));
        if let Some(up) = self.pane.parent.borrow_mut().deref() {
            up.propagate_responses_upward(&self.id(), resps);
        }
    }

    fn get_store_item(&self, key: &str) -> Option<Vec<u8>> {
        self.el_store.borrow().get(key).cloned()
    }

    fn set_store_item(&self, key: &str, value: Vec<u8>) {
        self.el_store.borrow_mut().insert(key.to_string(), value);
    }
}
