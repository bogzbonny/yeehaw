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

    pub fn inner_ctx(&self, ctx: &Context) -> Context {
        let s = self.size.borrow();
        let width = ctx.get_width().min(s.width);
        let height = ctx.get_height().min(s.height);
        ctx.clone().with_width(width).with_height(height)
    }
}

#[yeehaw_derive::impl_element_from(inner)]
impl Element for PaneLimiter {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let inner_ctx = self.inner_ctx(ctx);
        self.inner.receive_event_inner(&inner_ctx, ev)
    }
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        let inner_ctx = self.inner_ctx(ctx);
        self.inner.drawing(&inner_ctx, force_update)
    }
}
