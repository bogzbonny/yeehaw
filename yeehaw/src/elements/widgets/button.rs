use {
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
};

/// TODO DynVal button width

#[derive(Clone)]
pub struct Button {
    pub pane: SelectablePane,
    pub text: Rc<RefCell<String>>,
    pub button_style: Rc<RefCell<ButtonStyle>>,
    pub clicked_down: Rc<RefCell<bool>>,
    /// activated when mouse is clicked down while over button
    /// function which executes when button moves from pressed -> unpressed
    #[allow(clippy::type_complexity)]
    pub clicked_fn: Rc<RefCell<ButtonFn>>,
}

pub type ButtonFn = Box<dyn FnMut(Button, Context) -> EventResponses>;

#[derive(Clone)]
pub enum ButtonStyle {
    Basic(Option<Style>),
    /// style when depressed
    Sides(ButtonSides),
    Shadow(ButtonShadow),
    // a very thin righthand shadow: `░▏`
    MicroShadow(ButtonMicroShadow),
}

/// ideas
/// ]button[  ⡇button⢸
/// ]button[  ⢸button⡇
/// ⎤button⎣  ❳button❲ ⎣⦘button⦗⎤
#[derive(Clone)]
pub struct ButtonSides {
    pub depressed_style: Style,
    pub left: String,
    pub right: String,
    pub left_depressed: String,
    /// while clicked
    pub right_depressed: String,
}

impl Default for ButtonSides {
    fn default() -> Self {
        ButtonSides {
            depressed_style: Style::default_const()
                .with_fg(Color::BLACK)
                .with_bg(Color::WHITE),
            left: "]".to_string(),
            right: "[".to_string(),
            left_depressed: " ".to_string(),
            right_depressed: " ".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ButtonShadow {
    pub shadow_style: Option<Color>,
    /// if None will use a darkened version of the button's bg color
    pub bottom_left: char,
    pub bottom_middle: char,
    pub bottom_right: char,
    /// beside the button
    pub right: char,
    pub top_right: char,
}

impl Default for ButtonShadow {
    fn default() -> Self {
        ButtonShadow {
            shadow_style: None,
            bottom_left: '▝',
            bottom_middle: '▀',
            bottom_right: '▘',
            right: '▌',
            top_right: '▖',
        }
    }
}

#[derive(Clone)]
pub struct ButtonMicroShadow {
    pub shadow_style: Option<Color>,
    pub depressed_style: Style,
    pub right: char,
}

impl Default for ButtonMicroShadow {
    fn default() -> Self {
        ButtonMicroShadow {
            shadow_style: None,
            depressed_style: Style::default_const()
                .with_fg(Color::BLACK)
                .with_bg(Color::BLUE),
            right: '▎',
        }
    }
}

impl ButtonMicroShadow {
    pub fn new(shadow_style: Option<Color>, depressed_sty: Style) -> Self {
        ButtonMicroShadow {
            shadow_style,
            depressed_style: depressed_sty,
            right: '▎',
        }
    }
}

impl Button {
    const KIND: &'static str = "button";

    const STYLE: SelStyles = SelStyles {
        selected_style: Style::new_const(Color::BLACK, Color::LIGHT_YELLOW2),
        ready_style: Style::new_const(Color::BLACK, Color::WHITE),
        unselectable_style: Style::new_const(Color::BLACK, Color::GREY15),
    };

    pub fn default_receivable_events() -> ReceivableEvents {
        ReceivableEvents(vec![(KB::KEY_ENTER.into())]) // when "active" hitting enter will click the button
    }

    pub fn new(ctx: &Context, text: &str) -> Self {
        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_focused_receivable_events(Self::default_receivable_events())
            .with_styles(Self::STYLE);

        let b = Button {
            pane,
            text: Rc::new(RefCell::new(text.to_string())),
            button_style: Rc::new(RefCell::new(ButtonStyle::Shadow(ButtonShadow::default()))),
            clicked_down: Rc::new(RefCell::new(false)),
            clicked_fn: Rc::new(RefCell::new(Box::new(|_, _| EventResponses::default()))),
        };

        let d = b.button_drawing(ctx);

        b.pane.set_dyn_width(DynVal::new_fixed(d.width() as i32));
        b.pane.set_dyn_height(DynVal::new_fixed(d.height() as i32));
        b.pane.set_content(d);

        let b_ = b.clone();
        let ctx_ = ctx.clone();
        b.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                b_.pane.set_content(b_.button_drawing(&ctx_));
            }));
        b
    }

    pub fn button_drawing(&self, ctx: &Context) -> DrawChs2D {
        match self.button_style.borrow().clone() {
            ButtonStyle::Basic(depressed_sty) => {
                let sty = if let Some(dsty) = depressed_sty {
                    if *self.clicked_down.borrow() {
                        dsty
                    } else {
                        self.pane.get_current_style()
                    }
                } else {
                    self.pane.get_current_style()
                };
                DrawChs2D::from_string(self.text.borrow().clone(), sty)
            }
            ButtonStyle::Sides(sides) => {
                let left =
                    if *self.clicked_down.borrow() { &sides.left_depressed } else { &sides.left };
                let right =
                    if *self.clicked_down.borrow() { &sides.right_depressed } else { &sides.right };
                let sty = if *self.clicked_down.borrow() {
                    &sides.depressed_style
                } else {
                    &self.pane.get_current_style()
                };
                DrawChs2D::from_string(
                    format!("{}{}{}", left, self.text.borrow(), right),
                    sty.clone(),
                )
            }
            ButtonStyle::Shadow(shadow) => {
                let text_sty = self.pane.get_current_style();
                if *self.clicked_down.borrow() {
                    let non_button_sty = Style::default_const().with_bg(Color::TRANSPARENT);
                    let mut chs =
                        DrawChs2D::from_string(format!("{}", self.text.borrow()), text_sty.clone());
                    let padding = DrawCh::new(' ', text_sty.clone());
                    let blank = DrawCh::new(' ', non_button_sty.clone());
                    chs.pad_left(padding.clone(), 1);
                    chs.pad_right(padding.clone(), 1);
                    chs.pad_left(blank.clone(), 1);
                    chs.pad_bottom(blank.clone(), 1);
                    chs
                } else {
                    let shadow_sty = match shadow.shadow_style {
                        Some(c) => Style::default_const()
                            .with_bg(Color::TRANSPARENT)
                            .with_fg(c),
                        None => {
                            let fg = text_sty
                                .bg
                                .clone()
                                .unwrap_or_default()
                                .0
                                .darken(&ctx.color_store);
                            Style::default_const()
                                .with_bg(Color::TRANSPARENT)
                                .with_fg(fg)
                        }
                    };
                    let mut top =
                        DrawChs2D::from_string(format!("{}", self.text.borrow()), text_sty.clone());
                    let padding = DrawCh::new(' ', text_sty.clone());
                    top.pad_left(padding.clone(), 1);
                    top.pad_right(padding.clone(), 1);

                    let right_shadow = DrawCh::new(shadow.right, shadow_sty.clone());
                    top.pad_right(right_shadow.clone(), 1);

                    // adjust the top right corner to be the shadow's top right corner character
                    let top_right_shadow = DrawCh::new(shadow.top_right, shadow_sty.clone());
                    top.set_ch(top.width() - 1, 0, top_right_shadow);

                    let bottom_shadow = format!(
                        "{}{}{}",
                        shadow.bottom_left,
                        shadow.bottom_middle.to_string().repeat(top.width() - 2),
                        shadow.bottom_right
                    );
                    let bottom = DrawChs2D::from_string(bottom_shadow, shadow_sty.clone());
                    top.concat_top_bottom(bottom)
                }
            }
            ButtonStyle::MicroShadow(shadow) => {
                let text_sty = self.pane.get_current_style();
                if *self.clicked_down.borrow() {
                    let sty = shadow.depressed_style;
                    let mut chs =
                        DrawChs2D::from_string(format!("{}", self.text.borrow()), sty.clone());
                    let shadow_sty = Style::default_const()
                        .with_bg(Color::TRANSPARENT)
                        .with_fg(sty.bg.clone().unwrap_or_default().0);
                    let right_shadow = DrawCh::new(shadow.right, shadow_sty);
                    chs.pad_right(right_shadow.clone(), 1);
                    chs
                } else {
                    let shadow_sty = match shadow.shadow_style {
                        Some(c) => Style::default_const()
                            .with_bg(Color::TRANSPARENT)
                            .with_fg(c),
                        None => {
                            let fg = text_sty
                                .bg
                                .clone()
                                .unwrap_or_default()
                                .0
                                .darken(&ctx.color_store);
                            Style::default_const()
                                .with_bg(Color::TRANSPARENT)
                                .with_fg(fg)
                        }
                    };
                    let mut chs =
                        DrawChs2D::from_string(format!("{}", self.text.borrow()), text_sty);
                    let right_shadow = DrawCh::new(shadow.right, shadow_sty.clone());
                    chs.pad_right(right_shadow.clone(), 1);
                    chs
                }
            }
        }
    }

    // ----------------------------------------------
    // decorators

    pub fn with_fn(self, f: ButtonFn) -> Self {
        self.set_fn(f);
        self
    }

    pub fn set_fn(&self, f: ButtonFn) {
        *self.clicked_fn.borrow_mut() = f;
    }

    pub fn with_styles(self, ctx: &Context, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self.pane.set_content(self.button_drawing(ctx));
        self
    }

    pub fn basic_button(self, ctx: &Context, sty: Option<Style>) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::Basic(sty);
        let d = self.button_drawing(ctx);
        self.pane
            .pane
            .set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.pane
            .pane
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self.pane.set_content(d);
        self
    }
    pub fn with_sides(self, ctx: &Context, sides: ButtonSides) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::Sides(sides);
        let d = self.button_drawing(ctx);
        self.pane
            .pane
            .set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.pane
            .pane
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self.pane.set_content(d);
        self
    }

    pub fn with_shadow(self, ctx: &Context, shadow: ButtonShadow) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::Shadow(shadow);
        let d = self.button_drawing(ctx);
        self.pane
            .pane
            .set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.pane
            .pane
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self.pane.set_content(d);
        self
    }

    pub fn with_micro_shadow(self, ctx: &Context, shadow: ButtonMicroShadow) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::MicroShadow(shadow);
        let d = self.button_drawing(ctx);
        self.pane
            .pane
            .set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.pane
            .pane
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self.pane.set_content(d);
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    // ----------------------------------------------
    pub fn click(&self, ctx: &Context) -> EventResponses {
        (self.clicked_fn.borrow_mut())(self.clone(), ctx.clone())
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Button {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        if captured {
            return (true, resps);
        }
        if self.pane.get_selectability() == Selectability::Unselectable {
            return (false, resps);
        }
        match ev {
            Event::KeyCombo(ke) => {
                if self.pane.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, resps);
                }
                if ke[0] == KB::KEY_ENTER {
                    let resps_ = self.click(ctx);
                    resps.extend(resps_);
                    self.pane.set_content(self.button_drawing(ctx));
                    return (true, resps);
                }
            }
            Event::Mouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        self.pane.set_content(self.button_drawing(ctx));
                        return (true, resps);
                    }
                    MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        *self.clicked_down.borrow_mut() = false;
                        let resps_ = self.click(ctx);
                        resps.extend(resps_);
                        self.pane.set_content(self.button_drawing(ctx));
                        return (true, resps);
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                        self.pane.set_content(self.button_drawing(ctx));
                    }
                }
            }
            Event::ExternalMouse(_) => {
                let clicked_down = *self.clicked_down.borrow();
                if clicked_down {
                    *self.clicked_down.borrow_mut() = false;
                    self.pane.set_content(self.button_drawing(ctx));
                }
            }
            _ => {}
        }
        (false, resps)
    }
}
