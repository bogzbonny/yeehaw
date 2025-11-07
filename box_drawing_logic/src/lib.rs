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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideAttribute {
    Thin,
    Thick,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dashes {
    Double,
    Triple,
    Quadruple,
}

// NOTE if connected_[side] is true, but the corresponding char at for that side
// does not have a box drawing side in the direction of the connection then no connection will
// be attempted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxAdjContext {
    pub left: char,
    pub right: char,
    pub up: char,
    pub down: char,
    pub connected_left: bool,
    pub connected_right: bool,
    pub connected_up: bool,
    pub connected_down: bool,
}

impl BoxDrawingCh {
    pub fn new(left: bool, right: bool, up: bool, down: bool) -> Self {
        Self::new_with_side_attr(left, right, up, down, SideAttribute::Thin)
    }

    pub fn new_thick(left: bool, right: bool, up: bool, down: bool) -> Self {
        Self::new_with_side_attr(left, right, up, down, SideAttribute::Thick)
    }

    pub fn new_double(left: bool, right: bool, up: bool, down: bool) -> Self {
        Self::new_with_side_attr(left, right, up, down, SideAttribute::Double)
    }

    pub fn new_with_side_attr(
        left: bool, right: bool, up: bool, down: bool, side_attr: SideAttribute,
    ) -> Self {
        let left = if left { Some(side_attr) } else { None };
        let right = if right { Some(side_attr) } else { None };
        let up = if up { Some(side_attr) } else { None };
        let down = if down { Some(side_attr) } else { None };
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
        self.to_char_permissive().is_some()
    }

    // This function get's the primary side attribute of the box character
    // for instance ┠ should return SideAttribute::Thick.
    // Ties of the most common attribute should be resolved by the following order:
    //  - Thin
    //  - Thick
    //  - Double
    // So for example ┑ should return SideAttribute::Thin.
    // If there are no sides (space) then the function will return None.
    pub fn primary_side_attribute(&self) -> Option<SideAttribute> {
        let mut attr_count = [
            (SideAttribute::Thin, 0),
            (SideAttribute::Thick, 0),
            (SideAttribute::Double, 0),
        ];
        for side in [&self.left, &self.right, &self.up, &self.down].iter() {
            match side {
                Some(SideAttribute::Thin) => attr_count[0].1 += 1,
                Some(SideAttribute::Thick) => attr_count[1].1 += 1,
                Some(SideAttribute::Double) => attr_count[2].1 += 1,
                None => (),
            }
        }

        // sort the attributes by count
        attr_count.sort_by(|a, b| b.1.cmp(&a.1));

        if attr_count[0].1 == 0 {
            return None;
        }

        // get the most common attribute, if there is a tie, return all tied attributes
        let mut winners = vec![attr_count[0].0];
        if attr_count[0].1 == attr_count[1].1 {
            winners.push(attr_count[1].0);
        }
        if attr_count[0].1 == attr_count[2].1 {
            winners.push(attr_count[2].0);
        }

        if winners.len() == 1 {
            Some(winners[0])
        } else {
            // preference for ties
            if winners.contains(&SideAttribute::Thin) {
                Some(SideAttribute::Thin)
            } else if winners.contains(&SideAttribute::Thick) {
                Some(SideAttribute::Thick)
            } else {
                Some(SideAttribute::Double)
            }
        }
    }

    // Given the input box character, and the surrounding box characters, return the appropriate
    // box character which is viable. Always attempt to create the connection if possible even if
    // this means changing the SideAttribute within the box character side to make the BoxDrawingCh
    // viable, or removing the curved or dashes attributes.
    pub fn connect_to_surroundings(&mut self, bctx: &BoxAdjContext) {
        let mut out = *self;

        // first determine the attribute of the connection sides.
        let left_conn_attr = if bctx.connected_left {
            match BoxDrawingCh::from_char(bctx.left) {
                Some(box_ch) => box_ch.right,
                None => None,
            }
        } else {
            None
        };
        let right_conn_attr = if bctx.connected_right {
            match BoxDrawingCh::from_char(bctx.right) {
                Some(box_ch) => box_ch.left,
                None => None,
            }
        } else {
            None
        };
        let up_conn_attr = if bctx.connected_up {
            match BoxDrawingCh::from_char(bctx.up) {
                Some(box_ch) => box_ch.down,
                None => None,
            }
        } else {
            None
        };
        let down_conn_attr = if bctx.connected_down {
            match BoxDrawingCh::from_char(bctx.down) {
                Some(box_ch) => box_ch.up,
                None => None,
            }
        } else {
            None
        };

        // modify the attributes of the to connect
        if let Some(attr) = left_conn_attr {
            if out.left.is_none() {
                out.left = Some(attr);
            }
        }
        if let Some(attr) = right_conn_attr {
            if out.right.is_none() {
                out.right = Some(attr);
            }
        }
        if let Some(attr) = up_conn_attr {
            if out.up.is_none() {
                out.up = Some(attr);
            }
        }
        if let Some(attr) = down_conn_attr {
            if out.down.is_none() {
                out.down = Some(attr);
            }
        }

        if out.viable() {
            *self = out;
        }
    }

    /// overlay self with other, combines box characters, if the combination is un-viable (doesn't
    /// have a corresponding box character) replace self with other which is considered to be on
    /// top.
    pub fn overlay_with(&mut self, other: Self) {
        let mut combined = *self;

        // Combine sides, taking the other's side if present
        if other.left.is_some() {
            combined.left = other.left;
        }
        if other.right.is_some() {
            combined.right = other.right;
        }
        if other.up.is_some() {
            combined.up = other.up;
        }
        if other.down.is_some() {
            combined.down = other.down;
        }

        // Take curved and dashes from other if present
        if other.curved {
            combined.curved = true;
        }
        if other.dashes.is_some() {
            combined.dashes = other.dashes;
        }

        // If the combined character is viable, use it
        // Otherwise keep the other character which is considered to be on top
        if combined.viable() {
            *self = combined;
        } else {
            *self = other;
        }
    }

    // TODO this functionality should be added to this crate, however it's really complicated,
    // and I don't need it right now, so I'm just leaving this commented code here for now.
    //#[derive(Debug, Clone, Copy, PartialEq, Eq)]
    //pub enum Direction {
    //    Left,
    //    Right,
    //    Up,
    //    Down,
    //}
    //
    // MAYBE we should be drawing in half increments instead of full increments? takes
    // some of the guess work out of it... something to think about.
    //
    // This function is used for drawing box characters, with a cursor (think DrawIt).
    // NOTE we maybe should add in the origin_ctx and next_ctx??
    //  - `origin` - this is the box character at the origin where the next box drawing character
    //     is being draw from. The origin may be modified as a part of this draw process (for
    //     instance turned into a corner if the direction changes).
    //  - `origin_fixed` - whether or not the existing sides on the origin are fixed. If the origin is fixed, then this function will never
    //     remove a side from the origin, only add too it.
    //  - `existing_at_next` - the existing character at the next position which is being drawn to.
    //     the output next character will overlay this character if it is a box character.
    //  - `direction` - the direction which the next box drawing character is being drawn from
    //  - `curve` whether or not to curve the origin if it turned into a corner.
    //  - `side_attr` the `SideAttribute` to apply to the next character being created
    //  - `dashes` dashes to apply to the next character being created
    // Returns the next BoxDrawingCh to be created, and also if the next character is fixed (i.e.
    // feeds into the next origin_fixed).
    //pub fn draw_next(
    //    origin: &mut Self, origin_fixed: bool, existing_at_next: char, direction: Direction,
    //    curve: bool, side_attr: SideAttribute, dashes: Option<Dashes>,
    //) -> (Self, bool) {
    //}

    /// curves the edge of a box character if possible
    /// otherwise returning the input
    pub fn add_curve_edge(&mut self) {
        // Only allow curving if all present sides are thin
        let all_thin = self
            .left
            .as_ref()
            .is_none_or(|s| matches!(s, SideAttribute::Thin))
            && self
                .right
                .as_ref()
                .is_none_or(|s| matches!(s, SideAttribute::Thin))
            && self
                .up
                .as_ref()
                .is_none_or(|s| matches!(s, SideAttribute::Thin))
            && self
                .down
                .as_ref()
                .is_none_or(|s| matches!(s, SideAttribute::Thin));

        // Only allow curving for corner pieces (exactly two adjacent sides)
        let side_count = [&self.left, &self.right, &self.up, &self.down]
            .iter()
            .filter(|s| s.is_some())
            .count();

        let is_corner = side_count == 2
            && ((self.left.is_some() && self.up.is_some())
                || (self.left.is_some() && self.down.is_some())
                || (self.right.is_some() && self.up.is_some())
                || (self.right.is_some() && self.down.is_some()));

        if all_thin && is_corner {
            self.curved = true;
        }
    }

    /// removes a curved edge if there is one
    pub fn remove_curve_edge(&mut self) {
        self.curved = false;
    }

    /// adds double dashes to the box character if it's a horizontal or vertical line
    pub fn add_double_dashes(&mut self) {
        self.add_dashes(Dashes::Double);
    }

    /// adds triple dashes to the box character if it's a horizontal or vertical line
    pub fn add_triple_dash(&mut self) {
        self.add_dashes(Dashes::Triple);
    }

    /// adds quadruple dashes to the box character if it's a horizontal or vertical line
    pub fn add_quadruple_dash(&mut self) {
        self.add_dashes(Dashes::Quadruple);
    }

    /// adds dashes to the box character if it's a horizontal or vertical line
    pub fn add_dashes(&mut self, dashes: Dashes) {
        let is_horizontal =
            self.left.is_some() && self.right.is_some() && self.up.is_none() && self.down.is_none();
        let is_vertical =
            self.up.is_some() && self.down.is_some() && self.left.is_none() && self.right.is_none();

        if is_horizontal || is_vertical {
            // Convert any double lines to thin lines first
            if self.left == Some(SideAttribute::Double) || self.right == Some(SideAttribute::Double)
            {
                self.left = Some(SideAttribute::Thin);
                self.right = Some(SideAttribute::Thin);
            }
            if self.up == Some(SideAttribute::Double) || self.down == Some(SideAttribute::Double) {
                self.up = Some(SideAttribute::Thin);
                self.down = Some(SideAttribute::Thin);
            }

            self.dashes = Some(dashes);
        }
    }

    /// removes dashes from the box character
    pub fn remove_dashes(&mut self) {
        self.dashes = None;
    }

    /// converts all the line parts within a box character to a thick line parts
    pub fn make_thick(&mut self) {
        if self.left.is_some() {
            self.left = Some(SideAttribute::Thick);
        }
        if self.right.is_some() {
            self.right = Some(SideAttribute::Thick);
        }
        if self.up.is_some() {
            self.up = Some(SideAttribute::Thick);
        }
        if self.down.is_some() {
            self.down = Some(SideAttribute::Thick);
        }
        self.curved = false;
        self.dashes = None;
    }

    /// converts all the line parts within a box character to a thin line parts
    pub fn make_thin(&mut self) {
        if self.left.is_some() {
            self.left = Some(SideAttribute::Thin);
        }
        if self.right.is_some() {
            self.right = Some(SideAttribute::Thin);
        }
        if self.up.is_some() {
            self.up = Some(SideAttribute::Thin);
        }
        if self.down.is_some() {
            self.down = Some(SideAttribute::Thin);
        }
    }

    /// converts all the line parts within a box character to a double line parts
    pub fn make_double(&mut self) {
        if self.left.is_some() {
            self.left = Some(SideAttribute::Double);
        }
        if self.right.is_some() {
            self.right = Some(SideAttribute::Double);
        }
        if self.up.is_some() {
            self.up = Some(SideAttribute::Double);
        }
        if self.down.is_some() {
            self.down = Some(SideAttribute::Double);
        }
        self.curved = false;
        self.dashes = None;
    }
}

impl Display for BoxDrawingCh {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.to_char_permissive().unwrap_or(' ').fmt(f)
    }
}

// ---------------------------------------------------------------

// given the input box character, and the surrounding box characters, return the appropriate box character
// adjust the "─" character to possibly have an upper or bottom connection
pub fn connect_to_surroundings(c: char, _bctx: &BoxAdjContext) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.connect_to_surroundings(_bctx);
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

pub fn overlay(bottom: char, top: char) -> char {
    let Some(mut bottom_bx) = BoxDrawingCh::from_char(bottom) else {
        return top;
    };
    let Some(top_bx) = BoxDrawingCh::from_char(top) else {
        return bottom;
    };
    bottom_bx.overlay_with(top_bx);
    bottom_bx.to_char_permissive().unwrap_or(top)
}

/// curves the edge of a box character if possible
/// otherwise returning the input
pub fn add_curve_edge(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.add_curve_edge();
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// removes a curved edge if there is one
pub fn remove_curve_edge(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.remove_curve_edge();
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// removes dashes from the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
pub fn remove_dash(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.remove_dashes();
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// adds double dashes to the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
/// double lines are converted into single thin lines
pub fn add_double_dash(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.add_double_dashes();
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// adds triple dashes to the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
/// triple lines are converted into single thin lines
pub fn add_triple_dash(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.add_triple_dash();
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// adds quadruple dashes to the box character if it's a horizontal or vertical line
/// taking into account the thickness of the line.
/// quadruple lines are converted into single thin lines
pub fn add_quadruple_dash(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.add_quadruple_dash();
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// converts all the line parts within a box character to a thick line parts
pub fn make_thick(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.make_thick();
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// converts all the line parts within a box character to a thin line parts
pub fn make_thin(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.make_thin();
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// converts all the line parts within a box character to a double line parts
pub fn make_double(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.make_double();
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// removes the left side of a box character
pub fn remove_left(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.left = None;
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// removes the right side of a box character
pub fn remove_right(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.right = None;
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// removes the top side of a box character
pub fn remove_up(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.up = None;
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
}

/// removes the bottom side of a box character
pub fn remove_down(c: char) -> char {
    if let Some(mut box_ch) = BoxDrawingCh::from_char(c) {
        box_ch.down = None;
        box_ch.to_char_permissive().unwrap_or(c)
    } else {
        c
    }
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
