use {
    crate::{
        widgets::{
            HorizontalSBPositions, HorizontalScrollbar, VerticalSBPositions, VerticalScrollbar,
        },
        Context, DrawCh, DrawChPos, DrawChPosVec, DynLocationSet, DynVal, Element, ElementID,
        Event, EventResponses, Loc, Parent, ParentPane, Priority, ReceivableEventChanges,
        SelfReceivableEvents, Size, Style,
    },
    crossterm::event::{KeyModifiers, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

/// PaneScrollable is a simple pane which exhibits offsets for the content.
/// The size of the view is fixed, determined by the view_height and view_width.
/// Additionally mouse scroll functionality is provided.
#[derive(Clone)]
pub struct PaneScrollable {
    pane: ParentPane,
    content_width: Rc<RefCell<usize>>, // TODO will need to adjust with scrollbar hooks
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
                // adjust the pos of the mouse event
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

#[derive(Clone)]
pub struct PaneWithScrollbars {
    pane: ParentPane,
    inner_pane: PaneScrollable,

    pub last_size: Rc<RefCell<Size>>, // needed for knowing when to resize scrollbars

    pub x_scrollbar: Rc<RefCell<Option<HorizontalScrollbar>>>,
    pub y_scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
    // for when there are two scrollbars
    pub corner_decor: Rc<RefCell<DrawCh>>,
}

impl PaneWithScrollbars {
    pub const KIND: &'static str = "pane_with_scrollbars";

    pub fn new(
        ctx: &Context, width: usize, height: usize, x_scrollbar_op: HorizontalSBPositions,
        y_scrollbar_op: VerticalSBPositions,
    ) -> Self {
        let pane = ParentPane::new(ctx, Self::KIND);
        let x_scrollbar = Rc::new(RefCell::new(None));
        let y_scrollbar = Rc::new(RefCell::new(None));
        let corner_decor = Rc::new(RefCell::new(DrawCh::new('⁙', Style::default_const())));

        let inner_pane = PaneScrollable::new(ctx, width, height);

        // THERE is a strange issue with the scrollbars here, using the HorizontalScrollbar as an
        // example:
        //  - consider the example were both horizontal and vertical scrollbars are present:
        //     - VerticalSBPositions is ToTheRight
        //     - HorizontalSBPositions is Below
        //  - we want the width of the horizontal scrollbar to be the width of the inner_pane
        //    aka. flex(1.0)-fixed(1).
        //  - however internally the width kind of needs to be flex(1.0) as when it comes time
        //    to draw the scrollbar the context is calculated given its provided dimensions
        //    and thus the drawing would apply the width of flex(1.0)-fixed(1) to the context which has
        //    already had the fixed(1) subtracted from it, thus resulting in two subtractions.
        //  - the solution is to have the width as a fixed size, and adjust it with each resize

        let (x_sb_size, x_sc_start_x) = match y_scrollbar_op {
            VerticalSBPositions::None => (DynVal::new_full(), DynVal::new_fixed(0)),
            VerticalSBPositions::ToTheLeft => (
                DynVal::new_full()
                    .minus(1.into())
                    .get_val(ctx.s.width)
                    .into(),
                DynVal::new_fixed(1),
            ),
            VerticalSBPositions::ToTheRight => (
                DynVal::new_full()
                    .minus(1.into())
                    .get_val(ctx.s.width)
                    .into(),
                DynVal::new_fixed(0),
            ),
        };
        let x_sb = HorizontalScrollbar::new(ctx, x_sb_size, *inner_pane.content_width.borrow());
        match x_scrollbar_op {
            HorizontalSBPositions::None => {}
            HorizontalSBPositions::Above => {
                x_sb.set_at(x_sc_start_x, DynVal::new_fixed(0));
            }
            HorizontalSBPositions::Below => {
                x_sb.set_at(x_sc_start_x, DynVal::new_full().minus(1.into()));
            }
        }

        let (y_sb_size, y_sc_start_y) = match x_scrollbar_op {
            HorizontalSBPositions::None => (DynVal::new_full(), DynVal::new_fixed(0)),
            HorizontalSBPositions::Above => (
                DynVal::new_full()
                    .minus(1.into())
                    .get_val(ctx.s.height)
                    .into(),
                DynVal::new_fixed(1),
            ),
            HorizontalSBPositions::Below => (
                DynVal::new_full()
                    .minus(1.into())
                    .get_val(ctx.s.height)
                    .into(),
                DynVal::new_fixed(0),
            ),
        };

        let y_sb = VerticalScrollbar::new(ctx, y_sb_size, *inner_pane.content_height.borrow());
        match y_scrollbar_op {
            VerticalSBPositions::None => {}
            VerticalSBPositions::ToTheRight => {
                y_sb.set_at(DynVal::new_full().minus(1.into()), y_sc_start_y);
            }
            VerticalSBPositions::ToTheLeft => {
                y_sb.set_at(DynVal::new_fixed(0), y_sc_start_y);
            }
        }

        let inner_pane_start_x = if matches!(y_scrollbar_op, VerticalSBPositions::ToTheLeft) {
            DynVal::new_fixed(1)
        } else {
            DynVal::new_fixed(0)
        };
        let inner_pane_start_y = if matches!(x_scrollbar_op, HorizontalSBPositions::Above) {
            DynVal::new_fixed(1)
        } else {
            DynVal::new_fixed(0)
        };
        let inner_pane_width = if matches!(y_scrollbar_op, VerticalSBPositions::None) {
            DynVal::new_full()
        } else {
            DynVal::new_full().minus(DynVal::new_fixed(1))
        };
        let inner_pane_height = if matches!(x_scrollbar_op, HorizontalSBPositions::None) {
            DynVal::new_full()
        } else {
            DynVal::new_full().minus(DynVal::new_fixed(1))
        };

        let loc = inner_pane.get_dyn_location_set();
        loc.borrow_mut().set_start_x(inner_pane_start_x);
        loc.borrow_mut().set_start_y(inner_pane_start_y);
        loc.borrow_mut().set_dyn_width(inner_pane_width);
        loc.borrow_mut().set_dyn_height(inner_pane_height);

        if !matches!(x_scrollbar_op, HorizontalSBPositions::None) {
            let inner_pane_ = inner_pane.clone();
            let hook = Box::new(move |ctx, x| inner_pane_.set_content_x_offset(&ctx, x));
            *x_sb.position_changed_hook.borrow_mut() = Some(hook);
            *x_scrollbar.borrow_mut() = Some(x_sb.clone());
            pane.add_element(Box::new(x_sb.clone()));
        }

        if !matches!(y_scrollbar_op, VerticalSBPositions::None) {
            let inner_pane_ = inner_pane.clone();
            let hook = Box::new(move |ctx, y| inner_pane_.set_content_y_offset(&ctx, y));
            *y_sb.position_changed_hook.borrow_mut() = Some(hook);
            *y_scrollbar.borrow_mut() = Some(y_sb.clone());
            pane.add_element(Box::new(y_sb.clone()));
        }

        pane.add_element(Box::new(inner_pane.clone()));
        inner_pane.change_priority(Priority::Focused);
        pane.change_priority(Priority::Focused);

        Self {
            pane,
            inner_pane,
            x_scrollbar,
            last_size: Rc::new(RefCell::new(ctx.s)),
            y_scrollbar,
            corner_decor,
        }
    }

    pub fn add_element(&self, el: Box<dyn Element>) {
        self.inner_pane.add_element(el.clone());
    }
    pub fn remove_element(&self, el_id: &ElementID) {
        self.inner_pane.remove_element(el_id);
    }
    pub fn clear_elements(&self) {
        self.inner_pane.clear_elements();
    }

    pub fn ensure_scrollbar_size(&self, ctx: &Context) {
        if *self.last_size.borrow() != ctx.s {
            let x_sb = self.x_scrollbar.borrow();
            if let Some(x_sb) = x_sb.as_ref() {
                let w: DynVal = DynVal::new_full()
                    .minus(DynVal::new_fixed(1))
                    .get_val(ctx.s.width)
                    .into();
                x_sb.set_dyn_width(w.clone(), w, None);
            }
            let y_sb = self.y_scrollbar.borrow();
            if let Some(y_sb) = y_sb.as_ref() {
                let h: DynVal = DynVal::new_full()
                    .minus(DynVal::new_fixed(1))
                    .get_val(ctx.s.height)
                    .into();
                y_sb.set_dyn_height(h.clone(), h, None);
            }
            *self.last_size.borrow_mut() = ctx.s;
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for PaneWithScrollbars {
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.ensure_scrollbar_size(ctx);
        self.pane.drawing(ctx)
    }
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.ensure_scrollbar_size(ctx);

        let out = self.pane.receive_event_inner(ctx, ev);
        if let Some(sb) = self.x_scrollbar.borrow().as_ref() {
            sb.external_change(
                ctx,
                *self.inner_pane.content_offset_x.borrow(),
                *self.inner_pane.content_width.borrow(),
            );
        }
        if let Some(sb) = self.y_scrollbar.borrow().as_ref() {
            sb.external_change(
                ctx,
                *self.inner_pane.content_offset_y.borrow(),
                *self.inner_pane.content_height.borrow(),
            );
        }
        out
    }
}
