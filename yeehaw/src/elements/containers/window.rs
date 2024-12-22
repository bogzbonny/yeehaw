use {
    crate::*,
    crossterm::event::{MouseButton, MouseEventKind},
};

// TODO Top bar movements should be added to the Top Bar logic instead of the window logic
// TODO animation for minimize/restore, should have one frame in the middle
//      in the middle location between the restore position and start position.
//      for that animation frame should just be the minimized top-bar

#[derive(Clone)]
pub struct WindowPane {
    pub pane: ParentPane,
    pub top_bar: Box<dyn Element>,
    pub inner: Box<dyn Element>,
    pub dragging: Rc<RefCell<Option<(u16, u16)>>>,
    /// start location of the drag
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
    pub fn new(ctx: &Context, inner: Box<dyn Element>, title: &str) -> Self {
        let pane = ParentPane::new(ctx, Self::KIND)
            .with_transparent()
            .with_focused(false);
        let top_bar = Box::new(BasicWindowTopBar::new(ctx, title, true, true, true));

        // adjust the inner size to account for the top bar
        let mut loc = inner.get_dyn_location_set().clone();
        loc.set_start_y(1);
        loc.set_dyn_height(DynVal::FULL.minus(1.into()));
        inner.set_dyn_location_set(loc);
        inner.set_focused(true);

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

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, x: D, y: D2) -> Self {
        self.pane.pane.set_at(x.into(), y.into());
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

    pub fn with_corner_resizer(mut self, ctx: &Context) -> Self {
        self.set_corner_resizer(ctx);
        self
    }

    pub fn set_corner_resizer(&mut self, ctx: &Context) {
        let corner_ch = DrawCh::new(
            '◢',
            Style::default()
                .with_fg(Color::WHITE)
                .with_bg(Color::TRANSPARENT),
        );
        let ca = BorderCorner::new(
            ctx,
            corner_ch,
            CornerPos::BottomRight,
            BorderPropertyCnr::DragResize,
        )
        .at(DynVal::FULL.minus(1.into()), DynVal::FULL.minus(1.into()));
        self.pane.add_element(Box::new(ca));

        use crate::organizer::ElDetails;
        let mut eoz: Vec<(ElementID, ElDetails)> = Vec::new();
        for (el_id, details) in self.pane.eo.els.borrow().iter() {
            eoz.push((el_id.clone(), details.clone()));
        }
    }

    /// partially process the resp for the inner elements of the window
    ///                                                                                 just-minimized
    pub fn partially_process_inner_resp(&self, ctx: &Context, resps: &mut EventResponses) -> bool {
        let mut resps_ = EventResponses::default();
        let mut just_minimized = false;
        for resp in resps.iter_mut() {
            let changes = match resp {
                EventResponse::Move(m) => Some((m.dx, m.dx, m.dy, m.dy)),
                EventResponse::Resize(r) => Some((r.left_dx, r.right_dx, r.top_dy, r.bottom_dy)),
                _ => None,
            };

            if let Some(changes) = changes {
                let (left_dx, right_dx, top_dy, bottom_dy) = changes;

                // NOTE must set to a a fixed value (aka need to get the size for the pane DynVal
                // using ctx here. if we do not then the next pane position drag will be off
                let start_x = self.pane.pane.get_start_x(ctx);
                let start_y = self.pane.pane.get_start_y(ctx);
                let end_x = self.pane.pane.get_end_x(ctx);
                let end_y = self.pane.pane.get_end_y(ctx);
                let mut start_x_adj = start_x + left_dx;
                let mut start_y_adj = start_y + top_dy;
                let mut end_x_adj = end_x + right_dx;
                let mut end_y_adj = end_y + bottom_dy;

                if end_x_adj - start_x_adj < 2 || start_x_adj < 0 || start_y_adj < 0 {
                    start_x_adj = start_x;
                    end_x_adj = end_x;
                }

                // 3 = 1 (top bar) + 2 inner
                if end_y_adj - start_y_adj < 3 || start_x_adj < 0 || start_y_adj < 0 {
                    start_y_adj = start_y;
                    end_y_adj = end_y;
                }

                self.pane.pane.set_start_x(start_x_adj);
                self.pane.pane.set_start_y(start_y_adj);
                self.pane.pane.set_end_x(end_x_adj);
                self.pane.pane.set_end_y(end_y_adj);

                let inner_ctx = ctx.clone().with_size(Size::new(
                    self.pane.pane.get_width(ctx) as u16,
                    (self.pane.pane.get_height(ctx) as u16).saturating_sub(1),
                ));

                let mut top_bar_ctx = ctx.clone();
                top_bar_ctx.size.height = 1;

                let (_, r) = self.inner.receive_event(&inner_ctx, Event::Resize);
                resps_.extend(r);

                // reset the maximizer button
                if self.maximized_restore.borrow().is_some() {
                    self.maximized_restore.replace(None);
                    let (_, r) = self.top_bar.receive_event(
                        &top_bar_ctx,
                        Event::Custom(
                            Self::WINDOW_RESET_MAXIMIZER_EV_KEY.to_string(),
                            Vec::with_capacity(0),
                        ),
                    );
                    resps_.extend(r);
                }

                *resp = EventResponse::None;
                continue;
            }

            let mut close_window = false;
            if let EventResponse::Custom(key, _) = resp {
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
            if let EventResponse::Custom(key, _) = resp {
                if key == Self::MAXIMIZE_WINDOW_MD_KEY {
                    maximize_window = true;
                }
            }
            if maximize_window {
                let mr = (*self.maximized_restore.borrow()).clone();
                match mr {
                    Some(restore_loc) => {
                        let mut pane_ctx = ctx.clone();
                        pane_ctx.size.height = restore_loc.height(ctx) as u16;
                        pane_ctx.size.width = restore_loc.width(ctx) as u16;

                        let mut top_bar_ctx = pane_ctx.clone();
                        top_bar_ctx.size.height = 1;

                        let mut inner_ctx = pane_ctx.clone();
                        inner_ctx.size.height -= 1;

                        self.pane.pane.set_dyn_location(restore_loc);

                        let (_, r) = self.top_bar.receive_event(&top_bar_ctx, Event::Resize);
                        resps_.extend(r);
                        let (_, r) = self.inner.receive_event(&inner_ctx, Event::Resize);
                        resps_.extend(r);

                        self.maximized_restore.replace(None);
                    }
                    None => {
                        let restore_loc = self.pane.pane.get_dyn_location();
                        let l = DynLocation::full();
                        self.pane.pane.set_dyn_location(l);

                        let mut top_bar_ctx = ctx.clone();
                        top_bar_ctx.size.height = 1;

                        let mut inner_ctx = ctx.clone();
                        inner_ctx.size.height -= 1;

                        let (_, r) = self.top_bar.receive_event(&top_bar_ctx, Event::Resize);
                        resps_.extend(r);
                        let (_, r) = self.inner.receive_event(&inner_ctx, Event::Resize);
                        resps_.extend(r);

                        self.maximized_restore.replace(Some(restore_loc));
                    }
                }
                *resp = EventResponse::None;
            }

            let mut minimize_window = false;
            if let EventResponse::Custom(key, _) = resp {
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
                self.pane.pane.set_start_y(DynVal::FULL.minus(1.into()));
                self.pane.pane.set_end_y(DynVal::FULL);
                let mut pane_ctx = ctx.clone();
                pane_ctx.size.height = 1;
                pane_ctx.size.width = minimize_width;

                // send an event telling the top bar to hide its buttons
                let (_, r) = self.top_bar.receive_event(
                    &pane_ctx,
                    Event::Custom(
                        Self::WINDOW_MINIMIZE_EV_KEY.to_string(),
                        Vec::with_capacity(0),
                    ),
                );
                resps_.extend(r);

                // resize events
                let (_, r) = self.top_bar.receive_event(&pane_ctx, Event::Resize);
                resps_.extend(r);
                self.inner.set_visible(false);
                *resp = EventResponse::None;
                self.minimized_restore.replace(Some(restore_loc));
                continue;
            }
        }
        resps.extend(resps_);
        just_minimized
    }

    pub fn focus_on_click(&self, ev: &Event) -> EventResponses {
        let mut resps = EventResponses::default();
        if let Event::Mouse(me) = ev {
            if let MouseEventKind::Down(_) = me.kind {
                resps.push(EventResponse::BringToFront);
                resps.push(EventResponse::UnfocusOthers);
                resps.push(EventResponse::Focus);
            }
        }
        resps
    }

    pub fn process_dragging(
        &self, ctx: &Context, ev: &Event, dragging: bool, just_minimized: bool,
        captured: &mut bool, resps: &mut EventResponses,
    ) {
        match ev {
            Event::Mouse(me) => {
                *captured = true;
                //if let MouseEventKind::Down(_) = me.kind {
                //    resps.push(EventResponse::BringToFront);
                //    resps.push(EventResponse::UnfocusOthers);
                //    resps.push(EventResponse::Focus);
                //}

                let mr = (*self.minimized_restore.borrow()).clone();
                match me.kind {
                    MouseEventKind::Up(MouseButton::Left) if !just_minimized && mr.is_some() => {
                        let restore_loc = mr.expect("impossible");

                        // maximize from minimized

                        let mut pane_ctx = ctx.clone();
                        pane_ctx.size.height = restore_loc.height(ctx) as u16;
                        pane_ctx.size.width = restore_loc.width(ctx) as u16;

                        let mut top_bar_ctx = pane_ctx.clone();
                        top_bar_ctx.size.height = 1;

                        self.pane.pane.set_dyn_location(restore_loc);

                        // send an event telling the top bar to hide its buttons
                        let (_, r) = self.top_bar.receive_event(
                            &pane_ctx,
                            Event::Custom(
                                Self::WINDOW_MINIMIZE_RESTORE_EV_KEY.to_string(),
                                Vec::with_capacity(0),
                            ),
                        );
                        resps.extend(r);

                        let (_, r) = self.top_bar.receive_event(&top_bar_ctx, Event::Resize);
                        resps.extend(r);
                        self.inner.set_visible(true);

                        self.minimized_restore.replace(None);
                    }
                    MouseEventKind::Down(MouseButton::Left) if !dragging && mr.is_none() => {
                        let top_height = self.top_bar.get_dyn_location_set().l.height(ctx);
                        if me.row as usize >= top_height {
                            return;
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
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for WindowPane {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
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
        let mut resps = self.focus_on_click(&ev);

        let (mut captured, resps_) = if event_to_inner {
            self.pane.receive_event(ctx, ev.clone())
        } else {
            (false, EventResponses::default())
        };
        resps.extend(resps_);

        // check if the inner pane has been removed from the parent in which case close this window
        // NOTE this happens with terminal
        if self.pane.eo.get_element(&self.inner.id()).is_none() {
            resps.push(EventResponse::Destruct);
            return (true, resps);
        }

        let just_minimized = self.partially_process_inner_resp(ctx, &mut resps);
        if captured {
            return (captured, resps);
        }

        // process dragging
        self.process_dragging(
            ctx,
            &ev,
            dragging,
            just_minimized,
            &mut captured,
            &mut resps,
        );

        // check if the inner pane has been removed from the parent in which case close this window
        if self.pane.eo.get_element(&self.inner.id()).is_none() {
            resps.push(EventResponse::Destruct);
        }

        (captured, resps)
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
        ctx: &Context, title: &str, close_button: bool, maximize_button: bool,
        minimize_button: bool,
    ) -> Self {
        let pane = ParentPane::new(ctx, "basic_window_top_bar")
            .with_dyn_height(DynVal::new_fixed(1))
            .with_dyn_width(DynVal::FULL)
            .with_style(Style::default().with_bg(Color::WHITE).with_fg(Color::BLACK));

        let btn_styles = SelStyles::new(
            Style::default()
                .with_bg(Color::GREY22)
                .with_fg(Color::BLACK),
            Style::default()
                .with_bg(Color::GREY22)
                .with_fg(Color::BLACK),
            Style::default()
                .with_bg(Color::GREY22)
                .with_fg(Color::BLACK),
        );

        let shadow_sty = ButtonMicroShadow::new(
            Some(Color::GREY19),
            Style::default().with_fg(Color::BLACK).with_bg(Color::BLUE),
        );

        let mut button_rhs_spaces = 2;

        if close_button {
            let close_button = Button::new(
                ctx,
                "x",
                Box::new(|btn, _ctx| {
                    let mut resps = btn.pane.deselect();
                    let resp = EventResponse::Custom(
                        WindowPane::CLOSE_WINDOW_MD_KEY.to_string(),
                        Vec::with_capacity(0),
                    );
                    resps.push(resp);
                    resps
                }),
            )
            .with_micro_shadow(shadow_sty.clone())
            .with_styles(btn_styles.clone())
            .at(
                DynVal::FULL.minus(button_rhs_spaces.into()),
                DynVal::new_fixed(0),
            );
            pane.add_element(Box::new(close_button));
            button_rhs_spaces += 2;
        }

        let mut maximizer_button = None;
        if maximize_button {
            let maximize_button = Button::new(
                ctx,
                "□",
                Box::new(|btn, _ctx| {
                    let mut resps = btn.pane.deselect();
                    // change the text to the restore icon or back
                    let existing_icon = btn.text.borrow().clone();
                    *btn.text.borrow_mut() = match existing_icon.as_str() {
                        "□" => "◱".to_string(),
                        _ => "□".to_string(),
                    };
                    let resp = EventResponse::Custom(
                        WindowPane::MAXIMIZE_WINDOW_MD_KEY.to_string(),
                        Vec::with_capacity(0),
                    );
                    resps.push(resp);
                    resps
                }),
            )
            .with_micro_shadow(shadow_sty.clone())
            .with_styles(btn_styles.clone())
            .at(
                DynVal::FULL.minus(button_rhs_spaces.into()),
                DynVal::new_fixed(0),
            );
            let b = Box::new(maximize_button);
            pane.add_element(b.clone());
            maximizer_button = Some(b);
            button_rhs_spaces += 2;
        }

        if minimize_button {
            let minimize_button = Button::new(
                ctx,
                "ˍ",
                Box::new(|btn, _ctx| {
                    let mut resps = btn.pane.deselect();
                    let resp = EventResponse::Custom(
                        WindowPane::MINIMIZE_WINDOW_MD_KEY.to_string(),
                        Vec::with_capacity(0),
                    );
                    resps.push(resp);
                    resps
                }),
            )
            .with_micro_shadow(shadow_sty)
            .with_styles(btn_styles)
            .at(
                DynVal::FULL.minus(button_rhs_spaces.into()),
                DynVal::new_fixed(0),
            );
            pane.add_element(Box::new(minimize_button));
        }

        let title_label = Box::new(
            Label::new(ctx, title)
                .with_style(Style::transparent())
                .at(DynVal::new_fixed(1), DynVal::new_fixed(0)),
        );
        let decor_label = Box::new(
            Label::new(ctx, "◹")
                .with_style(Style::transparent())
                .at(DynVal::FULL.minus(2.into()), DynVal::new_fixed(0)),
        );
        pane.add_element(title_label.clone());
        pane.add_element(decor_label.clone());
        decor_label.set_visible(false);

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
        self.title.set_visible(true);
        self.minimized_decor.set_visible(true);
    }

    /// useful for restore after minimization
    pub fn minimize_restore(&self) {
        self.pane.eo.set_all_visibility(true);
        self.minimized_decor.set_visible(false);
    }

    pub fn reset_maximizer_button(&self) {
        let maximizer_button = self.maximizer_button.borrow_mut();
        if let Some(mb) = maximizer_button.as_ref() {
            mb.text.replace("□".to_string());
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for BasicWindowTopBar {
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
}
