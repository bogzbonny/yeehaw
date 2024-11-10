use {
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
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
    pub clicked_fn: Rc<RefCell<dyn FnMut(Button, Context) -> EventResponses>>,
}

#[derive(Clone)]
pub enum ButtonStyle {
    Basic(Option<Style>),
    /// style when depressed
    Sides(ButtonSides),
    Shadow(ButtonShadow),
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
    pub left: char,
    pub middle: char,
    pub right: char,
    /// beside the button
    pub top_right: char,
}

impl Default for ButtonShadow {
    fn default() -> Self {
        ButtonShadow {
            shadow_style: None,
            left: '▝',
            middle: '▀',
            right: '▘',
            top_right: '▖',
        }
    }
}

impl Button {
    const KIND: &'static str = "widget_button";

    const STYLE: SelStyles = SelStyles {
        selected_style: Style::new_const(Color::BLACK, Color::LIGHT_YELLOW2),
        ready_style: Style::new_const(Color::BLACK, Color::WHITE),
        unselectable_style: Style::new_const(Color::BLACK, Color::GREY15),
    };

    pub fn default_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![(KB::KEY_ENTER.into(), Priority::Focused)]) // / when "active" hitting enter will click the button
    }

    pub fn new(
        ctx: &Context, text: &str, clicked_fn: Box<dyn FnMut(Button, Context) -> EventResponses>,
    ) -> Self {
        let pane = SelectablePane::new(ctx, Self::KIND);
        pane.pane
            .pane
            .set_self_receivable_events(Self::default_receivable_events());
        pane.set_styles(Self::STYLE);

        let b = Button {
            pane,
            text: Rc::new(RefCell::new(text.to_string())),
            button_style: Rc::new(RefCell::new(ButtonStyle::Shadow(ButtonShadow::default()))),
            clicked_down: Rc::new(RefCell::new(false)),
            clicked_fn: Rc::new(RefCell::new(clicked_fn)),
        };

        let d = b.button_drawing();
        b.pane
            .pane
            .set_dyn_width(DynVal::new_fixed(d.width() as i32));
        b.pane
            .pane
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        b
    }

    pub fn button_drawing(&self) -> DrawChs2D {
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
                    let left = DrawChs2D::from_string(" ".to_string(), non_button_sty.clone());
                    let top = DrawChs2D::from_string(format!(" {} ", self.text.borrow()), text_sty);
                    let top = left.concat_left_right(top).unwrap();
                    let width = top.width();
                    // bottom all spaces
                    let bottom = DrawChs2D::from_string(" ".repeat(width), non_button_sty.clone());
                    top.concat_top_bottom(bottom)
                } else {
                    let shadow_sty = match shadow.shadow_style {
                        Some(c) => Style::default_const()
                            .with_bg(Color::TRANSPARENT)
                            .with_fg(c),
                        None => {
                            let fg = text_sty.bg.clone().unwrap_or_default().0.darken();
                            Style::default_const()
                                .with_bg(Color::TRANSPARENT)
                                .with_fg(fg)
                        }
                    };
                    let top = DrawChs2D::from_string(
                        format!(" {} ", self.text.borrow()),
                        text_sty.clone(),
                    );
                    let right =
                        DrawChs2D::from_string(shadow.top_right.to_string(), shadow_sty.clone());
                    let top = top.concat_left_right(right).unwrap();
                    let bottom_text = format!(
                        "{}{}{}",
                        shadow.left,
                        shadow.middle.to_string().repeat(top.width() - 2),
                        shadow.right
                    );
                    let bottom = DrawChs2D::from_string(bottom_text, shadow_sty.clone());
                    top.concat_top_bottom(bottom)
                }
            }
        }
    }

    // ----------------------------------------------
    /// decorators

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self
    }

    pub fn with_sides(self, sides: ButtonSides) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::Sides(sides);
        let d = self.button_drawing();
        self.pane
            .pane
            .set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.pane
            .pane
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self
    }

    pub fn with_shadow(self, shadow: ButtonShadow) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::Shadow(shadow);
        let d = self.button_drawing();
        self.pane
            .pane
            .set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.pane
            .pane
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self
    }

    pub fn basic_button(self, sty: Option<Style>) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::Basic(sty);
        let d = self.button_drawing();
        self.pane
            .pane
            .set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.pane
            .pane
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.pane.pane.set_at(loc_x, loc_y);
        self
    }

    // ----------------------------------------------
    pub fn click(&self, ctx: &Context) -> EventResponses {
        (self.clicked_fn.borrow_mut())(self.clone(), ctx.clone())
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Button {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        if captured {
            return (true, resps);
        }
        match ev {
            Event::KeyCombo(ke) => {
                if self.pane.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, resps);
                }
                if ke[0] == KB::KEY_ENTER {
                    let resps_ = self.click(ctx);
                    resps.extend(resps_);
                    return (true, resps);
                }
            }
            Event::Mouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, resps);
                    }
                    MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        *self.clicked_down.borrow_mut() = false;
                        let resps_ = self.click(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }
            }
            Event::ExternalMouse(_) => {
                let clicked_down = *self.clicked_down.borrow();
                if clicked_down {
                    *self.clicked_down.borrow_mut() = false;
                }
            }
            _ => {}
        }
        (false, EventResponses::default())
    }

    fn drawing(&self, _ctx: &Context) -> Vec<DrawChPos> {
        self.button_drawing().to_draw_ch_pos(0, 0)
    }
}
