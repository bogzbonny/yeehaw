pub mod font_ansi_reg;
pub mod font_ansi_shadow;
pub mod font_pagga;
pub mod font_stick;

use {crate::DrawChs2D, std::collections::HashMap};

// map the rune to the megafont glyph
pub struct Megafont {
    chs: HashMap<char, DrawChs2D>,
    height: u16, // height of the characters
}

impl Megafont {
    pub fn new(chs: HashMap<char, DrawChs2D>) -> Self {
        let mut height = 0;
        if let Some(mega_ch) = chs.get(&'a') {
            height = mega_ch.0.len();
        }
        Megafont {
            chs,
            height: height as u16,
        }
    }

    pub fn mega_ch(&self, r: char) -> DrawChs2D {
        self.chs[&r].clone()
    }

    pub fn get_mega_text(&self, input: &str) -> DrawChs2D {
        let mut out = DrawChs2D::default();
        for r in input.chars() {
            out = out
                .concat_left_right(self.mega_ch(r))
                .expect("mega text has inconsistent height");
        }
        out
    }
}

// -------------------------------------------------------
// common helpers

// take a vertical section of a [y][x]rune
// startX is inclusive, endX is exclusive
pub fn slice_from_group(rm: &Vec<Vec<char>>, start_x: usize, end_x: usize) -> Vec<Vec<char>> {
    let mut out = vec![];
    for rns in rm.iter() {
        out.push(rns[start_x..end_x].to_vec());
    }
    out
}

// FontInputGroup is a helper type for inputting multiple characters
// at once
//#[derive(Clone, Default)]

// XXX need to redo more like the golang code.
//                        (char     , glyph         , glyph-width)
//pub struct FontInputGroup((Vec<char>, Vec<Vec<char>>, Vec<u8)>);

//impl FontInputGroup {
//    pub fn add_glyphs_to_map(&mut self, map: &mut HashMap<char, DrawChs2D>, sty: Style) {
//        let mut gx = 0; // glyph x index
//        for (ch, glyphs, width) in self.0.iter_mut() {
//            let mut sliced_glyphs = slice_from_group(&glyphs, gx, gx + *width as usize);
//            map.insert(*ch, DrawChs2D::from_runes(&mut sliced_glyphs, sty));
//            gx += *width as usize;
//        }
//    }
//}
