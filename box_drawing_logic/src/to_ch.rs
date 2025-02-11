use crate::{BoxDrawingCh, Dashes, SideAttribute};

impl BoxDrawingCh {
    /// Converts this BoxDrawingCh back into its corresponding character
    /// if a matching character exists.
    /// 
    /// This is the reverse operation of `get_from_char`.
    pub fn to_char(&self) -> char {
        // Special case for empty box (space)
        if self.left.is_none() && self.right.is_none() && 
           self.up.is_none() && self.down.is_none() &&
           !self.curved && self.dashes.is_none() {
            return ' ';
        }

        // Convert all the parts into a pattern that can be matched
        match (
            &self.left,
            &self.right,
            &self.up,
            &self.down,
            self.curved,
            &self.dashes
        ) {
            // Simple horizontal lines
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, None, false, None) => '─',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, None, false, None) => '━',
            (Some(SideAttribute::Double), Some(SideAttribute::Double), None, None, false, None) => '═',

            // Simple vertical lines
            (None, None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '│',
            (None, None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => '┃',
            (None, None, Some(SideAttribute::Double), Some(SideAttribute::Double), false, None) => '║',

            // Dashed horizontal lines
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, None, false, Some(Dashes::Triple)) => '┄',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, None, false, Some(Dashes::Triple)) => '┅',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, None, false, Some(Dashes::Quadruple)) => '┈',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, None, false, Some(Dashes::Quadruple)) => '┉',

            // Dashed vertical lines
            (None, None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, Some(Dashes::Triple)) => '┆',
            (None, None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, Some(Dashes::Triple)) => '┇',
            (None, None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, Some(Dashes::Quadruple)) => '┊',
            (None, None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, Some(Dashes::Quadruple)) => '┋',

            // Simple corners
            (None, Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), false, None) => '┌',
            (None, Some(SideAttribute::Thick), None, Some(SideAttribute::Thin), false, None) => '┍',
            (None, Some(SideAttribute::Thin), None, Some(SideAttribute::Thick), false, None) => '┎',
            (None, Some(SideAttribute::Thick), None, Some(SideAttribute::Thick), false, None) => '┏',
            (Some(SideAttribute::Thin), None, None, Some(SideAttribute::Thin), false, None) => '┐',
            (Some(SideAttribute::Thick), None, None, Some(SideAttribute::Thin), false, None) => '┑',
            (Some(SideAttribute::Thin), None, None, Some(SideAttribute::Thick), false, None) => '┒',
            (Some(SideAttribute::Thick), None, None, Some(SideAttribute::Thick), false, None) => '┓',
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, false, None) => '└',
            (None, Some(SideAttribute::Thick), Some(SideAttribute::Thin), None, false, None) => '┕',
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thick), None, false, None) => '┖',
            (None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, false, None) => '┗',
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), None, false, None) => '┘',
            (Some(SideAttribute::Thick), None, Some(SideAttribute::Thin), None, false, None) => '┙',
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thick), None, false, None) => '┚',
            (Some(SideAttribute::Thick), None, Some(SideAttribute::Thick), None, false, None) => '┛',

            // T junctions
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '├',
            (None, Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '┝',
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => '┞',
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => '┟',
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => '┠',
            (None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => '┡',
            (None, Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => '┢',
            (None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => '┣',
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '┤',
            (Some(SideAttribute::Thick), None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '┥',
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => '┦',
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => '┧',
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => '┨',
            (Some(SideAttribute::Thick), None, Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => '┩',
            (Some(SideAttribute::Thick), None, Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => '┪',
            (Some(SideAttribute::Thick), None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => '┫',

            // Horizontal T junctions
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), false, None) => '┬',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), false, None) => '┭',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), None, Some(SideAttribute::Thin), false, None) => '┮',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, Some(SideAttribute::Thin), false, None) => '┯',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, Some(SideAttribute::Thick), false, None) => '┰',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), None, Some(SideAttribute::Thick), false, None) => '┱',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), None, Some(SideAttribute::Thick), false, None) => '┲',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, Some(SideAttribute::Thick), false, None) => '┳',
            
            // Bottom T junctions
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, false, None) => '┴',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, false, None) => '┵',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thin), None, false, None) => '┶',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thin), None, false, None) => '┷',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thick), None, false, None) => '┸',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thick), None, false, None) => '┹',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, false, None) => '┺',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, false, None) => '┻',

            // Cross junctions
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '┼',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '┽',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '┾',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '┿',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => '╀',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => '╁',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => '╂',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => '╃',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => '╄',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => '╅',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => '╆',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => '╇',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => '╈',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => '╉',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => '╊',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, None) => '╋',

            // Double dashes
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, None, false, Some(Dashes::Double)) => '╌',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thick), None, None, false, Some(Dashes::Double)) => '╍',
            (None, None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, Some(Dashes::Double)) => '╎',
            (None, None, Some(SideAttribute::Thick), Some(SideAttribute::Thick), false, Some(Dashes::Double)) => '╏',

            // Mixed single/double line corners
            (None, Some(SideAttribute::Double), None, Some(SideAttribute::Thin), false, None) => '╒',
            (None, Some(SideAttribute::Thin), None, Some(SideAttribute::Double), false, None) => '╓',
            (None, Some(SideAttribute::Double), None, Some(SideAttribute::Double), false, None) => '╔',
            (Some(SideAttribute::Double), None, None, Some(SideAttribute::Thin), false, None) => '╕',
            (Some(SideAttribute::Thin), None, None, Some(SideAttribute::Double), false, None) => '╖',
            (Some(SideAttribute::Double), None, None, Some(SideAttribute::Double), false, None) => '╗',
            (None, Some(SideAttribute::Double), Some(SideAttribute::Thin), None, false, None) => '╘',
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Double), None, false, None) => '╙',
            (None, Some(SideAttribute::Double), Some(SideAttribute::Double), None, false, None) => '╚',
            (Some(SideAttribute::Double), None, Some(SideAttribute::Thin), None, false, None) => '╛',
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Double), None, false, None) => '╜',
            (Some(SideAttribute::Double), None, Some(SideAttribute::Double), None, false, None) => '╝',

            // Mixed single/double line T junctions
            (None, Some(SideAttribute::Double), Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '╞',
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Double), Some(SideAttribute::Double), false, None) => '╟',
            (None, Some(SideAttribute::Double), Some(SideAttribute::Double), Some(SideAttribute::Double), false, None) => '╠',
            (Some(SideAttribute::Double), None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '╡',
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Double), Some(SideAttribute::Double), false, None) => '╢',
            (Some(SideAttribute::Double), None, Some(SideAttribute::Double), Some(SideAttribute::Double), false, None) => '╣',
            (Some(SideAttribute::Double), Some(SideAttribute::Double), None, Some(SideAttribute::Thin), false, None) => '╤',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, Some(SideAttribute::Double), false, None) => '╥',
            (Some(SideAttribute::Double), Some(SideAttribute::Double), None, Some(SideAttribute::Double), false, None) => '╦',
            (Some(SideAttribute::Double), Some(SideAttribute::Double), Some(SideAttribute::Thin), None, false, None) => '╧',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Double), None, false, None) => '╨',
            (Some(SideAttribute::Double), Some(SideAttribute::Double), Some(SideAttribute::Double), None, false, None) => '╩',

            // Mixed single/double line cross junctions
            (Some(SideAttribute::Double), Some(SideAttribute::Double), Some(SideAttribute::Thin), Some(SideAttribute::Thin), false, None) => '╪',
            (Some(SideAttribute::Thin), Some(SideAttribute::Thin), Some(SideAttribute::Double), Some(SideAttribute::Double), false, None) => '╫',
            (Some(SideAttribute::Double), Some(SideAttribute::Double), Some(SideAttribute::Double), Some(SideAttribute::Double), false, None) => '╬',

            // Curved corners
            (None, Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), true, None) => '╭',
            (Some(SideAttribute::Thin), None, None, Some(SideAttribute::Thin), true, None) => '╮',
            (Some(SideAttribute::Thin), None, Some(SideAttribute::Thin), None, true, None) => '╯',
            (None, Some(SideAttribute::Thin), Some(SideAttribute::Thin), None, true, None) => '╰',

            // Line endings
            (Some(SideAttribute::Thin), None, None, None, false, None) => '╴',
            (None, None, Some(SideAttribute::Thin), None, false, None) => '╵',
            (None, Some(SideAttribute::Thin), None, None, false, None) => '╶',
            (None, None, None, Some(SideAttribute::Thin), false, None) => '╷',
            (Some(SideAttribute::Thick), None, None, None, false, None) => '╸',
            (None, None, Some(SideAttribute::Thick), None, false, None) => '╹',
            (None, Some(SideAttribute::Thick), None, None, false, None) => '╺',
            (None, None, None, Some(SideAttribute::Thick), false, None) => '╻',

            // Mixed thin/thick lines
            (Some(SideAttribute::Thin), Some(SideAttribute::Thick), None, None, false, None) => '╼',
            (None, None, Some(SideAttribute::Thin), Some(SideAttribute::Thick), false, None) => '╽',
            (Some(SideAttribute::Thick), Some(SideAttribute::Thin), None, None, false, None) => '╾',
            (None, None, Some(SideAttribute::Thick), Some(SideAttribute::Thin), false, None) => '╿',

            // If we can't find a match, return space
            _ => ' ',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_drawing_conversion() {
        let test_chars = " ─━│┃┄┅┆┇┈┉┊┋┌┍┎┏┐┑┒┓└┕┖┗┘┙┚┛├┝┞┟┠┡┢┣┤┥┦┧┨┩┪┫┬┭┮┯┰┱┲┳┴┵┶┷┸┹┺┻┼┽┾┿╀╁╂╃╄╅╆╇╈╉╊╋╌╍╎╏═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬╭╮╯╰╴╵╶╷╸╹╺╻╼╽╾╿";
        
        for ch in test_chars.chars() {
            if let Some(box_ch) = BoxDrawingCh::get_from_char(ch) {
                let result = box_ch.to_char();
                assert_eq!(result, ch, "Failed conversion for char: {}", ch);
            }
        }
    }
}
