use {
    crate::{Location, Size, Style},
    anyhow::{anyhow, Error},
};

// DrawCh is a character with a style and transparency
#[derive(Clone)]
pub struct DrawCh {
    pub ch: char,
    pub transparent: bool, // aka do not draw this character
    pub style: Style,
}

impl DrawCh {
    pub fn new(ch: char, transparent: bool, style: Style) -> DrawCh {
        DrawCh {
            ch,
            transparent,
            style,
        }
    }
    pub fn at(self, x: u16, y: u16) -> DrawChPos {
        DrawChPos { ch: self, x, y }
    }

    pub fn str_to_draw_chs(s: &str, sty: Style) -> Vec<DrawCh> {
        s.chars().map(|c| DrawCh::new(c, false, sty)).collect()
    }
}

// ----------------------------------------------------

// DrawChPos is a DrawCh with an X and Y position
// The positions X and Y are local positions within the element
// and begin from 0 and count right and down respecively.
pub struct DrawChPos {
    pub ch: DrawCh,
    pub x: u16,
    pub y: u16,
}

impl DrawChPos {
    pub fn new(ch: DrawCh, x: u16, y: u16) -> DrawChPos {
        DrawChPos { ch, x, y }
    }
    pub fn adjust_by_location(&mut self, loc: Location) {
        self.x += loc.start_x as u16; // TODO check for overflow
        self.y += loc.start_y as u16;
    }
}

// ----------------------------------------------------

#[derive(Clone)]
pub struct DrawChs2D(Vec<Vec<DrawCh>>); // y, x

impl DrawChs2D {
    pub fn new(chs: Vec<Vec<DrawCh>>) -> DrawChs2D {
        DrawChs2D(chs)
    }

    pub fn from_string(text: String, sty: Style) -> DrawChs2D {
        let lines = text.split('\n');
        let mut chs = Vec::new();
        for line in lines {
            chs.push(DrawCh::str_to_draw_chs(line, sty));
        }
        DrawChs2D(chs)
    }

    pub fn from_runes(text: &mut Vec<Vec<char>>, sty: Style) -> DrawChs2D {
        let mut out = Vec::new();
        for line in text.iter_mut() {
            let mut new_line = Vec::new();
            for c in line.iter_mut() {
                new_line.push(DrawCh::new(*c, false, sty));
            }
            out.push(new_line);
        }
        DrawChs2D(out)
    }

    pub fn width(&self) -> usize {
        if self.0.is_empty() {
            return 0;
        }
        self.0[0].len()
    }
    pub fn height(&self) -> usize {
        self.0.len()
    }

    pub fn clone(&self) -> DrawChs2D {
        let mut out = Vec::new();
        for line in &self.0 {
            out.push(line.clone());
        }
        DrawChs2D(out)
    }

    pub fn size(&self) -> Size {
        Size::new(self.width() as u16, self.height() as u16)
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for line in &self.0 {
            for ch in line {
                out.push(ch.ch);
            }
            out.push('\n');
        }
        out
    }

    // TODO rename concat_right
    // concats the two arrays with self to the left of chs2
    pub fn concat_left_right(&self, chs2: DrawChs2D) -> Result<DrawChs2D, Error> {
        if self.0.is_empty() && !chs2.0.is_empty() {
            return Ok(chs2.clone());
        }
        if !self.0.is_empty() && chs2.0.is_empty() {
            return Ok(self.clone());
        }
        if self.height() != chs2.height() {
            return Err(anyhow!(
                "DrawChs2D ConcatRuneMatrices: chs.len() != chs2.len()",
            ));
        }
        let mut chs = self.clone();
        for i in 0..chs.height() {
            chs.0[i].append(&mut chs2.0[i].clone());
        }
        Ok(chs)
    }

    // concats the two arrays with chs on top of chs2
    pub fn concat_top_bottom(&self, chs2: DrawChs2D) -> DrawChs2D {
        if self.0.is_empty() && !chs2.0.is_empty() {
            return chs2.clone();
        }
        if !self.0.is_empty() && chs2.0.is_empty() {
            return self.clone();
        }
        let mut chs = self.clone();
        chs.0.append(&mut chs2.0.clone());
        chs
    }

    // Changes the style of all the characters in the array
    // at provided y line.
    pub fn change_style_at_xy(&mut self, x: usize, y: usize, sty: Style) {
        if y >= self.0.len() {
            return;
        }
        if x >= self.0[y].len() {
            return;
        }
        self.0[y][x].style = sty;
    }

    // Changes the style of all the characters in the array
    // at provided y line.
    pub fn change_style_along_y(&mut self, y: usize, sty: Style) {
        if y >= self.0.len() {
            return;
        }
        for x in 0..self.0[y].len() {
            self.0[y][x].style = sty;
        }
    }

    // Changes the style of all the characters in the array
    // at provided x column.
    pub fn change_style_along_x(&mut self, x: usize, sty: Style) {
        if x >= self.0[0].len() {
            return;
        }
        for y in 0..self.0.len() {
            self.0[y][x].style = sty;
        }
    }

    pub fn rotate_90_deg(&self) -> DrawChs2D {
        let mut new_chs = Vec::with_capacity(self.width());
        for _ in 0..self.width() {
            new_chs.push(Vec::with_capacity(self.height()));
        }
        for y in 0..self.height() {
            for x in 0..self.width() {
                new_chs[x].push(self.0[y][x].clone());
            }
        }
        DrawChs2D(new_chs)
    }
}
