pub mod from_ch;
pub mod to_ch;

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

// curves the edge of a box character if possible
pub fn add_curve_edge(input: char) -> char {
    todo!()
}

pub fn remove_curve_edge(input: char) -> char {
    todo!()
}

pub fn add_double_dash(input: char) -> char {
    todo!()
}

pub fn remove_double_dash(input: char) -> char {
    todo!()
}

pub fn add_triple_dash(input: char) -> char {
    todo!()
}

pub fn remove_triple_dash(input: char) -> char {
    todo!()
}

pub fn add_quadruple_dash(input: char) -> char {
    todo!()
}

pub fn remove_quadruple_dash(input: char) -> char {
    todo!()
}

pub fn make_thick(input: char) -> char {
    todo!()
}

pub fn make_thin(input: char) -> char {
    todo!()
}

pub fn make_double(input: char) -> char {
    todo!()
}

//// combinations
//pub struct BoxAdjContext {
//    left: char,
//    right: char,
//    up: char,
//    down: char,
//    connected_left: bool,
//    connected_right: bool,
//    connected_up: bool,
//    connected_down: bool,
//}

//// given the input box character, and the surrounding box characters, return the appropriate box character

//// adjust the "─" character to possibly have an upper or bottom connection
//fn adjust_light_horizontal(bctx: &BoxAdjContext) -> char {
//    let mut ch = '─';
//    if bctx.connected_up == true {
//        if bctx.connected_down == true {
//            return '─';
//        } else {
//            return '┬';
//        }
//    } else {
//        if bctx.connected_down == true {
//            return '┴';
//        } else {
//            return '─';
//        }
//    }
//}
