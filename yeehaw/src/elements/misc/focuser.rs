use {
    crate::*,
    crossterm::event::{MouseButton, MouseEventKind},
};

/// The Focuser element is a wrapper around another element that will focus the inner element when
/// the left mouse button is clicked on it. This is useful for usage with stack panes
#[derive(Clone)]
pub struct Focuser {
    pub inner: Box<dyn Element>,
}

impl Focuser {
    pub const KIND: &'static str = "focuser";
    pub fn new(inner: Box<dyn Element>) -> Focuser {
        Focuser { inner }
    }
}

#[yeehaw_derive::impl_element_from(inner)]
impl Element for Focuser {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let mut resps = EventResponses::default();
        if let Event::Mouse(me) = ev {
            match me.kind {
                MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left) => {
                    // set focused if unfocused
                    if !self.get_focused() {
                        resps.push(EventResponse::UnfocusOthers);
                        resps.push(EventResponse::Focus);
                    }
                }
                _ => {}
            }
        }
        let (captured, resps_) = self.inner.receive_event(ctx, ev.clone());
        resps.extend(resps_);
        (captured, resps)
    }
}
