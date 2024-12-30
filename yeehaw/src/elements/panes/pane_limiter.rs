use crate::*;

/// limits the context size to the dimensions provided
#[derive(Clone)]
pub struct PaneLimiter {
    inner: Box<dyn Element>,
    size: Rc<RefCell<Size>>,
}

impl PaneLimiter {
    pub const KIND: &'static str = "pane_limiter";

    pub fn new(el: Box<dyn Element>, width: u16, height: u16) -> Self {
        let size = Size::new(width, height);
        let size = Rc::new(RefCell::new(size));
        PaneLimiter { inner: el, size }
    }

    pub fn inner_dr(&self, dr: &DrawRegion) -> DrawRegion {
        let s = self.size.borrow();
        let width = dr.get_width().min(s.width);
        let height = dr.get_height().min(s.height);
        dr.clone().with_width(width).with_height(height)
    }
}

#[yeehaw_derive::impl_element_from(inner)]
impl Element for PaneLimiter {
    fn receive_event(&self, ctx: &Context, mut ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::Mouse(ref mut me) => {
                me.dr = self.inner_dr(&me.dr);
            }
            Event::ExternalMouse(ref mut ke) => {
                ke.dr = self.inner_dr(&ke.dr);
            }
            _ => {}
        }
        self.inner.receive_event(ctx, ev)
    }
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        let inner_dr = self.inner_dr(dr);
        self.inner.drawing(ctx, &inner_dr, force_update)
    }
}
