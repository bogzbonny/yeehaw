use {
    crate::{
        element::ReceivableEventChanges, Context, DrawChPos, Element, ElementID, Event,
        EventResponses, Pane, Priority, RgbColour, SclLocationSet, Style, UpwardPropagator,
    },
    std::{cell::RefCell, rc::Rc},
};

// displays the size
pub struct SizePane {
    pub pane: Pane,
}

impl Element for SizePane {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.pane.receive_event(ctx, ev.clone())
    }
    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let size = ctx.s;
        let s = format!("{}x{}", size.width, size.height);
        DrawChPos::new_from_string(
            s,
            0,
            0,
            Style::default()
                .with_bg(RgbColour::BLACK)
                .with_fg(RgbColour::WHITE),
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
    fn get_scl_location_set(&self) -> Rc<RefCell<SclLocationSet>> {
        self.pane.get_scl_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.pane.get_visible()
    }
}
