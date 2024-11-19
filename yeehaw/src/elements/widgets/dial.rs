use {
    super::arb_selector::SelChanges,
    crate::*,
    std::{cell::RefCell, rc::Rc},
};

/// Arbitrary Selector is a selector object which can be used to construct
/// cool selectors with arbitrary selection positions such as dials.
#[derive(Clone)]
pub struct Dial {
    pane: ArbSelector,
    dial_color: Rc<RefCell<Color>>,
    dial_knob_color: Rc<RefCell<Color>>,
    label_color: Rc<RefCell<Color>>,
    label_selected_color: Rc<RefCell<Style>>,
    labels: Rc<RefCell<Vec<(usize, String)>>>, // position and strings
    spacing: Rc<RefCell<Spacing>>,
}

impl Dial {
    const KIND: &'static str = "dial";

    const DEFAULT_DIAL_COLOR: Color = Color::AQUA;
    const DEFAULT_LABEL_COLOR: Color = Color::GREY13;
    const DEFAULT_LABEL_SEL_COLOR: Color = Color::YELLOW;

    /// create a new Dial, the Dial spacing will be chosen based on the number of labels
    pub fn new<S: Into<String>>(ctx: &Context, labels: Vec<S>) -> Self {
        let spacing = match labels.len() {
            0..=8 => Spacing::Compact,
            9..=12 => Spacing::SemiCompact,
            _ => panic!("Dial can only have 12 or fewer labels"), // TODO error
        };

        // assign positions to labels
        let labels: Vec<(usize, String)> = labels
            .into_iter()
            .enumerate()
            .map(|(i, s)| (i, s.into()))
            .collect();
        Self::new_inner(ctx, labels, spacing)
    }

    /// Create a new ArbSelector with the given drawing style and base drawing
    /// and the positions map. The positions map is a 2D array of letters where 'A' is the
    /// 1st position, 'B' is the 2nd position, etc.
    pub fn new_ultra_compact<S: Into<String>>(ctx: &Context, labels: Vec<(usize, S)>) -> Self {
        Self::new_inner(ctx, labels, Spacing::UltraCompact)
    }

    pub fn new_compact<S: Into<String>>(ctx: &Context, labels: Vec<(usize, S)>) -> Self {
        Self::new_inner(ctx, labels, Spacing::Compact)
    }

    pub fn new_semi_compact<S: Into<String>>(ctx: &Context, labels: Vec<(usize, S)>) -> Self {
        Self::new_inner(ctx, labels, Spacing::SemiCompact)
    }

    pub fn new_spacious<S: Into<String>>(ctx: &Context, labels: Vec<(usize, S)>) -> Self {
        Self::new_inner(ctx, labels, Spacing::Spacious)
    }

    fn new_inner<S: Into<String>>(
        ctx: &Context, labels: Vec<(usize, S)>, spacing: Spacing,
    ) -> Self {
        // convert labels to strings
        let labels: Vec<(usize, String)> = labels.into_iter().map(|(i, s)| (i, s.into())).collect();

        let pane = spacing.dial_arb_selector(
            ctx,
            Self::DEFAULT_DIAL_COLOR,
            Self::DEFAULT_DIAL_COLOR,
            Self::DEFAULT_LABEL_COLOR,
            Style::new_const(Self::DEFAULT_LABEL_SEL_COLOR, Color::BLACK),
            labels.clone(),
        );
        pane.pane.set_kind(Self::KIND);
        Self {
            pane,
            dial_color: Rc::new(RefCell::new(Self::DEFAULT_DIAL_COLOR)),
            dial_knob_color: Rc::new(RefCell::new(Self::DEFAULT_DIAL_COLOR)),
            label_color: Rc::new(RefCell::new(Self::DEFAULT_LABEL_COLOR)),
            label_selected_color: Rc::new(RefCell::new(Style::new_const(
                Self::DEFAULT_LABEL_SEL_COLOR,
                Color::BLACK,
            ))),
            labels: Rc::new(RefCell::new(labels)),
            spacing: Rc::new(RefCell::new(spacing.clone())),
        }
    }

    pub fn with_dial_color(mut self, ctx: &Context, dial_color: Color) -> Self {
        *self.dial_color.borrow_mut() = dial_color;
        self.reset_arb_selector(ctx);
        self
    }

    pub fn with_dial_knob_color(mut self, ctx: &Context, dial_knob_color: Color) -> Self {
        *self.dial_knob_color.borrow_mut() = dial_knob_color;
        self.reset_arb_selector(ctx);
        self
    }

    pub fn with_label_color(mut self, ctx: &Context, label_color: Color) -> Self {
        *self.label_color.borrow_mut() = label_color;
        self.reset_arb_selector(ctx);
        self
    }

    pub fn with_label_selected_color(mut self, ctx: &Context, label_selected_color: Style) -> Self {
        *self.label_selected_color.borrow_mut() = label_selected_color;
        self.reset_arb_selector(ctx);
        self
    }

    pub fn reset_arb_selector(&mut self, ctx: &Context) {
        let pane = self.spacing.borrow().dial_arb_selector(
            ctx,
            self.dial_color.borrow().clone(),
            self.dial_knob_color.borrow().clone(),
            self.label_color.borrow().clone(),
            Style::new_const(Self::DEFAULT_LABEL_SEL_COLOR, Color::BLACK),
            self.labels.borrow().clone(),
        );
        pane.pane.set_kind(Self::KIND);
        self.pane = pane;
    }

    pub fn at(self, x: DynVal, y: DynVal) -> Self {
        self.pane.pane.set_at(x, y);
        self
    }
}

/// dial spacing determines how the labels are drawn around the dial
#[derive(Clone)]
pub enum Spacing {
    /// ultra compact dial spacing:
    /// ```text
    ///   H__A  
    /// G ╱° ╲ B
    /// F ╲__╱ C
    ///   E  D
    /// ```
    UltraCompact,
    /// compact dial spacing:
    /// ```text
    ///   OptionH __ OptionA  
    /// OptionG  ╱° ╲  OptionB
    /// OptionF  ╲__╱  OptionC
    ///   OptionE    OptionD
    /// ```
    Compact,

    /// semi-compact dial spacing:
    /// ```text
    ///     OptionL  OptionA    
    ///  OptionK   __   OptionB
    /// OptionJ   ╱  ╲   OptionC
    /// OptionI   °__╱   OptionD
    ///  OptionH        OptionE
    ///     OptionG  OptionF    
    /// ```
    SemiCompact,
    /// spacious dial spacing:
    /// ```text
    ///      OptionL  OptionA     
    ///  OptionK    __    OptionB  
    /// OptionJ    ╱  ╲    OptionC
    /// OptionI    °__╱    OptionD
    ///  OptionH          OptionE  
    ///      OptionG  OptionF     
    /// ```
    Spacious,
}

impl Spacing {
    /// returns the starting x position of the dial
    /// option is the usize option position (OptionA is 0, OptionB is 1, etc.)
    pub fn start_x_from_center(&self, option: &usize) -> isize {
        match &self {
            Spacing::UltraCompact => match option {
                0 => 1,
                1 => 3,
                2 => 3,
                3 => 1,
                4 => -1,
                5 => -3,
                6 => -3,
                7 => -1,
                _ => panic!("Invalid option, UltraCompact dial only has 8 options"), // TODO error
            },
            Spacing::Compact => match option {
                0 => 2,
                1 => 4,
                2 => 4,
                3 => 2,
                4 => -2,
                5 => -4,
                6 => -4,
                7 => -2,
                _ => panic!("Invalid option, Compact dial only has 8 options"), // TODO error
            },
            Spacing::SemiCompact => match option {
                0 => 1,
                1 => 4,
                2 => 5,
                3 => 5,
                4 => 4,
                5 => 1,
                6 => -1,
                7 => -4,
                8 => -5,
                9 => -5,
                10 => -4,
                11 => -1,
                _ => panic!("Invalid option, SemiCompact dial only has 12 options"), // TODO error
            },
            Spacing::Spacious => match option {
                0 => 1,
                1 => 5,
                2 => 6,
                3 => 6,
                4 => 5,
                5 => 1,
                6 => -1,
                7 => -5,
                8 => -6,
                9 => -6,
                10 => -5,
                11 => -1,
                _ => panic!("Invalid option, Spacious dial only has 12 options"), // TODO error
            },
        }
    }

    /// the maximum width of the right hand side of the dial
    pub fn max_righthand_width(&self, labels: Vec<(usize, String)>) -> usize {
        match &self {
            Spacing::UltraCompact | Spacing::Compact => {
                let mut max_width = 2;
                for (i, label) in labels.iter() {
                    if *i < 4 {
                        let len = label.len() as isize + self.start_x_from_center(i);
                        max_width = max_width.max(len);
                    }
                }
                max_width as usize
            }
            Spacing::SemiCompact | Spacing::Spacious => {
                let mut max_width = 3;
                for (i, label) in labels.iter() {
                    if *i < 6 {
                        let len = label.len() as isize + self.start_x_from_center(i);
                        max_width = max_width.max(len);
                    }
                }
                max_width as usize
            }
        }
    }

    pub fn max_lefthand_width(&self, labels: Vec<(usize, String)>) -> usize {
        match &self {
            Spacing::UltraCompact | Spacing::Compact => {
                let mut max_width = 2;
                for (i, label) in labels.iter() {
                    if *i >= 4 {
                        let len = label.len() as isize - self.start_x_from_center(i);
                        max_width = max_width.max(len);
                    }
                }
                max_width as usize
            }
            Spacing::SemiCompact | Spacing::Spacious => {
                let mut max_width = 3;
                for (i, label) in labels.iter() {
                    if *i >= 6 {
                        let len = label.len() as isize - self.start_x_from_center(i);
                        max_width = max_width.max(len);
                    }
                }
                max_width as usize
            }
        }
    }

    /// create the dial arb selector from the labels
    #[rustfmt::skip]
    pub fn dial_arb_selector(
        &self,
        ctx: &Context,
        dial_color: Color,
        dial_knob_color: Color,
        label_color: Color,
        label_selected_color: Style,
        labels: Vec<(usize, String)>, // position and strings
    ) -> ArbSelector {
        // get the label with 0 as the first value
        let max_lh_width = self.max_lefthand_width(labels.clone());
        let max_rh_width = self.max_righthand_width(labels.clone());
        debug!("max_lh_width: {}, max_rh_width: {}", max_lh_width, max_rh_width);
        let a = labels
            .iter()
            .find_map(|(i, l)| if *i == 0 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let b = labels
            .iter()
            .find_map(|(i, l)| if *i == 1 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let c = labels
            .iter()
            .find_map(|(i, l)| if *i == 2 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let d = labels
            .iter()
            .find_map(|(i, l)| if *i == 3 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let e = labels
            .iter()
            .find_map(|(i, l)| if *i == 4 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let f = labels
            .iter()
            .find_map(|(i, l)| if *i == 5 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let g = labels
            .iter()
            .find_map(|(i, l)| if *i == 6 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let h = labels
            .iter()
            .find_map(|(i, l)| if *i == 7 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let i = labels
            .iter()
            .find_map(|(i, l)| if *i == 8 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let j = labels
            .iter()
            .find_map(|(i, l)| if *i == 9 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let k = labels
            .iter()
            .find_map(|(i, l)| if *i == 10 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());
        let l = labels
            .iter()
            .find_map(|(i, l)| if *i == 11 { Some(l.clone()) } else { None })
            .unwrap_or("".to_string());

        let label_sty = Style::transparent().with_fg(label_color);
        let a = DrawCh::str_to_draw_chs(&a, label_sty.clone());
        let b = DrawCh::str_to_draw_chs(&b, label_sty.clone());
        let c = DrawCh::str_to_draw_chs(&c, label_sty.clone());
        let d = DrawCh::str_to_draw_chs(&d, label_sty.clone());
        let e = DrawCh::str_to_draw_chs(&e, label_sty.clone());
        let f = DrawCh::str_to_draw_chs(&f, label_sty.clone());
        let g = DrawCh::str_to_draw_chs(&g, label_sty.clone());
        let h = DrawCh::str_to_draw_chs(&h, label_sty.clone());
        let i = DrawCh::str_to_draw_chs(&i, label_sty.clone());
        let j = DrawCh::str_to_draw_chs(&j, label_sty.clone());
        let k = DrawCh::str_to_draw_chs(&k, label_sty.clone());
        let l = DrawCh::str_to_draw_chs(&l, label_sty.clone());
        let has_a = !a.is_empty();
        let has_b = !b.is_empty();
        let has_c = !c.is_empty();
        let has_d = !d.is_empty();
        let has_e = !e.is_empty();
        let has_f = !f.is_empty();
        let has_g = !g.is_empty();
        let has_h = !h.is_empty();
        let has_i = !i.is_empty();
        let has_j = !j.is_empty();
        let has_k = !k.is_empty();
        let has_l = !l.is_empty();
        let a_ch = if has_a {"A"} else {"0"};
        let b_ch = if has_b {"B"} else {"0"};
        let c_ch = if has_c {"C"} else {"0"};
        let d_ch = if has_d {"D"} else {"0"};
        let e_ch = if has_e {"E"} else {"0"};
        let f_ch = if has_f {"F"} else {"0"};
        let g_ch = if has_g {"G"} else {"0"};
        let h_ch = if has_h {"H"} else {"0"};
        let i_ch = if has_i {"I"} else {"0"};
        let j_ch = if has_j {"J"} else {"0"};
        let k_ch = if has_k {"K"} else {"0"};
        let l_ch = if has_l {"L"} else {"0"};

        let dial_knob_sty = Style::transparent().with_fg(dial_knob_color);
        let dial_sty = Style::transparent().with_fg(dial_color);
        let dial_y0 =  DrawCh::str_to_draw_chs("__", dial_sty.clone());
        let dial_y1 = DrawCh::str_to_draw_chs("╱  ╲", dial_sty.clone());
        let dial_y2 = DrawCh::str_to_draw_chs("╲__╱", dial_sty.clone());

        match self {
            Spacing::UltraCompact => {

                let max_lh_width = max_lh_width.max(3);
                let max_rh_width = max_rh_width.max(3);

                let a_spc_len = max_rh_width.saturating_sub(a.len()).saturating_sub(self.start_x_from_center(&0) as usize);
                let b_spc_len = max_rh_width.saturating_sub(b.len()).saturating_sub(self.start_x_from_center(&1) as usize);
                let c_spc_len = max_rh_width.saturating_sub(c.len()).saturating_sub(self.start_x_from_center(&2)as usize);
                let d_spc_len = max_rh_width.saturating_sub(d.len()).saturating_sub(self.start_x_from_center(&3) as usize);
                let e_spc_len = max_lh_width.saturating_sub(e.len()).saturating_sub((-self.start_x_from_center(&4)) as usize);
                let f_spc_len = max_lh_width.saturating_sub(f.len()).saturating_sub((-self.start_x_from_center(&5)) as usize);
                let g_spc_len = max_lh_width.saturating_sub(g.len()).saturating_sub((-self.start_x_from_center(&6)) as usize);
                let h_spc_len = max_lh_width.saturating_sub(h.len()).saturating_sub((-self.start_x_from_center(&7)) as usize);
                let a_spc = DrawCh::str_to_draw_chs(&" ".repeat(a_spc_len), Style::transparent());
                let b_spc = DrawCh::str_to_draw_chs(&" ".repeat(b_spc_len), Style::transparent());
                let c_spc = DrawCh::str_to_draw_chs(&" ".repeat(c_spc_len), Style::transparent());
                let d_spc = DrawCh::str_to_draw_chs(&" ".repeat(d_spc_len), Style::transparent());
                let e_spc = DrawCh::str_to_draw_chs(&" ".repeat(e_spc_len), Style::transparent());
                let f_spc = DrawCh::str_to_draw_chs(&" ".repeat(f_spc_len), Style::transparent());
                let g_spc = DrawCh::str_to_draw_chs(&" ".repeat(g_spc_len), Style::transparent());
                let h_spc = DrawCh::str_to_draw_chs(&" ".repeat(h_spc_len), Style::transparent());
                let spc_1 = DrawCh::str_to_draw_chs(" ", Style::transparent()); // one space

                debug!("a_spc_len: {}, b_spc_len: {}, c_spc_len: {}, d_spc_len: {}, e_spc_len: {},
                    f_spc_len: {}, g_spc_len: {}, h_spc_len: {}", a_spc_len, b_spc_len, c_spc_len,
                    d_spc_len, e_spc_len, f_spc_len, g_spc_len, h_spc_len);

                let y0 = [h_spc.clone(), h.clone(), dial_y0.clone(), a.clone(), a_spc.clone()].concat();
                let y1 = [g_spc.clone(), g.clone(), spc_1.clone(), dial_y1.clone(), spc_1.clone(), b.clone(), b_spc.clone()].concat();
                let y2 = [f_spc.clone(), f.clone(), spc_1.clone(), dial_y2.clone(), spc_1.clone(), c.clone(), c_spc.clone()].concat();
                let y3 = [e_spc.clone(), e.clone(), spc_1.clone(), spc_1.clone(), d.clone(), d_spc.clone()].concat();
                let y0 = DrawChs2D::from_draw_chs_horizontal(y0);
                let y1 = DrawChs2D::from_draw_chs_horizontal(y1);
                let y2 = DrawChs2D::from_draw_chs_horizontal(y2);
                let y3 = DrawChs2D::from_draw_chs_horizontal(y3);
                let mut base = y0.concat_top_bottom(y1)
                  .concat_top_bottom(y2)
                  .concat_top_bottom(y3);

                let map_len_b_y0 = a_spc_len * 3 / 5;
                let map_len_a_y0 = a.len() + a_spc_len - map_len_b_y0;
                let map_len_b_y1 = b.len() + b_spc_len;
                let map_len_c_y2 = c.len() + c_spc_len;
                let map_len_c_y3 = d_spc_len * 3 / 5;
                let map_len_d_y3 = d.len() + d_spc_len - map_len_c_y3;

                let map_len_g_y0 = h_spc_len * 3 / 5;
                let map_len_h_y0 = h.len() + h_spc_len - map_len_g_y0;
                debug!("map_len_g_y0: {}", map_len_g_y0);
                debug!("map_len_h_y0: {}", map_len_h_y0);

                let map_len_g_y1 = g.len() + g_spc_len;
                let map_len_f_y2 = f.len() + f_spc_len;
                let map_len_f_y3 = e_spc_len * 3 / 5;
                let map_len_e_y3 = e.len() + e_spc_len - map_len_f_y3;


                let map_a_y0 = a_ch.repeat(map_len_a_y0);
                let map_b_y0 = b_ch.repeat(map_len_b_y0);
                let map_b_y1 = b_ch.repeat(map_len_b_y1);
                let map_c_y2 = c_ch.repeat(map_len_c_y2);
                let map_c_y3 = c_ch.repeat(map_len_c_y3);
                let map_d_y3 = d_ch.repeat(map_len_d_y3);
                let map_e_y3 = e_ch.repeat(map_len_e_y3);
                let map_f_y3 = f_ch.repeat(map_len_f_y3);
                let map_f_y2 = f_ch.repeat(map_len_f_y2);
                let map_g_y1 = g_ch.repeat(map_len_g_y1);
                let map_g_y0 = g_ch.repeat(map_len_g_y0);
                let map_h_y0 = h_ch.repeat(map_len_h_y0);

                let mut pos_map_str = format!("{map_g_y0}{map_h_y0}{h_ch}{a_ch}{map_a_y0}{map_b_y0}\n")
                                       + &format!("{map_g_y1}{g_ch}{g_ch}{h_ch}{a_ch}{b_ch}{b_ch}{map_b_y1}\n")
                                       + &format!("{map_f_y2}{f_ch}{f_ch}{e_ch}{d_ch}{c_ch}{c_ch}{map_c_y2}\n")
                               + &format!("{map_f_y3}{map_e_y3}{e_ch}{d_ch}{map_d_y3}{map_c_y3}");

                let mut sel_changes = Vec::new();
                if has_a {
                    let mut a_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width + self.start_x_from_center(&0) as usize;
                    for ch in a.iter() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        a_sel_changes.push((ch, x, 0).into());
                        x += 1;
                    }
                    let x = max_lh_width;
                    let ch = DrawCh::new('°', dial_knob_sty.clone());
                    a_sel_changes.push((ch, x, 1).into());
                    sel_changes.push((0, a_sel_changes));
                }
                if has_b {
                    let mut b_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width + self.start_x_from_center(&1) as usize;
                    for ch in b.iter() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        b_sel_changes.push((ch, x, 1).into());
                        x += 1;
                    }
                    let x = max_lh_width + 1;
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
                    b_sel_changes.push((ch, x, 1).into());
                    sel_changes.push((1, b_sel_changes));
                }
                if has_c {
                    let mut c_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width + self.start_x_from_center(&2) as usize;
                    for ch in c.iter() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        c_sel_changes.push((ch, x, 2).into());
                        x += 1;
                    }
                    let x = max_lh_width + 1;
                    let ch = DrawCh::new('°', dial_knob_sty.clone());
                    c_sel_changes.push((ch, x, 2).into());
                    sel_changes.push((2, c_sel_changes));
                }
                if has_d {
                    let mut d_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width + self.start_x_from_center(&3) as usize;
                    for ch in d.iter() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        d_sel_changes.push((ch, x, 3).into());
                        x += 1;
                    }
                    let x = max_lh_width;
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
                    d_sel_changes.push((ch, x, 2).into());
                    sel_changes.push((3, d_sel_changes));
                }
                if has_e {
                    let mut e_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&4)) as usize;
                    for ch in e.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        e_sel_changes.push((ch, x, 3).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 1;
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
                    e_sel_changes.push((ch, x, 2).into());
                    sel_changes.push((4, e_sel_changes));
                }
                if has_f {
                    let mut f_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&5)) as usize;
                    for ch in f.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        f_sel_changes.push((ch, x, 2).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 2;
                    let ch = DrawCh::new('°', dial_knob_sty.clone());
                    f_sel_changes.push((ch, x, 2).into());
                    sel_changes.push((5, f_sel_changes));
                }
                if has_g {
                    let mut g_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&6)) as usize;
                    for ch in g.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        g_sel_changes.push((ch, x, 1).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 2;
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
                    g_sel_changes.push((ch, x, 1).into());
                    sel_changes.push((6, g_sel_changes));
                }
                if has_h {
                    let mut h_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&7)) as usize;
                    for ch in h.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        h_sel_changes.push((ch, x, 0).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 1;
                    let ch = DrawCh::new('°', dial_knob_sty.clone());
                    h_sel_changes.push((ch, x, 1).into());
                    sel_changes.push((7, h_sel_changes));
                }


                // remove one right hand column if there are no right hand labels
                if !has_a && !has_b && !has_c && !has_d {
                    base.remove_right(1);
                    pos_map_str = pos_map_str.split("\n").map(|line| {
                        let mut line = line.to_string();
                        line.pop();
                        line
                    }).collect::<Vec<String>>().join("\n");
                }

                // remove one left hand column if there are no left hand labels
                if !has_e && !has_f && !has_g && !has_h {
                    base.remove_left(1);
                    pos_map_str = pos_map_str.split("\n").map(|line| {
                        let mut line = line.to_string();
                        line.remove(0);
                        line
                    }).collect::<Vec<String>>().join("\n");

                    for sel_change in sel_changes.iter_mut() {
                        for ch in sel_change.1.iter_mut() {
                            ch.x -= 1;
                        }
                    }
                }

                // remove the bottom of base and pos_map_str if there are no bottom labels
                if !has_e && !has_d {
                    base.remove_bottom(1);
                    pos_map_str = pos_map_str.split("\n").take(3).collect::<Vec<&str>>().join("\n");
                }

                debug!("pos_map_str: \n{}", pos_map_str);
                debug!("base: \n{}", base);

                let pos_map = ArbSelector::positions_string_to_map(&pos_map_str);
                ArbSelector::new_inner(ctx, base, pos_map, sel_changes)
            }
            Spacing::Compact => {
                //let a_spc = " ".repeat(max_rh_width - a.len() - self.start_x_from_center(&0) as usize);
                //let b_spc = " ".repeat(max_rh_width - b.len() - self.start_x_from_center(&1) as usize);
                //let c_spc = " ".repeat(max_rh_width - c.len() - self.start_x_from_center(&2)as usize);
                //let d_spc = " ".repeat(max_rh_width - d.len() - self.start_x_from_center(&3) as usize);
                //let e_spc = " ".repeat(max_lh_width - e.len() - ((-self.start_x_from_center(&4)) as usize));
                //let f_spc = " ".repeat(max_lh_width - f.len() - ((-self.start_x_from_center(&5)) as usize));
                //let g_spc = " ".repeat(max_lh_width - g.len() - ((-self.start_x_from_center(&6)) as usize));
                //let h_spc = " ".repeat(max_lh_width - h.len() - ((-self.start_x_from_center(&7)) as usize));
                //let dial_chs = format!("{h_spc}{h} __ {a}{a_spc}\n")
                //          + &format!("{g_spc}{g}  ╱  ╲  {b}{b_spc}\n")
                //          + &format!("{f_spc}{f}  ╲__╱  {c}{c_spc}\n")
                //            + &format!("{e_spc}{e}    {d}{d_spc}");
                //dial_chs
                todo!()
            }
            Spacing::SemiCompact => {
                //let a_spc = " ".repeat(max_rh_width - a.len() - self.start_x_from_center(&0) as usize);
                //let b_spc = " ".repeat(max_rh_width - b.len() - self.start_x_from_center(&1) as usize);
                //let c_spc = " ".repeat(max_rh_width - c.len() - self.start_x_from_center(&2)as usize);
                //let d_spc = " ".repeat(max_rh_width - d.len() - self.start_x_from_center(&3) as usize);
                //let e_spc = " ".repeat(max_lh_width - e.len() - self.start_x_from_center(&4) as usize);
                //let f_spc = " ".repeat(max_lh_width - f.len() - self.start_x_from_center(&5) as usize);
                //let g_spc = " ".repeat(max_lh_width - g.len() - ((-self.start_x_from_center(&6)) as usize));
                //let h_spc = " ".repeat(max_lh_width - h.len() - ((-self.start_x_from_center(&7)) as usize));
                //let i_spc = " ".repeat(max_lh_width - i.len() - ((-self.start_x_from_center(&8)) as usize));
                //let j_spc = " ".repeat(max_lh_width - j.len() - ((-self.start_x_from_center(&9)) as usize));
                //let k_spc = " ".repeat(max_lh_width - k.len() - ((-self.start_x_from_center(&10)) as usize));
                //let l_spc = " ".repeat(max_lh_width - l.len() - ((-self.start_x_from_center(&11)) as usize));
                //let dial_chs = format!("{l_spc}{l}  {a}{a_spc}\n")
                //         + &format!("{k_spc}{k}   __   {b}{b_spc}\n")
                //        + &format!("{j_spc}{j}   ╱  ╲   {c}{c_spc}\n")
                //        + &format!("{i_spc}{i}   ╲__╱   {d}{d_spc}\n")
                //         + &format!("{h_spc}{h}        {e}{e_spc}\n")
                //            + &format!("{g_spc}{g}  {f}{f_spc}");
                //dial_chs
                todo!()
            }
            Spacing::Spacious => {
                //let a_spc = " ".repeat(max_rh_width - a.len() - self.start_x_from_center(&0) as usize);
                //let b_spc = " ".repeat(max_rh_width - b.len() - self.start_x_from_center(&1) as usize);
                //let c_spc = " ".repeat(max_rh_width - c.len() - self.start_x_from_center(&2)as usize);
                //let d_spc = " ".repeat(max_rh_width - d.len() - self.start_x_from_center(&3) as usize);
                //let e_spc = " ".repeat(max_lh_width - e.len() - self.start_x_from_center(&4) as usize);
                //let f_spc = " ".repeat(max_lh_width - f.len() - self.start_x_from_center(&5) as usize);
                //let g_spc = " ".repeat(max_lh_width - g.len() - ((-self.start_x_from_center(&6)) as usize));
                //let h_spc = " ".repeat(max_lh_width - h.len() - ((-self.start_x_from_center(&7)) as usize));
                //let i_spc = " ".repeat(max_lh_width - i.len() - ((-self.start_x_from_center(&8)) as usize));
                //let j_spc = " ".repeat(max_lh_width - j.len() - ((-self.start_x_from_center(&9)) as usize));
                //let k_spc = " ".repeat(max_lh_width - k.len() - ((-self.start_x_from_center(&10)) as usize));
                //let l_spc = " ".repeat(max_lh_width - l.len() - ((-self.start_x_from_center(&11)) as usize));
                //let dial_chs = format!("{l_spc}{l}  {a}{a_spc}\n")
                //        + &format!("{k_spc}{k}    __    {b}{b_spc}\n")
                //       + &format!("{j_spc}{j}    ╱  ╲    {c}{c_spc}\n")
                //       + &format!("{i_spc}{i}    ╲__╱    {d}{d_spc}\n")
                //        + &format!("{h_spc}{h}          {e}{e_spc}\n")
                //            + &format!("{g_spc}{g}  {f}{f_spc}");
                //dial_chs
                todo!()
            }
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Dial {}
