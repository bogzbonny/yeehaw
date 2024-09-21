use {
    crate::{Context, DynLocation, Size, Style},
    anyhow::{anyhow, Error},
    compact_str::CompactString,
    crossterm::style::{ContentStyle, StyledContent},
    std::ops::{Deref, DerefMut},
};

// DrawCh is a character with a style and transparency
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawCh {
    pub ch: ChPlus,
    pub style: Style,
}

/// ch+ more than just your regular char
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChPlus {
    Transparent, // no character, ch taken from underneath (NOTE fg and bg are still applied)
    Char(char),
    Str(CompactString),

    // skip this character entirely, useful for image viewers
    Skip,
}

// NOTE need to implement Default for DrawCh so that it is a space character
impl Default for DrawCh {
    fn default() -> DrawCh {
        DrawCh {
            ch: ChPlus::default(),
            style: Style::new(),
        }
    }
}

impl std::fmt::Display for ChPlus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChPlus::Transparent => write!(f, ""),
            ChPlus::Char(ch) => write!(f, "{}", ch),
            ChPlus::Str(s) => write!(f, "{}", s),
            ChPlus::Skip => write!(f, ""),
        }
    }
}

impl Default for ChPlus {
    fn default() -> Self {
        Self::Char(' ')
    }
}

impl From<char> for ChPlus {
    fn from(ch: char) -> ChPlus {
        ChPlus::Char(ch)
    }
}

impl From<&str> for ChPlus {
    fn from(s: &str) -> ChPlus {
        ChPlus::Str(CompactString::new(s))
    }
}

impl From<String> for ChPlus {
    fn from(s: String) -> ChPlus {
        ChPlus::Str(CompactString::new(&s))
    }
}

impl DrawCh {
    pub const fn const_new(ch: char, style: Style) -> DrawCh {
        DrawCh {
            ch: ChPlus::Char(ch),
            style,
        }
    }
    pub fn new<CH: Into<ChPlus>>(ch: CH, style: Style) -> DrawCh {
        DrawCh {
            ch: ch.into(),
            style,
        }
    }

    pub fn at(self, x: u16, y: u16) -> DrawChPos {
        DrawChPos { ch: self, x, y }
    }

    pub fn str_to_draw_chs(s: &str, sty: Style) -> Vec<DrawCh> {
        s.chars().map(|c| DrawCh::new(c, sty)).collect()
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
    pub fn adjust_by_dyn_location(&mut self, ctx: &Context, loc: &DynLocation) {
        let mut start_x = loc.get_start_x(ctx);
        let mut start_y = loc.get_start_y(ctx);
        // check for overflow
        if start_x < 0 {
            start_x = 0;
        }
        if start_y < 0 {
            start_y = 0;
        }
        self.x += start_x as u16;
        self.y += start_y as u16;
    }

    pub fn new_from_string(s: String, start_x: u16, start_y: u16, sty: Style) -> Vec<DrawChPos> {
        DrawChs2D::from_string(s.to_string(), sty).to_draw_ch_pos(start_x, start_y)
    }

    // get the content style for this DrawChPos given the underlying style
    pub fn get_content_style(&self, prev: &StyledContent<ChPlus>) -> StyledContent<ChPlus> {
        let (ch, attr) = if matches!(self.ch.ch, ChPlus::Transparent) {
            (prev.content(), prev.style().attributes)
        } else {
            (&self.ch.ch, self.ch.style.attr.into())
        };

        let (prev_fg, prev_bg, prev_ul) = (
            prev.style().foreground_color,
            prev.style().background_color,
            prev.style().background_color,
        );

        let fg = self.ch.style.fg.map(|fg| fg.to_crossterm_color(prev_fg));
        let bg = self.ch.style.bg.map(|bg| bg.to_crossterm_color(prev_bg));
        let ul = self
            .ch
            .style
            .underline
            .map(|ul| ul.to_crossterm_color(prev_ul));

        let cs = ContentStyle {
            foreground_color: fg,
            background_color: bg,
            underline_color: ul,
            attributes: attr,
        };
        StyledContent::new(cs, ch.clone())
    }
}

pub struct DrawChPosVec(pub Vec<DrawChPos>);
impl Deref for DrawChPosVec {
    type Target = Vec<DrawChPos>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DrawChPosVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<DrawChPos>> for DrawChPosVec {
    fn from(chs: Vec<DrawChPos>) -> DrawChPosVec {
        DrawChPosVec(chs)
    }
}

impl From<ratatui::buffer::Buffer> for DrawChPosVec {
    fn from(buf: ratatui::buffer::Buffer) -> Self {
        let mut out = Vec::new();

        buf.content.iter().enumerate().for_each(|(i, cell)| {
            let (x, y) = buf.pos_of(i);
            let mut ch: ChPlus = cell.symbol().into();

            if cell.skip {
                ch = ChPlus::Skip;
            }
            let sty = Style::from(cell.clone());
            out.push(DrawChPos::new(DrawCh::new(ch, sty), x, y));
        });
        DrawChPosVec(out)
    }
}

impl DrawChPosVec {
    pub fn new(chs: Vec<DrawChPos>) -> DrawChPosVec {
        DrawChPosVec(chs)
    }

    // adjust the position of the DrawChPos by the offset, if the position
    // is less than the offset or greater than max_x/y, then truncate the DrawChPos
    pub fn adjust_for_offset_and_truncate(
        &mut self, offset_x: usize, offset_y: usize, max_x: usize, max_y: usize,
    ) {
        let mut out: Vec<DrawChPos> = Vec::with_capacity(self.0.len());
        for mut ch in self.0.drain(..) {
            if (ch.x as usize) < offset_x
                || (ch.y as usize) < offset_y
                || (ch.x as usize) > max_x
                || (ch.y as usize) > max_y
            {
                continue;
            }
            ch.x = (ch.x as usize - offset_x) as u16;
            ch.y = (ch.y as usize - offset_y) as u16;
            out.push(ch);
        }
        self.0 = out;
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

    // filles a new empty DrawChs2D to the provided size
    pub fn new_empty_of_size(width: usize, height: usize, sty: Style) -> DrawChs2D {
        let mut out = Vec::new();
        for _ in 0..height {
            let mut line = Vec::new();
            for _ in 0..width {
                line.push(DrawCh::new(' ', sty));
            }
            out.push(line);
        }
        DrawChs2D(out)
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
                new_line.push(DrawCh::new(*c, sty));
            }
            out.push(new_line);
        }
        DrawChs2D(out)
    }

    pub fn to_draw_ch_pos(&self, start_x: u16, start_y: u16) -> Vec<DrawChPos> {
        let mut out = Vec::new();
        for (y, line) in self.0.iter().enumerate() {
            for (x, ch) in line.iter().enumerate() {
                out.push(DrawChPos::new(
                    ch.clone(),
                    start_x + x as u16,
                    start_y + y as u16,
                ));
            }
        }
        out
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
                new_chs[x].push(self.0[y][x].clone());
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

    // TODO fix text
    //#[test]
    //fn test_adjust_by_location() {
    //    let a = DrawCh::new('a', Style::new());
    //    let b = DrawCh::new('b', Style::new());
    //    let c = DrawCh::new('c', Style::new());
    //    let chs = vec![
    //        DrawChPos::new(a, 0, 0),
    //        DrawChPos::new(b, 1, 0),
    //        DrawChPos::new(c, 2, 3),
    //    ];
    //    let loc = Location::new(10, 20, 30, 40);

    //    let mut out = Vec::new();
    //    for mut ch in chs {
    //        ch.adjust_by_location(&loc);
    //        out.push(ch);
    //    }
    //    assert_eq!(out[0], DrawChPos::new(a, 10, 30));
    //    assert_eq!(out[1], DrawChPos::new(b, 11, 30));
    //    assert_eq!(out[2], DrawChPos::new(c, 12, 33));
    //}
}
