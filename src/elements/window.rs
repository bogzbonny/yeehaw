use {
    crate::{
        widgets::Button, widgets::Label, Color, Context, DrawChPos, DynLocationSet, DynVal,
        Element, ElementID, Event, EventResponse, EventResponses, ParentPane, Priority,
        ReceivableEventChanges, SortingHat, Style, UpwardPropagator, VerticalStack, ZIndex,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// TODO border

// ---------------------------------------------------
#[derive(Clone)]
pub struct WindowPane {
    pub pane: VerticalStack,
    pub top_bar: Rc<RefCell<dyn Element>>,
    pub inner: Rc<RefCell<dyn Element>>,
    pub dragging: Rc<RefCell<Option<(u16, u16)>>>, // start location of the drag
}

/// Window border characters
/// NOTE top characters will intersect with the upper bar of the window
//pub struct WindowBorder {
//    pub top: char,
//    pub top_left: char,
//    pub top_right: char,

//    pub left: char,
//    pub right: char,

//    pub bottom: char,
//    pub bottom_right: char,
//    pub bottom_left: char,
//}

impl WindowPane {
    const KIND: &'static str = "window_pane";
    const CLOSE_WINDOW_MD_KEY: &'static str = "close_window";
    pub const Z_INDEX: ZIndex = 90;
    pub fn new(
        hat: &SortingHat, ctx: &Context, inner: Rc<RefCell<dyn Element>>, title: &str,
    ) -> Self {
        let vs = VerticalStack::new_with_kind(hat, Self::KIND);
        let top_bar = Rc::new(RefCell::new(BasicWindowTopBar::new(hat, ctx, title)));
        vs.push(ctx, top_bar.clone());
        vs.push(ctx, inner.clone());
        Self {
            pane: vs,
            top_bar,
            inner,
            dragging: Rc::new(RefCell::new(None)),
        }
    }

    pub fn at(self, x: DynVal, y: DynVal) -> Self {
        self.pane.pane.set_x_y(x, y);
        self
    }

    pub fn with_size(self, w: DynVal, h: DynVal) -> Self {
        self.pane.pane.pane.set_dyn_width(w);
        self.pane.pane.pane.set_dyn_height(h);
        self
    }

    pub fn with_width(self, w: DynVal) -> Self {
        self.pane.pane.pane.set_dyn_width(w);
        self
    }

    pub fn with_height(self, h: DynVal) -> Self {
        self.pane.pane.pane.set_dyn_height(h);
        self
    }
}

impl Element for WindowPane {
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
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());

        for resp in resps.iter_mut() {
            let mut close_window = false;
            if let EventResponse::Metadata((key, _)) = resp {
                if key == Self::CLOSE_WINDOW_MD_KEY {
                    close_window = true;
                }
            }
            // swap out the response for a destruct if the window should close
            if close_window {
                *resp = EventResponse::Destruct;
            }
        }

        let top_height = self
            .top_bar
            .borrow()
            .get_dyn_location_set()
            .borrow()
            .l
            .height(ctx);

        // process dragging
        match ev {
            Event::Mouse(me) => {
                if me.row as usize > top_height {
                    return (captured, resps);
                }
                let dragging = self.dragging.borrow().is_some();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.dragging.borrow_mut() = Some((me.column, me.row));
                    }
                    MouseEventKind::Drag(MouseButton::Left) if dragging => {
                        let (start_x, start_y) = self.dragging.borrow().expect("impossible");
                        let dx = me.column - start_x;
                        let dy = me.row - start_y;
                        let x = self.pane.pane.pane.get_start_x(ctx) + dx as i32;
                        let y = self.pane.pane.pane.get_start_y(ctx) + dy as i32;
                        self.pane
                            .pane
                            .set_x_y(DynVal::new_fixed(x), DynVal::new_fixed(y));
                    }
                    _ => {
                        *self.dragging.borrow_mut() = None;
                    }
                }
            }
            Event::ExternalMouse(ref me) => {
                let dragging = self.dragging.borrow().is_some();
                if dragging {
                    match me.kind {
                        MouseEventKind::Drag(MouseButton::Left) => {}
                        _ => {
                            *self.dragging.borrow_mut() = None;
                        }
                    }
                }
            }
            _ => {}
        }

        (captured, resps)
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

pub struct BasicWindowTopBar {
    pub pane: ParentPane,
    pub title: String,
}

impl BasicWindowTopBar {
    pub fn new(hat: &SortingHat, ctx: &Context, title: &str) -> Self {
        let pane = ParentPane::new(hat, "basic_window_top_bar");

        let close_button = Button::new(
            hat,
            ctx,
            "X",
            Box::new(|_ctx| {
                EventResponse::Metadata((
                    WindowPane::CLOSE_WINDOW_MD_KEY.to_string(),
                    Vec::with_capacity(0),
                ))
                .into()
            }),
        )
        .basic_button(Some(
            Style::default().with_fg(Color::BLACK).with_bg(Color::WHITE),
        ))
        .at(DynVal::new_fixed(0), DynVal::new_fixed(1));
        let title_label =
            Label::new(hat, ctx, title).at(DynVal::new_fixed(0), DynVal::new_flex(0.5));
        pane.add_element(Rc::new(RefCell::new(close_button)));
        pane.add_element(Rc::new(RefCell::new(title_label)));
        let out = Self {
            pane,
            title: title.to_string(),
        };
        out.pane.pane.set_dyn_height(DynVal::new_fixed(1));
        out
    }
}

impl Element for BasicWindowTopBar {
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
        self.pane.receive_event(ctx, ev)
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
