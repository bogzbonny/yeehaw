use {
    crate::{Color, Size},
    crossterm::style::{Attribute as CrAttribute, Attributes as CrAttributes},
    ratatui::style::Modifier as RAttributes,
    std::time::Duration,
};

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Style {
    pub fg: Option<(Color, FgTranspSrc)>,
    pub bg: Option<(Color, BgTranspSrc)>,
    pub underline_color: Option<(Color, UlTranspSrc)>,
    pub attr: Attributes,
}

/// source of the underlying color for fg colors
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug, Default)]
pub enum FgTranspSrc {
    #[default]
    LowerFg,
    LowerBg,
    LowerUl,
    /// the bg color of the same cell
    ThisBg,
}

/// source of the underlying color for underline colors
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug, Default)]
pub enum UlTranspSrc {
    LowerFg,
    LowerBg,
    #[default]
    LowerUl,
    /// the bg color of the same cell
    ThisBg,
}

/// source of the underlying color for bg colors
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug, Default)]
pub enum BgTranspSrc {
    LowerFg,
    #[default]
    LowerBg,
    LowerUl,
}

impl Style {
    pub const fn default_const() -> Self {
        Self {
            fg: None,
            bg: None,
            underline_color: None,
            attr: Attributes::new(),
        }
    }

    pub const fn transparent() -> Self {
        Self {
            fg: Some((Color::TRANSPARENT, FgTranspSrc::LowerFg)),
            bg: Some((Color::TRANSPARENT, BgTranspSrc::LowerBg)),
            underline_color: Some((Color::TRANSPARENT, UlTranspSrc::LowerUl)),
            attr: Attributes::new(),
        }
    }

    /// create a style which is semi-transparent
    pub fn opaque(mut c: Color, alpha: u8) -> Self {
        c.set_alpha(alpha);
        Self {
            fg: Some((c.clone(), FgTranspSrc::LowerFg)),
            bg: Some((c, BgTranspSrc::LowerBg)),
            underline_color: Some((Color::TRANSPARENT, UlTranspSrc::LowerUl)),
            attr: Attributes::new(),
        }
    }

    pub const fn standard() -> Self {
        Self {
            fg: Some((Color::WHITE, FgTranspSrc::LowerFg)),
            bg: Some((Color::TRANSPARENT, BgTranspSrc::LowerBg)),
            underline_color: Some((Color::TRANSPARENT, UlTranspSrc::LowerUl)),
            attr: Attributes::new(),
        }
    }

    /// creates new style from fg, bg with a default style
    pub const fn new_const(fg: Color, bg: Color) -> Self {
        Self {
            fg: Some((fg, FgTranspSrc::LowerFg)),
            bg: Some((bg, BgTranspSrc::LowerBg)),
            underline_color: None,
            attr: Attributes::new(),
        }
    }

    pub fn with_fg(mut self, fg: Color) -> Self {
        self.set_fg(fg);
        self
    }

    pub fn with_bg(mut self, bg: Color) -> Self {
        self.set_bg(bg);
        self
    }

    pub fn with_underline_color(mut self, underline: Color) -> Self {
        self.underline_color = Some((underline, UlTranspSrc::default()));
        self
    }

    pub fn with_bg_transp_src(mut self, bg_transp_src: BgTranspSrc) -> Self {
        if let Some(bg) = self.bg.as_mut() {
            bg.1 = bg_transp_src;
        }
        self
    }

    pub fn with_fg_transp_src(mut self, fg_transp_src: FgTranspSrc) -> Self {
        if let Some(fg) = self.fg.as_mut() {
            fg.1 = fg_transp_src;
        }
        self
    }

    pub fn with_ul_transp_src(mut self, ul_transp_src: UlTranspSrc) -> Self {
        if let Some(ul) = self.underline_color.as_mut() {
            ul.1 = ul_transp_src;
        }
        self
    }

    pub const fn with_attrs(mut self, attr: Attributes) -> Self {
        self.attr = attr;
        self
    }

    pub fn set_fg(&mut self, fg: Color) {
        match self.fg {
            Some(ref mut fg_) => fg_.0 = fg,
            None => self.fg = Some((fg, FgTranspSrc::default())),
        }
    }
    pub fn set_bg(&mut self, bg: Color) {
        match self.bg {
            Some(ref mut bg_) => bg_.0 = bg,
            None => self.bg = Some((bg, BgTranspSrc::default())),
        }
    }

    pub fn set_underline_color(&mut self, underline: Color) {
        match self.underline_color {
            Some(ref mut underline_) => underline_.0 = underline,
            None => self.underline_color = Some((underline, UlTranspSrc::default())),
        }
    }
    pub fn set_attrs(&mut self, attr: Attributes) {
        self.attr = attr;
    }

    pub const fn with_bold(mut self) -> Self {
        self.attr.bold = true;
        self
    }

    pub const fn with_faded(mut self) -> Self {
        self.attr.faded = true;
        self
    }

    pub const fn with_italic(mut self) -> Self {
        self.attr.italic = true;
        self
    }

    pub const fn with_underlined(mut self) -> Self {
        self.attr.underlined = true;
        self
    }

    pub const fn with_doubleunderlined(mut self) -> Self {
        self.attr.doubleunderlined = true;
        self
    }

    pub const fn with_undercurled(mut self) -> Self {
        self.attr.undercurled = true;
        self
    }

    pub const fn with_underdotted(mut self) -> Self {
        self.attr.underdotted = true;
        self
    }

    pub const fn with_underdashed(mut self) -> Self {
        self.attr.underdashed = true;
        self
    }

    pub const fn with_slowblink(mut self) -> Self {
        self.attr.slowblink = true;
        self
    }

    pub const fn with_rapidblink(mut self) -> Self {
        self.attr.rapidblink = true;
        self
    }

    pub const fn with_reverse(mut self) -> Self {
        self.attr.reverse = true;
        self
    }

    pub const fn with_hidden(mut self) -> Self {
        self.attr.hidden = true;
        self
    }

    pub const fn with_crossedout(mut self) -> Self {
        self.attr.crossedout = true;
        self
    }

    pub const fn with_fraktur(mut self) -> Self {
        self.attr.fraktur = true;
        self
    }

    pub const fn with_framed(mut self) -> Self {
        self.attr.framed = true;
        self
    }

    pub const fn with_encircled(mut self) -> Self {
        self.attr.encircled = true;
        self
    }

    pub const fn with_overlined(mut self) -> Self {
        self.attr.overlined = true;
        self
    }

    pub fn update_colors_for_time_and_pos(
        &mut self, s: Size, dur_since_launch: Duration, x: u16, y: u16,
    ) {
        if let Some(fg) = self.fg.as_mut() {
            fg.0.update_color(s, dur_since_launch, x, y);
        }
        if let Some(bg) = self.bg.as_mut() {
            bg.0.update_color(s, dur_since_launch, x, y);
        }
        if let Some(ul) = self.underline_color.as_mut() {
            ul.0.update_color(s, dur_since_launch, x, y);
        }
    }

    pub fn set_draw_size_offset_colors(&mut self, s: Size, x: u16, y: u16) {
        if let Some(fg) = self.fg.as_mut() {
            fg.0.add_to_offset(x, y);
            fg.0.set_draw_size_if_unset(s);
        }
        if let Some(bg) = self.bg.as_mut() {
            bg.0.add_to_offset(x, y);
            bg.0.set_draw_size_if_unset(s);
        }
        if let Some(ul) = self.underline_color.as_mut() {
            ul.0.add_to_offset(x, y);
            ul.0.set_draw_size_if_unset(s);
        }
    }

    // overlays the overlay style on top of self colors
    pub fn overlay_style(&mut self, overlay: &Self) {
        let (under_fg, under_bg, under_ul) = (
            self.fg.clone(),
            self.bg.clone(),
            self.underline_color.clone(),
        );
        if let Some(ol_bg) = overlay.bg.clone() {
            let under = match ol_bg.1 {
                BgTranspSrc::LowerFg => under_fg.clone().map(|(fg, _)| fg),
                BgTranspSrc::LowerBg => under_bg.clone().map(|(bg, _)| bg),
                BgTranspSrc::LowerUl => under_ul.clone().map(|(ul, _)| ul),
            };
            if let (Some(under), Some((bg, _))) = (under, &mut self.bg) {
                *bg = under.overlay_color(ol_bg.0);
            }
        };
        let this_bg = self.bg.clone();

        if let Some(ol_fg) = overlay.fg.clone() {
            let under = match ol_fg.1 {
                FgTranspSrc::LowerFg => under_fg.clone().map(|(fg, _)| fg),
                FgTranspSrc::LowerBg => under_bg.clone().map(|(bg, _)| bg),
                FgTranspSrc::LowerUl => under_ul.clone().map(|(ul, _)| ul),
                FgTranspSrc::ThisBg => this_bg.clone().map(|(bg, _)| bg),
            };
            if let (Some(under), Some((fg, _))) = (under, &mut self.fg) {
                *fg = under.overlay_color(ol_fg.0);
            }
        };

        if let Some(ol_ul) = overlay.underline_color.clone() {
            let under = match ol_ul.1 {
                UlTranspSrc::LowerFg => under_fg.clone().map(|(fg, _)| fg),
                UlTranspSrc::LowerBg => under_bg.clone().map(|(bg, _)| bg),
                UlTranspSrc::LowerUl => under_ul.clone().map(|(ul, _)| ul),
                UlTranspSrc::ThisBg => this_bg.clone().map(|(bg, _)| bg),
            };
            if let (Some(under), Some((ul, _))) = (under, &mut self.underline_color) {
                *ul = under.overlay_color(ol_ul.0);
            }
        }
    }
}

impl From<(Color, Color)> for Style {
    fn from((fg, bg): (Color, Color)) -> Self {
        Self {
            fg: Some((fg, FgTranspSrc::LowerFg)),
            bg: Some((bg, BgTranspSrc::LowerBg)),
            underline_color: None,
            attr: Attributes::new(),
        }
    }
}

impl From<ratatui::buffer::Cell> for Style {
    fn from(cell: ratatui::buffer::Cell) -> Self {
        Self {
            fg: Some((cell.fg.into(), FgTranspSrc::default())),
            bg: Some((cell.bg.into(), BgTranspSrc::default())),
            underline_color: Some((cell.underline_color.into(), UlTranspSrc::default())),
            attr: cell.modifier.into(),
        }
    }
}

/// Attributes applied to the text
/// bitflags are not used here to help future proof this struct.
/// It's doubtful it would make a significant difference in performance.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct Attributes {
    pub bold: bool,
    pub faded: bool,
    pub italic: bool,
    pub underlined: bool,
    pub doubleunderlined: bool,
    pub undercurled: bool,
    pub underdotted: bool,
    pub underdashed: bool,
    pub slowblink: bool,
    pub rapidblink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub crossedout: bool,
    pub fraktur: bool,
    pub framed: bool,
    pub encircled: bool,
    pub overlined: bool,
}

impl Attributes {
    pub const fn new() -> Self {
        Self {
            bold: false,
            faded: false,
            italic: false,
            underlined: false,
            doubleunderlined: false,
            undercurled: false,
            underdotted: false,
            underdashed: false,
            slowblink: false,
            rapidblink: false,
            reverse: false,
            hidden: false,
            crossedout: false,
            fraktur: false,
            framed: false,
            encircled: false,
            overlined: false,
        }
    }

    pub const fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub const fn with_faded(mut self) -> Self {
        self.faded = true;
        self
    }

    pub const fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub const fn with_underlined(mut self) -> Self {
        self.underlined = true;
        self
    }

    pub const fn with_doubleunderlined(mut self) -> Self {
        self.doubleunderlined = true;
        self
    }

    pub const fn with_undercurled(mut self) -> Self {
        self.undercurled = true;
        self
    }

    pub const fn with_underdotted(mut self) -> Self {
        self.underdotted = true;
        self
    }

    pub const fn with_underdashed(mut self) -> Self {
        self.underdashed = true;
        self
    }

    pub const fn with_slowblink(mut self) -> Self {
        self.slowblink = true;
        self
    }

    pub const fn with_rapidblink(mut self) -> Self {
        self.rapidblink = true;
        self
    }

    pub const fn with_reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    pub const fn with_hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub const fn with_crossedout(mut self) -> Self {
        self.crossedout = true;
        self
    }

    pub const fn with_fraktur(mut self) -> Self {
        self.fraktur = true;
        self
    }

    pub const fn with_framed(mut self) -> Self {
        self.framed = true;
        self
    }

    pub const fn with_encircled(mut self) -> Self {
        self.encircled = true;
        self
    }

    pub const fn with_overlined(mut self) -> Self {
        self.overlined = true;
        self
    }

    pub const fn new_bold() -> Self {
        Self::new().with_bold()
    }
}

impl From<Attributes> for CrAttributes {
    fn from(attr: Attributes) -> Self {
        let mut att_out = CrAttributes::default();
        if attr.bold {
            att_out.set(CrAttribute::Bold);
        }
        if attr.faded {
            att_out.set(CrAttribute::Dim);
        }
        if attr.italic {
            att_out.set(CrAttribute::Italic);
        }
        if attr.underlined {
            att_out.set(CrAttribute::Underlined);
        }
        if attr.doubleunderlined {
            att_out.set(CrAttribute::DoubleUnderlined);
        }
        if attr.undercurled {
            att_out.set(CrAttribute::Undercurled);
        }
        if attr.underdotted {
            att_out.set(CrAttribute::Underdotted);
        }
        if attr.underdashed {
            att_out.set(CrAttribute::Underdashed);
        }
        if attr.slowblink {
            att_out.set(CrAttribute::SlowBlink);
        }
        if attr.rapidblink {
            att_out.set(CrAttribute::RapidBlink);
        }
        if attr.reverse {
            att_out.set(CrAttribute::Reverse);
        }
        if attr.hidden {
            att_out.set(CrAttribute::Hidden);
        }
        if attr.crossedout {
            att_out.set(CrAttribute::CrossedOut);
        }
        if attr.fraktur {
            att_out.set(CrAttribute::Fraktur);
        }
        if attr.framed {
            att_out.set(CrAttribute::Framed);
        }
        if attr.encircled {
            att_out.set(CrAttribute::Encircled);
        }
        if attr.overlined {
            att_out.set(CrAttribute::OverLined);
        }
        att_out
    }
}

impl From<CrAttribute> for Attributes {
    fn from(attr: CrAttribute) -> Self {
        let attrs = CrAttributes::from(attr);
        attrs.into()
    }
}

impl From<CrAttributes> for Attributes {
    fn from(attr: CrAttributes) -> Self {
        Self {
            bold: attr.has(CrAttribute::Bold),
            faded: attr.has(CrAttribute::Dim),
            italic: attr.has(CrAttribute::Italic),
            underlined: attr.has(CrAttribute::Underlined),
            doubleunderlined: attr.has(CrAttribute::DoubleUnderlined),
            undercurled: attr.has(CrAttribute::Undercurled),
            underdotted: attr.has(CrAttribute::Underdotted),
            underdashed: attr.has(CrAttribute::Underdashed),
            slowblink: attr.has(CrAttribute::SlowBlink),
            rapidblink: attr.has(CrAttribute::RapidBlink),
            reverse: attr.has(CrAttribute::Reverse),
            hidden: attr.has(CrAttribute::Hidden),
            crossedout: attr.has(CrAttribute::CrossedOut),
            fraktur: attr.has(CrAttribute::Fraktur),
            framed: attr.has(CrAttribute::Framed),
            encircled: attr.has(CrAttribute::Encircled),
            overlined: attr.has(CrAttribute::OverLined),
        }
    }
}

impl From<RAttributes> for Attributes {
    fn from(attr: RAttributes) -> Self {
        Self {
            bold: attr.contains(RAttributes::BOLD),
            faded: attr.contains(RAttributes::DIM),
            italic: attr.contains(RAttributes::ITALIC),
            underlined: attr.contains(RAttributes::UNDERLINED),
            doubleunderlined: false,
            undercurled: false,
            underdotted: false,
            underdashed: false,
            slowblink: attr.contains(RAttributes::SLOW_BLINK),
            rapidblink: attr.contains(RAttributes::RAPID_BLINK),
            reverse: attr.contains(RAttributes::REVERSED),
            hidden: attr.contains(RAttributes::HIDDEN),
            crossedout: attr.contains(RAttributes::CROSSED_OUT),
            fraktur: false,
            framed: false,
            encircled: false,
            overlined: false,
        }
    }
}
