use {
    crate::{
        //widgets::{
        //    common, HorizontalSBPositions, HorizontalScrollbar, Label, Selectability,
        //    VerticalSBPositions, VerticalScrollbar, WBStyles, Widget, WidgetBase, Widgets,
        //},
        Context,
        DrawChPos,
        DrawChPosVec,
        DynLocationSet,
        Element,
        ElementID,
        Event,
        EventResponses,
        Loc,
        ParentPane,
        Priority,
        ReceivableEventChanges,
        SortingHat,
        UpwardPropagator,
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
        self.pane.eo.add_element(el.clone(), None);
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
                        if dx < 0 {
                            let x = self
                                .content_offset_x
                                .borrow()
                                .saturating_sub((-dx) as usize);
                            *self.content_offset_x.borrow_mut() = x;
                        } else {
                            *self.content_offset_x.borrow_mut() += dx as usize;
                            if *self.content_offset_x.borrow() + ctx.s.width as usize
                                > *self.content_width.borrow()
                            {
                                *self.content_offset_x.borrow_mut() = self
                                    .content_width
                                    .borrow()
                                    .saturating_sub(ctx.s.width as usize);
                            }
                        }
                        if dy < 0 {
                            let y = self
                                .content_offset_y
                                .borrow()
                                .saturating_sub((-dy) as usize);
                            *self.content_offset_y.borrow_mut() = y;
                        } else {
                            *self.content_offset_y.borrow_mut() += dy as usize;
                            if *self.content_offset_y.borrow() + ctx.s.height as usize
                                > *self.content_height.borrow()
                            {
                                *self.content_offset_y.borrow_mut() = self
                                    .content_height
                                    .borrow()
                                    .saturating_sub(ctx.s.height as usize);
                            }
                        }

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

/*
#[derive(Clone)]
pub struct PaneScrollableWithScrollbars {
    pane: VerticalStack,

    pub inner_pane: Rc<RefCell<dyn Element>>,

    view_height: Rc<RefCell<DynVal>>,
    view_width: Rc<RefCell<DynVal>>,
    content_width: Rc<RefCell<usize>>,
    content_height: Rc<RefCell<usize>>,
    content_view_offset_x: Rc<RefCell<usize>>,
    content_view_offset_y: Rc<RefCell<usize>>,

    pub x_scrollbar: Rc<RefCell<Option<HorizontalScrollbar>>>,
    pub y_scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
    // for when there are two scrollbars
    pub corner_decor: Rc<RefCell<DrawCh>>,
}

impl PaneScrollableWithScrollbars {
    pub fn new(
        hat: &SortingHat, ctx: &Context, main_pane: Box<dyn Element>, x_scrollbar_op: HorizontalSBPositions,
        y_scrollbar_op: VerticalSBPositions,
    ) -> Self {
        let pane = VerticalStack::new(hat);
        let x_scrollbar = Rc::new(RefCell::new(None));
        let y_scrollbar = Rc::new(RefCell::new(None));
        let corner_decor = Rc::new(RefCell::new(DrawCh::new('⁙', false, Style::new())));

        let main_height =

        match (y_scrollbar_op, x_scrollbar_op) {
            (VerticalSBPositions::None, HorizontalSBPositions::None) => {}
            (VerticalSBPositions::Right, HorizontalSBPositions::None) => {
                let y_scrollbar = VerticalScrollbar::new(hat, y_scrollbar_op);
                pane.push(ctx, y_scrollbar.clone());
                *y_scrollbar.borrow_mut() = Some(y_scrollbar);
            }
        }

        Self {
            pane,
            x_scrollbar_op,
            y_scrollbar_op,
            x_scrollbar,
            y_scrollbar,
            corner_decor,
        }
    }
}
*/
