use {
    crate::*,
    crossterm::event::{MouseButton, MouseEventKind},
};

/// displays the size
#[derive(Clone)]
pub struct Bordered {
    pub pane: ParentPane,
    pub inner: Rc<RefCell<Box<dyn Element>>>,

    /// left border element
    pub left: Rc<RefCell<Option<VerticalSide>>>,
    /// right border element
    pub right: Rc<RefCell<Option<VerticalSide>>>,
    /// top border element
    pub top: Rc<RefCell<Option<HorizontalSide>>>,
    /// bottom border element
    pub bottom: Rc<RefCell<Option<HorizontalSide>>>,

    pub last_size: Rc<RefCell<Size>>,
    /// needed for knowing when to resize scrollbars
    pub x_scrollbar: Rc<RefCell<Option<HorizontalScrollbar>>>,
    pub y_scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,

    /// how much less the x scrollbar is from the full width
    pub x_scrollbar_sub_from_full: Rc<RefCell<usize>>,

    /// how much less the y scrollbar is from the full height
    pub y_scrollbar_sub_from_full: Rc<RefCell<usize>>,

    /// how much less the x scrollbar view width is from the full width
    pub x_scrollbar_view_sub_from_full: Rc<RefCell<usize>>,

    /// how much less the y scrollbar view width is from the full width
    pub y_scrollbar_view_sub_from_full: Rc<RefCell<usize>>,
}

/// property for the border
#[derive(Clone, Copy)]
pub enum PropertyCnr {
    /// just for display
    None,
    /// drag to resize
    DragResize,
    /// drag to move
    DragMove,
}

// XXX TODO remove once accordian stack implemented
/*
// Collapsing triangle in the vertical direction
#[derive(Clone)]
pub struct CollapserCnr {
    is_collapsed: bool,
    /// the size of the collapsed dimension when non-collapsed
    non_collapse_val: DynVal,
    vertical: bool, // collapse in the vertical direction
    open_ch: char,
    collapsed_ch: char,
}

/// Collapsing triangle in the vertical direction
#[derive(Clone, Copy)]
pub struct CollapserSide {
    is_collapsed: bool,
    vertical: bool, // collapse in the vertical direction
    open_ch: char,
    collapsed_ch: char,
}

impl CollapserCnr {
    pub fn new(
        vertical: bool, non_collapse_val: DynVal, open_ch: char, collapsed_ch: char,
    ) -> Self {
        Self {
            is_collapsed: false,
            non_collapse_val,
            vertical,
            open_ch,
            collapsed_ch,
        }
    }

    pub fn new_left_horizontal() -> Self {
        Self {
            is_collapsed: false,
            non_collapse_val: 0.25.into(),
            vertical: false,
            open_ch: '▼',
            collapsed_ch: '▶',
        }
    }

    pub fn new_right_horizontal() -> Self {
        Self {
            is_collapsed: false,
            non_collapse_val: 0.25.into(),
            vertical: false,
            open_ch: '▼',
            collapsed_ch: '◀',
        }
    }

    pub fn new_right_vertical() -> Self {
        Self {
            is_collapsed: false,
            non_collapse_val: 0.25.into(),
            vertical: true,
            open_ch: '▶',
            collapsed_ch: '▼',
        }
    }

    pub fn new_left_vertical() -> Self {
        Self {
            is_collapsed: false,
            non_collapse_val: 0.25.into(),
            vertical: true,
            open_ch: '◀',
            collapsed_ch: '▼',
        }
    }

    pub fn toggle_collapser(&mut self) {
        self.is_collapsed = !self.is_collapsed;
    }

    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.is_collapsed = collapsed;
    }

    pub fn get_ch(&self) -> char {
        if self.is_collapsed {
            self.collapsed_ch
        } else {
            self.open_ch
        }
    }
}
*/

/// property for the border
/// including vertical scrollbar option
#[derive(Clone)]
pub enum PropertyVrt {
    /// just for display
    None,
    /// drag to resize
    DragResize,
    /// drag to move
    DragMove,
    Scrollbar(VerticalScrollbar),
}

/// property for the border
/// including horizontal scrollbar option
#[derive(Clone)]
pub enum PropertyHzt {
    /// just for display
    None,
    /// drag to resize
    DragResize,
    /// drag to move
    DragMove,
    Scrollbar(HorizontalScrollbar),
}

#[derive(Clone)]
pub struct BorderProperies {
    pub left: Option<PropertyVrt>,
    /// if None, then no left border
    pub right: Option<PropertyVrt>,
    /// if None, then no right border
    pub top: Option<PropertyHzt>,
    /// if None, then no top border
    pub bottom: Option<PropertyHzt>,
    /// if None, then no bottom border
    pub top_corner: PropertyCnr,
    pub bottom_corner: PropertyCnr,
    pub left_corner: PropertyCnr,
    pub right_corner: PropertyCnr,
}

impl BorderProperies {
    pub fn new_basic() -> Self {
        Self {
            left: Some(PropertyVrt::None),
            right: Some(PropertyVrt::None),
            top: Some(PropertyHzt::None),
            bottom: Some(PropertyHzt::None),
            top_corner: PropertyCnr::None,
            bottom_corner: PropertyCnr::None,
            left_corner: PropertyCnr::None,
            right_corner: PropertyCnr::None,
        }
    }

    pub fn new_basic_with_scrollbars(ctx: &Context, sty: Style) -> Self {
        Self {
            left: Some(PropertyVrt::None),
            right: Some(PropertyVrt::Scrollbar(
                VerticalScrollbar::new(ctx, 0.into(), Size::default(), 0)
                    .with_scrollbar_sty(ScrollbarSty::vertical_for_thin_box(sty.clone())),
            )),
            top: Some(PropertyHzt::None),
            bottom: Some(PropertyHzt::Scrollbar(
                HorizontalScrollbar::new(ctx, 0.into(), Size::default(), 0)
                    .with_scrollbar_sty(ScrollbarSty::horizontal_for_thin_box(sty)),
            )),
            top_corner: PropertyCnr::None,
            bottom_corner: PropertyCnr::None,
            left_corner: PropertyCnr::None,
            right_corner: PropertyCnr::None,
        }
    }

    pub fn new_borderless_with_scrollbars(ctx: &Context) -> Self {
        Self {
            left: None,
            right: Some(PropertyVrt::Scrollbar(VerticalScrollbar::new(
                ctx,
                0.into(),
                Size::default(),
                0,
            ))),
            top: None,
            bottom: Some(PropertyHzt::Scrollbar(HorizontalScrollbar::new(
                ctx,
                0.into(),
                Size::default(),
                0,
            ))),
            top_corner: PropertyCnr::None,
            bottom_corner: PropertyCnr::None,
            left_corner: PropertyCnr::None,
            right_corner: PropertyCnr::None,
        }
    }

    pub fn new_resizer() -> Self {
        Self {
            left: Some(PropertyVrt::DragResize),
            right: Some(PropertyVrt::DragResize),
            top: Some(PropertyHzt::DragResize),
            bottom: Some(PropertyHzt::DragResize),
            top_corner: PropertyCnr::DragResize,
            bottom_corner: PropertyCnr::DragResize,
            left_corner: PropertyCnr::DragResize,
            right_corner: PropertyCnr::DragResize,
        }
    }

    pub fn new_left_resizer() -> Self {
        Self {
            left: Some(PropertyVrt::DragResize),
            right: None,
            top: None,
            bottom: None,
            top_corner: PropertyCnr::DragResize,
            bottom_corner: PropertyCnr::DragResize,
            left_corner: PropertyCnr::DragResize,
            right_corner: PropertyCnr::DragResize,
        }
    }

    pub fn new_right_resizer() -> Self {
        Self {
            left: None,
            right: Some(PropertyVrt::DragResize),
            top: None,
            bottom: None,
            top_corner: PropertyCnr::DragResize,
            bottom_corner: PropertyCnr::DragResize,
            left_corner: PropertyCnr::DragResize,
            right_corner: PropertyCnr::DragResize,
        }
    }

    pub fn new_top_resizer() -> Self {
        Self {
            left: None,
            right: None,
            top: Some(PropertyHzt::DragResize),
            bottom: None,
            top_corner: PropertyCnr::DragResize,
            bottom_corner: PropertyCnr::DragResize,
            left_corner: PropertyCnr::DragResize,
            right_corner: PropertyCnr::DragResize,
        }
    }

    pub fn new_bottom_resizer() -> Self {
        Self {
            left: None,
            right: None,
            top: None,
            bottom: Some(PropertyHzt::DragResize),
            top_corner: PropertyCnr::DragResize,
            bottom_corner: PropertyCnr::DragResize,
            left_corner: PropertyCnr::DragResize,
            right_corner: PropertyCnr::DragResize,
        }
    }

    pub fn new_resizer_with_scrollbars(ctx: &Context, sty: Style) -> Self {
        Self {
            left: Some(PropertyVrt::DragResize),
            right: Some(PropertyVrt::Scrollbar(
                VerticalScrollbar::new(ctx, 0.into(), Size::default(), 0)
                    .with_scrollbar_sty(ScrollbarSty::vertical_for_thin_box(sty.clone())),
            )),
            top: Some(PropertyHzt::DragResize),
            bottom: Some(PropertyHzt::Scrollbar(
                HorizontalScrollbar::new(ctx, 0.into(), Size::default(), 0)
                    .with_scrollbar_sty(ScrollbarSty::horizontal_for_thin_box(sty)),
            )),
            top_corner: PropertyCnr::DragResize,
            bottom_corner: PropertyCnr::DragResize,
            left_corner: PropertyCnr::DragResize,
            right_corner: PropertyCnr::DragResize,
        }
    }

    pub fn new_mover() -> Self {
        Self {
            left: Some(PropertyVrt::DragMove),
            right: Some(PropertyVrt::DragMove),
            top: Some(PropertyHzt::DragMove),
            bottom: Some(PropertyHzt::DragMove),
            top_corner: PropertyCnr::DragMove,
            bottom_corner: PropertyCnr::DragMove,
            left_corner: PropertyCnr::DragMove,
            right_corner: PropertyCnr::DragMove,
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
    pub fn new_borderless(sty: Style) -> Self {
        Self {
            left: DrawCh::new(' ', sty.clone()),
            right: DrawCh::new(' ', sty.clone()),
            top: DrawCh::new(' ', sty.clone()),
            bottom: DrawCh::new(' ', sty.clone()),
            bottom_left: DrawCh::new(' ', sty.clone()),
            bottom_right: DrawCh::new(' ', sty.clone()),
            top_left: DrawCh::new(' ', sty.clone()),
            top_right: DrawCh::new(' ', sty),
        }
    }

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

    /// ```text
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

    /// ```text
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

    /// ```text
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

    /// ```text
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

    /// ```text
    /// ▗▄▄▄▄▄▄▖
    /// ▐      ▌
    /// ▐      ▌
    /// ▝▀▀▀▀▀▀▘
    /// ```
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

    /// ```text
    /// ▛▀▀▀▀▀▀▜
    /// ▌      ▐
    /// ▌      ▐
    /// ▙▄▄▄▄▄▄▟
    /// ```
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

    /// NOTE this uses unicode characters that may not supported by all terminals
    /// (Symbols for Legacy Computing)
    /// ```text
    /// 🭽▔▔▔▔▔▔🭾
    /// ▏      ▕
    /// ▏      ▕
    /// 🭼▁▁▁▁▁▁🭿
    /// ```
    pub fn new_large_eighth_block(sty: Style) -> Self {
        Self {
            left: DrawCh::new('▏', sty.clone()),
            right: DrawCh::new('▕', sty.clone()),
            top: DrawCh::new('▔', sty.clone()),
            bottom: DrawCh::new('▁', sty.clone()),
            top_left: DrawCh::new('🭽', sty.clone()),
            top_right: DrawCh::new('🭾', sty.clone()),
            bottom_left: DrawCh::new('🭼', sty.clone()),
            bottom_right: DrawCh::new('🭿', sty),
        }
    }

    /// ```text
    ///  ▁▁▁▁▁▁▁▁   
    /// ▕hello   ▏
    /// ▕whats up▏
    ///  ▔▔▔▔▔▔▔▔
    /// ```
    pub fn new_tight_eighth_block(sty: Style) -> Self {
        Self {
            left: DrawCh::new('▕', sty.clone()),
            right: DrawCh::new('▏', sty.clone()),
            top: DrawCh::new('▁', sty.clone()),
            bottom: DrawCh::new('▔', sty.clone()),
            top_left: DrawCh::new(' ', sty.clone()),
            top_right: DrawCh::new(' ', sty.clone()),
            bottom_left: DrawCh::new(' ', sty.clone()),
            bottom_right: DrawCh::new(' ', sty),
        }
    }

    /// a border composed of all the same character
    /// some recommendations: █ ▓ ▒ ░
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

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl Bordered {
    pub const KIND: &'static str = "bordered";

    pub fn new_basic(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_basic();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_large(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_large_eighth_block(sty);
        let properties = BorderProperies::new_basic();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_tight(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_tight_eighth_block(sty);
        let properties = BorderProperies::new_basic();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_double(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_double(sty);
        let properties = BorderProperies::new_basic();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_basic_with_scrollbars(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty.clone());
        let properties = BorderProperies::new_basic_with_scrollbars(ctx, sty);
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_borderless_with_scrollbars(
        ctx: &Context, inner: Box<dyn Element>, sty: Style,
    ) -> Self {
        let properties = BorderProperies::new_borderless_with_scrollbars(ctx);
        let chs = BorderSty::new_borderless(sty);
        Self::new(ctx, inner, chs, properties)
    }

    /// a special case there there is no bottom left corner element
    /// that position is taken by the scrollbar
    pub fn new_borderless_with_scrollbars_and_thin_left(
        ctx: &Context, inner: Box<dyn Element>, sty: Style,
    ) -> Self {
        let properties = BorderProperies::new_borderless_with_scrollbars(ctx);
        let chs = BorderSty::new_borderless(sty.clone());
        let bordered = Self::new(ctx, inner, chs, properties);

        let start_y: DynVal = 0.into();
        let end_y = DynVal::FULL.minus(1.into());
        let left_loc = DynLocation::new(0.into(), 1.into(), start_y.clone(), end_y);

        let chs_left = DrawCh::new('▏', sty);
        let side = VerticalSide::new(ctx, chs_left.clone(), VerticalPos::Left, PropertyVrt::None);
        side.pane.set_dyn_location(left_loc);
        bordered.pane.add_element(Box::new(side.clone()));
        bordered.left.borrow_mut().replace(side);

        let mut l = bordered.inner.borrow().get_dyn_location_set().clone();
        l.l.set_start_x(1.into());
        //l.l.set_end_x(DynVal::FULL.minus(2));
        bordered.inner.borrow_mut().set_dyn_location_set(l);

        // shrink the view portal as understood by the x scrollbar
        *bordered.x_scrollbar_view_sub_from_full.borrow_mut() += 1;
        bordered.ensure_scrollbar_size(ctx);

        bordered
    }

    pub fn new_resizer(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_resizer();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_left_resizer(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_left_resizer();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_right_resizer(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_right_resizer();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_top_resizer(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_top_resizer();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_bottom_resizer(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_bottom_resizer();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_resizer_with_scrollbars(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty.clone());
        let properties = BorderProperies::new_resizer_with_scrollbars(ctx, sty);
        Self::new(ctx, inner, chs, properties)
    }

    pub fn new_mover(ctx: &Context, inner: Box<dyn Element>, sty: Style) -> Self {
        let chs = BorderSty::new_thick_single(sty);
        let properties = BorderProperies::new_mover();
        Self::new(ctx, inner, chs, properties)
    }

    pub fn with_title<S: Into<String>>(self, title: S) -> Self {
        if let Some(ref top) = *self.top.borrow() {
            (*top).set_left_text(title);
        }
        self
    }

    pub fn set_title<S: Into<String>>(&self, title: S) {
        if let Some(ref top) = *self.top.borrow() {
            (*top).set_left_text(title);
        }
    }

    pub fn with_title_right<S: Into<String>>(self, title: S) -> Self {
        if let Some(ref top) = *self.top.borrow() {
            (*top).set_right_text(title);
        }
        self
    }

    pub fn set_title_right<S: Into<String>>(&self, title: S) {
        if let Some(ref top) = *self.top.borrow() {
            (*top).set_right_text(title);
        }
    }

    pub fn with_title_center<S: Into<String>>(self, title: S) -> Self {
        if let Some(ref top) = *self.top.borrow() {
            (*top).set_center_text(title);
        }
        self
    }

    pub fn set_title_center<S: Into<String>>(&self, title: S) {
        if let Some(ref top) = *self.top.borrow() {
            (*top).set_center_text(title);
        }
    }

    pub fn with_bottom_left_text<S: Into<String>>(self, text: S) -> Self {
        if let Some(ref bottom) = *self.bottom.borrow() {
            (*bottom).set_left_text(text);
        }
        self
    }

    pub fn set_bottom_left_text<S: Into<String>>(&self, text: S) {
        if let Some(ref bottom) = *self.bottom.borrow() {
            (*bottom).set_left_text(text);
        }
    }

    pub fn with_bottom_right_text<S: Into<String>>(self, text: S) -> Self {
        if let Some(ref bottom) = *self.bottom.borrow() {
            (*bottom).set_right_text(text);
        }
        self
    }

    pub fn set_bottom_right_text<S: Into<String>>(&self, text: S) {
        if let Some(ref bottom) = *self.bottom.borrow() {
            (*bottom).set_right_text(text);
        }
    }

    pub fn with_bottom_center_text<S: Into<String>>(self, text: S) -> Self {
        if let Some(ref bottom) = *self.bottom.borrow() {
            (*bottom).set_center_text(text);
        }
        self
    }

    pub fn set_bottom_center_text<S: Into<String>>(&self, text: S) {
        if let Some(ref bottom) = *self.bottom.borrow() {
            (*bottom).set_center_text(text);
        }
    }

    pub fn with_left_top_text<S: Into<String>>(self, text: S) -> Self {
        if let Some(ref left) = *self.left.borrow() {
            (*left).set_top_text(text);
        }
        self
    }

    pub fn set_left_top_text<S: Into<String>>(&self, text: S) {
        if let Some(ref left) = *self.left.borrow() {
            (*left).set_top_text(text);
        }
    }

    pub fn with_left_bottom_text<S: Into<String>>(self, text: S) -> Self {
        if let Some(ref left) = *self.left.borrow() {
            (*left).set_bottom_text(text);
        }
        self
    }

    pub fn set_left_bottom_text<S: Into<String>>(&self, text: S) {
        if let Some(ref left) = *self.left.borrow() {
            (*left).set_bottom_text(text);
        }
    }

    pub fn with_left_center_text<S: Into<String>>(self, text: S) -> Self {
        if let Some(ref left) = *self.left.borrow() {
            (*left).set_center_text(text);
        }
        self
    }

    pub fn set_left_center_text<S: Into<String>>(&self, text: S) {
        if let Some(ref left) = *self.left.borrow() {
            (*left).set_center_text(text);
        }
    }

    pub fn with_right_top_text<S: Into<String>>(self, text: S) -> Self {
        if let Some(ref right) = *self.right.borrow() {
            (*right).set_top_text(text);
        }
        self
    }

    pub fn set_right_top_text<S: Into<String>>(&self, text: S) {
        if let Some(ref right) = *self.right.borrow() {
            (*right).set_top_text(text);
        }
    }

    pub fn with_right_bottom_text<S: Into<String>>(self, text: S) -> Self {
        if let Some(ref right) = *self.right.borrow() {
            (*right).set_bottom_text(text);
        }
        self
    }

    pub fn set_right_bottom_text<S: Into<String>>(&self, text: S) {
        if let Some(ref right) = *self.right.borrow() {
            (*right).set_bottom_text(text);
        }
    }

    pub fn with_right_center_text<S: Into<String>>(self, text: S) -> Self {
        if let Some(ref right) = *self.right.borrow() {
            (*right).set_center_text(text);
        }
        self
    }

    pub fn set_right_center_text<S: Into<String>>(&self, text: S) {
        if let Some(ref right) = *self.right.borrow() {
            (*right).set_center_text(text);
        }
    }

    pub fn at(self, x: DynVal, y: DynVal) -> Self {
        self.set_at(x, y);
        self
    }

    /// TODO I think the method described in this note is nolonger required after context system
    /// refactor. Refactor out if possible.
    ///
    /// NOTE SB-1
    /// THERE is a strange issue with the scrollbars here, using the HorizontalScrollbar as an
    /// example:
    ///  - consider the example were both horizontal and vertical scrollbars are present:
    ///     - VerticalSBPositions is ToTheRight
    ///     - HorizontalSBPositions is Below
    ///  - we want the width of the horizontal scrollbar to be the width of the inner_pane
    ///    aka. flex(1.0)-fixed(1).
    ///  - however internally the width kind of needs to be flex(1.0) as when it comes time
    ///    to draw the scrollbar the context is calculated given its provided dimensions
    ///    and thus the drawing would apply the width of flex(1.0)-fixed(1) to the context which has
    ///    already had the fixed(1) subtracted from it, thus resulting in two subtractions.
    ///  - the solution is to have the width as a fixed size, and adjust it with each resize
    ///     - this is done with the ensure_scrollbar_size function.
    pub fn new(
        ctx: &Context, inner: Box<dyn Element>, chs: BorderSty, mut properties: BorderProperies,
    ) -> Self {
        // deref chs
        let BorderSty {
            left: chs_left,
            right: chs_right,
            top: chs_top,
            bottom: chs_bottom,
            top_left: chs_top_left,
            top_right: chs_top_right,
            bottom_left: chs_bottom_left,
            bottom_right: chs_bottom_right,
        } = chs;

        let pane = ParentPane::new(ctx, Self::KIND).with_transparent();
        let bordered = Self {
            pane,
            inner: Rc::new(RefCell::new(inner.clone())),

            left: Rc::new(RefCell::new(None)),
            right: Rc::new(RefCell::new(None)),
            top: Rc::new(RefCell::new(None)),
            bottom: Rc::new(RefCell::new(None)),

            last_size: Rc::new(RefCell::new(ctx.size)),
            x_scrollbar: Rc::new(RefCell::new(None)),
            y_scrollbar: Rc::new(RefCell::new(None)),
            x_scrollbar_sub_from_full: Rc::new(RefCell::new(0)),
            y_scrollbar_sub_from_full: Rc::new(RefCell::new(0)),
            x_scrollbar_view_sub_from_full: Rc::new(RefCell::new(0)),
            y_scrollbar_view_sub_from_full: Rc::new(RefCell::new(0)),
        };

        let has_top_left_corner = properties.top.is_some() && properties.left.is_some();
        let has_top_right_corner = properties.top.is_some() && properties.right.is_some();
        let has_bottom_left_corner = properties.bottom.is_some() && properties.left.is_some();
        let has_bottom_right_corner = properties.bottom.is_some() && properties.right.is_some();

        let inner_start_x: DynVal = if properties.left.is_some() { 1.into() } else { 0.into() };
        let inner_start_y: DynVal = if properties.top.is_some() { 1.into() } else { 0.into() };
        let inner_end_x =
            if properties.right.is_some() { DynVal::FULL.minus(1.into()) } else { DynVal::FULL };
        let inner_end_y =
            if properties.bottom.is_some() { DynVal::FULL.minus(1.into()) } else { DynVal::FULL };
        let inner_loc = DynLocation::new(
            inner_start_x.clone(),
            inner_end_x,
            inner_start_y.clone(),
            inner_end_y,
        );
        inner.set_dyn_location(inner_loc.clone());

        if has_top_left_corner {
            let corner = Corner::new(
                ctx,
                chs_top_left.clone(),
                CornerPos::TopLeft,
                properties.top_corner,
            )
            .at(0, 0);
            bordered.pane.add_element(Box::new(corner));
        }
        if has_top_right_corner {
            let corner = Corner::new(
                ctx,
                chs_top_right.clone(),
                CornerPos::TopRight,
                properties.top_corner,
            )
            .at(DynVal::FULL.minus(1.into()), 0);
            bordered.pane.add_element(Box::new(corner));
        }
        if has_bottom_left_corner {
            let corner = Corner::new(
                ctx,
                chs_bottom_left.clone(),
                CornerPos::BottomLeft,
                properties.bottom_corner,
            )
            .at(0, DynVal::FULL.minus(1.into()));
            bordered.pane.add_element(Box::new(corner));
        }
        if has_bottom_right_corner {
            let corner = Corner::new(
                ctx,
                chs_bottom_right.clone(),
                CornerPos::BottomRight,
                properties.bottom_corner,
            )
            .at(DynVal::FULL.minus(1.into()), DynVal::FULL.minus(1.into()));
            bordered.pane.add_element(Box::new(corner));
        }

        if let Some(left_property) = properties.left.take() {
            let mut y_less = 0;
            let start_y: DynVal = if has_top_left_corner {
                y_less += 1;
                1.into()
            } else {
                0.into()
            };
            let end_y = if has_bottom_left_corner {
                y_less += 1;
                DynVal::FULL.minus(1.into())
            } else {
                DynVal::FULL
            };
            *bordered.y_scrollbar_sub_from_full.borrow_mut() = y_less;
            *bordered.y_scrollbar_view_sub_from_full.borrow_mut() = y_less;
            let left_loc = DynLocation::new(0.into(), 1.into(), start_y.clone(), end_y);

            if let PropertyVrt::Scrollbar(sb) = left_property {
                let inner_ = inner.clone();
                let hook = Box::new(move |ctx, y| inner_.set_content_y_offset(&ctx, y));
                *sb.position_changed_hook.borrow_mut() = Some(hook);

                // set the scrollbar dimensions/location (as it wasn't done earlier)
                // see NOTE SB-1 as to why we don't use the dynamic width/height
                let sb_height: DynVal = left_loc.height(ctx).into();
                sb.set_dyn_location(left_loc);

                let inner_ctx = ctx.child_context(&inner.get_dyn_location_set().l);
                sb.set_dyn_height(
                    sb_height.clone(),
                    sb_height,
                    Some(inner.get_content_height(&inner_ctx)),
                );

                bordered.y_scrollbar.borrow_mut().replace(sb.clone());
                bordered.pane.add_element(Box::new(sb));
            } else {
                let side =
                    VerticalSide::new(ctx, chs_left.clone(), VerticalPos::Left, left_property);
                side.pane.set_dyn_location(left_loc);
                bordered.pane.add_element(Box::new(side.clone()));
                bordered.left.borrow_mut().replace(side);
            }
        }

        if let Some(right_property) = properties.right.take() {
            let mut y_less = 0;
            let start_y: DynVal = if has_top_right_corner {
                y_less += 1;
                1.into()
            } else {
                0.into()
            };
            let end_y = if has_bottom_right_corner {
                y_less += 1;
                DynVal::FULL.minus(1.into())
            } else {
                DynVal::FULL
            };
            *bordered.y_scrollbar_sub_from_full.borrow_mut() = y_less;
            *bordered.y_scrollbar_view_sub_from_full.borrow_mut() = y_less;

            let right_loc = DynLocation::new(
                DynVal::FULL.minus(1.into()),
                DynVal::FULL,
                start_y.clone(),
                end_y,
            );

            if let PropertyVrt::Scrollbar(sb) = right_property {
                let inner_ = inner.clone();
                let hook = Box::new(move |ctx, y| inner_.set_content_y_offset(&ctx, y));
                *sb.position_changed_hook.borrow_mut() = Some(hook);

                // set the scrollbar dimensions/location (as it wasn't done earlier)
                // see NOTE SB-1 as to why we don't use the dynamic width/height
                let sb_height: DynVal = right_loc.height(ctx).into();
                sb.set_dyn_location(right_loc);

                let inner_ctx = ctx.child_context(&inner.get_dyn_location_set().l);
                sb.set_dyn_height(
                    sb_height.clone(),
                    sb_height,
                    Some(inner.get_content_height(&inner_ctx)),
                );

                bordered.y_scrollbar.borrow_mut().replace(sb.clone());
                bordered.pane.add_element(Box::new(sb));
            } else {
                let side =
                    VerticalSide::new(ctx, chs_right.clone(), VerticalPos::Right, right_property);
                side.pane.set_dyn_location(right_loc);
                bordered.pane.add_element(Box::new(side.clone()));
                bordered.right.borrow_mut().replace(side);
            }
        }

        if let Some(top_property) = properties.top.take() {
            let mut x_less = 0;
            let start_x: DynVal = if has_top_left_corner {
                x_less += 1;
                1.into()
            } else {
                0.into()
            };
            let end_x = if has_top_right_corner {
                x_less += 1;
                DynVal::FULL.minus(1.into())
            } else {
                DynVal::FULL
            };
            *bordered.x_scrollbar_sub_from_full.borrow_mut() = x_less;
            *bordered.x_scrollbar_view_sub_from_full.borrow_mut() = x_less;

            let top_loc = DynLocation::new(start_x, end_x, 0.into(), 1.into());
            if let PropertyHzt::Scrollbar(sb) = top_property {
                let inner_ = inner.clone();
                let hook = Box::new(move |ctx, x| inner_.set_content_x_offset(&ctx, x));
                *sb.position_changed_hook.borrow_mut() = Some(hook);

                // set the scrollbar dimensions/location (as it wasn't done earlier)
                // see NOTE SB-1 as to why we don't use the dynamic width/height
                let sb_width: DynVal = top_loc.width(ctx).into();
                sb.set_dyn_location(top_loc);

                let inner_ctx = ctx.child_context(&inner.get_dyn_location_set().l);
                sb.set_dyn_width(
                    sb_width.clone(),
                    sb_width,
                    Some(inner.get_content_width(&inner_ctx)),
                );

                bordered.x_scrollbar.borrow_mut().replace(sb.clone());
                bordered.pane.add_element(Box::new(sb));
            } else {
                let side =
                    HorizontalSide::new(ctx, chs_top.clone(), HorizontalPos::Top, top_property);
                side.pane.set_dyn_location(top_loc);
                bordered.pane.add_element(Box::new(side.clone()));
                bordered.top.borrow_mut().replace(side);
            }
        }

        if let Some(bottom_property) = properties.bottom.take() {
            let mut x_less = 0;
            let start_x: DynVal = if has_bottom_left_corner {
                x_less += 1;
                1.into()
            } else {
                0.into()
            };
            let end_x = if has_bottom_right_corner {
                x_less += 1;
                DynVal::FULL.minus(1.into())
            } else {
                DynVal::FULL
            };
            *bordered.x_scrollbar_sub_from_full.borrow_mut() = x_less;
            *bordered.x_scrollbar_view_sub_from_full.borrow_mut() = x_less;

            let bottom_loc =
                DynLocation::new(start_x, end_x, DynVal::FULL.minus(1.into()), DynVal::FULL);
            if let PropertyHzt::Scrollbar(sb) = bottom_property {
                let inner_ = inner.clone();
                let hook = Box::new(move |ctx, x| {
                    inner_.set_content_x_offset(&ctx, x);
                });
                *sb.position_changed_hook.borrow_mut() = Some(hook);

                // set the scrollbar dimensions/location (as it wasn't done earlier)
                // see NOTE SB-1 as to why we don't use the dynamic width/height
                let sb_width: DynVal = bottom_loc.width(ctx).into();
                sb.set_dyn_location(bottom_loc);
                let inner_ctx = ctx.child_context(&inner.get_dyn_location_set().l);
                sb.set_dyn_width(
                    sb_width.clone(),
                    sb_width,
                    Some(inner.get_content_width(&inner_ctx)),
                );

                bordered.x_scrollbar.borrow_mut().replace(sb.clone());
                bordered.pane.add_element(Box::new(sb));
            } else {
                let side = HorizontalSide::new(
                    ctx,
                    chs_bottom.clone(),
                    HorizontalPos::Bottom,
                    bottom_property,
                );
                side.pane.set_dyn_location(bottom_loc);
                bordered.pane.add_element(Box::new(side.clone()));
                bordered.bottom.borrow_mut().replace(side);
            }
        }

        bordered.pane.add_element(inner);
        bordered
    }

    pub fn ensure_scrollbar_size(&self, ctx: &Context) {
        if *self.last_size.borrow() != ctx.size {
            let x_sb = self.x_scrollbar.borrow();
            if let Some(x_sb) = x_sb.as_ref() {
                let view_w: DynVal = DynVal::FULL
                    .minus((*self.x_scrollbar_view_sub_from_full.borrow()).into())
                    .get_val(ctx.size.width)
                    .into();
                let w: DynVal = DynVal::FULL
                    .minus((*self.x_scrollbar_sub_from_full.borrow()).into())
                    .get_val(ctx.size.width)
                    .into();
                x_sb.set_dyn_width(view_w, w, None);
            }
            let y_sb = self.y_scrollbar.borrow();
            if let Some(y_sb) = y_sb.as_ref() {
                let view_h: DynVal = DynVal::FULL
                    .minus((*self.y_scrollbar_view_sub_from_full.borrow()).into())
                    .get_val(ctx.size.height)
                    .into();
                let h: DynVal = DynVal::FULL
                    .minus((*self.y_scrollbar_sub_from_full.borrow()).into())
                    .get_val(ctx.size.height)
                    .into();
                y_sb.set_dyn_height(view_h, h, None);
            }
            *self.last_size.borrow_mut() = ctx.size;
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Bordered {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.ensure_scrollbar_size(ctx);

        let (captured, mut resps) = self.pane.receive_event(ctx, ev);
        let inner_size = self.inner.borrow().get_dyn_location_set().l.get_size(ctx);
        let inner_ctx = ctx.child_context(&self.inner.borrow().get_dyn_location_set().l);
        if let Some(sb) = self.x_scrollbar.borrow().as_ref() {
            sb.external_change(
                self.inner.borrow().get_content_x_offset(),
                self.inner.borrow().get_content_width(&inner_ctx),
                inner_size,
            );
        }
        if let Some(sb) = self.y_scrollbar.borrow().as_ref() {
            sb.external_change(
                self.inner.borrow().get_content_y_offset(),
                self.inner.borrow().get_content_height(&inner_ctx),
                inner_size,
            );
        }

        // update resize/move responses to this borders id
        for resp in resps.iter_mut() {
            match resp {
                EventResponse::Resize(r) => {
                    r.el_id = self.id();
                }
                EventResponse::Move(r) => {
                    r.el_id = self.id();
                }
                _ => {}
            }
        }

        (captured, resps)
    }
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        self.ensure_scrollbar_size(ctx);
        self.pane.drawing(ctx, dr, force_update)
    }
}

// ------------------------------------------------------------------------------------------

/// a small element (one character) that can be used to send resize requests to parent elements
#[derive(Clone)]
pub struct Corner {
    pub pane: Pane,
    pub pos: Rc<RefCell<CornerPos>>,
    pub property: Rc<RefCell<PropertyCnr>>,
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

    pub fn new(ctx: &Context, ch: DrawCh, pos: CornerPos, property: PropertyCnr) -> Self {
        let pane = Pane::new(ctx, "resize_corner")
            .with_dyn_height(1)
            .with_dyn_width(1)
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

    pub fn set_ch(&self, ch: DrawCh) {
        self.pane.set_content(ch.into());
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, x: D, y: D2) -> Self {
        self.pane.set_at(x.into(), y.into());
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Corner {
    fn receive_event(&self, _ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let property = *self.property.borrow();
        if matches!(property, PropertyCnr::None) {
            return (true, EventResponses::default()); // still capture just don't do anything
        }
        let cur_dragging = *self.dragging.borrow();
        match ev {
            Event::Mouse(me) => match me.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    *self.dragging.borrow_mut() = true;
                    return (true, EventResponses::default());
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    return (true, EventResponses::default());
                }
                _ => *self.dragging.borrow_mut() = false,
            },
            Event::ExternalMouse(me) => match me.kind {
                MouseEventKind::Drag(MouseButton::Left) if cur_dragging => {
                    let dx = me.column;
                    let dy = me.row;

                    match property {
                        PropertyCnr::DragResize => {
                            let (left_dx, right_dx, top_dy, bottom_dy) = match *self.pos.borrow() {
                                CornerPos::TopLeft => (dx, 0, dy, 0),
                                CornerPos::TopRight => (0, dx, dy, 0),
                                CornerPos::BottomLeft => (dx, 0, 0, dy),
                                CornerPos::BottomRight => (0, dx, 0, dy),
                            };
                            let resp = ResizeResponse {
                                el_id: self.id(),
                                left_dx,
                                right_dx,
                                top_dy,
                                bottom_dy,
                            };
                            return (true, EventResponse::Resize(resp).into());
                        }
                        PropertyCnr::DragMove => {
                            let resp = MoveResponse {
                                el_id: self.id(),
                                dx,
                                dy,
                            };
                            return (true, EventResponse::Move(resp).into());
                        }
                        PropertyCnr::None => {}
                    }
                }
                _ => *self.dragging.borrow_mut() = false,
            },
            _ => {}
        }
        (false, EventResponses::default())
    }
}

// ------------------------------------------------------------------------------------------

pub enum Justification {
    /// left or top justified
    Start,
    Center,
    /// right or bottom justified
    End,
}

/// a vertical side of the border
#[derive(Clone)]
pub struct VerticalSide {
    pub pane: Pane,
    pub ch: Rc<RefCell<DrawCh>>,
    pub pos: Rc<RefCell<VerticalPos>>,
    pub property: Rc<RefCell<PropertyVrt>>,
    #[allow(clippy::type_complexity)]
    pub text: Rc<RefCell<Option<(Vec<DrawCh>, Justification)>>>,
    /// postion where dragging started x, y
    pub dragging_start_pos: Rc<RefCell<Option<(i32, i32)>>>,

    pub last_draw: Rc<RefCell<Vec<DrawChPos>>>,
}

/// corner position
#[derive(Clone, Copy)]
pub enum VerticalPos {
    Left,
    Right,
}

impl VerticalSide {
    const Z_INDEX: ZIndex = 200;

    pub fn new(ctx: &Context, ch: DrawCh, pos: VerticalPos, property: PropertyVrt) -> Self {
        let pane = Pane::new(ctx, "vertical_side")
            .with_dyn_height(DynVal::FULL)
            .with_dyn_width(1);
        pane.set_z(Self::Z_INDEX);
        Self {
            pane,
            ch: Rc::new(RefCell::new(ch)),
            pos: Rc::new(RefCell::new(pos)),
            property: Rc::new(RefCell::new(property)),
            text: Rc::new(RefCell::new(None)),
            dragging_start_pos: Rc::new(RefCell::new(None)),
            last_draw: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn with_top_text<S: Into<String>>(self, text: S) -> Self {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::Start;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
        self
    }

    pub fn set_top_text<S: Into<String>>(&self, text: S) {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::Start;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
    }

    pub fn with_bottom_text<S: Into<String>>(self, text: S) -> Self {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::End;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
        self
    }

    pub fn set_bottom_text<S: Into<String>>(&self, text: S) {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::End;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
    }

    pub fn with_center_text<S: Into<String>>(self, text: S) -> Self {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::Center;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
        self
    }

    pub fn set_center_text<S: Into<String>>(&self, text: S) {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::Center;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, x: D, y: D2) -> Self {
        self.pane.set_at(x.into(), y.into());
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for VerticalSide {
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        let out = if let Some((ref text, ref j)) = *self.text.borrow() {
            let text_height = text.len() as u16;
            let (start_y, end_y) = match j {
                Justification::Start => (0u16, text_height),
                Justification::Center => {
                    let start_y = (ctx.size.height - text_height) / 2;
                    (start_y, start_y + text_height)
                }
                Justification::End => (ctx.size.height - text_height, ctx.size.height),
            };
            let mut out = Vec::with_capacity(ctx.size.height as usize);
            let mut text_i = 0;
            for y in 0..ctx.size.height {
                if y >= start_y && y < end_y {
                    if let Some(ch) = text.get(text_i) {
                        out.push(DrawChPos::new(ch.clone(), 0, y));
                        text_i += 1;
                        continue;
                    }
                }
                out.push(DrawChPos::new(self.ch.borrow().clone(), 0, y));
            }
            out
        } else {
            DrawChPos::new_repeated_vertical(self.ch.borrow().clone(), 0, 0, ctx.size.height)
        };

        if out != *self.last_draw.borrow() || force_update {
            *self.last_draw.borrow_mut() = out.clone();
            return DrawUpdate::update(out).into();
        }
        Vec::with_capacity(0)
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let property = self.property.borrow();
        if matches!(*property, PropertyVrt::None) {
            return (true, EventResponses::default()); // still capture just don't do anything
        }

        if let PropertyVrt::Scrollbar(ref sb) = *property {
            return sb.receive_event(ctx, ev);
        }

        let dragging_start_pos = *self.dragging_start_pos.borrow();
        match ev {
            Event::Mouse(me) => match me.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    *self.dragging_start_pos.borrow_mut() = Some((me.column.into(), me.row.into()));
                    return (true, EventResponses::default());
                }
                MouseEventKind::Drag(MouseButton::Left) if dragging_start_pos.is_some() => {
                    let (_, start_y) = dragging_start_pos.expect("impossible");
                    let dy = me.row as i32 - start_y;

                    if matches!(*property, PropertyVrt::DragMove) {
                        let resp = MoveResponse {
                            el_id: self.id(),
                            dx: 0,
                            dy,
                        };
                        return (true, EventResponse::Move(resp).into());
                    }
                }
                _ => *self.dragging_start_pos.borrow_mut() = None,
            },
            Event::ExternalMouse(me) => match me.kind {
                MouseEventKind::Drag(MouseButton::Left) if dragging_start_pos.is_some() => {
                    let dx = me.column;
                    let dy = me.row;

                    match *property {
                        PropertyVrt::DragResize => {
                            let (left_dx, right_dx) = match *self.pos.borrow() {
                                VerticalPos::Left => (dx, 0),
                                VerticalPos::Right => (0, dx),
                            };
                            let resp = ResizeResponse {
                                el_id: self.id(),
                                left_dx,
                                right_dx,
                                top_dy: 0,
                                bottom_dy: 0,
                            };
                            return (true, EventResponse::Resize(resp).into());
                        }
                        PropertyVrt::DragMove => {
                            let (start_x, start_y) = dragging_start_pos.expect("impossible");
                            let dx = dx - start_x;
                            let dy = dy - start_y;
                            let resp = MoveResponse {
                                el_id: self.id(),
                                dx,
                                dy,
                            };
                            return (true, EventResponse::Move(resp).into());
                        }
                        PropertyVrt::Scrollbar(_) => {} // handled earlier
                        PropertyVrt::None => {}
                    }
                }
                _ => *self.dragging_start_pos.borrow_mut() = None,
            },
            _ => {}
        }
        (false, EventResponses::default())
    }
}

// ------------------------------------------------------------------------------------------

/// a vertical side of the border
#[derive(Clone)]
pub struct HorizontalSide {
    pub pane: Pane,
    pub ch: Rc<RefCell<DrawCh>>,
    pub pos: Rc<RefCell<HorizontalPos>>,
    pub property: Rc<RefCell<PropertyHzt>>,
    #[allow(clippy::type_complexity)]
    pub text: Rc<RefCell<Option<(Vec<DrawCh>, Justification)>>>,
    /// x, y
    pub dragging_start_pos: Rc<RefCell<Option<(i32, i32)>>>,

    pub last_draw: Rc<RefCell<Vec<DrawChPos>>>,
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
    pub fn new(ctx: &Context, ch: DrawCh, pos: HorizontalPos, property: PropertyHzt) -> Self {
        let pane = Pane::new(ctx, "horizontal_side")
            .with_dyn_height(1)
            .with_dyn_width(1.0);
        pane.set_z(Self::Z_INDEX);
        Self {
            pane,
            ch: Rc::new(RefCell::new(ch)),
            pos: Rc::new(RefCell::new(pos)),
            property: Rc::new(RefCell::new(property)),
            text: Rc::new(RefCell::new(None)),
            dragging_start_pos: Rc::new(RefCell::new(None)),
            last_draw: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn with_left_text<S: Into<String>>(self, text: S) -> Self {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::Start;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
        self
    }

    pub fn set_left_text<S: Into<String>>(&self, text: S) {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::Start;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
    }

    pub fn with_right_text<S: Into<String>>(self, text: S) -> Self {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::End;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
        self
    }

    pub fn set_right_text<S: Into<String>>(&self, text: S) {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::End;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
    }

    pub fn with_center_text<S: Into<String>>(self, text: S) -> Self {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::Center;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
        self
    }

    pub fn set_center_text<S: Into<String>>(&self, text: S) {
        let sty = self.ch.borrow().style.clone();
        let j = Justification::Center;
        let text = DrawCh::str_to_draw_chs(&text.into(), sty);
        *self.text.borrow_mut() = Some((text, j));
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, x: D, y: D2) -> Self {
        self.pane.set_at(x.into(), y.into());
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for HorizontalSide {
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        let out = if let Some((ref text, ref j)) = *self.text.borrow() {
            let text_width = text.len() as u16;
            let (start_x, end_x) = match j {
                Justification::Start => (0u16, text_width),
                Justification::Center => {
                    let start_x = (ctx.size.width - text_width) / 2;
                    (start_x, start_x + text_width)
                }
                Justification::End => (ctx.size.width - text_width, ctx.size.width),
            };
            let mut out = Vec::with_capacity(ctx.size.width as usize);
            let mut text_i = 0;
            for x in 0..ctx.size.width {
                if x >= start_x && x < end_x {
                    if let Some(ch) = text.get(text_i) {
                        out.push(DrawChPos::new(ch.clone(), x, 0));
                        text_i += 1;
                        continue;
                    }
                }
                out.push(DrawChPos::new(self.ch.borrow().clone(), x, 0));
            }
            out
        } else {
            DrawChPos::new_repeated_horizontal(self.ch.borrow().clone(), 0, 0, ctx.size.width)
        };

        if out != *self.last_draw.borrow() || force_update {
            *self.last_draw.borrow_mut() = out.clone();
            return DrawUpdate::update(out).into();
        }
        Vec::with_capacity(0)
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let property = self.property.borrow();
        if matches!(*property, PropertyHzt::None) {
            return (true, EventResponses::default()); // still capture just don't do anything
        }

        if let PropertyHzt::Scrollbar(ref sb) = *property {
            return sb.receive_event(ctx, ev);
        }

        let dragging_start_pos = *self.dragging_start_pos.borrow();
        match ev {
            Event::Mouse(me) => match me.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    *self.dragging_start_pos.borrow_mut() = Some((me.column.into(), me.row.into()));
                    return (true, EventResponses::default());
                }
                MouseEventKind::Drag(MouseButton::Left) if dragging_start_pos.is_some() => {
                    let (start_x, _) = dragging_start_pos.expect("impossible");
                    let dx = me.column as i32 - start_x;

                    if matches!(*property, PropertyHzt::DragMove) {
                        let resp = MoveResponse {
                            el_id: self.id(),
                            dx,
                            dy: 0,
                        };
                        return (true, EventResponse::Move(resp).into());
                    }
                }
                _ => *self.dragging_start_pos.borrow_mut() = None,
            },
            Event::ExternalMouse(me) => match me.kind {
                MouseEventKind::Drag(MouseButton::Left) if dragging_start_pos.is_some() => {
                    let dx = me.column;
                    let dy = me.row;

                    match *property {
                        PropertyHzt::DragResize => {
                            let (top_dy, bottom_dy) = match *self.pos.borrow() {
                                HorizontalPos::Top => (dy, 0),
                                HorizontalPos::Bottom => (0, dy),
                            };
                            let resp = ResizeResponse {
                                el_id: self.id(),
                                left_dx: 0,
                                right_dx: 0,
                                top_dy,
                                bottom_dy,
                            };
                            return (true, EventResponse::Resize(resp).into());
                        }
                        PropertyHzt::DragMove => {
                            let (start_x, start_y) = dragging_start_pos.expect("impossible");
                            let dx = dx - start_x;
                            let dy = dy - start_y;
                            let resp = MoveResponse {
                                el_id: self.id(),
                                dx,
                                dy,
                            };
                            return (true, EventResponse::Move(resp).into());
                        }
                        PropertyHzt::Scrollbar(_) => {} // handled earlier
                        PropertyHzt::None => {}
                    }
                }
                _ => *self.dragging_start_pos.borrow_mut() = None,
            },
            _ => {}
        }
        (false, EventResponses::default())
    }
}
