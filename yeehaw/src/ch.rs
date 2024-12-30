use {
    crate::{BgTranspSrc, Color, Context, DynLocation, FgTranspSrc, Size, Style, UlTranspSrc},
    anyhow::{anyhow, Error},
    compact_str::CompactString,
    crossterm::style::{ContentStyle, StyledContent},
    std::ops::{Deref, DerefMut},
};

/// DrawCh is a character with a style and transparency
#[derive(Clone, Debug, PartialEq)]
pub struct DrawCh {
    pub ch: ChPlus,
    pub style: Style,
}

/// ch+ more than just your regular char
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChPlus {
    /// no character, ch taken from underneath (NOTE fg and bg are still applied)
    Transparent,
    /// regular character
    Char(char),
    /// more complex display information (useful for image protocols)
    Str(CompactString),
    /// skip this character entirely, useful for image viewers / mirroring ratatui buffer
    Skip,
}

/// NOTE need to implement Default for DrawCh so that it is a space character
impl Default for DrawCh {
    fn default() -> DrawCh {
        DrawCh {
            ch: ChPlus::default(),
            style: Style::default_const(),
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

    pub const fn transparent() -> DrawCh {
        DrawCh {
            ch: ChPlus::Transparent,
            style: Style::transparent(),
        }
    }

    pub const fn skip() -> DrawCh {
        DrawCh {
            ch: ChPlus::Skip,
            style: Style::transparent(),
        }
    }

    pub fn at(self, x: u16, y: u16) -> DrawChPos {
        DrawChPos { ch: self, x, y }
    }

    pub fn str_to_draw_chs(s: &str, sty: Style) -> Vec<DrawCh> {
        s.chars().map(|c| DrawCh::new(c, sty.clone())).collect()
    }

    pub fn overlay_style(&mut self, ctx: &Context, sty: &Style) {
        self.style.overlay_style(ctx, sty);
    }

    pub fn with_overlay_style(mut self, ctx: &Context, sty: &Style) -> Self {
        self.style.overlay_style(ctx, sty);
        self
    }
}

// ----------------------------------------------------

/// DrawChPos is a DrawCh with an X and Y position
/// The positions X and Y are local positions within the element
/// and begin from 0 and count right and down respecively.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawChPos {
    pub ch: DrawCh,
    pub x: u16,
    pub y: u16,
}

impl From<(DrawCh, u16, u16)> for DrawChPos {
    fn from((ch, x, y): (DrawCh, u16, u16)) -> DrawChPos {
        DrawChPos { ch, x, y }
    }
}

impl From<(DrawCh, usize, usize)> for DrawChPos {
    fn from((ch, x, y): (DrawCh, usize, usize)) -> DrawChPos {
        DrawChPos {
            ch,
            x: x as u16,
            y: y as u16,
        }
    }
}

impl DrawChPos {
    pub fn new(ch: DrawCh, x: u16, y: u16) -> DrawChPos {
        DrawChPos { ch, x, y }
    }
    pub fn adjust_by_dyn_location(&mut self, s: Size, loc: &DynLocation) {
        let mut start_x = loc.get_start_x_from_size(s);
        let mut start_y = loc.get_start_y_from_size(s);
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

    /// apply offsets to all position based colors, also set the draw size if unset
    pub fn set_draw_size_offset_colors(&mut self, s: Size, offset_x: i32, offset_y: i32) {
        self.ch
            .style
            .set_draw_size_offset_colors(s, offset_x, offset_y);
    }

    /// add to the offsets for any gradient colors in the style, used for scrollable areas
    pub fn add_to_offset_colors(&mut self, offset_x: i32, offset_y: i32) {
        self.ch.style.add_to_offset_colors(offset_x, offset_y);
    }

    pub fn new_from_string(s: String, start_x: u16, start_y: u16, sty: Style) -> Vec<DrawChPos> {
        DrawChs2D::from_string(s.to_string(), sty).to_draw_ch_pos(start_x, start_y)
    }

    pub fn new_repeated_vertical(
        ch: DrawCh, start_x: u16, start_y: u16, height: u16,
    ) -> Vec<DrawChPos> {
        let mut out = Vec::new();
        for y in 0..height {
            out.push(DrawChPos::new(ch.clone(), start_x, start_y + y));
        }
        out
    }

    pub fn new_repeated_horizontal(
        ch: DrawCh, start_x: u16, start_y: u16, width: u16,
    ) -> Vec<DrawChPos> {
        let mut out = Vec::new();
        for x in 0..width {
            out.push(DrawChPos::new(ch.clone(), start_x + x, start_y));
        }
        out
    }

    /// get the content style for this DrawChPos given the underlying style
    pub fn get_content_style(
        &self, ctx: &Context, draw_size: &Size, prev: &StyledContent<ChPlus>,
    ) -> StyledContent<ChPlus> {
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

        let bg = self.ch.style.bg.clone().map(|bg| {
            let transp_src = match bg.1 {
                BgTranspSrc::LowerFg => prev_fg,
                BgTranspSrc::LowerBg => prev_bg,
                BgTranspSrc::LowerUl => prev_ul,
            };
            bg.0.to_crossterm_color(ctx, draw_size, transp_src, self.x, self.y)
        });

        let fg = self.ch.style.fg.clone().map(|fg| {
            let transp_src = match fg.1 {
                FgTranspSrc::LowerFg => prev_fg,
                FgTranspSrc::LowerBg => prev_bg,
                FgTranspSrc::LowerUl => prev_ul,
                FgTranspSrc::ThisBg => bg,
            };
            fg.0.to_crossterm_color(ctx, draw_size, transp_src, self.x, self.y)
        });
        let ul = self.ch.style.underline_color.clone().map(|ul| {
            let transp_src = match ul.1 {
                UlTranspSrc::LowerFg => prev_fg,
                UlTranspSrc::LowerBg => prev_bg,
                UlTranspSrc::LowerUl => prev_ul,
                UlTranspSrc::ThisBg => bg,
            };
            ul.0.to_crossterm_color(ctx, draw_size, transp_src, self.x, self.y)
        });

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

impl From<ratatui::buffer::Buffer> for DrawChs2D {
    fn from(buf: ratatui::buffer::Buffer) -> Self {
        let mut out = Vec::new();
        for (i, cell) in buf.content.iter().enumerate() {
            let (x, y) = buf.pos_of(i);
            let mut ch: ChPlus = cell.symbol().into();

            if cell.skip {
                ch = ChPlus::Skip;
            }
            let sty = Style::from(cell.clone());
            let line = if let Some(line) = out.get_mut(y as usize) {
                line
            } else {
                out.push(Vec::new());
                out.last_mut().expect("impossible")
            };
            let x = x as usize;
            if x >= line.len() {
                line.resize(x + 1, DrawCh::default());
            }
            line[x] = DrawCh::new(ch, sty);
        }
        DrawChs2D(out)
    }
}

impl DrawChPosVec {
    pub fn new(chs: Vec<DrawChPos>) -> DrawChPosVec {
        DrawChPosVec(chs)
    }
}

// ----------------------------------------------------

#[derive(Clone, Default, Debug, PartialEq)]
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

impl From<Vec<Vec<DrawCh>>> for DrawChs2D {
    fn from(chs: Vec<Vec<DrawCh>>) -> DrawChs2D {
        DrawChs2D(chs)
    }
}

impl From<DrawCh> for DrawChs2D {
    fn from(chs: DrawCh) -> DrawChs2D {
        DrawChs2D(vec![vec![chs]])
    }
}

impl DrawChs2D {
    pub fn new(chs: Vec<Vec<DrawCh>>) -> DrawChs2D {
        DrawChs2D(chs)
    }

    /// filles a new empty DrawChs2D to the provided size
    pub fn new_empty_of_size(width: usize, height: usize, sty: Style) -> DrawChs2D {
        let mut out = Vec::new();
        for _ in 0..height {
            let mut line = Vec::new();
            for _ in 0..width {
                line.push(DrawCh::new(' ', sty.clone()));
            }
            out.push(line);
        }
        DrawChs2D(out)
    }

    pub fn from_string(text: String, sty: Style) -> DrawChs2D {
        let s = Size::get_text_size(&text);
        let mut out = Self::new_empty_of_size(s.width as usize, s.height as usize, sty.clone());

        let lines = text.lines();
        for (y, line) in lines.enumerate() {
            for (x, c) in line.chars().enumerate() {
                out[y][x] = DrawCh::new(c, sty.clone());
            }
        }
        out
    }

    pub fn from_char(ch: char, sty: Style) -> DrawChs2D {
        DrawChs2D(vec![vec![DrawCh::new(ch, sty.clone())]])
    }

    pub fn from_runes(text: &mut [Vec<char>], sty: Style) -> DrawChs2D {
        let mut out = Vec::new();
        for line in text.iter_mut() {
            let mut new_line = Vec::new();
            for c in line.iter_mut() {
                new_line.push(DrawCh::new(*c, sty.clone()));
            }
            out.push(new_line);
        }
        DrawChs2D(out)
    }

    pub fn from_draw_chs_horizontal(chs: Vec<DrawCh>) -> DrawChs2D {
        DrawChs2D(vec![chs])
    }

    pub fn from_draw_chs_vertical(chs: Vec<DrawCh>) -> DrawChs2D {
        let mut out = Vec::new();
        for ch in chs {
            out.push(vec![ch]);
        }
        DrawChs2D(out)
    }

    pub fn from_vec_draw_ch_pos(chs: Vec<DrawChPos>, default_ch: DrawCh) -> DrawChs2D {
        // get the max x and y
        let mut max_x = 0;
        let mut max_y = 0;
        for ch in chs.iter() {
            if ch.x > max_x {
                max_x = ch.x;
            }
            if ch.y > max_y {
                max_y = ch.y;
            }
        }
        let mut out = vec![vec![default_ch.clone(); max_x as usize + 1]; max_y as usize + 1];
        for ch in chs.iter() {
            out[ch.y as usize][ch.x as usize] = ch.ch.clone();
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

    pub fn apply_vec_draw_ch_pos(&mut self, chs: Vec<DrawChPos>) {
        for ch in chs {
            self.apply_draw_ch_pos(ch);
        }
    }

    pub fn apply_draw_ch_pos(&mut self, ch: DrawChPos) {
        self.set_ch(ch.x as usize, ch.y as usize, ch.ch);
    }

    pub fn set_ch(&mut self, x: usize, y: usize, ch: DrawCh) {
        let Some(line) = self.0.get_mut(y) else {
            return;
        };
        let Some(chplus) = line.get_mut(x) else {
            return;
        };
        *chplus = ch;
    }

    /// pads the DrawChs2D with the provided character added to the lefthand side the
    /// `amount` number of columns
    pub fn pad_left(&mut self, ch: DrawCh, amount: usize) {
        for line in self.0.iter_mut() {
            for _ in 0..amount {
                line.insert(0, ch.clone());
            }
        }
    }

    /// pads the DrawChs2D with the provided character added to the righthand side the
    /// `amount` number of columns
    pub fn pad_right(&mut self, ch: DrawCh, amount: usize) {
        for line in self.0.iter_mut() {
            for _ in 0..amount {
                line.push(ch.clone());
            }
        }
    }

    /// pads the DrawChs2D with the provided character added to the top the
    /// `amount` number of rows
    pub fn pad_top(&mut self, ch: DrawCh, amount: usize) {
        for _ in 0..amount {
            self.0.insert(0, vec![ch.clone(); self.width()]);
        }
    }

    /// pads the DrawChs2D with the provided character added to the bottom the
    /// `amount` number of rows
    pub fn pad_bottom(&mut self, ch: DrawCh, amount: usize) {
        for _ in 0..amount {
            self.0.push(vec![ch.clone(); self.width()]);
        }
    }

    /// removes the leftmost `amount` number of columns from the DrawChs2D
    pub fn remove_left(&mut self, amount: usize) {
        for line in self.0.iter_mut() {
            for _ in 0..amount {
                line.remove(0);
            }
        }
    }

    /// removes the rightmost `amount` number of columns from the DrawChs2D
    pub fn remove_right(&mut self, amount: usize) {
        for line in self.0.iter_mut() {
            for _ in 0..amount {
                line.pop();
            }
        }
    }

    /// removes the topmost `amount` number of rows from the DrawChs2D
    pub fn remove_top(&mut self, amount: usize) {
        for _ in 0..amount {
            self.0.remove(0);
        }
    }

    /// removes the bottommost `amount` number of rows from the DrawChs2D
    pub fn remove_bottom(&mut self, amount: usize) {
        for _ in 0..amount {
            self.0.pop();
        }
    }

    pub fn trim_bottom_whitespace(&mut self) {
        let mut trim_lines = 0;
        for line in self.0.iter().rev() {
            let mut is_whitespace = true;
            for ch in line.iter() {
                if !matches!(ch.ch, ChPlus::Char(' ')) {
                    is_whitespace = false;
                    break;
                }
            }
            if is_whitespace {
                trim_lines += 1;
            } else {
                break;
            }
        }
        self.remove_bottom(trim_lines);
    }

    pub fn trim_top_whitespace(&mut self) {
        let mut trim_lines = 0;
        for line in self.0.iter() {
            let mut is_whitespace = true;
            for ch in line.iter() {
                if !matches!(ch.ch, ChPlus::Char(' ')) {
                    is_whitespace = false;
                    break;
                }
            }
            if is_whitespace {
                trim_lines += 1;
            } else {
                break;
            }
        }
        self.remove_top(trim_lines);
    }

    // TODO rename concat_right
    /// concats the two arrays with self to the left of chs2
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

    /// concats the two arrays with chs on top of chs2
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

    /// Changes the style of all the characters in the array
    /// at provided y line.
    pub fn change_style_at_xy(&mut self, x: usize, y: usize, sty: Style) {
        if y >= self.0.len() {
            return;
        }
        if x >= self.0[y].len() {
            return;
        }
        self.0[y][x].style = sty;
    }

    /// Changes the style of all the characters in the array
    /// at provided y line.
    pub fn change_style_along_y(&mut self, y: usize, sty: Style) {
        if y >= self.0.len() {
            return;
        }
        for x in 0..self.0[y].len() {
            self.0[y][x].style = sty.clone();
        }
    }

    /// Changes the style of all the characters in the array
    /// at provided x column.
    pub fn change_style_along_x(&mut self, x: usize, sty: Style) {
        if x >= self.0[0].len() {
            return;
        }
        for y in 0..self.0.len() {
            self.0[y][x].style = sty.clone();
        }
    }

    pub fn change_all_styles(&mut self, sty: Style) {
        for y in 0..self.0.len() {
            for x in 0..self.0[y].len() {
                self.0[y][x].style = sty.clone();
            }
        }
    }

    pub fn change_all_bg(&mut self, bg: &Color) {
        for y in 0..self.0.len() {
            for x in 0..self.0[y].len() {
                self.0[y][x].style.set_bg(bg.clone());
            }
        }
    }

    pub fn change_all_fg(&mut self, fg: Color) {
        for y in 0..self.0.len() {
            for x in 0..self.0[y].len() {
                self.0[y][x].style.set_fg(fg.clone());
            }
        }
    }

    pub fn change_all_underline_color(&mut self, ul: Color) {
        for y in 0..self.0.len() {
            for x in 0..self.0[y].len() {
                self.0[y][x].style.set_underline_color(ul.clone());
            }
        }
    }

    pub fn overlay_all_styles(&mut self, ctx: &Context, sty: &Style) {
        for y in 0..self.0.len() {
            for x in 0..self.0[y].len() {
                self.0[y][x].overlay_style(ctx, sty);
            }
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
        let chs = DrawChs2D::from_string("abc\ndef".to_string(), Style::default_const());
        assert_eq!(chs.width(), 3);
        assert_eq!(chs.height(), 2);
        assert_eq!(chs.size(), Size::new(3, 2));
        assert_eq!(chs.to_string(), "abc\ndef");

        let chs2 = DrawChs2D::from_string("123\n456".to_string(), Style::default_const());
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
    //    let a = DrawCh::new('a', Style::default_const());
    //    let b = DrawCh::new('b', Style::default_const());
    //    let c = DrawCh::new('c', Style::default_const());
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
