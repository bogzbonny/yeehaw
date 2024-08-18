use {
    crate::{Location, Size, Style},
    anyhow::{anyhow, Error},
    std::ops::{Deref, DerefMut},
};

// DrawCh is a character with a style and transparency
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DrawCh {
    pub ch: char,
    pub transparent: bool, // aka do not draw this character
    pub style: Style,
}

// NOTE need to implement Default for DrawCh so that it is a space character
impl Default for DrawCh {
    fn default() -> DrawCh {
        DrawCh {
            ch: ' ',
            transparent: false,
            style: Style::new(),
        }
    }
}

impl DrawCh {
    pub const fn new(ch: char, transparent: bool, style: Style) -> DrawCh {
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawChPos {
    pub ch: DrawCh,
    pub x: u16,
    pub y: u16,
}

impl DrawChPos {
    pub fn new(ch: DrawCh, x: u16, y: u16) -> DrawChPos {
        DrawChPos { ch, x, y }
    }
    pub fn adjust_by_location(&mut self, loc: &Location) {
        self.x += loc.start_x as u16; // TODO check for overflow
        self.y += loc.start_y as u16;
    }
}

// ----------------------------------------------------

#[derive(Clone, Default)]
pub struct DrawChs2D(pub Vec<Vec<DrawCh>>); // y, x

impl Deref for DrawChs2D {
    type Target = Vec<Vec<DrawCh>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DrawChs2D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for DrawChs2D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, line) in self.0.iter().enumerate() {
            for ch in line {
                write!(f, "{}", ch.ch)?;
            }
            if i < self.0.len() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

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

    pub fn from_runes(text: &mut [Vec<char>], sty: Style) -> DrawChs2D {
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

    // filles a new empty DrawChs2D to the provided size
    pub fn new_empty_of_size(width: usize, height: usize, sty: Style) -> DrawChs2D {
        let mut out = Vec::new();
        for _ in 0..height {
            let mut line = Vec::new();
            for _ in 0..width {
                line.push(DrawCh::new(' ', false, sty));
            }
            out.push(line);
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

    pub fn size(&self) -> Size {
        Size::new(self.width() as u16, self.height() as u16)
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

    #[allow(clippy::needless_range_loop)]
    pub fn rotate_90_deg(&self) -> DrawChs2D {
        let mut new_chs = Vec::with_capacity(self.width());
        for _ in 0..self.width() {
            new_chs.push(Vec::with_capacity(self.height()));
        }
        for y in 0..self.height() {
            for x in 0..self.width() {
                new_chs[x].push(self.0[y][x]);
            }
        }
        DrawChs2D(new_chs)
    }
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_chs2d() {
        let chs = DrawChs2D::from_string("abc\ndef".to_string(), Style::new());
        assert_eq!(chs.width(), 3);
        assert_eq!(chs.height(), 2);
        assert_eq!(chs.size(), Size::new(3, 2));
        assert_eq!(chs.to_string(), "abc\ndef");

        let chs2 = DrawChs2D::from_string("123\n456".to_string(), Style::new());
        assert_eq!(chs2.width(), 3);
        assert_eq!(chs2.height(), 2);
        assert_eq!(chs2.size(), Size::new(3, 2));
        assert_eq!(chs2.to_string(), "123\n456");

        let chs3 = chs.concat_top_bottom(chs2);
        assert_eq!(chs3.width(), 3);
        assert_eq!(chs3.height(), 4);
        assert_eq!(chs3.size(), Size::new(3, 4));
        assert_eq!(chs3.to_string(), "abc\ndef\n123\n456");
    }

    #[test]
    fn test_adjust_by_location() {
        let a = DrawCh::new('a', false, Style::new());
        let b = DrawCh::new('b', false, Style::new());
        let c = DrawCh::new('c', false, Style::new());
        let chs = vec![
            DrawChPos::new(a, 0, 0),
            DrawChPos::new(b, 1, 0),
            DrawChPos::new(c, 2, 3),
        ];
        let loc = Location::new(10, 20, 30, 40);

        let mut out = Vec::new();
        for mut ch in chs {
            ch.adjust_by_location(&loc);
            out.push(ch);
        }
        assert_eq!(out[0], DrawChPos::new(a, 10, 30));
        assert_eq!(out[1], DrawChPos::new(b, 11, 30));
        assert_eq!(out[2], DrawChPos::new(c, 12, 33));
    }
}
