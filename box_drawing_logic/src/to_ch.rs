use crate::{BoxDrawingCh, Dashes, SideAttribute};

impl BoxDrawingCh {
    /// Converts this BoxDrawingCh back into its corresponding character
    /// if a matching character exists.
    ///
    /// This is the reverse operation of `get_from_char`.
    pub fn to_char(&self) -> Option<char> {
        // Special case for empty box (space)
        if self.left.is_none()
            && self.right.is_none()
            && self.up.is_none()
            && self.down.is_none()
            && !self.curved
            && self.dashes.is_none()
        {
            return Some(' ');
        }

        // Convert all the parts into a pattern that can be matched
        match (
            &self.left,
            &self.right,
            &self.up,
            &self.down,
            self.curved,
            &self.dashes,
        ) {
            // Simple horizontal lines
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, None, false, None) => {
                Some('─')
            }
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, None, false, None) => {
                Some('━')
            }
            (Some(SideAttribute::Double), Some(SideAttribute::Double), None, None, false, None) => {
                Some('═')
            }

            // Simple vertical lines
            (None, None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => {
                Some('│')
            }
            (None, None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => {
                Some('┃')
            }
            (None, None, Some(SideAttribute::Double), Some(SideAttribute::Double), false, None) => {
                Some('║')
            }

            // Dashed horizontal lines
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                None,
                None,
                false,
                Some(Dashes::Triple),
            ) => Some('┄'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                None,
                None,
                false,
                Some(Dashes::Triple),
            ) => Some('┅'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                None,
                None,
                false,
                Some(Dashes::Quadruple),
            ) => Some('┈'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                None,
                None,
                false,
                Some(Dashes::Quadruple),
            ) => Some('┉'),

            // Dashed vertical lines
            (
                None,
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                Some(Dashes::Triple),
            ) => Some('┆'),
            (
                None,
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                Some(Dashes::Triple),
            ) => Some('┇'),
            (
                None,
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                Some(Dashes::Quadruple),
            ) => Some('┊'),
            (
                None,
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                Some(Dashes::Quadruple),
            ) => Some('┋'),

            // Simple corners
            (None, Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), false, None) => {
                Some('┌')
            }
            (None, Some(SideAttribute::Thick), None, Some(SideAttribute::Thin), false, None) => {
                Some('┍')
            }
            (None, Some(SideAttribute::Thin), None, Some(SideAttribute::Thick), false, None) => {
                Some('┎')
            }
            (None, Some(SideAttribute::Thick), None, Some(SideAttribute::Thick), false, None) => {
                Some('┏')
            }
            (Some(SideAttribute::Thin), None, None, Some(SideAttribute::Thin), false, None) => {
                Some('┐')
            }
            (Some(SideAttribute::Thick), None, None, Some(SideAttribute::Thin), false, None) => {
                Some('┑')
            }
            (Some(SideAttribute::Thin), None, None, Some(SideAttribute::Thick), false, None) => {
                Some('┒')
            }
            (Some(SideAttribute::Thick), None, None, Some(SideAttribute::Thick), false, None) => {
                Some('┓')
            }
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, false, None) => {
                Some('└')
            }
            (None, Some(SideAttribute::Thick), Some(SideAttribute::Thin), None, false, None) => {
                Some('┕')
            }
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thick), None, false, None) => {
                Some('┖')
            }
            (None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, false, None) => {
                Some('┗')
            }
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), None, false, None) => {
                Some('┘')
            }
            (Some(SideAttribute::Thick), None, Some(SideAttribute::Thin), None, false, None) => {
                Some('┙')
            }
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thick), None, false, None) => {
                Some('┚')
            }
            (Some(SideAttribute::Thick), None, Some(SideAttribute::Thick), None, false, None) => {
                Some('┛')
            }

            // T junctions
            (
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('├'),
            (
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┝'),
            (
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┞'),
            (
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┟'),
            (
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┠'),
            (
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┡'),
            (
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┢'),
            (
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┣'),
            (
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┤'),
            (
                Some(SideAttribute::Thick),
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┥'),
            (
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┦'),
            (
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┧'),
            (
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┨'),
            (
                Some(SideAttribute::Thick),
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┩'),
            (
                Some(SideAttribute::Thick),
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┪'),
            (
                Some(SideAttribute::Thick),
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┫'),

            // Horizontal T junctions
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┬'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┭'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                None,
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┮'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                None,
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┯'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┰'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┱'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                None,
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┲'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                None,
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('┳'),

            // Bottom T junctions
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                None,
                false,
                None,
            ) => Some('┴'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                None,
                false,
                None,
            ) => Some('┵'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                None,
                false,
                None,
            ) => Some('┶'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                None,
                false,
                None,
            ) => Some('┷'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                None,
                false,
                None,
            ) => Some('┸'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                None,
                false,
                None,
            ) => Some('┹'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                None,
                false,
                None,
            ) => Some('┺'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                None,
                false,
                None,
            ) => Some('┻'),

            // Cross junctions
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┼'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┽'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┾'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('┿'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('╀'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('╁'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('╂'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('╃'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('╄'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('╅'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('╆'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('╇'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('╈'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('╉'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('╊'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                None,
            ) => Some('╋'),

            // Double dashes
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                None,
                None,
                false,
                Some(Dashes::Double),
            ) => Some('╌'),
            (
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                None,
                None,
                false,
                Some(Dashes::Double),
            ) => Some('╍'),
            (
                None,
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                Some(Dashes::Double),
            ) => Some('╎'),
            (
                None,
                None,
                Some(SideAttribute::Thick),
                Some(SideAttribute::Thick),
                false,
                Some(Dashes::Double),
            ) => Some('╏'),

            // Mixed single/double line corners
            (None, Some(SideAttribute::Double), None, Some(SideAttribute::Thin), false, None) => {
                Some('╒')
            }
            (None, Some(SideAttribute::Thin), None, Some(SideAttribute::Double), false, None) => {
                Some('╓')
            }
            (None, Some(SideAttribute::Double), None, Some(SideAttribute::Double), false, None) => {
                Some('╔')
            }
            (Some(SideAttribute::Double), None, None, Some(SideAttribute::Thin), false, None) => {
                Some('╕')
            }
            (Some(SideAttribute::Thin), None, None, Some(SideAttribute::Double), false, None) => {
                Some('╖')
            }
            (Some(SideAttribute::Double), None, None, Some(SideAttribute::Double), false, None) => {
                Some('╗')
            }
            (None, Some(SideAttribute::Double), Some(SideAttribute::Thin), None, false, None) => {
                Some('╘')
            }
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Double), None, false, None) => {
                Some('╙')
            }
            (None, Some(SideAttribute::Double), Some(SideAttribute::Double), None, false, None) => {
                Some('╚')
            }
            (Some(SideAttribute::Double), None, Some(SideAttribute::Thin), None, false, None) => {
                Some('╛')
            }
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Double), None, false, None) => {
                Some('╜')
            }
            (Some(SideAttribute::Double), None, Some(SideAttribute::Double), None, false, None) => {
                Some('╝')
            }

            // Mixed single/double line T junctions
            (
                None,
                Some(SideAttribute::Double),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('╞'),
            (
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                false,
                None,
            ) => Some('╟'),
            (
                None,
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                false,
                None,
            ) => Some('╠'),
            (
                Some(SideAttribute::Double),
                None,
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('╡'),
            (
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                false,
                None,
            ) => Some('╢'),
            (
                Some(SideAttribute::Double),
                None,
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                false,
                None,
            ) => Some('╣'),
            (
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                None,
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('╤'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                None,
                Some(SideAttribute::Double),
                false,
                None,
            ) => Some('╥'),
            (
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                None,
                Some(SideAttribute::Double),
                false,
                None,
            ) => Some('╦'),
            (
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                Some(SideAttribute::Thin),
                None,
                false,
                None,
            ) => Some('╧'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Double),
                None,
                false,
                None,
            ) => Some('╨'),
            (
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                None,
                false,
                None,
            ) => Some('╩'),

            // Mixed single/double line cross junctions
            (
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                false,
                None,
            ) => Some('╪'),
            (
                Some(SideAttribute::Thin),
                Some(SideAttribute::Thin),
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                false,
                None,
            ) => Some('╫'),
            (
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                Some(SideAttribute::Double),
                false,
                None,
            ) => Some('╬'),

            // Curved corners
            (None, Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), true, None) => {
                Some('╭')
            }
            (Some(SideAttribute::Thin), None, None, Some(SideAttribute::Thin), true, None) => {
                Some('╮')
            }
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), None, true, None) => {
                Some('╯')
            }
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, true, None) => {
                Some('╰')
            }

            // Line endings
            (Some(SideAttribute::Thin), None, None, None, false, None) => Some('╴'),
            (None, None, Some(SideAttribute::Thin), None, false, None) => Some('╵'),
            (None, Some(SideAttribute::Thin), None, None, false, None) => Some('╶'),
            (None, None, None, Some(SideAttribute::Thin), false, None) => Some('╷'),
            (Some(SideAttribute::Thick), None, None, None, false, None) => Some('╸'),
            (None, None, Some(SideAttribute::Thick), None, false, None) => Some('╹'),
            (None, Some(SideAttribute::Thick), None, None, false, None) => Some('╺'),
            (None, None, None, Some(SideAttribute::Thick), false, None) => Some('╻'),

            // Mixed thin/thick lines
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), None, None, false, None) => {
                Some('╼')
            }
            (None, None, Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => {
                Some('╽')
            }
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), None, None, false, None) => {
                Some('╾')
            }
            (None, None, Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => {
                Some('╿')
            }

            // If we can't find a match, return space
            _ => None,
        }
    }
}
