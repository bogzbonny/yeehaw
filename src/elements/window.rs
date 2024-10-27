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

// TODO border

// ---------------------------------------------------
#[derive(Clone)]
pub struct WindowPane {
    pub pane: ParentPane,
    pub top_bar: Rc<RefCell<dyn Element>>,
    pub inner: Rc<RefCell<dyn Element>>,
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
    pub const Z_INDEX: ZIndex = 50;
    pub fn new(
        hat: &SortingHat, ctx: &Context, inner: Rc<RefCell<dyn Element>>, title: &str,
    ) -> Self {
        //let vs = VerticalStack::new_with_kind(hat, Self::KIND).with_style(Style::transparent());
        let vs = ParentPane::new(hat, Self::KIND).with_transparent();
        let top_bar = Rc::new(RefCell::new(BasicWindowTopBar::new(
            hat, ctx, title, true, true, true,
        )));

        // adjust the inner size to account for the top bar
        //let top_height = top_bar.borrow().get_dyn_location_set().borrow().l.height(ctx);
        //let inner_loc = inner.borrow().get_dyn_location_set().borrow().l.clone();
        //let inner_loc = DynLocation::new(
        //    DynVal::new_fixed(inner_loc.start_x(ctx)),
        //    DynVal::new_fixed(inner_loc.start_y(ctx) + top_height),
        //    DynVal::new_fixed(inner_loc.end_x(ctx)),
        //    DynVal::new_fixed(inner_loc.end_y(ctx)),
        //);
        //inner.borrow().get_dyn_location_set().replace(inner_loc);

        let mut loc = inner.borrow().get_dyn_location_set().borrow().clone();
        loc.set_start_y(1.into());
        loc.set_dyn_height(DynVal::new_flex(1.).minus(1.into()));
        inner.borrow().get_dyn_location_set().replace(loc);

        vs.add_element(top_bar.clone());
        vs.add_element(inner.clone());
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
        self.pane.add_element(Rc::new(RefCell::new(ca)));

        use crate::organizer::ElDetails;
        let mut eoz: Vec<(ElementID, ElDetails)> = Vec::new();
        for (el_id, details) in self.pane.eo.els.borrow().iter() {
            eoz.push((el_id.clone(), details.clone()));
        }

        /*
        // sort z index from low to high
        eoz.sort_by(|a, b| a.1.loc.borrow().z.cmp(&b.1.loc.borrow().z));

        // draw elements in order from highest z-index to lowest
        for el_id_z in eoz {
            debug!("\n element {:?}", el_id_z.0);
            let details = self
                .pane
                .pane
                .eo
                .get_element_details(&el_id_z.0)
                .expect("impossible");
            if !*details.vis.borrow() {
                continue;
            }
            if let Some(vis_loc) = ctx.visible_region {
                if !vis_loc.intersects_dyn_location_set(ctx, &details.loc.borrow()) {
                    continue;
                }
            }

            debug!("ctx {:?}", ctx);
            let size = el_id_z.1.loc.borrow().l.get_size(ctx);
            debug!("size {:?}", size);
            let start_x = el_id_z.1.loc.borrow().l.get_start_x(ctx);
            let start_y = el_id_z.1.loc.borrow().l.get_start_y(ctx);
            let end_x = el_id_z.1.loc.borrow().l.get_end_x(ctx);
            let end_y = el_id_z.1.loc.borrow().l.get_end_y(ctx);
            debug!("start_x {:?}", start_x);
            debug!("start_y {:?}", start_y);
            debug!("end_x {:?}", end_x);
            debug!("end_y {:?}", end_y);

            let child_ctx = self.pane.pane.eo.get_context_for_el(ctx, &el_id_z.1);

            debug!("child ctx {:?}", child_ctx);
        }
        */

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

                //let end_x = self.pane.pane.get_end_x(ctx) + dx;
                //let end_y = self.pane.pane.get_end_y(ctx) + dy;
                //self.pane.pane.set_end_x(DynVal::new_fixed(end_x));
                //self.pane.pane.set_end_y(DynVal::new_fixed(end_y));

                let mut end_x = self.pane.pane.get_dyn_end_x().plus(dx.into());
                let mut end_y = self.pane.pane.get_dyn_end_y().plus(dy.into());
                end_x.flatten_internal();
                end_y.flatten_internal();
                self.pane.pane.set_end_x(end_x);
                self.pane.pane.set_end_y(end_y);

                let inner_ctx = ctx.clone().with_size(Size::new(
                    self.pane.pane.get_width(ctx) as u16,
                    self.pane.pane.get_height(ctx) as u16 - 1,
                ));

                let (_, r) = self.inner.borrow().receive_event(&inner_ctx, Event::Resize);
                resps_.extend(r.0);

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
                        let x1 = self.pane.pane.get_start_x(ctx) + dx;
                        let y1 = self.pane.pane.get_start_y(ctx) + dy;
                        let x2 = self.pane.pane.get_end_x(ctx) + dx;
                        let y2 = self.pane.pane.get_end_y(ctx) + dy;

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
                            let dx = me.column - start_x;
                            let dy = me.row - start_y;
                            let x1 = self.pane.pane.get_start_x(ctx) + dx;
                            let y1 = self.pane.pane.get_start_y(ctx) + dy;
                            let x2 = self.pane.pane.get_end_x(ctx) + dx;
                            let y2 = self.pane.pane.get_end_y(ctx) + dy;
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
            pane.add_element(Rc::new(RefCell::new(maximize_button)));
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
            pane.add_element(Rc::new(RefCell::new(minimize_button)));
        }

        let title_label = Rc::new(RefCell::new(
            Label::new(hat, ctx, title)
                .with_style(ctx, Style::transparent())
                .at(DynVal::new_fixed(1), DynVal::new_fixed(0)),
        ));
        let decor_label = Rc::new(RefCell::new(
            Label::new(hat, ctx, "◹")
                .with_style(ctx, Style::transparent())
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

// ------------------------------------------------

/// a small element (one character) that can be used to send resize requests to parent elements
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
