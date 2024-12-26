use {
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
};

#[derive(Clone)]
pub struct Slider {
    pub pane: SelectablePane,

    pub is_dirty: Rc<RefCell<bool>>,

    /// position of the slider from 0.0 to 1.0
    pub position: Rc<RefCell<f64>>,

    pub filled: Rc<RefCell<DrawCh>>,
    pub empty: Rc<RefCell<DrawCh>>,
    pub head: Rc<RefCell<DrawCh>>,

    /// activated when mouse is clicked down while over button
    pub clicked_down: Rc<RefCell<bool>>,

    // called for each adjustment
    pub adjust_fn: Rc<RefCell<AdjustFn>>,
}

pub type AdjustFn = Box<dyn FnMut(Context, &Slider) -> EventResponses>;

impl Slider {
    const KIND: &'static str = "slider";

    pub fn default_receivable_events() -> ReceivableEvents {
        ReceivableEvents(vec![
            (KB::KEY_LEFT.into()),
            (KB::KEY_RIGHT.into()),
            (KB::KEY_H.into()),
            (KB::KEY_L.into()),
        ])
    }

    pub fn new_basic_block(ctx: &Context) -> Self {
        Self::new(
            ctx,
            DrawCh::new('■', Style::new_const(Color::AQUA, Color::GREY13)),
            DrawCh::new(' ', Style::new_const(Color::BLACK, Color::GREY13)),
            DrawCh::new('⛊', Style::new_const(Color::AQUA, Color::GREY13)),
        )
    }

    pub fn new_basic_line(ctx: &Context) -> Self {
        Self::new(
            ctx,
            DrawCh::new('━', Style::new_const(Color::AQUA, Color::GREY13)),
            DrawCh::new('─', Style::new_const(Color::BLACK, Color::GREY13)),
            DrawCh::new('⛊', Style::new_const(Color::AQUA, Color::GREY13)),
        )
    }

    pub fn new_basic_line_diamond(ctx: &Context) -> Self {
        Self::new(
            ctx,
            DrawCh::new('━', Style::new_const(Color::AQUA, Color::GREY13)),
            DrawCh::new('─', Style::new_const(Color::BLACK, Color::GREY13)),
            DrawCh::new('◆', Style::new_const(Color::AQUA, Color::GREY13)),
        )
    }

    /// creates a new slider
    /// some inspirations:
    /// ```text
    ///━━━━━━━━━━⛊─────────────────────
    ///━━━━━━━━━●
    ///━━━━━━━━━━━━━━━━━━╉─────────────
    ///███████████████╋━━━━━━━━━━━━━━━━
    /// ```
    pub fn new(ctx: &Context, filled: DrawCh, empty: DrawCh, head: DrawCh) -> Self {
        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_focused_receivable_events(Self::default_receivable_events())
            .with_styles(SelStyles::opaque(ctx))
            .with_dyn_width(DynVal::FULL)
            .with_dyn_height(DynVal::new_fixed(1));

        let t = Slider {
            pane,
            is_dirty: Rc::new(RefCell::new(true)),
            position: Rc::new(RefCell::new(0.0)),
            filled: Rc::new(RefCell::new(filled)),
            empty: Rc::new(RefCell::new(empty)),
            head: Rc::new(RefCell::new(head)),
            clicked_down: Rc::new(RefCell::new(false)),
            adjust_fn: Rc::new(RefCell::new(Box::new(|_, _| EventResponses::default()))),
        };

        let t_ = t.clone();
        t.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                t_.is_dirty.replace(true);
            }));
        t
    }

    // ----------------------------------------------
    // decorators

    pub fn with_gradient(self, start_color: Color, end_color: Color) -> Self {
        let gr = Gradient::new_x_grad_2_color(start_color, end_color);
        if let Some((fg, _)) = &mut self.filled.borrow_mut().style.fg {
            *fg = Color::Gradient(gr);
        }
        self
    }

    pub fn with_color(self, c: Color) -> Self {
        self.filled.borrow_mut().style.set_fg(c);
        self
    }

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self
    }

    pub fn with_fn(self, adjust_fn: AdjustFn) -> Self {
        *self.adjust_fn.borrow_mut() = adjust_fn;
        self
    }

    pub fn with_width(self, width: DynVal) -> Self {
        self.pane.set_dyn_width(width);
        self
    }

    pub fn with_position(self, pos: f64) -> Self {
        self.set_position(pos);
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    // ----------------------------------------------

    pub fn get_position(&self) -> f64 {
        *self.position.borrow()
    }

    pub fn set_position(&self, mut pos: f64) {
        pos = pos.clamp(0.0, 1.0);
        *self.position.borrow_mut() = pos;
        self.is_dirty.replace(true);
    }

    pub fn perform_adjustment(&self, ctx: &Context, pos: f64) -> EventResponses {
        self.set_position(pos);
        let mut adjust_fn = self.adjust_fn.borrow_mut();
        adjust_fn(ctx.clone(), self)
    }

    /// get the position from the mouse x position
    pub fn get_pos_from_x(&self, ctx: &Context, x: i32) -> f64 {
        if ctx.size.width == 0 {
            return 0.0;
        }
        let width = (ctx.size.width - 1) as f64;
        (x as f64) / width
    }

    pub fn get_x_from_pos(&self, ctx: &Context) -> i32 {
        let pos = self.get_position();
        if ctx.size.width == 0 {
            return 0;
        }
        let width = (ctx.size.width - 1) as f64;
        (pos * width).round_ties_even() as i32
    }

    pub fn increment_position(&self, ctx: &Context) -> EventResponses {
        let pos = self.get_pos_from_x(ctx, self.get_x_from_pos(ctx) + 1);
        self.perform_adjustment(ctx, pos)
    }

    pub fn decrement_position(&self, ctx: &Context) -> EventResponses {
        let pos = self.get_pos_from_x(ctx, self.get_x_from_pos(ctx) - 1);
        self.perform_adjustment(ctx, pos)
    }

    pub fn update_content(&self, ctx: &Context) {
        let width = ctx.size.width;
        let pos = *self.position.borrow();

        let sty = self.pane.get_current_style();
        let filled = self.filled.borrow().clone().with_overlay_style(ctx, &sty);
        let empty = self.empty.borrow().clone().with_overlay_style(ctx, &sty);
        let head = self.head.borrow().clone().with_overlay_style(ctx, &sty);

        let mut out = vec![];
        let chs_filled = (width as f64 * pos).round() as usize;
        for _ in 0..chs_filled.saturating_sub(1) {
            out.push(filled.clone());
        }
        out.push(head);
        for _ in chs_filled..width as usize {
            out.push(empty.clone());
        }
        let content = DrawChs2D::from_draw_chs_horizontal(out);
        self.pane.set_content(content);
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Slider {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        if captured {
            return (true, resps);
        }
        let sel = self.pane.get_selectability();
        match ev {
            Event::KeyCombo(ke) => {
                if self.pane.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, resps);
                }
                match true {
                    _ if ke[0] == KB::KEY_LEFT || ke[0] == KB::KEY_H => {
                        let resps_ = self.decrement_position(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ if ke[0] == KB::KEY_RIGHT || ke[0] == KB::KEY_L => {
                        let resps_ = self.increment_position(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ => {}
                }
                return (false, resps);
            }
            Event::Mouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollRight
                        if sel == Selectability::Selected =>
                    {
                        let resps_ = self.decrement_position(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    MouseEventKind::ScrollDown | MouseEventKind::ScrollLeft
                        if sel == Selectability::Selected =>
                    {
                        let resps_ = self.increment_position(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, resps);
                    }
                    MouseEventKind::Drag(MouseButton::Left)
                    | MouseEventKind::Up(MouseButton::Left)
                        if clicked_down =>
                    {
                        let x = me.column as usize;
                        let pos = self.get_pos_from_x(ctx, x as i32);
                        let resps_ = self.perform_adjustment(ctx, pos);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }
                return (false, resps);
            }
            Event::ExternalMouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::Drag(MouseButton::Left)
                    | MouseEventKind::Up(MouseButton::Left)
                        if clicked_down =>
                    {
                        let x = me.column as usize;
                        let pos = self.get_pos_from_x(ctx, x as i32);
                        let resps_ = self.perform_adjustment(ctx, pos);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ if clicked_down => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                    _ => {}
                }
            }
            Event::Resize => {
                self.is_dirty.replace(true);
                return (false, resps);
            }
            _ => {}
        }
        (false, resps)
    }
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        if self.is_dirty.replace(false) || force_update {
            self.update_content(ctx);
        }
        self.pane.drawing(ctx, force_update)
    }
}
