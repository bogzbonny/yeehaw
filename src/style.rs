use {
    crate::RgbColour,
    bincode::{Decode, Encode},
    crossterm::style::{Attribute, Attributes, ContentStyle},
};

#[derive(Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct Style {
    pub fg: Option<RgbColour>,
    pub bg: Option<RgbColour>,
    pub attr: AttributesMirror,
}

impl Style {
    // creates new style from fg, bg with a default style
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            attr: AttributesMirror::new(),
        }
    }

    pub const fn new_coloured(fg: RgbColour, bg: RgbColour) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            attr: AttributesMirror::new(),
        }
    }

    pub const fn new_coloured_op(fg: Option<RgbColour>, bg: Option<RgbColour>) -> Self {
        Self {
            fg,
            bg,
            attr: AttributesMirror::new(),
        }
    }

    pub const fn with_fg(mut self, fg: RgbColour) -> Self {
        self.fg = Some(fg);
        self
    }

    pub const fn with_bg(mut self, bg: RgbColour) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn set_fg(&mut self, fg: RgbColour) {
        self.fg = Some(fg);
    }
    pub fn set_bg(&mut self, bg: RgbColour) {
        self.bg = Some(bg);
    }
    pub fn set_attr(&mut self, attr: AttributesMirror) {
        self.attr = attr;
    }
}

impl From<Style> for ContentStyle {
    fn from(sty: Style) -> Self {
        let fg = sty.fg.map(|fg| fg.into());
        let bg = sty.bg.map(|bg| bg.into());
        ContentStyle {
            foreground_color: fg,
            background_color: bg,
            underline_color: None,
            attributes: sty.attr.into(),
        }
    }
}

impl From<(RgbColour, RgbColour)> for Style {
    fn from((fg, bg): (RgbColour, RgbColour)) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            attr: AttributesMirror::new(),
        }
    }
}

//#[derive(Encode, Decode, Clone, Copy)]
//pub struct LetterStyle {
//    pub ch:    char,
//    pub style: Style,
//}

//impl LetterStyle {
//    pub fn new(ch: char, style: Style) -> Self { LetterStyle { ch, style } }
//}

#[derive(Encode, Decode, Debug, PartialEq)]
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
#[derive(Encode, Decode, Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct AttributesMirror {
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

impl AttributesMirror {
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
}

impl From<AttributesMirror> for Attributes {
    fn from(attr: AttributesMirror) -> Self {
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

impl From<Attributes> for AttributesMirror {
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
