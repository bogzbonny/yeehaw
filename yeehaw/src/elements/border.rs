use {
    crate::*,
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// displays the size
#[derive(Clone)]
pub struct Bordered {
    pub pane: ParentPane,
    pub chs: Rc<RefCell<BorderSty>>,
    pub properties: Rc<RefCell<BorderProperies>>,
}

/// property for the border
#[derive(Clone, Copy)]
pub enum Property {
    None,       // just for display
    DragResize, // drag to resize
    DragMove,   // drag to move
}

/// property for the border
/// including vertical scrollbar option
#[derive(Clone)]
pub enum PropertyVSB {
    None,       // just for display
    DragResize, // drag to resize
    DragMove,   // drag to move
    Scrollbar(widgets::VerticalScrollbar),
}

/// property for the border
/// including horizontal scrollbar option
#[derive(Clone)]
pub enum PropertyHSB {
    None,       // just for display
    DragResize, // drag to resize
    DragMove,   // drag to move
    Scrollbar(widgets::HorizontalScrollbar),
}

#[derive(Clone)]
pub struct BorderProperies {
    pub left: Option<Property>,      // if None, then no left border
    pub right: Option<PropertyVSB>,  // if None, then no right border
    pub top: Option<Property>,       // if None, then no top border
    pub bottom: Option<PropertyHSB>, // if None, then no bottom border
    pub top_corner: Property,
    pub bottom_corner: Property,
    pub left_corner: Property,
    pub right_corner: Property,
}

#[derive(Clone)]
pub struct BorderSty {
    pub left: DrawCh,
    pub right: DrawCh,
    pub top: DrawCh,
    pub bottom: DrawCh,
    pub top_left: DrawCh,
    pub top_right: DrawCh,
    pub bottom_left: DrawCh,
    pub bottom_right: DrawCh,
}

impl BorderSty {
    pub fn new_thin_single(sty: Style) -> Self {
        Self {
            left: DrawCh::new('│', sty.clone()),
            right: DrawCh::new('│', sty.clone()),
            top: DrawCh::new('─', sty.clone()),
            bottom: DrawCh::new('─', sty.clone()),
            bottom_left: DrawCh::new('└', sty.clone()),
            bottom_right: DrawCh::new('┘', sty.clone()),
            top_left: DrawCh::new('┌', sty.clone()),
            top_right: DrawCh::new('┐', sty),
        }
    }

    pub fn new_thin_single_rounded(sty: Style) -> Self {
        Self {
            left: DrawCh::new('│', sty.clone()),
            right: DrawCh::new('│', sty.clone()),
            top: DrawCh::new('─', sty.clone()),
            bottom: DrawCh::new('─', sty.clone()),
            bottom_left: DrawCh::new('╰', sty.clone()),
            bottom_right: DrawCh::new('╯', sty.clone()),
            top_left: DrawCh::new('╭', sty.clone()),
            top_right: DrawCh::new('╮', sty),
        }
    }

    pub fn new_thick_single(sty: Style) -> Self {
        Self {
            left: DrawCh::new('┃', sty.clone()),
            right: DrawCh::new('┃', sty.clone()),
            top: DrawCh::new('━', sty.clone()),
            bottom: DrawCh::new('━', sty.clone()),
            top_left: DrawCh::new('┏', sty.clone()),
            top_right: DrawCh::new('┓', sty.clone()),
            bottom_left: DrawCh::new('┗', sty.clone()),
            bottom_right: DrawCh::new('┛', sty),
        }
    }

    pub fn new_double(sty: Style) -> Self {
        Self {
            left: DrawCh::new('║', sty.clone()),
            right: DrawCh::new('║', sty.clone()),
            top: DrawCh::new('═', sty.clone()),
            bottom: DrawCh::new('═', sty.clone()),
            top_left: DrawCh::new('╔', sty.clone()),
            top_right: DrawCh::new('╗', sty.clone()),
            bottom_left: DrawCh::new('╚', sty.clone()),
            bottom_right: DrawCh::new('╝', sty),
        }
    }

    pub fn new_double_vertical(sty: Style) -> Self {
        Self {
            left: DrawCh::new('║', sty.clone()),
            right: DrawCh::new('║', sty.clone()),
            bottom: DrawCh::new('─', sty.clone()),
            top: DrawCh::new('─', sty.clone()),
            top_left: DrawCh::new('╓', sty.clone()),
            top_right: DrawCh::new('╖', sty.clone()),
            bottom_left: DrawCh::new('╙', sty.clone()),
            bottom_right: DrawCh::new('╜', sty),
        }
    }

    pub fn new_double_horizontal(sty: Style) -> Self {
        Self {
            left: DrawCh::new('│', sty.clone()),
            right: DrawCh::new('│', sty.clone()),
            top: DrawCh::new('═', sty.clone()),
            bottom: DrawCh::new('═', sty.clone()),
            top_left: DrawCh::new('╒', sty.clone()),
            top_right: DrawCh::new('╕', sty.clone()),
            bottom_left: DrawCh::new('╘', sty.clone()),
            bottom_right: DrawCh::new('╛', sty),
        }
    }

    pub fn new_thin_double_dashed(sty: Style) -> Self {
        Self {
            left: DrawCh::new('╎', sty.clone()),
            right: DrawCh::new('╎', sty.clone()),
            top: DrawCh::new('╌', sty.clone()),
            bottom: DrawCh::new('╌', sty.clone()),
            top_left: DrawCh::new('┌', sty.clone()),
            top_right: DrawCh::new('┐', sty.clone()),
            bottom_left: DrawCh::new('└', sty.clone()),
            bottom_right: DrawCh::new('┘', sty),
        }
    }

    pub fn new_thick_double_dashed(sty: Style) -> Self {
        Self {
            left: DrawCh::new('╏', sty.clone()),
            right: DrawCh::new('╏', sty.clone()),
            top: DrawCh::new('╍', sty.clone()),
            bottom: DrawCh::new('╍', sty.clone()),
            top_left: DrawCh::new('┏', sty.clone()),
            top_right: DrawCh::new('┓', sty.clone()),
            bottom_left: DrawCh::new('┗', sty.clone()),
            bottom_right: DrawCh::new('┛', sty),
        }
    }

    pub fn new_thin_triple_dashed(sty: Style) -> Self {
        Self {
            left: DrawCh::new('┆', sty.clone()),
            right: DrawCh::new('┆', sty.clone()),
            top: DrawCh::new('┄', sty.clone()),
            bottom: DrawCh::new('┄', sty.clone()),
            top_left: DrawCh::new('┌', sty.clone()),
            top_right: DrawCh::new('┐', sty.clone()),
            bottom_left: DrawCh::new('└', sty.clone()),
            bottom_right: DrawCh::new('┘', sty),
        }
    }

    pub fn new_thick_triple_dashed(sty: Style) -> Self {
        Self {
            left: DrawCh::new('┇', sty.clone()),
            right: DrawCh::new('┇', sty.clone()),
            top: DrawCh::new('┅', sty.clone()),
            bottom: DrawCh::new('┅', sty.clone()),
            top_left: DrawCh::new('┏', sty.clone()),
            top_right: DrawCh::new('┓', sty.clone()),
            bottom_left: DrawCh::new('┗', sty.clone()),
            bottom_right: DrawCh::new('┛', sty),
        }
    }

    pub fn new_thin_quadruple_dashed(sty: Style) -> Self {
        Self {
            left: DrawCh::new('┊', sty.clone()),
            right: DrawCh::new('┊', sty.clone()),
            top: DrawCh::new('┈', sty.clone()),
            bottom: DrawCh::new('┈', sty.clone()),
            top_left: DrawCh::new('┌', sty.clone()),
            top_right: DrawCh::new('┐', sty.clone()),
            bottom_left: DrawCh::new('└', sty.clone()),
            bottom_right: DrawCh::new('┘', sty),
        }
    }

    pub fn new_thick_quadruple_dashed(sty: Style) -> Self {
        Self {
            left: DrawCh::new('┋', sty.clone()),
            right: DrawCh::new('┋', sty.clone()),
            top: DrawCh::new('┉', sty.clone()),
            bottom: DrawCh::new('┉', sty.clone()),
            top_left: DrawCh::new('┏', sty.clone()),
            top_right: DrawCh::new('┓', sty.clone()),
            bottom_left: DrawCh::new('┗', sty.clone()),
            bottom_right: DrawCh::new('┛', sty),
        }
    }

    /// ```
    ///  ________
    /// ⎥hello hi⎢
    ///  ‾‾‾‾‾‾‾‾
    /// ```
    pub fn new_tight_box(sty: Style) -> Self {
        Self {
            left: DrawCh::new('⎥', sty.clone()),
            right: DrawCh::new('⎢', sty.clone()),
            top: DrawCh::new('_', sty.clone()),
            bottom: DrawCh::new('‾', sty.clone()),
            top_left: DrawCh::new(' ', sty.clone()),
            top_right: DrawCh::new(' ', sty.clone()),
            bottom_left: DrawCh::new(' ', sty.clone()),
            bottom_right: DrawCh::new(' ', sty),
        }
    }

    /// ```
    /// ⌟________⌞
    /// ⎥hello hi⎢
    /// ⌝‾‾‾‾‾‾‾‾⌜
    /// ```
    pub fn new_tight_box_fancy(sty: Style) -> Self {
        Self {
            left: DrawCh::new('⎥', sty.clone()),
            right: DrawCh::new('⎢', sty.clone()),
            top: DrawCh::new('_', sty.clone()),
            bottom: DrawCh::new('‾', sty.clone()),
            top_left: DrawCh::new('⌟', sty.clone()),
            top_right: DrawCh::new('⌞', sty.clone()),
            bottom_left: DrawCh::new('⌝', sty.clone()),
            bottom_right: DrawCh::new('⌜', sty),
        }
    }

    /// ```
    /// ⎡‾‾‾‾‾‾‾‾‾⎤
    /// ⎢ welcome ⎥
    /// ⎢    to   ⎥
    /// ⎢  my box ⎥
    /// ⎣_________⎦
    /// ```
    pub fn new_large_box(sty: Style) -> Self {
        Self {
            left: DrawCh::new('⎢', sty.clone()),
            right: DrawCh::new('⎥', sty.clone()),
            top: DrawCh::new('‾', sty.clone()),
            bottom: DrawCh::new('_', sty.clone()),
            top_left: DrawCh::new('⎡', sty.clone()),
            top_right: DrawCh::new('⎤', sty.clone()),
            bottom_left: DrawCh::new('⎣', sty.clone()),
            bottom_right: DrawCh::new('⎦', sty),
        }
    }

    /// ```
    ///
    /// ⌜‾‾‾‾‾‾‾‾‾⌝
    /// ⎢ welcome ⎥
    /// ⎢    to   ⎥
    /// ⎢  my box ⎥
    /// ⌞_________⌟
    /// ```
    pub fn new_large_box_fancy(sty: Style) -> Self {
        Self {
            left: DrawCh::new('⎢', sty.clone()),
            right: DrawCh::new('⎥', sty.clone()),
            top: DrawCh::new('‾', sty.clone()),
            bottom: DrawCh::new('_', sty.clone()),
            top_left: DrawCh::new('⌜', sty.clone()),
            top_right: DrawCh::new('⌝', sty.clone()),
            bottom_left: DrawCh::new('⌞', sty.clone()),
            bottom_right: DrawCh::new('⌟', sty),
        }
    }

    pub fn new_tight_half_block(sty: Style) -> Self {
        Self {
            left: DrawCh::new('▐', sty.clone()),
            right: DrawCh::new('▌', sty.clone()),
            top: DrawCh::new('▄', sty.clone()),
            bottom: DrawCh::new('▀', sty.clone()),
            top_left: DrawCh::new('▗', sty.clone()),
            top_right: DrawCh::new('▖', sty.clone()),
            bottom_left: DrawCh::new('▝', sty.clone()),
            bottom_right: DrawCh::new('▘', sty),
        }
    }

    pub fn new_large_half_block(sty: Style) -> Self {
        Self {
            left: DrawCh::new('▌', sty.clone()),
            right: DrawCh::new('▐', sty.clone()),
            top: DrawCh::new('▀', sty.clone()),
            bottom: DrawCh::new('▄', sty.clone()),
            top_left: DrawCh::new('▛', sty.clone()),
            top_right: DrawCh::new('▜', sty.clone()),
            bottom_left: DrawCh::new('▙', sty.clone()),
            bottom_right: DrawCh::new('▟', sty),
        }
    }

    // a border composed of all the same character
    // some recommendations: █ ▓ ▒ ░
    pub fn new_uniform_ch(ch: char, sty: Style) -> Self {
        Self {
            left: DrawCh::new(ch, sty.clone()),
            right: DrawCh::new(ch, sty.clone()),
            top: DrawCh::new(ch, sty.clone()),
            bottom: DrawCh::new(ch, sty.clone()),
            top_left: DrawCh::new(ch, sty.clone()),
            top_right: DrawCh::new(ch, sty.clone()),
            bottom_left: DrawCh::new(ch, sty.clone()),
            bottom_right: DrawCh::new(ch, sty),
        }
    }

    /// overwrite the all corners with a character
    /// some nice recommendations:
    /// ꕤ ꕥ ꕢ ꕣ ꕕ ✿ ❀ ❁ ❃ ❋ ❂ ❀ ֍ ֎ ⁜ ᳀
    pub fn with_corners(mut self, ch: char) -> Self {
        self.top_left = DrawCh::new(ch, self.top_left.style);
        self.top_right = DrawCh::new(ch, self.top_right.style);
        self.bottom_left = DrawCh::new(ch, self.bottom_left.style);
        self.bottom_right = DrawCh::new(ch, self.bottom_right.style);
        self
    }
}

impl Bordered {
    const RESIZE_MD_KEY: &'static str = "adjust_size";
    const MOVE_MD_KEY: &'static str = "move";
    pub const KIND: &'static str = "bordered";

    pub fn thin(ctx: &Context, inner: Box<dyn Element>) -> Bordered {
        let shadow_color = Color::new_with_alpha(100, 100, 100, 100);
        let out = Bordered {
            inner,
            sh_sty: Rc::new(RefCell::new(BorderSty::new_thin(shadow_color))),
        };
        out.set_shadow_content(ctx);
        out
    }

    pub fn thin_with_color(
        ctx: &Context, inner: Box<dyn Element>, shadow_color: Color,
    ) -> Bordered {
        let out = Bordered {
            inner,
            sh_sty: Rc::new(RefCell::new(BorderSty::new_thin(shadow_color))),
        };
        out.set_shadow_content(ctx);
        out
    }

    // TODO could cache this
    pub fn set_border_content(&self, ctx: &Context) -> Vec<DrawChPos> {
        let size = ctx.s;
        let sh_sty = self.sh_sty.borrow();

        let mut out = vec![];

        out.push(DrawChPos::new(sh_sty.bottom_left.clone(), 0, size.height));
        out.push(DrawChPos::new(sh_sty.top_right.clone(), size.width, 0));
        out.push(DrawChPos::new(
            sh_sty.bottom_right.clone(),
            size.width,
            size.height,
        ));
        for x in 1..size.width {
            out.push(DrawChPos::new(sh_sty.bottom.clone(), x, size.height));
        }
        for y in 1..size.height {
            out.push(DrawChPos::new(sh_sty.right.clone(), size.width, y));
        }

        out
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Bordered {}

// ------------------------------------------------------------------------------------------

/// a small element (one character) that can be used to send resize requests to parent elements
#[derive(Clone)]
pub struct Corner {
    pub pane: Pane,
    pub pos: Rc<RefCell<CornerPos>>,
    pub property: Rc<RefCell<Property>>,
    pub dragging: Rc<RefCell<bool>>,
}

/// corner position
#[derive(Clone, Copy)]
pub enum CornerPos {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    const Z_INDEX: ZIndex = 200;

    pub fn new(ctx: &Context, ch: DrawCh, pos: CornerPos, property: Property) -> Self {
        let pane = Pane::new(ctx, "resize_corner")
            .with_dyn_height(1.into())
            .with_dyn_width(1.into())
            .with_content(ch.into());
        pane.set_z(Self::Z_INDEX);
        Self {
            pane,
            pos: Rc::new(RefCell::new(pos)),
            property: Rc::new(RefCell::new(property)),
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

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Corner {
    fn receive_event_inner(&self, _ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let property = *self.property.borrow();
        if matches!(property, Property::None) {
            return (true, EventResponses::default()); // still capture just don't do anything
        }
        let cur_dragging = *self.dragging.borrow();
        let mut captured = false;
        match ev {
            Event::Mouse(me) => {
                captured = true;
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => *self.dragging.borrow_mut() = true,
                    MouseEventKind::Drag(MouseButton::Left) => {}
                    _ => *self.dragging.borrow_mut() = false,
                }
            }
            Event::ExternalMouse(me) => match me.kind {
                MouseEventKind::Drag(MouseButton::Left) if cur_dragging => {
                    let dx = me.column;
                    let dy = me.row;

                    match property {
                        Property::DragResize => {
                            let (left_dx, right_dx, top_dy, bottom_dy) = match *self.pos.borrow() {
                                CornerPos::TopLeft => (dx, 0, dy, 0),
                                CornerPos::TopRight => (0, dx, dy, 0),
                                CornerPos::BottomLeft => (dx, 0, 0, dy),
                                CornerPos::BottomRight => (0, dx, 0, dy),
                            };
                            let resp = ResizeResponse {
                                left_dx,
                                right_dx,
                                top_dy,
                                bottom_dy,
                            };
                            return (true, EventResponse::Resize(resp).into());
                        }
                        Property::DragMove => {
                            let resp = MoveResponse { dx, dy };
                            return (true, EventResponse::Move(resp).into());
                        }
                        Property::None => {}
                    }
                }
                _ => *self.dragging.borrow_mut() = false,
            },
            _ => {}
        }
        (captured, EventResponses::default())
    }
}

// ------------------------------------------------------------------------------------------

/// a vertical side of the border
#[derive(Clone)]
pub struct VerticalSide {
    pub pane: Pane,
    pub ch: Rc<RefCell<DrawCh>>,
    pub pos: Rc<RefCell<VerticalPos>>,
    pub property: Rc<RefCell<Property>>,
    pub dragging_start_y: Rc<RefCell<Option<i32>>>,
}

/// corner position
#[derive(Clone, Copy)]
pub enum VerticalPos {
    Left,
    Right,
}

impl VerticalSide {
    const Z_INDEX: ZIndex = 200;

    /// The context provided determines the size of this element
    pub fn new(ctx: &Context, ch: DrawCh, pos: VerticalPos, property: Property) -> Self {
        let pane = Pane::new(ctx, "resize_corner")
            .with_dyn_height(1.0.into())
            .with_dyn_width(1.into());
        pane.set_z(Self::Z_INDEX);
        Self {
            pane,
            ch: Rc::new(RefCell::new(ch)),
            pos: Rc::new(RefCell::new(pos)),
            property: Rc::new(RefCell::new(property)),
            dragging_start_y: Rc::new(RefCell::new(None)),
        }
    }

    pub fn at(self, x: DynVal, y: DynVal) -> Self {
        self.pane.set_at(x, y);
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for VerticalSide {
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        DrawChPos::new_repeated_vertical(self.ch.borrow().clone(), 0, 0, ctx.s.height)
    }

    fn receive_event_inner(&self, _ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let property = *self.property.borrow();
        if matches!(property, Property::None) {
            return (true, EventResponses::default()); // still capture just don't do anything
        }
        let dragging_start_y = *self.dragging_start_y.borrow();
        let mut captured = false;
        match ev {
            Event::Mouse(me) => {
                captured = true;
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.dragging_start_y.borrow_mut() = Some(me.row.into())
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        let start_y = dragging_start_y.expect("impossible");
                        let dy = me.row as i32 - start_y as i32;

                        if matches!(property, Property::DragMove) {
                            let resp = MoveResponse { dx: 0, dy };
                            return (true, EventResponse::Move(resp).into());
                        }
                    }
                    _ => *self.dragging_start_y.borrow_mut() = None,
                }
            }
            Event::ExternalMouse(me) => match me.kind {
                MouseEventKind::Drag(MouseButton::Left) if dragging_start_y.is_some() => {
                    let dx = me.column;
                    let dy = me.row;

                    match property {
                        Property::DragResize => {
                            let (left_dx, right_dx) = match *self.pos.borrow() {
                                VerticalPos::Left => (dx, 0),
                                VerticalPos::Right => (0, dx),
                            };
                            let resp = ResizeResponse {
                                left_dx,
                                right_dx,
                                top_dy: 0,
                                bottom_dy: 0,
                            };
                            return (true, EventResponse::Resize(resp).into());
                        }
                        Property::DragMove => {
                            let resp = MoveResponse { dx, dy };
                            return (true, EventResponse::Move(resp).into());
                        }
                        Property::None => {}
                    }
                }
                _ => *self.dragging_start_y.borrow_mut() = None,
            },
            _ => {}
        }
        (captured, EventResponses::default())
    }
}

// ------------------------------------------------------------------------------------------

/// a vertical side of the border
#[derive(Clone)]
pub struct HorizontalSide {
    pub pane: Pane,
    pub ch: Rc<RefCell<DrawCh>>,
    pub pos: Rc<RefCell<HorizontalPos>>,
    pub property: Rc<RefCell<Property>>,
    pub dragging_start_x: Rc<RefCell<Option<i32>>>,
}

/// corner position
#[derive(Clone, Copy)]
pub enum HorizontalPos {
    Top,
    Bottom,
}

impl HorizontalSide {
    const Z_INDEX: ZIndex = 200;

    /// The context provided determines the size of this element
    pub fn new(ctx: &Context, ch: DrawCh, pos: HorizontalPos, property: Property) -> Self {
        let pane = Pane::new(ctx, "resize_corner")
            .with_dyn_height(1.0.into())
            .with_dyn_width(1.into());
        pane.set_z(Self::Z_INDEX);
        Self {
            pane,
            ch: Rc::new(RefCell::new(ch)),
            pos: Rc::new(RefCell::new(pos)),
            property: Rc::new(RefCell::new(property)),
            dragging_start_x: Rc::new(RefCell::new(None)),
        }
    }

    pub fn at(self, x: DynVal, y: DynVal) -> Self {
        self.pane.set_at(x, y);
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for HorizontalSide {
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        DrawChPos::new_repeated_vertical(self.ch.borrow().clone(), 0, 0, ctx.s.height)
    }

    fn receive_event_inner(&self, _ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let property = *self.property.borrow();
        if matches!(property, Property::None) {
            return (true, EventResponses::default()); // still capture just don't do anything
        }
        let dragging_start_x = *self.dragging_start_x.borrow();
        let mut captured = false;
        match ev {
            Event::Mouse(me) => {
                captured = true;
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.dragging_start_x.borrow_mut() = Some(me.column.into())
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        let start_x = dragging_start_x.expect("impossible");
                        let dx = me.column as i32 - start_x as i32;

                        if matches!(property, Property::DragMove) {
                            let resp = MoveResponse { dx, dy: 0 };
                            return (true, EventResponse::Move(resp).into());
                        }
                    }
                    _ => *self.dragging_start_x.borrow_mut() = None,
                }
            }
            Event::ExternalMouse(me) => match me.kind {
                MouseEventKind::Drag(MouseButton::Left) if dragging_start_x.is_some() => {
                    let dx = me.column;
                    let dy = me.row;

                    match property {
                        Property::DragResize => {
                            let (top_dy, bottom_dy) = match *self.pos.borrow() {
                                HorizontalPos::Top => (dy, 0),
                                HorizontalPos::Bottom => (0, dy),
                            };
                            let resp = ResizeResponse {
                                left_dx: 0,
                                right_dx: 0,
                                top_dy,
                                bottom_dy,
                            };
                            return (true, EventResponse::Resize(resp).into());
                        }
                        Property::DragMove => {
                            let resp = MoveResponse { dx, dy };
                            return (true, EventResponse::Move(resp).into());
                        }
                        Property::None => {}
                    }
                }
                _ => *self.dragging_start_x.borrow_mut() = None,
            },
            _ => {}
        }
        (captured, EventResponses::default())
    }
}
