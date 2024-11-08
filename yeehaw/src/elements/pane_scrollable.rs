use {
    crate::*,
    crossterm::event::{KeyModifiers, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

/// PaneScrollable is a simple pane which exhibits offsets for the content.
/// The size of the view is fixed, determined by the view_height and view_width.
/// Additionally mouse scroll functionality is provided.
#[derive(Clone)]
pub struct PaneScrollable {
    pane: ParentPane,
    content_width: Rc<RefCell<usize>>, /// TODO will need to adjust with scrollbar hooks
    content_height: Rc<RefCell<usize>>,
    content_offset_x: Rc<RefCell<usize>>,
    content_offset_y: Rc<RefCell<usize>>,

    /// how many characters to scroll on a scroll event, if None, then disable scroll
    pub scroll_rate: Rc<RefCell<Option<i16>>>,
}

impl PaneScrollable {
    pub const KIND: &'static str = "pane_scrollable";

    pub fn new(ctx: &Context, width: usize, height: usize) -> Self {
        Self {
            pane: ParentPane::new(ctx, Self::KIND).with_transparent(),
            content_width: Rc::new(RefCell::new(width)),
            content_height: Rc::new(RefCell::new(height)),
            content_offset_x: Rc::new(RefCell::new(0)),
            content_offset_y: Rc::new(RefCell::new(0)),
            scroll_rate: Rc::new(RefCell::new(Some(3))),
        }
    }

    pub fn add_element(&self, el: Box<dyn Element>) {
        self.pane.add_element(el.clone());
    }

    pub fn remove_element(&self, el_id: &ElementID) {
        self.pane.eo.remove_element(el_id);
    }

    pub fn clear_elements(&self) {
        self.pane.eo.clear_elements();
    }

    // ------------------------------------

    pub fn inner_ctx(&self, ctx: &Context) -> Context {
        let mut inner_ctx = ctx.clone();
        inner_ctx.s.height = *self.content_height.borrow() as u16;
        inner_ctx.s.width = *self.content_width.borrow() as u16;
        let x1 = *self.content_offset_x.borrow() as u16;
        let y1 = *self.content_offset_y.borrow() as u16;
        let x2 = x1 + ctx.s.width;
        let y2 = y1 + ctx.s.height;
        //debug!(
        //    "visible region: \n\tx1: {}, \n\tx2: {}, \n\ty1: {}, \n\ty2: {}",
        //    x1, x2, y1, y2
        //);
        let visible_region = Loc::new(x1, x2, y1, y2);
        inner_ctx.visible_region = Some(visible_region);
        inner_ctx
    }

    pub fn get_width_val(&self, ctx: &Context) -> usize {
        self.pane.get_dyn_location_set().borrow().get_width_val(ctx)
    }

    pub fn get_height_val(&self, ctx: &Context) -> usize {
        self.pane
            .get_dyn_location_set()
            .borrow()
            .get_height_val(ctx)
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for PaneScrollable {
    fn receive_event_inner(&self, ctx: &Context, mut ev: Event) -> (bool, EventResponses) {
        match &mut ev {
            Event::Mouse(me) => {
                /// adjust the pos of the mouse event
                me.column += *self.content_offset_x.borrow() as u16;
                me.row += *self.content_offset_y.borrow() as u16;

                let Some(sc_rate) = *self.scroll_rate.borrow() else {
                    let inner_ctx = self.inner_ctx(ctx);
                    return self.pane.receive_event(&inner_ctx, ev);
                };

                let scroll = match me.kind {
                    MouseEventKind::ScrollDown if me.modifiers == KeyModifiers::NONE => {
                        Some((0i16, sc_rate))
                    }
                    MouseEventKind::ScrollUp if me.modifiers == KeyModifiers::NONE => {
                        Some((0, -sc_rate))
                    }
                    MouseEventKind::ScrollDown if me.modifiers == KeyModifiers::SHIFT => {
                        Some((sc_rate, 0))
                    }
                    MouseEventKind::ScrollUp if me.modifiers == KeyModifiers::SHIFT => {
                        Some((-sc_rate, 0))
                    }
                    MouseEventKind::ScrollLeft => Some((-sc_rate, 0)),
                    MouseEventKind::ScrollRight => Some((sc_rate, 0)),
                    _ => None,
                };
                match scroll {
                    Some((dx, dy)) => {
                        let x = if dx < 0 {
                            self.content_offset_x
                                .borrow()
                                .saturating_sub((-dx) as usize)
                        } else {
                            *self.content_offset_x.borrow() + dx as usize
                        };
                        let y = if dy < 0 {
                            self.content_offset_y
                                .borrow()
                                .saturating_sub((-dy) as usize)
                        } else {
                            *self.content_offset_y.borrow() + dy as usize
                        };
                        self.set_content_x_offset(ctx, x);
                        self.set_content_y_offset(ctx, y);
                        (true, EventResponses::default())
                    }
                    None => {
                        let inner_ctx = self.inner_ctx(ctx);
                        self.pane.receive_event(&inner_ctx, ev)
                    }
                }
            }
            _ => {
                let inner_ctx = self.inner_ctx(ctx);
                self.pane.receive_event(&inner_ctx, ev)
            }
        }
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let inner_ctx = self.inner_ctx(ctx);
        let out = self.pane.drawing(&inner_ctx);
        let mut out = DrawChPosVec::new(out);
        let x_off = *self.content_offset_x.borrow();
        let y_off = *self.content_offset_y.borrow();
        let max_x = x_off + *self.content_width.borrow();
        let max_y = y_off + *self.content_height.borrow();
        out.adjust_for_offset_and_truncate(
            *self.content_offset_x.borrow(),
            *self.content_offset_y.borrow(),
            max_x,
            max_y,
        );
        out.0
    }

    fn set_content_x_offset(&self, ctx: &Context, x: usize) {
        let x = if x > self
            .content_width
            .borrow()
            .saturating_sub(ctx.s.width.into())
        {
            self.content_width
                .borrow()
                .saturating_sub(ctx.s.width.into())
        } else {
            x
        };
        *self.content_offset_x.borrow_mut() = x
    }

    fn set_content_y_offset(&self, ctx: &Context, y: usize) {
        let y = if y > self
            .content_height
            .borrow()
            .saturating_sub(ctx.s.height.into())
        {
            self.content_height
                .borrow()
                .saturating_sub(ctx.s.height.into())
        } else {
            y
        };
        *self.content_offset_y.borrow_mut() = y;
    }

    fn get_content_x_offset(&self) -> usize {
        *self.content_offset_x.borrow()
    }
    fn get_content_y_offset(&self) -> usize {
        *self.content_offset_y.borrow()
    }
    fn get_content_width(&self) -> usize {
        *self.content_width.borrow()
    }
    fn get_content_height(&self) -> usize {
        *self.content_height.borrow()
    }
}
