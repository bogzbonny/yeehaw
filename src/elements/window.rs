use {
    crate::{
        widgets::{Button, Label, WBStyles},
        Color, Context, DrawCh, DrawChPos, DrawChs2D, DynLocation, DynLocationSet, DynVal, Element,
        ElementID, Event, EventResponse, EventResponses, Pane, Parent, ParentPane, Priority,
        ReceivableEventChanges, Size, SortingHat, Style, ZIndex,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// TODO integrate in border pane size-adjustments
// TODO animation for minimize/restore, should have one frame in the middle
//      in the middle location between the restore position and start position.
//      for that animation frame should just be the minimized top-bar

// ---------------------------------------------------
#[derive(Clone)]
pub struct WindowPane {
    pub pane: ParentPane,
    pub top_bar: Box<dyn Element>,
    pub inner: Box<dyn Element>,
    pub dragging: Rc<RefCell<Option<(u16, u16)>>>, // start location of the drag
    pub maximized_restore: Rc<RefCell<Option<DynLocation>>>,
    pub minimized_restore: Rc<RefCell<Option<DynLocation>>>,
    pub minimized_width: Rc<RefCell<u16>>,
}

impl WindowPane {
    const KIND: &'static str = "window_pane";
    const CLOSE_WINDOW_MD_KEY: &'static str = "close_window";
    const MINIMIZE_WINDOW_MD_KEY: &'static str = "minimize_window";
    const MAXIMIZE_WINDOW_MD_KEY: &'static str = "maximize_window";
    const WINDOW_MINIMIZE_EV_KEY: &'static str = "window_minimize";
    const WINDOW_MINIMIZE_RESTORE_EV_KEY: &'static str = "window_minimize_restore";
    const WINDOW_RESET_MAXIMIZER_EV_KEY: &'static str = "window_reset_maximizer";
    pub const Z_INDEX: ZIndex = 50;
    pub fn new(hat: &SortingHat, ctx: &Context, inner: Box<dyn Element>, title: &str) -> Self {
        //let vs = VerticalStack::new_with_kind(hat, Self::KIND).with_style(Style::transparent());
        let pane = ParentPane::new(hat, Self::KIND).with_transparent();
        let top_bar = Box::new(BasicWindowTopBar::new(hat, ctx, title, true, true, true));
        //debug!(
        //    "***********************************new window pane, pane priority: {}",
        //    pane.pane.get_element_priority()
        //);

        // adjust the inner size to account for the top bar
        let mut loc = inner.get_dyn_location_set().borrow().clone();
        loc.set_start_y(1.into());
        loc.set_dyn_height(DynVal::new_flex(1.).minus(1.into()));
        inner.get_dyn_location_set().replace(loc);

        pane.add_element(top_bar.clone());
        pane.add_element(inner.clone());
        pane.pane.set_z(Self::Z_INDEX);

        Self {
            pane,
            top_bar,
            inner,
            dragging: Rc::new(RefCell::new(None)),
            maximized_restore: Rc::new(RefCell::new(None)),
            minimized_restore: Rc::new(RefCell::new(None)),
            minimized_width: Rc::new(RefCell::new(20)),
        }
    }

    pub fn at(self, x: DynVal, y: DynVal) -> Self {
        self.pane.pane.set_at(x, y);
        self
    }

    pub fn with_size(self, w: DynVal, h: DynVal) -> Self {
        self.pane.pane.set_dyn_width(w);
        self.pane.pane.set_dyn_height(h);
        self
    }

    pub fn with_minimum_width(self, w: u16) -> Self {
        *self.minimized_width.borrow_mut() = w;
        self
    }

    pub fn with_width(self, w: DynVal) -> Self {
        self.pane.pane.set_dyn_width(w);
        self
    }

    pub fn with_height(self, h: DynVal) -> Self {
        self.pane.pane.set_dyn_height(h);
        self
    }

    pub fn with_corner_adjuster(self, hat: &SortingHat, ctx: &Context) -> Self {
        let ca = CornerAdjuster::new(hat, ctx).at(
            DynVal::new_flex(1.).minus(1.into()),
            DynVal::new_flex(1.).minus(1.into()),
        );
        self.pane.add_element(Box::new(ca));

        use crate::organizer::ElDetails;
        let mut eoz: Vec<(ElementID, ElDetails)> = Vec::new();
        for (el_id, details) in self.pane.eo.els.borrow().iter() {
            eoz.push((el_id.clone(), details.clone()));
        }

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
        //debug!("Window({}) receive_event_inner: {:?}", self.id(), ev);
        // Skip sending the event into the pane if the event is a drag event and we're currently
        // dragging
        let mut event_to_inner = true;
        let dragging = self.dragging.borrow().is_some();
        if dragging {
            if let Event::Mouse(me) = ev {
                if let MouseEventKind::Drag(MouseButton::Left) = me.kind {
                    event_to_inner = false;
                }
            }
        };

        let (captured, mut resps) = if event_to_inner {
            self.pane.receive_event(ctx, ev.clone())
        } else {
            (true, EventResponses::default())
        };

        let mut resps_ = EventResponses::default();
        let mut just_minimized = false;
        for resp in resps.iter_mut() {
            let mut adjust_size = None;
            if let EventResponse::Metadata(key, adj_bz) = resp {
                if key == CornerAdjuster::ADJUST_SIZE_MD_KEY {
                    adjust_size = Some(adj_bz);
                }
            }
            if let Some(bz) = adjust_size {
                // get the adjust size event
                let adj_size_ev: AdjustSizeEvent = match serde_json::from_slice(bz) {
                    Ok(v) => v,
                    Err(_e) => {
                        // TODO log error
                        continue;
                    }
                };
                let mut dx = adj_size_ev.dx;
                let mut dy = adj_size_ev.dy;
                let width = self.pane.pane.get_width(ctx) as i32;
                let height = self.pane.pane.get_height(ctx) as i32;
                if width + dx < 2 {
                    dx = 0
                }
                if height + dy < 2 {
                    dy = 0
                }

                if dx == 0 && dy == 0 {
                    *resp = EventResponse::None;
                    continue;
                }

                // NOTE must set to a a fixed value (aka need to get the size for the pane DynVal
                // using ctx here. if we do not then the next pane position drag will be off
                let end_x = self.pane.pane.get_end_x(ctx) + dx;
                let end_y = self.pane.pane.get_end_y(ctx) + dy;
                self.pane.pane.set_end_x(end_x.into());
                self.pane.pane.set_end_y(end_y.into());

                let inner_ctx = ctx.clone().with_size(Size::new(
                    self.pane.pane.get_width(ctx) as u16,
                    self.pane.pane.get_height(ctx) as u16 - 1,
                ));

                let (_, r) = self.inner.receive_event(&inner_ctx, Event::Resize);
                resps_.extend(r.0);

                // reset the maximizer button
                if self.maximized_restore.borrow().is_some() {
                    self.maximized_restore.replace(None);
                    let (_, r) = self.top_bar.receive_event(
                        &Context::default(),
                        Event::Custom(
                            Self::WINDOW_RESET_MAXIMIZER_EV_KEY.to_string(),
                            Vec::with_capacity(0),
                        ),
                    );
                    resps_.extend(r.0);
                }

                *resp = EventResponse::None;
                continue;
            }

            let mut close_window = false;
            if let EventResponse::Metadata(key, _) = resp {
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
            if let EventResponse::Metadata(key, _) = resp {
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

                        let (_, r) = self.top_bar.receive_event(&top_bar_ctx, Event::Resize);
                        resps_.extend(r.0);
                        let (_, r) = self.inner.receive_event(&inner_ctx, Event::Resize);
                        resps_.extend(r.0);

                        self.maximized_restore.replace(None);
                    }
                    None => {
                        let restore_loc = self.pane.pane.get_dyn_location();
                        let l = DynLocation::new_full();
                        self.pane.pane.set_dyn_location(l);

                        let mut top_bar_ctx = ctx.clone();
                        top_bar_ctx.s.height = 1;

                        let mut inner_ctx = ctx.clone();
                        inner_ctx.s.height -= 1;

                        let (_, r) = self.top_bar.receive_event(&top_bar_ctx, Event::Resize);
                        resps_.extend(r.0);
                        let (_, r) = self.inner.receive_event(&inner_ctx, Event::Resize);
                        resps_.extend(r.0);

                        self.maximized_restore.replace(Some(restore_loc));
                    }
                }
                *resp = EventResponse::None;
            }

            let mut minimize_window = false;
            if let EventResponse::Metadata(key, _) = resp {
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
                let (_, r) = self.top_bar.receive_event(
                    &pane_ctx,
                    Event::Custom(
                        Self::WINDOW_MINIMIZE_EV_KEY.to_string(),
                        Vec::with_capacity(0),
                    ),
                );
                resps_.extend(r.0);

                // resize events
                let (_, r) = self.top_bar.receive_event(&pane_ctx, Event::Resize);
                resps_.extend(r.0);
                self.inner.get_visible().replace(false);
                *resp = EventResponse::None;
                self.minimized_restore.replace(Some(restore_loc));
                continue;
            }
        }

        // Won't work until mouse capturing is reformatted
        //if captured {
        //    return (captured, resps);
        //}

        let top_height = self.top_bar.get_dyn_location_set().borrow().l.height(ctx);

        // process dragging
        match ev {
            Event::Mouse(me) => {
                if let MouseEventKind::Down(_) = me.kind {
                    self.pane.pane.focus();
                    resps.push(EventResponse::BringToFront)
                }

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
                        let (_, r) = self.top_bar.receive_event(
                            &pane_ctx,
                            Event::Custom(
                                Self::WINDOW_MINIMIZE_RESTORE_EV_KEY.to_string(),
                                Vec::with_capacity(0),
                            ),
                        );
                        resps_.extend(r.0);

                        let (_, r) = self.top_bar.receive_event(&top_bar_ctx, Event::Resize);
                        resps_.extend(r.0);
                        self.inner.get_visible().replace(true);

                        self.minimized_restore.replace(None);
                    }
                    MouseEventKind::Down(MouseButton::Left) if !dragging && mr.is_none() => {
                        if me.row as usize >= top_height {
                            return (captured, resps);
                        }
                        *self.dragging.borrow_mut() = Some((me.column, me.row));
                    }
                    MouseEventKind::Drag(MouseButton::Left) if dragging && mr.is_none() => {
                        let (start_x, start_y) = self.dragging.borrow().expect("impossible");
                        let mut dx = me.column as i32 - start_x as i32;
                        let mut dy = me.row as i32 - start_y as i32;
                        let loc = self.pane.pane.get_dyn_location();
                        if loc.get_start_x(ctx) + dx < 0 {
                            dx = loc.get_start_x(ctx);
                        }
                        if loc.get_start_y(ctx) + dy < 0 {
                            dy = loc.get_start_y(ctx);
                        }
                        let x1 = loc.get_start_x(ctx) + dx;
                        let y1 = loc.get_start_y(ctx) + dy;
                        let x2 = loc.get_end_x(ctx) + dx;
                        let y2 = loc.get_end_y(ctx) + dy;
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
                            let (start_x, start_y) = self.dragging.borrow().expect("impossible");
                            let (start_x, start_y) = (start_x as i32, start_y as i32);
                            let mut dx = me.column - start_x;
                            let mut dy = me.row - start_y;
                            let loc = self.pane.pane.get_dyn_location();
                            if loc.get_start_x(ctx) + dx < 0 {
                                dx = loc.get_start_x(ctx);
                            }
                            if loc.get_start_y(ctx) + dy < 0 {
                                dy = loc.get_start_y(ctx);
                            }
                            let x1 = loc.get_start_x(ctx) + dx;
                            let y1 = loc.get_start_y(ctx) + dy;
                            let x2 = loc.get_end_x(ctx) + dx;
                            let y2 = loc.get_end_y(ctx) + dy;
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

        // check if the inner pane has been removed from the parent in which case close this window
        if self.pane.eo.get_element(&self.inner.id()).is_none() {
            resps.push(EventResponse::Destruct);
        }

        (captured, resps)
    }

    fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority( p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let out = self.pane.drawing(ctx);

        // check if the inner pane has been removed from the parent in which case close this window
        // NOTE this happens with terminal
        if self.pane.eo.get_element(&self.inner.id()).is_none() {
            //resps.push(EventResponse::Destruct);
            self.pane
                .pane
                .propagate_responses_upward(EventResponse::Destruct.into());
            Vec::with_capacity(0)
        } else {
            out
        }
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_parent(&self, up: Box<dyn Parent>) {
        self.pane.set_parent(up)
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
pub struct BasicWindowTopBar {
    pub pane: ParentPane,
    pub maximizer_button: Rc<RefCell<Option<Box<Button>>>>,
    pub title: Box<Label>,
    pub minimized_decor: Box<Label>,
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
                    EventResponse::Metadata(
                        WindowPane::CLOSE_WINDOW_MD_KEY.to_string(),
                        Vec::with_capacity(0),
                    )
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
            pane.add_element(Box::new(close_button));
            button_rhs_spaces += 2;
        }

        let mut maximizer_button = None;
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
                    EventResponse::Metadata(
                        WindowPane::MAXIMIZE_WINDOW_MD_KEY.to_string(),
                        Vec::with_capacity(0),
                    )
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
            let b = Box::new(maximize_button);
            pane.add_element(b.clone());
            maximizer_button = Some(b);
            button_rhs_spaces += 2;
        }

        if minimize_button {
            let minimize_button = Button::new(
                hat,
                ctx,
                "ˍ",
                Box::new(|_, _ctx| {
                    EventResponse::Metadata(
                        WindowPane::MINIMIZE_WINDOW_MD_KEY.to_string(),
                        Vec::with_capacity(0),
                    )
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
            pane.add_element(Box::new(minimize_button));
        }

        let title_label = Box::new(
            Label::new(hat, ctx, title)
                .with_style(ctx, Style::transparent())
                .at(DynVal::new_fixed(1), DynVal::new_fixed(0)),
        );
        let decor_label = Box::new(
            Label::new(hat, ctx, "◹")
                .with_style(ctx, Style::transparent())
                .at(DynVal::new_flex(1.).minus(2.into()), DynVal::new_fixed(0)),
        );
        pane.add_element(title_label.clone());
        pane.add_element(decor_label.clone());
        decor_label.get_visible().replace(false);

        Self {
            pane,
            maximizer_button: Rc::new(RefCell::new(maximizer_button)),
            title: title_label,
            minimized_decor: decor_label,
        }
    }

    /// useful for minimization
    pub fn minimize(&self) {
        self.pane.eo.set_all_visibility(false);
        self.title.get_visible().replace(true);
        self.minimized_decor.get_visible().replace(true);
    }

    /// useful for restore after minimization
    pub fn minimize_restore(&self) {
        self.pane.eo.set_all_visibility(true);
        self.minimized_decor.get_visible().replace(false);
    }

    pub fn reset_maximizer_button(&self) {
        let maximizer_button = self.maximizer_button.borrow_mut();
        if let Some(mb) = maximizer_button.as_ref() {
            mb.text.replace("□".to_string());
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
        match ev {
            Event::Custom(ref key, _) if key == WindowPane::WINDOW_MINIMIZE_EV_KEY => {
                self.minimize();
                (false, EventResponses::default())
            }
            Event::Custom(ref key, _) if key == WindowPane::WINDOW_MINIMIZE_RESTORE_EV_KEY => {
                self.minimize_restore();
                (false, EventResponses::default())
            }
            Event::Custom(ref key, _) if key == WindowPane::WINDOW_RESET_MAXIMIZER_EV_KEY => {
                self.reset_maximizer_button();
                (false, EventResponses::default())
            }
            _ => self.pane.receive_event(ctx, ev),
        }
    }
    fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority( p)
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
    fn set_parent(&self, up: Box<dyn Parent>) {
        self.pane.set_parent(up)
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

// ------------------------------------------------

/// a small element (one character) that can be used to send resize requests to parent elements
#[derive(Clone)]
pub struct CornerAdjuster {
    pub pane: Pane,
    pub dragging: Rc<RefCell<bool>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AdjustSizeEvent {
    pub dx: i32,
    pub dy: i32,
}

impl CornerAdjuster {
    const ADJUST_SIZE_MD_KEY: &'static str = "adjust_size";
    const Z_INDEX: ZIndex = 200;

    pub fn new(hat: &SortingHat, _ctx: &Context) -> Self {
        let pane = Pane::new(hat, "resize_corner")
            .with_dyn_height(1.into())
            .with_dyn_width(1.into())
            .with_style(Style::default().with_bg(Color::WHITE).with_fg(Color::BLACK))
            .with_content(DrawChs2D::from_char(
                '◢',
                Style::default()
                    .with_fg(Color::WHITE)
                    .with_bg(Color::TRANSPARENT),
            ));
        pane.set_z(Self::Z_INDEX);
        Self {
            pane,
            dragging: Rc::new(RefCell::new(false)),
        }
    }

    pub fn with_ch(self, ch: DrawCh) -> Self {
        self.pane.set_content(ch.into());
        self
    }

    pub fn at(self, x: DynVal, y: DynVal) -> Self {
        self.pane.set_at(x, y);
        self
    }
}

impl Element for CornerAdjuster {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }
    fn receive_event_inner(&self, _ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let cur_dragging = *self.dragging.borrow();
        match ev {
            Event::Mouse(me) => match me.kind {
                MouseEventKind::Down(MouseButton::Left) => *self.dragging.borrow_mut() = true,
                MouseEventKind::Drag(MouseButton::Left) => {}
                _ => *self.dragging.borrow_mut() = false,
            },
            Event::ExternalMouse(me) => match me.kind {
                MouseEventKind::Drag(MouseButton::Left) if cur_dragging => {
                    let dx = me.column;
                    let dy = me.row;
                    let adj_size_ev = AdjustSizeEvent { dx, dy };
                    let bz = match serde_json::to_vec(&adj_size_ev) {
                        Ok(v) => v,
                        Err(_e) => {
                            // TODO log error
                            return (false, EventResponses::default());
                        }
                    };
                    return (
                        true,
                        EventResponse::Metadata(Self::ADJUST_SIZE_MD_KEY.to_string(), bz).into(),
                    );
                }
                _ => *self.dragging.borrow_mut() = false,
            },
            _ => {}
        }
        (false, EventResponses::default())
    }
    fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority( p)
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
    fn set_parent(&self, up: Box<dyn Parent>) {
        self.pane.set_parent(up)
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
