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
    pub last_size: Rc<RefCell<Size>>, // needed for knowing when to resize scrollbars
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
    pub left: Option<Property>, // if None, then no left border
    //pub right: Option<PropertyVSB>,  // if None, then no right border
    pub right: Option<Property>, // if None, then no right border
    pub top: Option<Property>,   // if None, then no top border
    //pub bottom: Option<PropertyHSB>, // if None, then no bottom border
    pub bottom: Option<Property>, // if None, then no bottom border
    pub top_corner: Property,
    pub bottom_corner: Property,
    pub left_corner: Property,
    pub right_corner: Property,
}

impl BorderProperies {
    pub fn new_basic() -> Self {
        Self {
            left: Some(Property::None),
            right: Some(Property::None),
            top: Some(Property::None),
            bottom: Some(Property::None),
            top_corner: Property::None,
            bottom_corner: Property::None,
            left_corner: Property::None,
            right_corner: Property::None,
        }
    }

    pub fn new_resizer() -> Self {
        Self {
            left: Some(Property::DragResize),
            right: Some(Property::DragResize),
            top: Some(Property::DragResize),
            bottom: Some(Property::DragResize),
            top_corner: Property::DragResize,
            bottom_corner: Property::DragResize,
            left_corner: Property::DragResize,
            right_corner: Property::DragResize,
        }
    }

    pub fn new_mover() -> Self {
        Self {
            left: Some(Property::DragMove),
            right: Some(Property::DragMove),
            top: Some(Property::DragMove),
            bottom: Some(Property::DragMove),
            top_corner: Property::DragMove,
            bottom_corner: Property::DragMove,
            left_corner: Property::DragMove,
            right_corner: Property::DragMove,
        }
    }
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
    pub const KIND: &'static str = "bordered";

    pub fn new_basic(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_basic();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_resizer(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_resizer();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_mover(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_mover();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new(
        ctx: &Context, inner: Box<dyn Element>, chs: BorderSty, properties: BorderProperies,
    ) -> Self {
        let pane = ParentPane::new(ctx, Self::KIND).with_transparent();

        let has_top_left_corner = properties.top.is_some() && properties.left.is_some();
        let has_top_right_corner = properties.top.is_some() && properties.right.is_some();
        let has_bottom_left_corner = properties.bottom.is_some() && properties.left.is_some();
        let has_bottom_right_corner = properties.bottom.is_some() && properties.right.is_some();

        let inner_start_x: DynVal = if properties.left.is_some() { 1.into() } else { 0.into() };
        let inner_start_y: DynVal = if properties.top.is_some() { 1.into() } else { 0.into() };
        let inner_end_x = if properties.right.is_some() {
            DynVal::new_full().minus(1.into())
        } else {
            DynVal::new_full()
        };
        let inner_end_y = if properties.bottom.is_some() {
            DynVal::new_full().minus(1.into())
        } else {
            DynVal::new_full()
        };
        let inner_loc = DynLocation::new(
            inner_start_x.clone(),
            inner_end_x,
            inner_start_y.clone(),
            inner_end_y,
        );
        inner.get_dyn_location_set().borrow_mut().l = inner_loc.clone();
        pane.add_element(inner);

        if has_top_left_corner {
            let corner = Corner::new(
                ctx,
                chs.top_left.clone(),
                CornerPos::TopLeft,
                properties.top_corner,
            )
            .at(0.into(), 0.into());
            pane.add_element(Box::new(corner));
        }
        if has_top_right_corner {
            let corner = Corner::new(
                ctx,
                chs.top_right.clone(),
                CornerPos::TopRight,
                properties.top_corner,
            )
            .at(DynVal::new_full().minus(1.into()), 0.into());
            pane.add_element(Box::new(corner));
        }
        if has_bottom_left_corner {
            let corner = Corner::new(
                ctx,
                chs.bottom_left.clone(),
                CornerPos::BottomLeft,
                properties.bottom_corner,
            )
            .at(0.into(), DynVal::new_full().minus(1.into()));
            pane.add_element(Box::new(corner));
        }
        if has_bottom_right_corner {
            let corner = Corner::new(
                ctx,
                chs.bottom_right.clone(),
                CornerPos::BottomRight,
                properties.bottom_corner,
            )
            .at(
                DynVal::new_full().minus(1.into()),
                DynVal::new_full().minus(1.into()),
            );
            pane.add_element(Box::new(corner));
        }

        if let Some(left_property) = properties.left {
            let start_y: DynVal = if has_top_left_corner { 1.into() } else { 0.into() };
            let end_y = if has_bottom_left_corner {
                DynVal::new_full().minus(1.into())
            } else {
                DynVal::new_full()
            };
            let left_loc = DynLocation::new(0.into(), 1.into(), start_y.clone(), end_y);
            let side = VerticalSide::new(ctx, chs.left.clone(), VerticalPos::Left, left_property);
            side.pane.get_dyn_location_set().borrow_mut().l = left_loc;
            pane.add_element(Box::new(side));
        }

        if let Some(right_property) = properties.right {
            let start_y: DynVal = if has_top_right_corner { 1.into() } else { 0.into() };
            let end_y = if has_bottom_right_corner {
                DynVal::new_full().minus(1.into())
            } else {
                DynVal::new_full()
            };
            let right_loc = DynLocation::new(
                DynVal::new_full().minus(1.into()),
                DynVal::new_full(),
                start_y.clone(),
                end_y,
            );
            let side =
                VerticalSide::new(ctx, chs.right.clone(), VerticalPos::Right, right_property);
            side.pane.get_dyn_location_set().borrow_mut().l = right_loc;
            pane.add_element(Box::new(side));
        }

        if let Some(top_property) = properties.top {
            let start_x: DynVal = if has_top_left_corner { 1.into() } else { 0.into() };
            let end_x = if has_top_right_corner {
                DynVal::new_full().minus(1.into())
            } else {
                DynVal::new_full()
            };
            let top_loc = DynLocation::new(start_x, end_x, 0.into(), 1.into());
            let side = HorizontalSide::new(ctx, chs.top.clone(), HorizontalPos::Top, top_property);
            side.pane.get_dyn_location_set().borrow_mut().l = top_loc;
            pane.add_element(Box::new(side));
        }

        if let Some(bottom_property) = properties.bottom {
            let start_x: DynVal = if has_bottom_left_corner { 1.into() } else { 0.into() };
            let end_x = if has_bottom_right_corner {
                DynVal::new_full().minus(1.into())
            } else {
                DynVal::new_full()
            };
            let bottom_loc = DynLocation::new(
                start_x,
                end_x,
                DynVal::new_full().minus(1.into()),
                DynVal::new_full(),
            );
            let side = HorizontalSide::new(
                ctx,
                chs.bottom.clone(),
                HorizontalPos::Bottom,
                bottom_property,
            );
            side.pane.get_dyn_location_set().borrow_mut().l = bottom_loc;
            pane.add_element(Box::new(side));
        }

        Self {
            pane,
            chs: Rc::new(RefCell::new(chs)),
            last_size: Rc::new(RefCell::new(ctx.s)),
            properties: Rc::new(RefCell::new(properties)),
        }
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
    pub dragging_start_pos: Rc<RefCell<Option<(i32, i32)>>>, // x, y
}

/// corner position
#[derive(Clone, Copy)]
pub enum VerticalPos {
    Left,
    Right,
}

impl VerticalSide {
    const Z_INDEX: ZIndex = 200;

    pub fn new(ctx: &Context, ch: DrawCh, pos: VerticalPos, property: Property) -> Self {
        let pane = Pane::new(ctx, "resize_corner")
            .with_dyn_height(DynVal::new_full())
            .with_dyn_width(1.into());
        pane.set_z(Self::Z_INDEX);
        Self {
            pane,
            ch: Rc::new(RefCell::new(ch)),
            pos: Rc::new(RefCell::new(pos)),
            property: Rc::new(RefCell::new(property)),
            dragging_start_pos: Rc::new(RefCell::new(None)),
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
        let dragging_start_pos = *self.dragging_start_pos.borrow();
        let mut captured = false;
        match ev {
            Event::Mouse(me) => {
                captured = true;
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.dragging_start_pos.borrow_mut() =
                            Some((me.column.into(), me.row.into()))
                    }
                    MouseEventKind::Drag(MouseButton::Left) if dragging_start_pos.is_some() => {
                        let (_, start_y) = dragging_start_pos.expect("impossible");
                        let dy = me.row as i32 - start_y;

                        if matches!(property, Property::DragMove) {
                            let resp = MoveResponse { dx: 0, dy };
                            return (true, EventResponse::Move(resp).into());
                        }
                    }
                    _ => *self.dragging_start_pos.borrow_mut() = None,
                }
            }
            Event::ExternalMouse(me) => match me.kind {
                MouseEventKind::Drag(MouseButton::Left) if dragging_start_pos.is_some() => {
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
                            let (start_x, start_y) = dragging_start_pos.expect("impossible");
                            let dx = dx - start_x;
                            let dy = dy - start_y;
                            let resp = MoveResponse { dx, dy };
                            return (true, EventResponse::Move(resp).into());
                        }
                        Property::None => {}
                    }
                }
                _ => *self.dragging_start_pos.borrow_mut() = None,
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
    pub dragging_start_pos: Rc<RefCell<Option<(i32, i32)>>>, // x, y
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
            .with_dyn_height(1.into())
            .with_dyn_width(1.0.into());
        pane.set_z(Self::Z_INDEX);
        Self {
            pane,
            ch: Rc::new(RefCell::new(ch)),
            pos: Rc::new(RefCell::new(pos)),
            property: Rc::new(RefCell::new(property)),
            dragging_start_pos: Rc::new(RefCell::new(None)),
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
        DrawChPos::new_repeated_horizontal(self.ch.borrow().clone(), 0, 0, ctx.s.width)
    }

    fn receive_event_inner(&self, _ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let property = *self.property.borrow();
        if matches!(property, Property::None) {
            return (true, EventResponses::default()); // still capture just don't do anything
        }
        let dragging_start_pos = *self.dragging_start_pos.borrow();
        let mut captured = false;
        match ev {
            Event::Mouse(me) => {
                captured = true;
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.dragging_start_pos.borrow_mut() =
                            Some((me.column.into(), me.row.into()))
                    }
                    MouseEventKind::Drag(MouseButton::Left) if dragging_start_pos.is_some() => {
                        let (start_x, _) = dragging_start_pos.expect("impossible");
                        let dx = me.column as i32 - start_x;

                        if matches!(property, Property::DragMove) {
                            let resp = MoveResponse { dx, dy: 0 };
                            return (true, EventResponse::Move(resp).into());
                        }
                    }
                    _ => *self.dragging_start_pos.borrow_mut() = None,
                }
            }
            Event::ExternalMouse(me) => match me.kind {
                MouseEventKind::Drag(MouseButton::Left) if dragging_start_pos.is_some() => {
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
                            let (start_x, start_y) = dragging_start_pos.expect("impossible");
                            let dx = dx - start_x;
                            let dy = dy - start_y;
                            let resp = MoveResponse { dx, dy };
                            return (true, EventResponse::Move(resp).into());
                        }
                        Property::None => {}
                    }
                }
                _ => *self.dragging_start_pos.borrow_mut() = None,
            },
            _ => {}
        }
        (captured, EventResponses::default())
    }
}
