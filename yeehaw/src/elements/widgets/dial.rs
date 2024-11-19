use {
    super::arb_selector::SelChanges,
    crate::*,
    std::{cell::RefCell, rc::Rc},
};

// welcome to heck
////////////////

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
    const DEFAULT_DIAL_KNOB_COLOR: Color = Color::LIME;
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
            Self::DEFAULT_DIAL_KNOB_COLOR,
            Self::DEFAULT_LABEL_COLOR,
            Style::new_const(Self::DEFAULT_LABEL_SEL_COLOR, Color::BLACK),
            labels.clone(),
        );
        pane.pane.set_kind(Self::KIND);
        Self {
            pane,
            dial_color: Rc::new(RefCell::new(Self::DEFAULT_DIAL_COLOR)),
            dial_knob_color: Rc::new(RefCell::new(Self::DEFAULT_DIAL_KNOB_COLOR)),
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
    ///   OptionK   __   OptionB  
    /// OptionJ    ╱  ╲    OptionC
    /// OptionI    °__╱    OptionD
    ///   OptionH        OptionE  
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
                1 => 4,
                2 => 6,
                3 => 6,
                4 => 4,
                5 => 1,
                6 => -1,
                7 => -4,
                8 => -6,
                9 => -6,
                10 => -4,
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
        //debug!("max_lh_width: {}, max_rh_width: {}", max_lh_width, max_rh_width);
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
            Spacing::UltraCompact | Spacing::Compact => {
                
                let ultra = matches!(self, Spacing::UltraCompact);

                let (max_lh_width, max_rh_width) = if ultra {
                    (max_lh_width.max(3), max_rh_width.max(3))
                } else {
                    (max_lh_width.max(4), max_rh_width.max(4))  
                };

                let a_spc_len = max_rh_width.saturating_sub(a.len()).saturating_sub(self.start_x_from_center(&0) as usize);
                let b_spc_len = max_rh_width.saturating_sub(b.len()).saturating_sub(self.start_x_from_center(&1) as usize);
                let c_spc_len = max_rh_width.saturating_sub(c.len()).saturating_sub(self.start_x_from_center(&2) as usize);
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
                let spc_1 = DrawCh::str_to_draw_chs(" ", Style::transparent()); 
                let spc_2 = DrawCh::str_to_draw_chs(&" ".repeat(2), Style::transparent()); 
                let spc_4 = DrawCh::str_to_draw_chs(&" ".repeat(4), Style::transparent()); 

                //    H__A  
                //  G ╱° ╲ B
                //  F ╲__╱ C
                //    E  D
                //    OptionH __ OptionA  
                //  OptionG  ╱° ╲  OptionB
                //  OptionF  ╲__╱  OptionC
                //    OptionE    OptionD

                let (y0, y1, y2, y3) = if ultra {
                    ([h_spc.clone(), h.clone(), dial_y0.clone(), a.clone(), a_spc.clone()].concat(),
                     [g_spc.clone(), g.clone(), spc_1.clone(), dial_y1.clone(), spc_1.clone(), b.clone(), b_spc.clone()].concat(),
                     [f_spc.clone(), f.clone(), spc_1.clone(), dial_y2.clone(), spc_1.clone(), c.clone(), c_spc.clone()].concat(),
                     [e_spc.clone(), e.clone(), spc_2.clone(), d.clone(), d_spc.clone()].concat())
                } else {
                    ([h_spc.clone(), h.clone(), spc_1.clone(), dial_y0.clone(),spc_1.clone(), a.clone(), a_spc.clone()].concat(),
                     [g_spc.clone(), g.clone(), spc_2.clone(), dial_y1.clone(), spc_2.clone(), b.clone(), b_spc.clone()].concat(),
                     [f_spc.clone(), f.clone(), spc_2.clone(), dial_y2.clone(), spc_2.clone(), c.clone(), c_spc.clone()].concat(),
                     [e_spc.clone(), e.clone(), spc_4.clone(), d.clone(), d_spc.clone()].concat())
                };
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

                let mut pos_map_str = if ultra {
                         format!("{map_g_y0}{map_h_y0}{h_ch}{a_ch}{map_a_y0}{map_b_y0}\n")
                    + &format!("{map_g_y1}{g_ch}{g_ch}{h_ch}{a_ch}{b_ch}{b_ch}{map_b_y1}\n")
                    + &format!("{map_f_y2}{f_ch}{f_ch}{e_ch}{d_ch}{c_ch}{c_ch}{map_c_y2}\n")
                      + &format!("{map_f_y3}{map_e_y3}{e_ch}{d_ch}{map_d_y3}{map_c_y3}")
                } else {
                         format!("{map_g_y0}{map_h_y0}{h_ch}{h_ch}{a_ch}{a_ch}{map_a_y0}{map_b_y0}\n")
                    + &format!("{map_g_y1}{g_ch}{g_ch}{g_ch}{h_ch}{a_ch}{b_ch}{b_ch}{b_ch}{map_b_y1}\n")
                    + &format!("{map_f_y2}{f_ch}{f_ch}{f_ch}{e_ch}{d_ch}{c_ch}{c_ch}{c_ch}{map_c_y2}\n")
                      + &format!("{map_f_y3}{map_e_y3}{e_ch}{e_ch}{d_ch}{d_ch}{map_d_y3}{map_c_y3}")
                };

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


                // remove 1 or 2 right hand column if there are no right hand labels
                if !has_a && !has_b && !has_c && !has_d {
                    let n = if ultra {1} else {2};
                    base.remove_right(n);
                    pos_map_str = pos_map_str.split("\n").map(|line| {
                        let mut line = line.to_string();
                        line.pop();
                        if !ultra {
                            line.pop();
                        }
                        line
                    }).collect::<Vec<String>>().join("\n");
                }

                // remove 1 or 2 left hand column if there are no left hand labels
                if !has_e && !has_f && !has_g && !has_h {
                    let n = if ultra {1} else {2};
                    base.remove_left(n);
                    pos_map_str = pos_map_str.split("\n").map(|line| {
                        let mut line = line.to_string();
                        line.remove(0);
                        if !ultra {
                            line.remove(0);
                        }
                        line
                    }).collect::<Vec<String>>().join("\n");

                    for sel_change in sel_changes.iter_mut() {
                        for ch in sel_change.1.iter_mut() {
                            ch.x -= n as u16;
                        }
                    }
                }

                // remove the bottom of base and pos_map_str if there are no bottom labels
                if !has_e && !has_d {
                    base.remove_bottom(1);
                    pos_map_str = pos_map_str.split("\n").take(3).collect::<Vec<&str>>().join("\n");
                }

                //debug!("pos_map_str: \n{}", pos_map_str);
                //debug!("base: \n{}", base);

                let pos_map = ArbSelector::positions_string_to_map(&pos_map_str);
                ArbSelector::new_inner(ctx, base, pos_map, sel_changes)
            }
            Spacing::SemiCompact | Spacing::Spacious=> {

                let semi = matches!(self, Spacing::SemiCompact);

                let (max_lh_width, max_rh_width) = if semi {
                    (max_lh_width.max(5), max_rh_width.max(5))
                } else {
                    (max_lh_width.max(6), max_rh_width.max(6))  
                };

                let a_spc_len = max_rh_width.saturating_sub(a.len()).saturating_sub(self.start_x_from_center(&0) as usize);
                let b_spc_len = max_rh_width.saturating_sub(b.len()).saturating_sub(self.start_x_from_center(&1) as usize);
                let c_spc_len = max_rh_width.saturating_sub(c.len()).saturating_sub(self.start_x_from_center(&2) as usize);
                let d_spc_len = max_rh_width.saturating_sub(d.len()).saturating_sub(self.start_x_from_center(&3) as usize);
                let e_spc_len = max_rh_width.saturating_sub(e.len()).saturating_sub(self.start_x_from_center(&4) as usize);
                let f_spc_len = max_rh_width.saturating_sub(f.len()).saturating_sub(self.start_x_from_center(&5) as usize);

                let g_spc_len = max_lh_width.saturating_sub(g.len()).saturating_sub((-self.start_x_from_center(&6)) as usize);
                let h_spc_len = max_lh_width.saturating_sub(h.len()).saturating_sub((-self.start_x_from_center(&7)) as usize);
                let i_spc_len = max_lh_width.saturating_sub(i.len()).saturating_sub((-self.start_x_from_center(&8)) as usize);
                let j_spc_len = max_lh_width.saturating_sub(j.len()).saturating_sub((-self.start_x_from_center(&9)) as usize);
                let k_spc_len = max_lh_width.saturating_sub(k.len()).saturating_sub((-self.start_x_from_center(&10)) as usize);
                let l_spc_len = max_lh_width.saturating_sub(l.len()).saturating_sub((-self.start_x_from_center(&11)) as usize);

                let a_spc = DrawCh::str_to_draw_chs(&" ".repeat(a_spc_len), Style::transparent());
                let b_spc = DrawCh::str_to_draw_chs(&" ".repeat(b_spc_len), Style::transparent());
                let c_spc = DrawCh::str_to_draw_chs(&" ".repeat(c_spc_len), Style::transparent());
                let d_spc = DrawCh::str_to_draw_chs(&" ".repeat(d_spc_len), Style::transparent());
                let e_spc = DrawCh::str_to_draw_chs(&" ".repeat(e_spc_len), Style::transparent());
                let f_spc = DrawCh::str_to_draw_chs(&" ".repeat(f_spc_len), Style::transparent());
                let g_spc = DrawCh::str_to_draw_chs(&" ".repeat(g_spc_len), Style::transparent());
                let h_spc = DrawCh::str_to_draw_chs(&" ".repeat(h_spc_len), Style::transparent());
                let i_spc = DrawCh::str_to_draw_chs(&" ".repeat(i_spc_len), Style::transparent());
                let j_spc = DrawCh::str_to_draw_chs(&" ".repeat(j_spc_len), Style::transparent());
                let k_spc = DrawCh::str_to_draw_chs(&" ".repeat(k_spc_len), Style::transparent());
                let l_spc = DrawCh::str_to_draw_chs(&" ".repeat(l_spc_len), Style::transparent());

                let spc_2 = DrawCh::str_to_draw_chs(&" ".repeat(2), Style::transparent()); 
                let spc_3 = DrawCh::str_to_draw_chs(&" ".repeat(3), Style::transparent()); 
                let spc_4 = DrawCh::str_to_draw_chs(&" ".repeat(4), Style::transparent()); 
                let spc_8 = DrawCh::str_to_draw_chs(&" ".repeat(8), Style::transparent()); 

                let (y0, y1, y2, y3, y4, y5) = if semi {
                    //     OptionL  OptionA    
                    //  OptionK   __   OptionB
                    // OptionJ   ╱  ╲   OptionC
                    // OptionI   °__╱   OptionD
                    //  OptionH        OptionE
                    //     OptionG  OptionF    
                    ([l_spc.clone(), l.clone(), spc_2.clone(), a.clone(), a_spc.clone()].concat(),
                     [k_spc.clone(), k.clone(), spc_3.clone(), dial_y0.clone(), spc_3.clone(), b.clone(), b_spc.clone()].concat(),
                     [j_spc.clone(), j.clone(), spc_3.clone(), dial_y1.clone(), spc_3.clone(), c.clone(), c_spc.clone()].concat(),
                     [i_spc.clone(), i.clone(), spc_3.clone(), dial_y2.clone(), spc_3.clone(), d.clone(), d_spc.clone()].concat(),
                     [h_spc.clone(), h.clone(), spc_8.clone(), e.clone(), e_spc.clone()].concat(),
                     [g_spc.clone(), g.clone(), spc_2.clone(), f.clone(), f_spc.clone()].concat())
                } else {
                    //      OptionL  OptionA     
                    //   OptionK   __   OptionB  
                    // OptionJ    ╱  ╲    OptionC
                    // OptionI    °__╱    OptionD
                    //   OptionH        OptionE  
                    //      OptionG  OptionF     
                    ([l_spc.clone(), l.clone(), spc_2.clone(), a.clone(), a_spc.clone()].concat(),
                     [k_spc.clone(), k.clone(), spc_3.clone(), dial_y0.clone(), spc_3.clone(), b.clone(), b_spc.clone()].concat(),
                     [j_spc.clone(), j.clone(), spc_4.clone(), dial_y1.clone(), spc_4.clone(), c.clone(), c_spc.clone()].concat(),
                     [i_spc.clone(), i.clone(), spc_4.clone(), dial_y2.clone(), spc_4.clone(), d.clone(), d_spc.clone()].concat(),
                     [h_spc.clone(), h.clone(), spc_8.clone(), e.clone(), e_spc.clone()].concat(),
                     [g_spc.clone(), g.clone(), spc_2.clone(), f.clone(), f_spc.clone()].concat())
                };
                let y0 = DrawChs2D::from_draw_chs_horizontal(y0);
                let y1 = DrawChs2D::from_draw_chs_horizontal(y1);
                let y2 = DrawChs2D::from_draw_chs_horizontal(y2);
                let y3 = DrawChs2D::from_draw_chs_horizontal(y3);
                let y4 = DrawChs2D::from_draw_chs_horizontal(y4);
                let y5 = DrawChs2D::from_draw_chs_horizontal(y5);
                let mut base = y0.concat_top_bottom(y1)
                  .concat_top_bottom(y2)
                  .concat_top_bottom(y3)
                  .concat_top_bottom(y4)
                  .concat_top_bottom(y5);

                let map_len_b_y0 = a_spc_len * 3 / 5;
                let map_len_a_y0 = a.len() + a_spc_len - map_len_b_y0;
                let map_len_c_y1 = b_spc_len * 2 / 5;
                let map_len_b_y1 = b.len() + b_spc_len - map_len_c_y1;
                let map_len_c_y2 = c.len() + c_spc_len;
                let map_len_d_y3 = d.len() + d_spc_len;
                let map_len_d_y4 = e_spc_len * 2 / 5;
                let map_len_e_y4 = e.len() + e_spc_len - map_len_d_y4;
                let map_len_e_y5 = f_spc_len * 3 / 5;
                let map_len_f_y5 = f.len() + f_spc_len - map_len_e_y5;

                let map_len_k_y0 = l_spc_len * 3 / 5;
                let map_len_l_y0 = l.len() + l_spc_len - map_len_k_y0;
                let map_len_j_y1 = k_spc_len * 2 / 5;
                let map_len_k_y1 = k.len() + k_spc_len - map_len_j_y1;
                let map_len_j_y2 = j.len() + j_spc_len;
                let map_len_i_y3 = i.len() + i_spc_len;
                let map_len_i_y4 = h_spc_len * 2 / 5;
                let map_len_h_y4 = h.len() + h_spc_len - map_len_i_y4;
                let map_len_h_y5 = g_spc_len * 3 / 5;
                let map_len_g_y5 = g.len() + g_spc_len - map_len_h_y5;

                let map_b_y0 = b_ch.repeat(map_len_b_y0);
                let map_a_y0 = a_ch.repeat(map_len_a_y0);
                let map_c_y1 = c_ch.repeat(map_len_c_y1);
                let map_b_y1 = b_ch.repeat(map_len_b_y1);
                let map_c_y2 = c_ch.repeat(map_len_c_y2);
                let map_d_y3 = d_ch.repeat(map_len_d_y3);
                let map_d_y4 = d_ch.repeat(map_len_d_y4);
                let map_e_y4 = e_ch.repeat(map_len_e_y4);
                let map_e_y5 = e_ch.repeat(map_len_e_y5);
                let map_f_y5 = f_ch.repeat(map_len_f_y5);
                let map_k_y0 = k_ch.repeat(map_len_k_y0);
                let map_l_y0 = l_ch.repeat(map_len_l_y0);
                let map_j_y1 = j_ch.repeat(map_len_j_y1);
                let map_k_y1 = k_ch.repeat(map_len_k_y1);
                let map_j_y2 = j_ch.repeat(map_len_j_y2);
                let map_i_y3 = i_ch.repeat(map_len_i_y3);
                let map_i_y4 = i_ch.repeat(map_len_i_y4);
                let map_h_y4 = h_ch.repeat(map_len_h_y4);
                let map_h_y5 = h_ch.repeat(map_len_h_y5);
                let map_g_y5 = g_ch.repeat(map_len_g_y5);

                let a_ch_2 = a_ch.repeat(2);
                let b_ch_2 = b_ch.repeat(2);
                let l_ch_2 = l_ch.repeat(2);
                let k_ch_2 = k_ch.repeat(2);
                let e_ch_2 = e_ch.repeat(2);
                let f_ch_2 = f_ch.repeat(2);
                let g_ch_2 = g_ch.repeat(2);
                let h_ch_2 = h_ch.repeat(2);
                let c_ch_5 = c_ch.repeat(5);
                let d_ch_5 = d_ch.repeat(5);
                let j_ch_5 = j_ch.repeat(5);
                let i_ch_5 = i_ch.repeat(5);

                let c_ch_6 = c_ch.repeat(6);
                let d_ch_6 = d_ch.repeat(6);
                let j_ch_6 = j_ch.repeat(6);
                let i_ch_6 = i_ch.repeat(6);

                let mut pos_map_str = if semi {
                    //     OptionL  OptionA    
                    //  OptionK   __   OptionB
                    // OptionJ   ╱  ╲   OptionC
                    // OptionI   °__╱   OptionD
                    //  OptionH        OptionE
                    //     OptionG  OptionF    
                                 format!("{map_k_y0}{map_l_y0}{l_ch}{a_ch}{map_a_y0}{map_b_y0}\n")
                    + &format!("{map_j_y1}{map_k_y1}{k_ch_2}{l_ch_2}{a_ch_2}{b_ch_2}{map_b_y1}{map_c_y1}\n")
                                      + &format!("{map_j_y2}{j_ch_5}{c_ch_5}{map_c_y2}\n")
                                      + &format!("{map_i_y3}{i_ch_5}{d_ch_5}{map_d_y3}\n")
                    + &format!("{map_i_y4}{map_h_y4}{h_ch_2}{g_ch_2}{f_ch_2}{e_ch_2}{map_e_y4}{map_d_y4}\n")
                              + &format!("{map_h_y5}{map_g_y5}{g_ch}{f_ch}{map_f_y5}{map_e_y5}")
                } else {
                    //      OptionL  OptionA     
                    //   OptionK   __   OptionB  
                    // OptionJ    ╱  ╲    OptionC
                    // OptionI    °__╱    OptionD
                    //   OptionH        OptionE  
                    //      OptionG  OptionF     
                                 format!("{map_k_y0}{map_l_y0}{l_ch}{a_ch}{map_a_y0}{map_b_y0}\n")
                    + &format!("{map_j_y1}{map_k_y1}{k_ch_2}{l_ch_2}{a_ch_2}{b_ch_2}{map_b_y1}{map_c_y1}\n")
                                      + &format!("{map_j_y2}{j_ch_6}{c_ch_6}{map_c_y2}\n")
                                      + &format!("{map_i_y3}{i_ch_6}{d_ch_6}{map_d_y3}\n")
                    + &format!("{map_i_y4}{map_h_y4}{h_ch_2}{g_ch_2}{f_ch_2}{e_ch_2}{map_e_y4}{map_d_y4}\n")
                              + &format!("{map_h_y5}{map_g_y5}{g_ch}{f_ch}{map_f_y5}{map_e_y5}")
                };

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
                    a_sel_changes.push((ch, x, 2).into());
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
                    let ch = DrawCh::new('°', dial_knob_sty.clone());
                    b_sel_changes.push((ch, x, 2).into());
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
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
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
                    let x = max_lh_width + 1;
                    let ch = DrawCh::new('°', dial_knob_sty.clone());
                    d_sel_changes.push((ch, x, 3).into());
                    sel_changes.push((3, d_sel_changes));
                }
                if has_e {
                    let mut e_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width + self.start_x_from_center(&4) as usize;
                    for ch in e.iter() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        e_sel_changes.push((ch, x, 4).into());
                        x += 1
                    }
                    let x = max_lh_width + 1;
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
                    e_sel_changes.push((ch, x, 3).into());
                    sel_changes.push((4, e_sel_changes));
                }
                if has_f {
                    let mut f_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width + self.start_x_from_center(&5) as usize;
                    for ch in f.iter() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        f_sel_changes.push((ch, x, 5).into());
                        x += 1
                    }
                    let x = max_lh_width;
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
                    f_sel_changes.push((ch, x, 3).into());
                    sel_changes.push((5, f_sel_changes));
                }
                if has_g {
                    let mut g_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&6)) as usize;
                    for ch in g.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        g_sel_changes.push((ch, x, 5).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 1;
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
                    g_sel_changes.push((ch, x, 3).into());
                    sel_changes.push((6, g_sel_changes));
                }
                if has_h {
                    let mut h_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&7)) as usize;
                    for ch in h.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        h_sel_changes.push((ch, x, 4).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 2;
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
                    h_sel_changes.push((ch, x, 3).into());
                    sel_changes.push((7, h_sel_changes));
                }
                if has_i {
                    let mut i_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&8)) as usize;
                    for ch in i.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        i_sel_changes.push((ch, x, 3).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 2;
                    let ch = DrawCh::new('°', dial_knob_sty.clone());
                    i_sel_changes.push((ch, x, 3).into());
                    sel_changes.push((8, i_sel_changes));
                }
                if has_j {
                    let mut j_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&9)) as usize;
                    for ch in j.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        j_sel_changes.push((ch, x, 2).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 2;
                    let ch = DrawCh::new('⚬', dial_knob_sty.clone());
                    j_sel_changes.push((ch, x, 2).into());
                    sel_changes.push((9, j_sel_changes));
                }
                if has_k {
                    let mut k_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&10)) as usize;
                    for ch in k.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        k_sel_changes.push((ch, x, 1).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 2;
                    let ch = DrawCh::new('°', dial_knob_sty.clone());
                    k_sel_changes.push((ch, x, 2).into());
                    sel_changes.push((10, k_sel_changes));
                }
                if has_l {
                    let mut l_sel_changes: SelChanges = SelChanges::default();
                    let mut x = max_lh_width - 1 - (-self.start_x_from_center(&11)) as usize;
                    for ch in l.iter().rev() {
                        let ch = DrawCh::new(ch.ch.clone(), label_selected_color.clone());
                        l_sel_changes.push((ch, x, 0).into());
                        x = x.saturating_sub(1);
                    }
                    let x = max_lh_width - 1;
                    let ch = DrawCh::new('°', dial_knob_sty.clone());
                    l_sel_changes.push((ch, x, 2).into());
                    sel_changes.push((11, l_sel_changes));
                }

                //debug!("PRE removing");
                //debug!("pos_map_str: \n{}", pos_map_str);
                //debug!("base: \n{}", base);

                // remove 3 or 4 right hand column if there are no right hand labels
                if !has_a && !has_b && !has_c && !has_d && !has_e && !has_f{
                    let n = if semi {3} else {4};
                    base.remove_right(n);
                    pos_map_str = pos_map_str.split("\n").map(|line| {
                        let mut line = line.to_string();
                        line.pop();
                        line.pop();
                        line.pop();
                        if !semi {
                            line.pop();
                        }
                        line
                    }).collect::<Vec<String>>().join("\n");
                }

                // remove 3 or 4 left hand column if there are no left hand labels
                if !has_g && !has_h && !has_i && !has_j && !has_k && !has_l {
                    let n = if semi {3} else {4};
                    base.remove_left(n);
                    pos_map_str = pos_map_str.split("\n").map(|line| {
                        let mut line = line.to_string();
                        line.remove(0);
                        line.remove(0);
                        line.remove(0);
                        if !semi {
                            line.remove(0);
                        }
                        line
                    }).collect::<Vec<String>>().join("\n");

                    for sel_change in sel_changes.iter_mut() {
                        for ch in sel_change.1.iter_mut() {
                            ch.x -= n as u16;
                        }
                    }
                }

                // remove the top if there are no top labels
                if !has_l && !has_a {
                    //debug!("removing top");
                    base.remove_top(1);
                    pos_map_str = pos_map_str.split("\n").skip(1).collect::<Vec<&str>>().join("\n"); 
                    for sel_change in sel_changes.iter_mut() {
                        for ch in sel_change.1.iter_mut() {
                            ch.y -= 1;
                        }
                    }
                }

                //   OptionH        OptionE  
                //      OptionG  OptionF     
                // remove the bottom of base and pos_map_str if there are no bottom labels
                if !has_g && !has_f {
                    //debug!("removing bottom");
                    base.remove_bottom(1);
                    let line_count = pos_map_str.split("\n").count();
                    pos_map_str = pos_map_str.split("\n").take(line_count - 1).collect::<Vec<&str>>().join("\n");
                }
                // remove the bottom again if there are no bottom labels for last two rows
                if !has_g && !has_f && !has_h && !has_e {
                    //debug!("removing bottom again");
                    base.remove_bottom(1);
                    let line_count = pos_map_str.split("\n").count();
                    pos_map_str = pos_map_str.split("\n").take(line_count - 1).collect::<Vec<&str>>().join("\n");
                }

                //debug!("POST removing");
                //debug!("pos_map_str: \n{}", pos_map_str);
                //debug!("base: \n{}", base);

                let pos_map = ArbSelector::positions_string_to_map(&pos_map_str);
                ArbSelector::new_inner(ctx, base, pos_map, sel_changes)
            }
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Dial {}
