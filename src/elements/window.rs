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
// TODO use ◢ in the lower righthand corner for rescaling

// ---------------------------------------------------
#[derive(Clone)]
pub struct WindowPane {
    pub pane: VerticalStack,
    pub top_bar: Rc<RefCell<dyn Element>>,
    pub inner: Rc<RefCell<dyn Element>>,
    pub dragging: Rc<RefCell<Option<(u16, u16)>>>, // start location of the drag
    pub maximized_restore: Rc<RefCell<Option<DynLocation>>>,
    pub minimized_restore: Rc<RefCell<Option<DynLocation>>>,
    pub minimized_width: Rc<RefCell<u16>>,
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
    const WINDOW_MINIMIZE_EV_KEY: &'static str = "window_minimize";
    const WINDOW_MINIMIZE_RESTORE_EV_KEY: &'static str = "window_minimize_restore";
    pub const Z_INDEX: ZIndex = 50;
    pub fn new(
        hat: &SortingHat, ctx: &Context, inner: Rc<RefCell<dyn Element>>, title: &str,
    ) -> Self {
        //let vs = VerticalStack::new_with_kind(hat, Self::KIND).with_style(Style::transparent());
        let vs = VerticalStack::new_with_kind(hat, Self::KIND).with_transparent();
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
            minimized_restore: Rc::new(RefCell::new(None)),
            minimized_width: Rc::new(RefCell::new(20)),
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

    pub fn with_minimum_width(self, w: u16) -> Self {
        *self.minimized_width.borrow_mut() = w;
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
        let mut just_minimized = false;
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

            let mut minimize_window = false;
            if let EventResponse::Metadata((key, _)) = resp {
                if key == Self::MINIMIZE_WINDOW_MD_KEY {
                    minimize_window = true;
                    just_minimized = true;
                }
            }

            let mr = (*self.minimized_restore.borrow()).clone();
            if minimize_window && mr.is_none() {
                let minimize_width = *self.minimized_width.borrow();
                let restore_loc = self.pane.pane.get_dyn_location();
                self.pane.pane.set_start_x(DynVal::new_fixed(0));
                self.pane
                    .pane
                    .set_end_x(DynVal::new_fixed(minimize_width.into()));
                self.pane
                    .pane
                    .set_start_y(DynVal::new_flex(1.).minus(1.into()));
                self.pane.pane.set_end_y(DynVal::new_flex(1.));
                let mut pane_ctx = ctx.clone();
                pane_ctx.s.height = 1;
                pane_ctx.s.width = minimize_width;

                // send an event telling the top bar to hide its buttons
                let (_, r) = self.top_bar.borrow().receive_event(
                    &pane_ctx,
                    Event::Custom(
                        Self::WINDOW_MINIMIZE_EV_KEY.to_string(),
                        Vec::with_capacity(0),
                    ),
                );
                resps_.extend(r.0);

                // resize events
                let (_, r) = self
                    .top_bar
                    .borrow()
                    .receive_event(&pane_ctx, Event::Resize);
                resps_.extend(r.0);
                self.inner.borrow().get_visible().replace(false);
                *resp = EventResponse::None;
                self.minimized_restore.replace(Some(restore_loc));
                continue;
            }
        }

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

                let mr = (*self.minimized_restore.borrow()).clone();
                match me.kind {
                    MouseEventKind::Up(MouseButton::Left) if !just_minimized && mr.is_some() => {
                        let restore_loc = mr.expect("impossible");

                        // maximize from minimized

                        let mut pane_ctx = ctx.clone();
                        pane_ctx.s.height = restore_loc.height(ctx) as u16;
                        pane_ctx.s.width = restore_loc.width(ctx) as u16;

                        let mut top_bar_ctx = pane_ctx.clone();
                        top_bar_ctx.s.height = 1;

                        self.pane.pane.set_dyn_location(restore_loc);

                        // send an event telling the top bar to hide its buttons
                        let (_, r) = self.top_bar.borrow().receive_event(
                            &pane_ctx,
                            Event::Custom(
                                Self::WINDOW_MINIMIZE_RESTORE_EV_KEY.to_string(),
                                Vec::with_capacity(0),
                            ),
                        );
                        resps_.extend(r.0);

                        let (_, r) = self
                            .top_bar
                            .borrow()
                            .receive_event(&top_bar_ctx, Event::Resize);
                        resps_.extend(r.0);
                        self.inner.borrow().get_visible().replace(true);

                        self.minimized_restore.replace(None);
                    }
                    MouseEventKind::Down(MouseButton::Left) if !dragging && mr.is_none() => {
                        *self.dragging.borrow_mut() = Some((me.column, me.row));
                    }
                    MouseEventKind::Drag(MouseButton::Left) if dragging && mr.is_none() => {
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

        resps.extend(resps_.0);
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
    pub minimized_decor: Rc<RefCell<Label>>,
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

        let mut button_rhs_spaces = 2;

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
            .at(
                DynVal::new_flex(1.).minus(button_rhs_spaces.into()),
                DynVal::new_fixed(0),
            );
            pane.add_element(Rc::new(RefCell::new(close_button)));
            button_rhs_spaces += 2;
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
            .at(
                DynVal::new_flex(1.).minus(button_rhs_spaces.into()),
                DynVal::new_fixed(0),
            );
            pane.add_element(Rc::new(RefCell::new(maximize_button)));
            button_rhs_spaces += 2;
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
            .at(
                DynVal::new_flex(1.).minus(button_rhs_spaces.into()),
                DynVal::new_fixed(0),
            );
            pane.add_element(Rc::new(RefCell::new(minimize_button)));
        }

        let title_label = Rc::new(RefCell::new(
            Label::new(hat, ctx, title).at(DynVal::new_fixed(1), DynVal::new_fixed(0)),
        ));
        let decor_label = Rc::new(RefCell::new(
            Label::new(hat, ctx, "◹")
                .at(DynVal::new_flex(1.).minus(2.into()), DynVal::new_fixed(0)),
        ));
        pane.add_element(title_label.clone());
        pane.add_element(decor_label.clone());
        decor_label.borrow().get_visible().replace(false);

        Self {
            pane,
            title: title_label,
            minimized_decor: decor_label,
        }
    }

    /// useful for minimization
    pub fn minimize(&self) {
        self.pane.eo.set_all_visibility(false);
        self.title.borrow().get_visible().replace(true);
        self.minimized_decor.borrow().get_visible().replace(true);
    }

    /// useful for restore after minimization
    pub fn minimize_restore(&self) {
        self.pane.eo.set_all_visibility(true);
        self.minimized_decor.borrow().get_visible().replace(false);
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
        match ev {
            Event::Custom(ref key, _) if key == WindowPane::WINDOW_MINIMIZE_EV_KEY => {
                self.minimize();
                (false, EventResponses::default())
            }
            Event::Custom(ref key, _) if key == WindowPane::WINDOW_MINIMIZE_RESTORE_EV_KEY => {
                self.minimize_restore();
                (false, EventResponses::default())
            }
            _ => self.pane.receive_event(ctx, ev),
        }
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
