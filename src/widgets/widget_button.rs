use {
    super::{Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Color, Context, DrawChPos, DrawChs2D, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Keyboard as KB, Priority, ReceivableEventChanges, SortingHat, Style,
        UpwardPropagator,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// TODO DynVal button width

#[derive(Clone)]
pub struct Button {
    pub base: WidgetBase,
    pub text: Rc<RefCell<String>>,
    pub button_style: Rc<RefCell<ButtonStyle>>,
    pub clicked_down: Rc<RefCell<bool>>, // activated when mouse is clicked down while over button
    // function which executes when button moves from pressed -> unpressed
    #[allow(clippy::type_complexity)]
    pub clicked_fn: Rc<RefCell<dyn FnMut(Context) -> EventResponses>>,
}

#[derive(Clone)]
pub enum ButtonStyle {
    Basic(Option<Style>), // style when depressed
    Sides(ButtonSides),
    Shadow(ButtonShadow),
}

// ideas
//	]button[  ⡇button⢸
//	]button[  ⢸button⡇
//	⎤button⎣  ❳button❲ ⎣⦘button⦗⎤
#[derive(Clone)]
pub struct ButtonSides {
    pub depressed_style: Style,
    pub left: String,
    pub right: String,
    pub left_depressed: String, // while clicked
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
            //left_depressed: "⢸".to_string(),
            //right_depressed: "⡇".to_string(),
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
    pub top_right: char, // beside the button
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

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new(Some(Color::BLACK), Some(Color::LIGHT_YELLOW2), None),
        ready_style: Style::new(Some(Color::BLACK), Some(Color::WHITE), None),
        unselectable_style: Style::new(Some(Color::BLACK), Some(Color::GREY15), None),
    };

    pub fn default_receivable_events() -> Vec<Event> {
        vec![KB::KEY_ENTER.into()] // when "active" hitting enter will click the button
    }

    pub fn button_drawing(&self) -> DrawChs2D {
        match self.button_style.borrow().clone() {
            ButtonStyle::Basic(depressed_sty) => {
                let sty = if let Some(dsty) = depressed_sty {
                    if *self.clicked_down.borrow() {
                        dsty
                    } else {
                        self.base.get_current_style()
                    }
                } else {
                    self.base.get_current_style()
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
                    &self.base.get_current_style()
                };
                DrawChs2D::from_string(
                    format!("{}{}{}", left, self.text.borrow(), right),
                    sty.clone(),
                )
            }
            ButtonStyle::Shadow(shadow) => {
                let text_sty = self.base.get_current_style();
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
                            let fg = text_sty.bg.clone().unwrap_or_default().darken();
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

    pub fn new(
        hat: &SortingHat, _ctx: &Context, text: String,
        clicked_fn: Box<dyn FnMut(Context) -> EventResponses>,
    ) -> Self {
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            DynVal::new_fixed(text.chars().count() as i32 + 2), // + 2 for sides
            DynVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        //_ = wb.set_selectability(ctx, Selectability::Unselectable);
        let b = Button {
            base: wb,
            text: Rc::new(RefCell::new(text)),
            button_style: Rc::new(RefCell::new(ButtonStyle::Shadow(ButtonShadow::default()))),
            clicked_down: Rc::new(RefCell::new(false)),
            clicked_fn: Rc::new(RefCell::new(clicked_fn)),
        };

        let d = b.button_drawing();
        b.base.set_dyn_width(DynVal::new_fixed(d.width() as i32));
        b.base.set_dyn_height(DynVal::new_fixed(d.height() as i32));
        b
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn with_sides(self, sides: ButtonSides) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::Sides(sides);
        let d = self.button_drawing();
        self.base.set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.base
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self
    }

    pub fn with_shadow(self, shadow: ButtonShadow) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::Shadow(shadow);
        let d = self.button_drawing();
        self.base.set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.base
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self
    }

    pub fn basic_button(self, sty: Option<Style>) -> Self {
        *self.button_style.borrow_mut() = ButtonStyle::Basic(sty);
        let d = self.button_drawing();
        self.base.set_dyn_width(DynVal::new_fixed(d.width() as i32));
        self.base
            .set_dyn_height(DynVal::new_fixed(d.height() as i32));
        self
    }

    pub fn at(mut self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }

    // ----------------------------------------------
    pub fn click(&self, ctx: &Context) -> EventResponses {
        (self.clicked_fn.borrow_mut())(ctx.clone())
    }
}

impl Widget for Button {}

impl Element for Button {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }
                if ke[0].matches_key(&KB::KEY_ENTER) {
                    return (true, self.click(ctx));
                }
            }
            Event::Mouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, EventResponses::default());
                    }
                    MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        *self.clicked_down.borrow_mut() = false;
                        return (true, self.click(ctx));
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

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, _ctx: &Context) -> Vec<DrawChPos> {
        self.button_drawing().to_draw_ch_pos(0, 0)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.base.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.base.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.base.set_upward_propagator(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.base.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.base.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.base.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.base.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.base.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.base.get_visible()
    }
}
