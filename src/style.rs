use {
    crate::Rgba,
    crossterm::style::{Attribute, Attributes},
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct Style {
    pub fg: Option<Rgba>,
    pub bg: Option<Rgba>,
    pub underline: Option<Rgba>,
    pub attr: YHAttributes,
}

impl Style {
    // creates new style from fg, bg with a default style
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            underline: None,
            attr: YHAttributes::new(),
        }
    }

    pub const fn new_coloured(fg: Rgba, bg: Rgba) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            underline: None,
            attr: YHAttributes::new(),
        }
    }

    pub const fn with_fg(mut self, fg: Rgba) -> Self {
        self.fg = Some(fg);
        self
    }

    pub const fn with_bg(mut self, bg: Rgba) -> Self {
        self.bg = Some(bg);
        self
    }

    pub const fn with_underline(mut self, underline: Rgba) -> Self {
        self.underline = Some(underline);
        self
    }

    pub const fn with_attr(mut self, attr: YHAttributes) -> Self {
        self.attr = attr;
        self
    }

    pub fn set_fg(&mut self, fg: Rgba) {
        self.fg = Some(fg);
    }
    pub fn set_bg(&mut self, bg: Rgba) {
        self.bg = Some(bg);
    }
    pub fn set_underline(&mut self, underline: Rgba) {
        self.underline = Some(underline);
    }
    pub fn set_attr(&mut self, attr: YHAttributes) {
        self.attr = attr;
    }
}

impl From<(Rgba, Rgba)> for Style {
    fn from((fg, bg): (Rgba, Rgba)) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            underline: None,
            attr: YHAttributes::new(),
        }
    }
}

//#[derive(Clone, Copy)]
//pub struct LetterStyle {
//    pub ch:    char,
//    pub style: Style,
//}

//impl LetterStyle {
//    pub fn new(ch: char, style: Style) -> Self { LetterStyle { ch, style } }
//}

#[derive(Debug, PartialEq)]
pub enum Justification {
    Left,
    Centre,
    Right,
}

impl Justification {
    //pub fn encode(&self) -> Result<Vec<u8>, Error> {
    //    Ok(bincode::encode_to_vec(self, BINCODE_CONFIG)?)
    //}

    //pub fn decode(bytes: &[u8]) -> Result<Justification, Error> {
    //    let (v, _) = bincode::decode_from_slice(bytes, BINCODE_CONFIG)?;
    //    Ok(v)
    //}

    pub fn is_left(&self) -> bool {
        matches!(self, Justification::Left)
    }

    pub fn is_centre(&self) -> bool {
        matches!(self, Justification::Centre)
    }

    pub fn is_right(&self) -> bool {
        matches!(self, Justification::Right)
    }
}

// mirroring the crossterm Attributes
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct YHAttributes {
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

impl YHAttributes {
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

impl From<YHAttributes> for Attributes {
    fn from(attr: YHAttributes) -> Self {
        let mut att_out = Attributes::default();
        if attr.bold {
            att_out.set(Attribute::Bold);
        }
        if attr.faded {
            att_out.set(Attribute::Dim);
        }
        if attr.italic {
            att_out.set(Attribute::Italic);
        }
        if attr.underlined {
            att_out.set(Attribute::Underlined);
        }
        if attr.doubleunderlined {
            att_out.set(Attribute::DoubleUnderlined);
        }
        if attr.undercurled {
            att_out.set(Attribute::Undercurled);
        }
        if attr.underdotted {
            att_out.set(Attribute::Underdotted);
        }
        if attr.underdashed {
            att_out.set(Attribute::Underdashed);
        }
        if attr.slowblink {
            att_out.set(Attribute::SlowBlink);
        }
        if attr.rapidblink {
            att_out.set(Attribute::RapidBlink);
        }
        if attr.reverse {
            att_out.set(Attribute::Reverse);
        }
        if attr.hidden {
            att_out.set(Attribute::Hidden);
        }
        if attr.crossedout {
            att_out.set(Attribute::CrossedOut);
        }
        if attr.fraktur {
            att_out.set(Attribute::Fraktur);
        }
        if attr.framed {
            att_out.set(Attribute::Framed);
        }
        if attr.encircled {
            att_out.set(Attribute::Encircled);
        }
        if attr.overlined {
            att_out.set(Attribute::OverLined);
        }
        att_out
    }
}

impl From<Attribute> for YHAttributes {
    fn from(attr: Attribute) -> Self {
        let attrs = Attributes::from(attr);
        attrs.into()
    }
}

impl From<Attributes> for YHAttributes {
    fn from(attr: Attributes) -> Self {
        Self {
            bold: attr.has(Attribute::Bold),
            faded: attr.has(Attribute::Dim),
            italic: attr.has(Attribute::Italic),
            underlined: attr.has(Attribute::Underlined),
            doubleunderlined: attr.has(Attribute::DoubleUnderlined),
            undercurled: attr.has(Attribute::Undercurled),
            underdotted: attr.has(Attribute::Underdotted),
            underdashed: attr.has(Attribute::Underdashed),
            slowblink: attr.has(Attribute::SlowBlink),
            rapidblink: attr.has(Attribute::RapidBlink),
            reverse: attr.has(Attribute::Reverse),
            hidden: attr.has(Attribute::Hidden),
            crossedout: attr.has(Attribute::CrossedOut),
            fraktur: attr.has(Attribute::Fraktur),
            framed: attr.has(Attribute::Framed),
            encircled: attr.has(Attribute::Encircled),
            overlined: attr.has(Attribute::OverLined),
        }
    }
}
