use {
    crate::Color,
    crossterm::style::{Attribute as CrAttribute, Attributes as CrAttributes},
    ratatui::style::Modifier as RAttributes,
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub underline: Option<Color>,
    pub attr: Attributes,
}

impl Style {
    // creates new style from fg, bg with a default style
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            underline: None,
            attr: Attributes::new(),
        }
    }

    pub const fn new_coloured(fg: Color, bg: Color) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            underline: None,
            attr: Attributes::new(),
        }
    }

    pub const fn with_fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    pub const fn with_bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    pub const fn with_underline(mut self, underline: Color) -> Self {
        self.underline = Some(underline);
        self
    }

    pub const fn with_attr(mut self, attr: Attributes) -> Self {
        self.attr = attr;
        self
    }

    pub fn set_fg(&mut self, fg: Color) {
        self.fg = Some(fg);
    }
    pub fn set_bg(&mut self, bg: Color) {
        self.bg = Some(bg);
    }
    pub fn set_underline(&mut self, underline: Color) {
        self.underline = Some(underline);
    }
    pub fn set_attr(&mut self, attr: Attributes) {
        self.attr = attr;
    }
}

impl From<(Color, Color)> for Style {
    fn from((fg, bg): (Color, Color)) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            underline: None,
            attr: Attributes::new(),
        }
    }
}

impl From<ratatui::buffer::Cell> for Style {
    fn from(cell: ratatui::buffer::Cell) -> Self {
        Self {
            fg: Some(cell.fg.into()),
            bg: Some(cell.bg.into()),
            underline: Some(cell.underline_color.into()),
            attr: cell.modifier.into(),
        }
    }
}

// mirroring the crossterm Attributes
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
