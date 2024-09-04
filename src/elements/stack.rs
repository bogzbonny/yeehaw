use {
    crate::{
        Context, DrawChPos, DynLocationSet, Element, ElementID, Event, EventResponses, ParentPane,
        Priority, ReceivableEventChanges, SortingHat, UpwardPropagator,
    },
    std::{cell::RefCell, rc::Rc},
};

pub struct VerticalStack {
    pub pane: ParentPane,
    pub els: Vec<Rc<RefCell<dyn Element>>>,
}

impl VerticalStack {
    pub fn new(hat: &SortingHat) -> Self {
        Self {
            pane: ParentPane::new(hat, "vertical_stack"),
            els: Vec::new(),
        }
    }

    // add an element to the end of the stack resizing the other elements
    // in order to fit the new element
    pub fn push(&mut self, el: Rc<RefCell<dyn Element>>) {
        // determine the min and max dimension of the new element
        let (min, max) = el.borrow().get_dyn_location_set().borrow().get_height();

        self.els.push(el.clone());
        self.pane.add_element(el);
    }

    //pub fn insert(&mut self, idx: usize, el: Rc<RefCell<dyn Element>>) {
    //    self.els.insert(idx, el.clone());
    //    self.pane.add_element(el);
    //    // TODO set the locations
    //    // XXX
    //}

    //pub fn remove(&mut self, idx: usize) {
    //    self.els.remove(idx);
    //    self.pane.remove_element(idx);
    //    // TODO set the locations
    //    // XXX
    //}

    //pub fn clear(&mut self) {
    //    self.els.clear();
    //    self.pane.clear_elements();
    //    // TODO set the locations
    //    // XXX
    //}
}

pub struct HorizontalStack {
    pub pane: ParentPane,
    pub els: Vec<Rc<RefCell<dyn Element>>>,
}

impl Element for VerticalStack {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.pane.receive_event(ctx, ev.clone())
    }
    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.pane.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
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

impl Element for HorizontalStack {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.pane.receive_event(ctx, ev.clone())
    }
    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.pane.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
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
