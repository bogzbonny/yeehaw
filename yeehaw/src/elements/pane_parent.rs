use {
    crate::{
        Color, Context, DrawCh, DrawChs2D, DrawUpdate, DynLocation, DynLocationSet, DynVal,
        Element, ElementID, ElementOrganizer, Event, EventResponses, Pane, Parent,
        ReceivableEvents, Size, Style, ZIndex,
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
        let pane = Pane::new(ctx, kind).with_focused(true);
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

    pub fn focused(self) -> Self {
        *self.pane.focused.borrow_mut() = true;
        self
    }

    pub fn unfocused(self) -> Self {
        *self.pane.focused.borrow_mut() = false;
        self
    }

    pub fn with_element(self, el: Box<dyn Element>) -> Self {
        self.add_element(el); // ignore the response as this is used during initialization
        self
    }

    pub fn add_element(&self, el: Box<dyn Element>) {
        self.eo.add_element(el, Some(Box::new(self.clone())))
    }

    pub fn remove_element(&self, el_id: &ElementID) {
        self.eo.remove_element(el_id)
    }

    pub fn clear_elements(&self) {
        self.eo.clear_elements()
    }

    pub fn has_elements(&self) -> bool {
        !self.eo.els.borrow().is_empty()
    }

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

    pub fn focus(&self) {
        self.set_focused(true);
    }

    pub fn unfocus(&self) {
        self.set_focused(false);
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
    fn can_receive(&self, ev: &Event) -> bool {
        self.get_focused() && (self.pane.can_receive(ev) || self.eo.can_receive(ev))
    }

    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>> {
        if self.get_focused() {
            let mut rec = self.eo.receivable();
            rec.extend(self.pane.receivable());
            rec
        } else {
            Vec::with_capacity(0)
        }
    }

    //                                                     (captured, resp         )
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        //if let Event::Mouse(_) = ev {
        //    debug!("id: {}, focused: {}", self.id(), self.get_focused());
        //}
        self.eo.event_process(ctx, ev, Box::new(self.clone()))
    }

    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        if !self.get_visible() {
            return Vec::with_capacity(0);
        }
        let mut out = self.pane.drawing(ctx, force_update);
        out.extend(self.eo.all_drawing_updates(ctx, force_update));
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

    fn get_parent_focused(&self) -> bool {
        self.pane.get_focused()
    }

    fn get_id(&self) -> ElementID {
        self.pane.id()
    }
}
