pub mod from_ch;
pub mod to_ch;

use std::fmt::{self, Display, Formatter};

/// The BoxDrawingCh struct represents a possible box drawing character
/// Many combinations of attributes will give rise to non-existent characters.
///
/// for reference here are all 128 box drawing characters:
/// ─━│┃┄┅┆┇┈┉┊┋┌┍┎┏
/// ┐┑┒┓└┕┖┗┘┙┚┛├┝┞┟
/// ┠┡┢┣┤┥┦┧┨┩┪┫┬┭┮┯
/// ┰┱┲┳┴┵┶┷┸┹┺┻┼┽┾┿
/// ╀╁╂╃╄╅╆╇╈╉╊╋╌╍╎╏
/// ═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟
/// ╠╡╢╣╤╥╦╧╨╩╪╫╬╭╮╯
/// ╰╱╲╳╴╵╶╷╸╹╺╻╼╽╾╿
pub struct BoxDrawingCh {
    pub left: Option<SideAttribute>,
    pub right: Option<SideAttribute>,
    pub up: Option<SideAttribute>,
    pub down: Option<SideAttribute>,

    /// only possible for Thin edges
    pub curved: bool,

    /// only possible for full horizontal or vertical lines
    pub dashes: Option<Dashes>,
}

pub enum SideAttribute {
    Thin,
    Thick,
    Double,
}

pub enum Dashes {
    Double,
    Triple,
    Quadruple,
}

pub struct BoxAdjContext {
    left: char,
    right: char,
    up: char,
    down: char,
    connected_left: bool,
    connected_right: bool,
    connected_up: bool,
    connected_down: bool,
}

// given the input box character, and the surrounding box characters, return the appropriate box character
// adjust the "─" character to possibly have an upper or bottom connection
fn connect_to_surroundings(c: char, bctx: &BoxAdjContext) -> char {
    todo!()
}

fn combine_box_chars(c: char, bctx: &BoxAdjContext) -> char {
    todo!()
}

impl BoxDrawingCh {
    pub fn new(left: bool, right: bool, up: bool, down: bool) -> Self {
        let left = if left { Some(SideAttribute::Thin) } else { None };
        let right = if right { Some(SideAttribute::Thin) } else { None };
        let up = if up { Some(SideAttribute::Thin) } else { None };
        let down = if down { Some(SideAttribute::Thin) } else { None };

        Self {
            left,
            right,
            up,
            down,
            curved: false,
            dashes: None,
        }
    }

    pub fn new_thick(left: bool, right: bool, up: bool, down: bool) -> Self {
        let left = if left { Some(SideAttribute::Thick) } else { None };
        let right = if right { Some(SideAttribute::Thick) } else { None };
        let up = if up { Some(SideAttribute::Thick) } else { None };
        let down = if down { Some(SideAttribute::Thick) } else { None };

        Self {
            left,
            right,
            up,
            down,
            curved: false,
            dashes: None,
        }
    }

    pub fn new_double(left: bool, right: bool, up: bool, down: bool) -> Self {
        let left = if left { Some(SideAttribute::Double) } else { None };
        let right = if right { Some(SideAttribute::Double) } else { None };
        let up = if up { Some(SideAttribute::Double) } else { None };
        let down = if down { Some(SideAttribute::Double) } else { None };

        Self {
            left,
            right,
            up,
            down,
            curved: false,
            dashes: None,
        }
    }

    /// does the BoxDrawingCh have a corresponding character
    pub fn viable(&self) -> bool {
        self.to_char().is_some()
    }

    // given the input box character, and the surrounding box characters, return the appropriate box character
    // adjust the "─" character to possibly have an upper or bottom connection
    fn connect_to_surroundings(&self, bctx: &BoxAdjContext) -> char {
        todo!()
    }

    /// combines box characters, if the combination is un-viable (doesn't have a corresponding box character)
    /// the function should attempt to keep the most important attributes in this order:
    ///  - keep all the sides of the line, never remove one of the line-parts in order to keep any other attribute
    ///  - keep the SideAttribute of each line-part, if necessary attempt to change some of the SideAttribute to make the combination viable
    ///  - discard the dashes if necessary
    ///  - discard the curved attribute if necessary
    fn combine_box_chars(&self, other: Self) -> Self {
        todo!()
    }
}

impl Display for BoxDrawingCh {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.to_char().unwrap_or(' ').fmt(f)
    }
}

/// curves the edge of a box character if possible
/// otherwise returning the input
pub fn add_curve_edge(c: char) -> char {
    todo!()
}

/// removes a curved edge if there is one
pub fn remove_curve_edge(c: char) -> char {
    todo!()
}

/// adds double dashes to the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
/// double lines are converted into single thin lines
pub fn add_double_dash(c: char) -> char {
    todo!()
}

/// removes double dashes from the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
pub fn remove_double_dash(c: char) -> char {
    todo!()
}

/// adds triple dashes to the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
/// triple lines are converted into single thin lines
pub fn add_triple_dash(c: char) -> char {
    todo!()
}

/// removes triple dashes from the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
pub fn remove_triple_dash(c: char) -> char {
    todo!()
}

/// adds quadruple dashes to the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
/// quadruple lines are converted into single thin lines
pub fn add_quadruple_dash(c: char) -> char {
    todo!()
}

/// removes quadruple dashes from the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
pub fn remove_quadruple_dash(c: char) -> char {
    todo!()
}

/// converts all the line parts within a box character to a thick line parts
pub fn make_thick(c: char) -> char {
    todo!()
}

/// converts all the line parts within a box character to a thin line parts
pub fn make_thin(c: char) -> char {
    todo!()
}

/// converts all the line parts within a box character to a double line parts
pub fn make_double(c: char) -> char {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_drawing_conversion() {
        let test_chars = " ─━│┃┄┅┆┇┈┉┊┋┌┍┎┏┐┑┒┓└┕┖┗┘┙┚┛├┝┞┟┠┡┢┣┤┥┦┧┨┩┪┫┬┭┮┯┰┱┲┳┴┵┶┷┸┹┺┻┼┽┾┿╀╁╂╃╄╅╆╇╈╉╊╋╌╍╎╏═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬╭╮╯╰╴╵╶╷╸╹╺╻╼╽╾╿";

        for ch in test_chars.chars() {
            if let Some(box_ch) = BoxDrawingCh::from_char(ch) {
                let result = box_ch.to_char().expect("Failed conversion");
                assert_eq!(result, ch, "Failed conversion for char: {}", ch);
            }
        }
    }
}
