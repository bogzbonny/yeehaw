use {
    crate::{
        widgets::{
            HorizontalSBPositions, HorizontalScrollbar, VerticalSBPositions, VerticalScrollbar,
        },
        Context, DrawCh, DrawChPos, DrawChPosVec, DynLocationSet, DynVal, Element, ElementID,
        Event, EventResponses, HorizontalStack, Loc, ParentPane, Priority, ReceivableEventChanges,
        SortingHat, Style, Parent, VerticalStack,
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

    pub fn new(hat: &SortingHat, width: usize, height: usize) -> Self {
        Self {
            pane: ParentPane::new(hat, Self::KIND),
            content_width: Rc::new(RefCell::new(width)),
            content_height: Rc::new(RefCell::new(height)),
            content_offset_x: Rc::new(RefCell::new(0)),
            content_offset_y: Rc::new(RefCell::new(0)),
            scroll_rate: Rc::new(RefCell::new(Some(3))),
        }
    }

    pub fn add_element(&self, el: Rc<RefCell<dyn Element>>) {
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

    pub fn set_content_x_offset(&self, ctx: &Context, x: usize) {
        *self.content_offset_x.borrow_mut() = if x > self
            .content_width
            .borrow()
            .saturating_sub(self.get_width_val(ctx))
        {
            self.content_width
                .borrow()
                .saturating_sub(self.get_width_val(ctx))
        } else {
            x
        };
    }

    pub fn set_content_y_offset(&self, ctx: &Context, y: usize) {
        *self.content_offset_y.borrow_mut() = if y > self
            .content_height
            .borrow()
            .saturating_sub(self.get_height_val(ctx))
        {
            self.content_height
                .borrow()
                .saturating_sub(self.get_height_val(ctx))
        } else {
            y
        };
    }
}

impl Element for PaneScrollable {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }

    fn receive_event_inner(&self, ctx: &Context, mut ev: Event) -> (bool, EventResponses) {
        let inner_ctx = self.inner_ctx(ctx);
        match &mut ev {
            Event::Mouse(me) => {
                // adjust the pos of the mouse event
                me.column += *self.content_offset_x.borrow() as u16;
                me.row += *self.content_offset_y.borrow() as u16;

                let Some(sc_rate) = *self.scroll_rate.borrow() else {
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
                    None => self.pane.receive_event(&inner_ctx, ev),
                }
            }
            _ => self.pane.receive_event(&inner_ctx, ev),
        }
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
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
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn Parent>) {
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

#[derive(Clone)]
pub struct PaneWithScrollbars {
    pane: HorizontalStack,
    inner_pane: PaneScrollable,

    pub x_scrollbar: Rc<RefCell<Option<HorizontalScrollbar>>>,
    pub y_scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
    // for when there are two scrollbars
    pub corner_decor: Rc<RefCell<DrawCh>>,
}

impl PaneWithScrollbars {
    pub fn new(
        hat: &SortingHat, ctx: &Context, width: usize, height: usize,
        x_scrollbar_op: HorizontalSBPositions, y_scrollbar_op: VerticalSBPositions,
    ) -> Self {
        let pane = HorizontalStack::new(hat);
        let x_scrollbar = Rc::new(RefCell::new(None));
        let y_scrollbar = Rc::new(RefCell::new(None));
        let corner_decor = Rc::new(RefCell::new(DrawCh::new('⁙', Style::default_const())));

        let inner_pane = PaneScrollable::new(hat, width, height);

        let y_scrollbar_size = if matches!(x_scrollbar_op, HorizontalSBPositions::None) {
            DynVal::new_flex(1.0)
        } else {
            DynVal::new_flex(1.0).minus(DynVal::new_fixed(1))
        };
        let x_scrollbar_size = DynVal::new_flex(1.0);

        let y_sb =
            VerticalScrollbar::new(hat, y_scrollbar_size, *inner_pane.content_height.borrow());
        let x_sb =
            HorizontalScrollbar::new(hat, x_scrollbar_size, *inner_pane.content_width.borrow());

        let inner_pane_ = inner_pane.clone();
        let hook = Box::new(move |ctx, x| inner_pane_.set_content_x_offset(&ctx, x));
        *x_sb.position_changed_hook.borrow_mut() = Some(hook);

        let inner_pane_ = inner_pane.clone();
        let hook = Box::new(move |ctx, y| inner_pane_.set_content_y_offset(&ctx, y));
        *y_sb.position_changed_hook.borrow_mut() = Some(hook);

        let vs = VerticalStack::new(hat);

        match (y_scrollbar_op, x_scrollbar_op) {
            (VerticalSBPositions::None, HorizontalSBPositions::None) => {}
            (VerticalSBPositions::ToTheRight, HorizontalSBPositions::None) => {
                *y_scrollbar.borrow_mut() = Some(y_sb.clone());
                pane.push(ctx, Rc::new(RefCell::new(inner_pane.clone())));
                pane.push(ctx, Rc::new(RefCell::new(y_sb.clone())));
            }
            (VerticalSBPositions::ToTheLeft, HorizontalSBPositions::None) => {
                *y_scrollbar.borrow_mut() = Some(y_sb.clone());
                pane.push(ctx, Rc::new(RefCell::new(y_sb.clone())));
                pane.push(ctx, Rc::new(RefCell::new(inner_pane.clone())));
            }
            (VerticalSBPositions::None, HorizontalSBPositions::Above) => {
                *x_scrollbar.borrow_mut() = Some(x_sb.clone());
                vs.push(ctx, Rc::new(RefCell::new(x_sb.clone())));
                vs.push(ctx, Rc::new(RefCell::new(inner_pane.clone())));
                pane.push(ctx, Rc::new(RefCell::new(vs.clone())));
            }
            (VerticalSBPositions::None, HorizontalSBPositions::Below) => {
                *x_scrollbar.borrow_mut() = Some(x_sb.clone());
                vs.push(ctx, Rc::new(RefCell::new(inner_pane.clone())));
                vs.push(ctx, Rc::new(RefCell::new(x_sb.clone())));
                pane.push(ctx, Rc::new(RefCell::new(vs.clone())));
            }
            (VerticalSBPositions::ToTheRight, HorizontalSBPositions::Above) => {
                *y_scrollbar.borrow_mut() = Some(y_sb.clone());
                *x_scrollbar.borrow_mut() = Some(x_sb.clone());
                vs.push(ctx, Rc::new(RefCell::new(x_sb.clone())));
                vs.push(ctx, Rc::new(RefCell::new(inner_pane.clone())));
                pane.push(ctx, Rc::new(RefCell::new(vs.clone())));
                pane.push(ctx, Rc::new(RefCell::new(y_sb.clone())));
            }
            (VerticalSBPositions::ToTheRight, HorizontalSBPositions::Below) => {
                *y_scrollbar.borrow_mut() = Some(y_sb.clone());
                *x_scrollbar.borrow_mut() = Some(x_sb.clone());
                vs.push(ctx, Rc::new(RefCell::new(inner_pane.clone())));
                vs.push(ctx, Rc::new(RefCell::new(x_sb.clone())));
                pane.push(ctx, Rc::new(RefCell::new(vs.clone())));
                pane.push(ctx, Rc::new(RefCell::new(y_sb.clone())));
            }
            (VerticalSBPositions::ToTheLeft, HorizontalSBPositions::Above) => {
                *y_scrollbar.borrow_mut() = Some(y_sb.clone());
                *x_scrollbar.borrow_mut() = Some(x_sb.clone());
                vs.push(ctx, Rc::new(RefCell::new(x_sb.clone())));
                vs.push(ctx, Rc::new(RefCell::new(inner_pane.clone())));
                pane.push(ctx, Rc::new(RefCell::new(y_sb.clone())));
                pane.push(ctx, Rc::new(RefCell::new(vs.clone())));
            }
            (VerticalSBPositions::ToTheLeft, HorizontalSBPositions::Below) => {
                *y_scrollbar.borrow_mut() = Some(y_sb.clone());
                *x_scrollbar.borrow_mut() = Some(x_sb.clone());
                vs.push(ctx, Rc::new(RefCell::new(inner_pane.clone())));
                vs.push(ctx, Rc::new(RefCell::new(x_sb.clone())));
                pane.push(ctx, Rc::new(RefCell::new(y_sb.clone())));
                pane.push(ctx, Rc::new(RefCell::new(vs.clone())));
            }
        };

        inner_pane.change_priority(ctx, Priority::FOCUSED);
        pane.change_priority(ctx, Priority::FOCUSED);
        vs.change_priority(ctx, Priority::FOCUSED);

        Self {
            pane,
            inner_pane,
            x_scrollbar,
            y_scrollbar,
            corner_decor,
        }
    }

    pub fn add_element(&self, el: Rc<RefCell<dyn Element>>) {
        self.inner_pane.add_element(el.clone());
    }
    pub fn remove_element(&self, el_id: &ElementID) {
        self.inner_pane.remove_element(el_id);
    }
    pub fn clear_elements(&self) {
        self.inner_pane.clear_elements();
    }
}

impl Element for PaneWithScrollbars {
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
    fn set_upward_propagator(&self, up: Box<dyn Parent>) {
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
