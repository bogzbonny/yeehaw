// Code in this file was originally adapted from https://github.com/ratatui/ansi-to-tui
// and is thereby licensed under the MIT license. (Nov 10, 2025)
// Original author: Uttarayan Mondal <email@uttarayan.me>
// Thank you to the ratatui team! :)

use {
    crate::*,
    crossterm::style::Color as CrosstermColor,
    nom::{
        AsChar, IResult, Parser,
        branch::alt,
        bytes::complete::*,
        character::complete::*,
        combinator::opt,
        multi::*,
        sequence::{delimited, preceded},
    },
};

pub fn get_chs_2d(s: &[u8], sty: Style) -> DrawChs2D {
    // Manual construction without relying on automatic expansion.
    let mut out: DrawChs2D = DrawChs2D::new(Vec::new());
    let mut cur_style = sty.clone();
    let mut x: usize = 0;
    let mut y: usize = 0;

    let mut i = 0usize;
    while i < s.len() {
        // Parse SGR escape sequence.
        if s[i] == 0x1b && i + 1 < s.len() && s[i + 1] == b'[' {
            if let Ok((rem, opt_style)) = style_f(&s[i..], cur_style.clone()) {
                if let Some(new_style) = opt_style {
                    cur_style = new_style;
                }
                i = s.len() - rem.len();
                continue;
            }
        }
        // Newline handling.
        if s[i] == b'\n' {
            y += 1;
            x = 0;
            i += 1;
            continue;
        }
        // Decode next UTF‑8 character.
        if let Ok(txt) = std::str::from_utf8(&s[i..]) {
            if let Some(ch) = txt.chars().next() {
                let ch_len = ch.len_utf8();
                out.set_ch_expand_if_necessary(x, y, DrawCh::new(ch, cur_style.clone()), &sty);
                x += 1;
                i += ch_len;
                continue;
            }
        }
        i += 1;
    }

    out
}

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum AnsiCode {
    /// Reset the terminal
    Reset,
    /// Set font to bold
    Bold,
    /// Set font to faint
    Faded,
    /// Set font to italic
    Italic,
    /// Set font to underline
    Underline,
    /// Set cursor to slowblink
    SlowBlink,
    /// Set cursor to rapidblink
    RapidBlink,
    /// Invert the colors
    Reverse,
    /// Hidden text
    Hidden,
    /// Display crossed out text
    CrossedOut,
    /// Choose primary font
    PrimaryFont,
    /// Choose alternate font
    AlternateFont,
    /// Choose alternate fonts 1-9
    #[allow(dead_code)]
    AlternateFonts(u8), // = 11..19, // from 11 to 19
    /// Fraktur
    Fraktur,
    /// Turn off bold
    BoldOff,
    /// Set text to normal
    Normal,
    /// Turn off Italic
    NotItalic,
    /// Turn off underline
    UnderlineOff,
    /// Turn off blinking
    BlinkOff,
    // 26 ?
    /// Don't invert colors
    InvertOff,
    /// Reveal text
    Reveal,
    /// Turn off Crossedout text
    CrossedOutOff,
    Framed,
    Encircled,
    Overlined,
    NeitherFramedNorEncircled,
    NotOverlined,
    /// Set foreground color (4-bit)
    ForegroundColor(Color), //, 31..37//Issue 60553 https://github.com/rust-lang/rust/issues/60553
    /// Set foreground color (8-bit and 24-bit)
    SetForegroundColor,
    /// Default foreground color
    DefaultForegroundColor,
    /// Set background color (4-bit)
    BackgroundColor(Color), // 41..47
    /// Set background color (8-bit and 24-bit)
    SetBackgroundColor,
    /// Default background color
    DefaultBackgroundColor, // 49
    /// Other / non supported escape codes
    Code(Vec<u8>),
}

impl From<u8> for AnsiCode {
    fn from(code: u8) -> Self {
        match code {
            0 => AnsiCode::Reset,
            1 => AnsiCode::Bold,
            2 => AnsiCode::Faded,
            3 => AnsiCode::Italic,
            4 => AnsiCode::Underline,
            5 => AnsiCode::SlowBlink,
            6 => AnsiCode::RapidBlink,
            7 => AnsiCode::Reverse,
            8 => AnsiCode::Hidden,
            9 => AnsiCode::CrossedOut,
            10 => AnsiCode::PrimaryFont,
            11 => AnsiCode::AlternateFont,
            // AlternateFont = 11..19,
            20 => AnsiCode::Fraktur,
            21 => AnsiCode::BoldOff,
            22 => AnsiCode::Normal,
            23 => AnsiCode::NotItalic,
            24 => AnsiCode::UnderlineOff,
            25 => AnsiCode::BlinkOff,
            // 26 unused (proportional spacing)
            27 => AnsiCode::InvertOff,
            28 => AnsiCode::Reveal,
            29 => AnsiCode::CrossedOutOff,
            30 => AnsiCode::ForegroundColor(CrosstermColor::Black.into()),
            31 => AnsiCode::ForegroundColor(CrosstermColor::DarkRed.into()),
            32 => AnsiCode::ForegroundColor(CrosstermColor::DarkGreen.into()),
            33 => AnsiCode::ForegroundColor(CrosstermColor::DarkYellow.into()),
            34 => AnsiCode::ForegroundColor(CrosstermColor::DarkBlue.into()),
            35 => AnsiCode::ForegroundColor(CrosstermColor::DarkMagenta.into()),
            36 => AnsiCode::ForegroundColor(CrosstermColor::DarkCyan.into()),
            37 => AnsiCode::ForegroundColor(CrosstermColor::Grey.into()),
            38 => AnsiCode::SetForegroundColor,
            39 => AnsiCode::DefaultForegroundColor,
            40 => AnsiCode::BackgroundColor(CrosstermColor::Black.into()),
            41 => AnsiCode::BackgroundColor(CrosstermColor::DarkRed.into()),
            42 => AnsiCode::BackgroundColor(CrosstermColor::DarkGreen.into()),
            43 => AnsiCode::BackgroundColor(CrosstermColor::DarkYellow.into()),
            44 => AnsiCode::BackgroundColor(CrosstermColor::DarkBlue.into()),
            45 => AnsiCode::BackgroundColor(CrosstermColor::DarkMagenta.into()),
            46 => AnsiCode::BackgroundColor(CrosstermColor::DarkCyan.into()),
            47 => AnsiCode::BackgroundColor(CrosstermColor::Grey.into()),
            48 => AnsiCode::SetBackgroundColor,
            49 => AnsiCode::DefaultBackgroundColor,
            // 50 unused (disable proporational spacing)
            51 => AnsiCode::Framed,
            52 => AnsiCode::Encircled,
            53 => AnsiCode::Overlined,
            54 => AnsiCode::NeitherFramedNorEncircled,
            55 => AnsiCode::NotOverlined,

            // TODO underline colors
            // TODO double underline, undercurl, underdot, underdash
            90 => AnsiCode::ForegroundColor(CrosstermColor::DarkGrey.into()),
            91 => AnsiCode::ForegroundColor(CrosstermColor::Red.into()),
            92 => AnsiCode::ForegroundColor(CrosstermColor::Green.into()),
            93 => AnsiCode::ForegroundColor(CrosstermColor::Yellow.into()),
            94 => AnsiCode::ForegroundColor(CrosstermColor::Blue.into()),
            95 => AnsiCode::ForegroundColor(CrosstermColor::Magenta.into()),
            96 => AnsiCode::ForegroundColor(CrosstermColor::Cyan.into()),
            97 => AnsiCode::ForegroundColor(CrosstermColor::White.into()),
            100 => AnsiCode::BackgroundColor(CrosstermColor::DarkGrey.into()),
            101 => AnsiCode::BackgroundColor(CrosstermColor::Red.into()),
            102 => AnsiCode::BackgroundColor(CrosstermColor::Green.into()),
            103 => AnsiCode::BackgroundColor(CrosstermColor::Yellow.into()),
            104 => AnsiCode::BackgroundColor(CrosstermColor::Blue.into()),
            105 => AnsiCode::BackgroundColor(CrosstermColor::Magenta.into()),
            106 => AnsiCode::BackgroundColor(CrosstermColor::Cyan.into()),
            107 => AnsiCode::ForegroundColor(CrosstermColor::White.into()),
            code => AnsiCode::Code(vec![code]),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ColorType {
    /// Eight Bit color
    EightBit,
    /// 24-bit color or true color
    TrueColor,
}

#[derive(Debug, Clone, PartialEq)]
struct AnsiItem {
    code: AnsiCode,
    color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
struct AnsiStates {
    pub items: smallvec::SmallVec<[AnsiItem; 2]>,
    pub style: Style,
}

impl From<AnsiStates> for Style {
    fn from(states: AnsiStates) -> Self {
        let mut style = states.style;
        if states.items.is_empty() {
            // [m is treated as a reset
            style = Style::default_const();
        }
        for item in states.items {
            match item.code {
                AnsiCode::Reset => style = Style::default_const(),
                AnsiCode::Bold => style.attr.bold = true,
                AnsiCode::Faded => style.attr.faded = true,
                AnsiCode::Normal => {
                    style.attr.bold = false;
                    style.attr.faded = false;
                }
                AnsiCode::Italic => style.attr.italic = true,
                AnsiCode::NotItalic => style.attr.italic = false,
                AnsiCode::Underline => style.attr.underlined = true,
                AnsiCode::UnderlineOff => style.attr.underlined = false,
                AnsiCode::SlowBlink => style.attr.slowblink = true,
                AnsiCode::RapidBlink => style.attr.rapidblink = true,
                AnsiCode::BlinkOff => {
                    style.attr.slowblink = false;
                    style.attr.rapidblink = false;
                }
                AnsiCode::Reverse => style.attr.reverse = true,
                AnsiCode::Hidden => style.attr.hidden = true,
                AnsiCode::Reveal => style.attr.hidden = false,
                AnsiCode::CrossedOut => style.attr.crossedout = true,
                AnsiCode::CrossedOutOff => style.attr.crossedout = false,
                AnsiCode::Fraktur => style.attr.fraktur = true,
                AnsiCode::Framed => style.attr.framed = true,
                AnsiCode::Encircled => style.attr.encircled = true,
                AnsiCode::Overlined => style.attr.overlined = true,
                AnsiCode::DefaultForegroundColor => style.set_fg(CrosstermColor::Reset.into()),
                AnsiCode::DefaultBackgroundColor => style.set_bg(CrosstermColor::Reset.into()),
                AnsiCode::SetForegroundColor => {
                    if let Some(color) = item.color {
                        style.set_fg(color)
                    }
                }
                AnsiCode::SetBackgroundColor => {
                    if let Some(color) = item.color {
                        style.set_bg(color)
                    }
                }
                AnsiCode::ForegroundColor(color) => style.set_fg(color),
                AnsiCode::BackgroundColor(color) => style.set_bg(color),
                _ => (),
            }
        }
        style
    }
}

fn style_f(s: &[u8], style: Style) -> IResult<&[u8], Option<Style>, nom::error::Error<&[u8]>> {
    let (s, r) = match opt(ansi_sgr_code).parse(s)? {
        (s, Some(r)) => (s, Some(r)),
        (s, None) => {
            let (s, _) = any_escape_sequence(s)?;
            (s, None)
        }
    };
    Ok((
        s,
        r.map(|r| {
            Style::from(AnsiStates {
                style: style.clone(),
                items: r,
            })
        }),
    ))
}

/// A complete ANSI SGR code
fn ansi_sgr_code(
    s: &[u8],
) -> IResult<&[u8], smallvec::SmallVec<[AnsiItem; 2]>, nom::error::Error<&[u8]>> {
    delimited(
        tag("\x1b["),
        fold_many0(ansi_sgr_item, smallvec::SmallVec::new, |mut items, item| {
            items.push(item);
            items
        }),
        char('m'),
    )
    .parse(s)
}

fn any_escape_sequence(s: &[u8]) -> IResult<&[u8], Option<&[u8]>> {
    // Attempt to consume most escape codes, including a single escape char.
    //
    // Most escape codes begin with ESC[ and are terminated by an alphabetic character,
    // but OSC codes begin with ESC] and are terminated by an ascii bell (\x07)
    // and a truncated/invalid code may just be a standalone ESC or not be terminated.
    //
    // We should try to consume as much of it as possible to match behavior of most terminals;
    // where we fail at that we should at least consume the escape char to avoid infinitely looping

    let (input, garbage) = preceded(
        char('\x1b'),
        opt(alt((
            delimited(char('['), take_till(AsChar::is_alpha), opt(take(1u8))),
            delimited(char(']'), take_till(|c| c == b'\x07'), opt(take(1u8))),
        ))),
    )
    .parse(s)?;
    Ok((input, garbage))
}

/// An ANSI SGR attribute
fn ansi_sgr_item(s: &[u8]) -> IResult<&[u8], AnsiItem> {
    let (s, c) = u8(s)?;
    let code = AnsiCode::from(c);
    let (s, color) = match code {
        AnsiCode::SetForegroundColor | AnsiCode::SetBackgroundColor => {
            let (s, _) = opt(tag(";")).parse(s)?;
            let (s, color) = color(s)?;
            (s, Some(color))
        }
        _ => (s, None),
    };
    let (s, _) = opt(tag(";")).parse(s)?;
    Ok((s, AnsiItem { code, color }))
}

fn color(s: &[u8]) -> IResult<&[u8], Color> {
    let (s, c_type) = color_type(s)?;
    let (s, _) = opt(tag(";")).parse(s)?;
    match c_type {
        ColorType::TrueColor => {
            let (s, (r, _, g, _, b)) = (u8, tag(";"), u8, tag(";"), u8).parse(s)?;
            Ok((s, Color::new(r, g, b)))
        }
        ColorType::EightBit => {
            let (s, index) = u8(s)?;
            Ok((s, CrosstermColor::AnsiValue(index).into()))
        }
    }
}

fn color_type(s: &[u8]) -> IResult<&[u8], ColorType> {
    let (s, t) = i64(s)?;
    // NOTE: This isn't opt because a color type must always be followed by a color
    // let (s, _) = opt(tag(";")).parse(s)?;
    let (s, _) = tag(";").parse(s)?;
    match t {
        2 => Ok((s, ColorType::TrueColor)),
        5 => Ok((s, ColorType::EightBit)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            s,
            nom::error::ErrorKind::Alt,
        ))),
    }
}

// ------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChPlus, Color};
    use crossterm::style::Color as CrosstermColor;

    #[test]
    fn test_get_chs_2d_simple() {
        let s = b"ab\nc";
        let chs = get_chs_2d(s, Style::default_const());
        eprintln!("matrix: {}", chs);
        assert_eq!(chs.width(), 2);
        assert_eq!(chs.height(), 2);
        assert_eq!(chs.get_at(0, 0).unwrap().ch, ChPlus::Char('a'));
        assert_eq!(chs.get_at(1, 0).unwrap().ch, ChPlus::Char('b'));
        assert_eq!(chs.get_at(0, 1).unwrap().ch, ChPlus::Char('c'));
    }

    #[test]
    fn test_get_chs_2d_ansi_color() {
        // Red foreground for 'X', then reset, then normal 'Y'
        let s = b"\x1b[31mX\x1b[0mY";
        let chs = get_chs_2d(s, Style::default_const());
        assert_eq!(chs.width(), 2);
        // First character should have red foreground
        match &chs.get_at(0, 0).unwrap().style.fg {
            Some((Color::ANSI(CrosstermColor::DarkRed), _)) => {}
            _ => panic!("First char does not have expected red foreground"),
        }
        // Second character should have no foreground set (default)
        assert!(chs.get_at(1, 0).unwrap().style.fg.is_none());
    }

    #[test]
    fn test_get_chs_2d_rgb_fg_bg() {
        // Set foreground to RGB (10,20,30), then background to RGB (100,110,120), then reset.
        let s = b"\x1b[38;2;10;20;30mA\x1b[48;2;100;110;120mB\x1b[0mC";
        let chs = get_chs_2d(s, Style::default_const());
        assert_eq!(chs.width(), 3);
        // First character: foreground RGB, no background.
        match &chs.get_at(0, 0).unwrap().style.fg {
            Some((Color::Rgba(rgba), _)) => {
                assert_eq!(rgba.r, 10);
                assert_eq!(rgba.g, 20);
                assert_eq!(rgba.b, 30);
            }
            _ => panic!("First char missing expected RGB foreground"),
        }
        assert!(chs.get_at(0, 0).unwrap().style.bg.is_none());
        // Second character: inherits foreground RGB, adds background RGB.
        match &chs.get_at(1, 0).unwrap().style.fg {
            Some((Color::Rgba(rgba), _)) => {
                assert_eq!(rgba.r, 10);
                assert_eq!(rgba.g, 20);
                assert_eq!(rgba.b, 30);
            }
            _ => panic!("Second char missing expected RGB foreground"),
        }
        match &chs.get_at(1, 0).unwrap().style.bg {
            Some((Color::Rgba(rgba), _)) => {
                assert_eq!(rgba.r, 100);
                assert_eq!(rgba.g, 110);
                assert_eq!(rgba.b, 120);
            }
            _ => panic!("Second char missing expected RGB background"),
        }
        // Third character after reset should have no fg/bg.
        assert!(chs.get_at(2, 0).unwrap().style.fg.is_none());
        assert!(chs.get_at(2, 0).unwrap().style.bg.is_none());
    }

    #[test]
    fn color_test() {
        let c = color(b"2;255;255;255").unwrap();
        assert_eq!(c.1, Color::new(255, 255, 255));
        let c = color(b"5;255").unwrap();
        assert_eq!(c.1, CrosstermColor::AnsiValue(255).into());
        let err = color(b"10;255");
        assert_ne!(err, Ok(c));
    }

    #[test]
    fn ansi_items_test() {
        let sc: Style = Default::default();
        let t = style_f(b"\x1b[38;2;3;3;3m", sc.clone()).unwrap().1.unwrap();
        assert_eq!(
            t,
            Style::from(AnsiStates {
                style: sc.clone(),
                items: vec![AnsiItem {
                    code: AnsiCode::SetForegroundColor,
                    color: Some(Color::new(3, 3, 3))
                }]
                .into()
            })
        );
        assert_eq!(
            style_f(b"\x1b[38;5;3m", sc.clone()).unwrap().1.unwrap(),
            Style::from(AnsiStates {
                style: sc.clone(),
                items: vec![AnsiItem {
                    code: AnsiCode::SetForegroundColor,
                    color: Some(CrosstermColor::AnsiValue(3).into())
                }]
                .into()
            })
        );
        assert_eq!(
            style_f(b"\x1b[38;5;3;48;5;3m", sc.clone())
                .unwrap()
                .1
                .unwrap(),
            Style::from(AnsiStates {
                style: sc.clone(),
                items: vec![
                    AnsiItem {
                        code: AnsiCode::SetForegroundColor,
                        color: Some(CrosstermColor::AnsiValue(3).into())
                    },
                    AnsiItem {
                        code: AnsiCode::SetBackgroundColor,
                        color: Some(CrosstermColor::AnsiValue(3).into())
                    }
                ]
                .into()
            })
        );
        assert_eq!(
            style_f(b"\x1b[38;5;3;48;5;3;1m", sc.clone())
                .unwrap()
                .1
                .unwrap(),
            Style::from(AnsiStates {
                style: sc.clone(),
                items: vec![
                    AnsiItem {
                        code: AnsiCode::SetForegroundColor,
                        color: Some(CrosstermColor::AnsiValue(3).into())
                    },
                    AnsiItem {
                        code: AnsiCode::SetBackgroundColor,
                        color: Some(CrosstermColor::AnsiValue(3).into())
                    },
                    AnsiItem {
                        code: AnsiCode::Bold,
                        color: None
                    }
                ]
                .into()
            })
        );
    }
}
