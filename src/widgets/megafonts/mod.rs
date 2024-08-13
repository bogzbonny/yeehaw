pub mod font_ansi_reg;
pub mod font_ansi_shadow;
pub mod font_pagga;
pub mod font_stick;

use {
    crate::{DrawChs2D, Style},
    std::collections::HashMap,
};

// Reference Material:
// http://www.roysac.com/thedrawfonts-tdf.html
// https://www.patorjk.com/software/taag/#p=display&f=Graffiti&t=Type%20Something%20
// https://www.asciiart.eu/text-to-ascii-art

// map the rune to the megafont glyph
pub struct Megafont {
    chs: HashMap<char, DrawChs2D>,
    //height: u16, // height of the characters
}

impl Megafont {
    pub fn new(chs: HashMap<char, DrawChs2D>) -> Self {
        //let mut height = 0;
        //if let Some(mega_ch) = chs.get(&'a') {
        //    height = mega_ch.0.len();
        //}
        Megafont {
            chs,
            //height: height as u16,
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
pub fn slice_from_group(rm: &[Vec<char>], start_x: usize, end_x: usize) -> Vec<Vec<char>> {
    let mut out = vec![];
    for rns in rm.iter() {
        out.push(rns[start_x..end_x].to_vec());
    }
    out
}

// FontInputGroup is a helper type for inputting multiple characters
// at once
pub struct FontInputGroup {
    pub glyphs: Vec<Vec<char>>, // the glyphs grouped together
    pub widths: Vec<usize>,     // the width of each glyph character within the glyphs group
    pub chars: Vec<char>,       // actual characters within the glyphs
}

impl FontInputGroup {
    pub fn add_glyphs_to_map(&mut self, map: &mut HashMap<char, DrawChs2D>, sty: Style) {
        if self.chars.len() != self.widths.len() {
            panic!("length of Chars must match length of Widths");
        }

        // sum of widths check
        let sum = self.widths.iter().fold(0, |acc, x| acc + *x);
        if sum != self.glyphs[0].len() {
            panic!("sum of widths must equal the width of the glyph");
        }

        let mut gx = 0; // glyph x index
        for (i, ch) in self.chars.iter().enumerate() {
            let width = self.widths[i];
            let mut sliced_glyphs = slice_from_group(&self.glyphs, gx, gx + width);
            map.insert(*ch, DrawChs2D::from_runes(&mut sliced_glyphs, sty));
            gx += width;
        }
    }
}
