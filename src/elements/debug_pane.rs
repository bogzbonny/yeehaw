use {
    crate::{
        element::ReceivableEventChanges, Color, Context, DrawChPos, DynLocationSet, DynVal,
        Element, ElementID, Event, EventResponses, Pane, Priority, SortingHat, Style,
        UpwardPropagator, ZIndex,
    },
    std::{cell::RefCell, rc::Rc},
};

// displays the size
#[derive(Clone)]
pub struct DebugSizePane {
    pub pane: Pane,
    pub text: Rc<RefCell<String>>,
}

impl DebugSizePane {
    pub fn new(hat: &SortingHat) -> DebugSizePane {
        DebugSizePane {
            pane: Pane::new(hat, "debug_size_pane"),
            text: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn with_text(self, text: String) -> Self {
        *self.text.borrow_mut() = text;
        self
    }

    pub fn with_height(self, h: DynVal) -> Self {
        self.pane.set_dyn_height(h);
        self
    }

    pub fn with_width(self, w: DynVal) -> Self {
        self.pane.set_dyn_width(w);
        self
    }

    pub fn with_z(self, z: ZIndex) -> Self {
        self.pane.set_z(z);
        self
    }
}

impl Element for DebugSizePane {
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
        let size = ctx.s;
        let s = format!("{}x{} {}", size.width, size.height, self.text.borrow());
        DrawChPos::new_from_string(
            s,
            0,
            0,
            Style::default_const().with_bg(Color::BLACK).with_fg(Color::WHITE),
        )
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
