use {
    crate::{
        widgets::{Button, Label, WBStyles},
        Color, Context, DrawChPos, DynLocation, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponse, EventResponses, Parent, ParentPane, Priority, ReceivableEventChanges,
        SortingHat, Style, VerticalStack, ZIndex,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// TODO border
// TODO work in z-index changes when clicked if there are other windows

// ---------------------------------------------------
#[derive(Clone)]
pub struct WindowPane {
    pub pane: VerticalStack,
    pub top_bar: Rc<RefCell<dyn Element>>,
    pub inner: Rc<RefCell<dyn Element>>,
    pub dragging: Rc<RefCell<Option<(u16, u16)>>>, // start location of the drag
    pub maximized_restore: Rc<RefCell<Option<DynLocation>>>,
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
    const MINIMIZE_WINDOW_MD_KEY: &'static str = "minimize_window";
    const MAXIMIZE_WINDOW_MD_KEY: &'static str = "maximize_window";
    pub const Z_INDEX: ZIndex = 50;
    pub fn new(
        hat: &SortingHat, ctx: &Context, inner: Rc<RefCell<dyn Element>>, title: &str,
    ) -> Self {
        let vs = VerticalStack::new_with_kind(hat, Self::KIND);
        let top_bar = Rc::new(RefCell::new(BasicWindowTopBar::new(
            hat, ctx, title, true, true, true,
        )));

        vs.push(ctx, top_bar.clone());
        vs.push(ctx, inner.clone());
        vs.pane.set_z(Self::Z_INDEX);

        Self {
            pane: vs,
            top_bar,
            inner,
            dragging: Rc::new(RefCell::new(None)),
            maximized_restore: Rc::new(RefCell::new(None)),
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

        let mut resps_ = EventResponses::default();
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
                continue;
            }

            let mut maximize_window = false;
            if let EventResponse::Metadata((key, _)) = resp {
                if key == Self::MAXIMIZE_WINDOW_MD_KEY {
                    maximize_window = true;
                }
            }
            if maximize_window {
                // swap out the response for a destruct if the window should close
                let mr = (*self.maximized_restore.borrow()).clone();
                match mr {
                    Some(restore_loc) => {
                        let mut pane_ctx = ctx.clone();
                        pane_ctx.s.height = restore_loc.height(ctx) as u16;
                        pane_ctx.s.width = restore_loc.width(ctx) as u16;

                        let mut top_bar_ctx = pane_ctx.clone();
                        top_bar_ctx.s.height = 1;

                        let mut inner_ctx = pane_ctx.clone();
                        inner_ctx.s.height -= 1;

                        self.pane.pane.set_dyn_location(restore_loc);

                        let (_, r) = self
                            .top_bar
                            .borrow()
                            .receive_event(&top_bar_ctx, Event::Resize);
                        resps_.extend(r.0);
                        let (_, r) = self.inner.borrow().receive_event(&inner_ctx, Event::Resize);
                        resps_.extend(r.0);

                        self.maximized_restore.replace(None);
                    }
                    None => {
                        let restore_loc = self.pane.pane.get_dyn_location();
                        self.pane.pane.set_start_x(DynVal::new_fixed(0));
                        self.pane.pane.set_start_y(DynVal::new_fixed(0));
                        self.pane.pane.set_end_x(DynVal::new_flex(1.));
                        self.pane.pane.set_end_y(DynVal::new_flex(1.));

                        let mut top_bar_ctx = ctx.clone();
                        top_bar_ctx.s.height = 1;

                        let mut inner_ctx = ctx.clone();
                        inner_ctx.s.height -= 1;

                        let (_, r) = self
                            .top_bar
                            .borrow()
                            .receive_event(&top_bar_ctx, Event::Resize);
                        resps_.extend(r.0);
                        let (_, r) = self.inner.borrow().receive_event(&inner_ctx, Event::Resize);
                        resps_.extend(r.0);

                        *resp = EventResponse::None;
                        self.maximized_restore.replace(Some(restore_loc));
                        continue;
                    }
                }
            }
        }
        resps.extend(resps_.0);

        // Won't work until mouse capturing is reformatted
        //if captured {
        //    return (captured, resps);
        //}

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
                if let MouseEventKind::Down(_) = me.kind {
                    resps.push(EventResponse::BringToFront)
                }

                if me.row as usize > top_height {
                    return (captured, resps);
                }
                let dragging = self.dragging.borrow().is_some();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) if !dragging => {
                        *self.dragging.borrow_mut() = Some((me.column, me.row));
                    }
                    MouseEventKind::Drag(MouseButton::Left) if dragging => {
                        let (start_x, start_y) = self.dragging.borrow().expect("impossible");
                        let dx = me.column as i32 - start_x as i32;
                        let dy = me.row as i32 - start_y as i32;
                        let x1 = self.pane.pane.pane.get_start_x(ctx) + dx;
                        let y1 = self.pane.pane.pane.get_start_y(ctx) + dy;
                        let x2 = self.pane.pane.pane.get_end_x(ctx) + dx;
                        let y2 = self.pane.pane.pane.get_end_y(ctx) + dy;

                        self.pane.pane.set_start_x(DynVal::new_fixed(x1));
                        self.pane.pane.set_start_y(DynVal::new_fixed(y1));
                        self.pane.pane.set_end_x(DynVal::new_fixed(x2));
                        self.pane.pane.set_end_y(DynVal::new_fixed(y2));
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
                        MouseEventKind::Drag(MouseButton::Left) => {
                            // logic is now relative to the parent context
                            let (start_x, start_y) = self.dragging.borrow().expect("impossible");
                            let dx = me.column as i32 - start_x as i32;
                            let dy = me.row as i32 - start_y as i32;
                            let x1 = dx;
                            let y1 = dy;
                            let x2 = self.pane.pane.pane.get_width(ctx) as i32 + dx;
                            let y2 = self.pane.pane.pane.get_height(ctx) as i32 + dy;
                            self.pane.pane.set_start_x(DynVal::new_fixed(x1));
                            self.pane.pane.set_start_y(DynVal::new_fixed(y1));
                            self.pane.pane.set_end_x(DynVal::new_fixed(x2));
                            self.pane.pane.set_end_y(DynVal::new_fixed(y2));
                        }
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

pub struct BasicWindowTopBar {
    pub pane: ParentPane,
    pub title: Rc<RefCell<Label>>,
}

impl BasicWindowTopBar {
    pub fn new(
        hat: &SortingHat, ctx: &Context, title: &str, close_button: bool, maximize_button: bool,
        minimize_button: bool,
    ) -> Self {
        let pane = ParentPane::new(hat, "basic_window_top_bar")
            .with_dyn_height(DynVal::new_fixed(1))
            .with_dyn_width(DynVal::new_flex(1.))
            .with_style(Style::default().with_bg(Color::WHITE).with_fg(Color::BLACK));

        let btn_styles = WBStyles::new(
            Style::default()
                .with_bg(Color::GREY20)
                .with_fg(Color::BLACK),
            Style::default()
                .with_bg(Color::GREY20)
                .with_fg(Color::BLACK),
            Style::default()
                .with_bg(Color::GREY20)
                .with_fg(Color::BLACK),
        );

        if close_button {
            let close_button = Button::new(
                hat,
                ctx,
                "x",
                Box::new(|_, _ctx| {
                    EventResponse::Metadata((
                        WindowPane::CLOSE_WINDOW_MD_KEY.to_string(),
                        Vec::with_capacity(0),
                    ))
                    .into()
                }),
            )
            .basic_button(Some(
                Style::default().with_fg(Color::BLACK).with_bg(Color::BLUE),
            ))
            .with_styles(btn_styles.clone())
            .at(DynVal::new_flex(1.).minus(2.into()), DynVal::new_fixed(0));
            pane.add_element(Rc::new(RefCell::new(close_button)));
        }

        if maximize_button {
            let maximize_button = Button::new(
                hat,
                ctx,
                "□",
                Box::new(|btn, _ctx| {
                    // change the text to the restore icon or back
                    let existing_icon = btn.text.borrow().clone();
                    *btn.text.borrow_mut() = match existing_icon.as_str() {
                        "□" => "◱".to_string(),
                        _ => "□".to_string(),
                    };
                    EventResponse::Metadata((
                        WindowPane::MAXIMIZE_WINDOW_MD_KEY.to_string(),
                        Vec::with_capacity(0),
                    ))
                    .into()
                }),
            )
            .basic_button(Some(
                Style::default().with_fg(Color::BLACK).with_bg(Color::BLUE),
            ))
            .with_styles(btn_styles.clone())
            .at(DynVal::new_flex(1.).minus(4.into()), DynVal::new_fixed(0));
            pane.add_element(Rc::new(RefCell::new(maximize_button)));
        }

        if minimize_button {
            let minimize_button = Button::new(
                hat,
                ctx,
                "ˍ",
                Box::new(|_, _ctx| {
                    EventResponse::Metadata((
                        WindowPane::MINIMIZE_WINDOW_MD_KEY.to_string(),
                        Vec::with_capacity(0),
                    ))
                    .into()
                }),
            )
            .basic_button(Some(
                Style::default().with_fg(Color::BLACK).with_bg(Color::BLUE),
            ))
            .with_styles(btn_styles)
            .at(DynVal::new_flex(1.).minus(6.into()), DynVal::new_fixed(0));
            pane.add_element(Rc::new(RefCell::new(minimize_button)));
        }

        let title_label = Rc::new(RefCell::new(
            Label::new(hat, ctx, title).at(DynVal::new_fixed(1), DynVal::new_fixed(0)),
        ));
        pane.add_element(title_label.clone());

        Self {
            pane,
            title: title_label,
        }
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
