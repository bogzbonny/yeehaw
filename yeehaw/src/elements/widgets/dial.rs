use {
    crate::*,
    std::{cell::RefCell, rc::Rc},
};

/// Arbitrary Selector is a selector object which can be used to construct
/// cool selectors with arbitrary selection positions such as dials.
#[derive(Clone)]
pub struct Dial {
    pane: ArbSelector,
    dial_color: Rc<RefCell<Color>>,
    label_color: Rc<RefCell<Color>>,
    label_selected_color: Rc<RefCell<Style>>,
    labels: Rc<RefCell<Vec<(usize, String)>>>, // position and strings
    spacing: Rc<RefCell<Spacing>>,
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
        label_color: Color,
        label_selected_color: Style,
        labels: Vec<(usize, String)>, // position and strings
    ) -> ArbSelector {
        // get the label with 0 as the first value
        let max_lh_width = self.max_lefthand_width(labels.clone());
        let max_rh_width = self.max_righthand_width(labels.clone());
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
        let a = DrawCh::str_to_draw_chs(&a, label_sty);
        let b = DrawCh::str_to_draw_chs(&b, label_sty);
        let c = DrawCh::str_to_draw_chs(&c, label_sty);
        let d = DrawCh::str_to_draw_chs(&d, label_sty);
        let e = DrawCh::str_to_draw_chs(&e, label_sty);
        let f = DrawCh::str_to_draw_chs(&f, label_sty);
        let g = DrawCh::str_to_draw_chs(&g, label_sty);
        let h = DrawCh::str_to_draw_chs(&h, label_sty);
        let i = DrawCh::str_to_draw_chs(&i, label_sty);
        let j = DrawCh::str_to_draw_chs(&j, label_sty);
        let k = DrawCh::str_to_draw_chs(&k, label_sty);
        let l = DrawCh::str_to_draw_chs(&l, label_sty);

        let dial_sty = Style::transparent().with_fg(dial_color);
        let dial_y0 =  DrawCh::str_to_draw_chs("__", dial_sty);
        let dial_y1 = DrawCh::str_to_draw_chs("╱  ╲", dial_sty);
        let dial_y2 = DrawCh::str_to_draw_chs("╲__╱", dial_sty);

        match self {
            Spacing::UltraCompact => {
                let a_spc = " ".repeat(max_rh_width - a.len() - self.start_x_from_center(&0) as usize);
                let b_spc = " ".repeat(max_rh_width - b.len() - self.start_x_from_center(&1) as usize);
                let c_spc = " ".repeat(max_rh_width - c.len() - self.start_x_from_center(&2)as usize);
                let d_spc = " ".repeat(max_rh_width - d.len() - self.start_x_from_center(&3) as usize);
                let e_spc = " ".repeat(max_lh_width - e.len() - ((-self.start_x_from_center(&4)) as usize));
                let f_spc = " ".repeat(max_lh_width - f.len() - ((-self.start_x_from_center(&5)) as usize));
                let g_spc = " ".repeat(max_lh_width - g.len() - ((-self.start_x_from_center(&6)) as usize));
                let h_spc = " ".repeat(max_lh_width - h.len() - ((-self.start_x_from_center(&7)) as usize));
                let a_spc = DrawCh::str_to_draw_chs(&a_spc, Style::transparent());
                let b_spc = DrawCh::str_to_draw_chs(&b_spc, Style::transparent());
                let c_spc = DrawCh::str_to_draw_chs(&c_spc, Style::transparent());
                let d_spc = DrawCh::str_to_draw_chs(&d_spc, Style::transparent());
                let e_spc = DrawCh::str_to_draw_chs(&e_spc, Style::transparent());
                let f_spc = DrawCh::str_to_draw_chs(&f_spc, Style::transparent());
                let g_spc = DrawCh::str_to_draw_chs(&g_spc, Style::transparent());
                let h_spc = DrawCh::str_to_draw_chs(&h_spc, Style::transparent());
                let spc_1 = DrawCh::str_to_draw_chs(" ", Style::transparent()); // one space

                let y0 = [h_spc.clone(), h.clone(), dial_y0.clone(), a.clone(), a_spc.clone()].concat();
                let y1 = [g_spc.clone(), g.clone(), spc_1.clone(), dial_y1.clone(), spc_1.clone(), b.clone(), b_spc.clone()].concat();
                let y2 = [f_spc.clone(), f.clone(), spc_1.clone(), dial_y2.clone(), spc_1.clone(), c.clone(), c_spc.clone()].concat();
                let y3 = [e_spc.clone(), e.clone(), spc_1.clone(), spc_1.clone(), d.clone(), d_spc.clone()].concat();
                let y0 = DrawChs2D::from_draw_chs_horizontal(y0);
                let y1 = DrawChs2D::from_draw_chs_horizontal(y1);
                let y2 = DrawChs2D::from_draw_chs_horizontal(y2);
                let y3 = DrawChs2D::from_draw_chs_horizontal(y3);
                y0.concat_top_bottom(y1)
                  .concat_top_bottom(y2)
                  .concat_top_bottom(y3);

                y0
            }
            Spacing::Compact => {
                let a_spc = " ".repeat(max_rh_width - a.len() - self.start_x_from_center(&0) as usize);
                let b_spc = " ".repeat(max_rh_width - b.len() - self.start_x_from_center(&1) as usize);
                let c_spc = " ".repeat(max_rh_width - c.len() - self.start_x_from_center(&2)as usize);
                let d_spc = " ".repeat(max_rh_width - d.len() - self.start_x_from_center(&3) as usize);
                let e_spc = " ".repeat(max_lh_width - e.len() - ((-self.start_x_from_center(&4)) as usize));
                let f_spc = " ".repeat(max_lh_width - f.len() - ((-self.start_x_from_center(&5)) as usize));
                let g_spc = " ".repeat(max_lh_width - g.len() - ((-self.start_x_from_center(&6)) as usize));
                let h_spc = " ".repeat(max_lh_width - h.len() - ((-self.start_x_from_center(&7)) as usize));
                let dial_chs = format!("{h_spc}{h} __ {a}{a_spc}\n")
                          + &format!("{g_spc}{g}  ╱  ╲  {b}{b_spc}\n")
                          + &format!("{f_spc}{f}  ╲__╱  {c}{c_spc}\n")
                            + &format!("{e_spc}{e}    {d}{d_spc}");
                dial_chs
            }
            Spacing::SemiCompact => {
                let a_spc = " ".repeat(max_rh_width - a.len() - self.start_x_from_center(&0) as usize);
                let b_spc = " ".repeat(max_rh_width - b.len() - self.start_x_from_center(&1) as usize);
                let c_spc = " ".repeat(max_rh_width - c.len() - self.start_x_from_center(&2)as usize);
                let d_spc = " ".repeat(max_rh_width - d.len() - self.start_x_from_center(&3) as usize);
                let e_spc = " ".repeat(max_lh_width - e.len() - self.start_x_from_center(&4) as usize);
                let f_spc = " ".repeat(max_lh_width - f.len() - self.start_x_from_center(&5) as usize);
                let g_spc = " ".repeat(max_lh_width - g.len() - ((-self.start_x_from_center(&6)) as usize));
                let h_spc = " ".repeat(max_lh_width - h.len() - ((-self.start_x_from_center(&7)) as usize));
                let i_spc = " ".repeat(max_lh_width - i.len() - ((-self.start_x_from_center(&8)) as usize));
                let j_spc = " ".repeat(max_lh_width - j.len() - ((-self.start_x_from_center(&9)) as usize));
                let k_spc = " ".repeat(max_lh_width - k.len() - ((-self.start_x_from_center(&10)) as usize));
                let l_spc = " ".repeat(max_lh_width - l.len() - ((-self.start_x_from_center(&11)) as usize));
                let dial_chs = format!("{l_spc}{l}  {a}{a_spc}\n")
                         + &format!("{k_spc}{k}   __   {b}{b_spc}\n")
                        + &format!("{j_spc}{j}   ╱  ╲   {c}{c_spc}\n")
                        + &format!("{i_spc}{i}   ╲__╱   {d}{d_spc}\n")
                         + &format!("{h_spc}{h}        {e}{e_spc}\n")
                            + &format!("{g_spc}{g}  {f}{f_spc}");
                dial_chs
            }
            Spacing::Spacious => {
                let a_spc = " ".repeat(max_rh_width - a.len() - self.start_x_from_center(&0) as usize);
                let b_spc = " ".repeat(max_rh_width - b.len() - self.start_x_from_center(&1) as usize);
                let c_spc = " ".repeat(max_rh_width - c.len() - self.start_x_from_center(&2)as usize);
                let d_spc = " ".repeat(max_rh_width - d.len() - self.start_x_from_center(&3) as usize);
                let e_spc = " ".repeat(max_lh_width - e.len() - self.start_x_from_center(&4) as usize);
                let f_spc = " ".repeat(max_lh_width - f.len() - self.start_x_from_center(&5) as usize);
                let g_spc = " ".repeat(max_lh_width - g.len() - ((-self.start_x_from_center(&6)) as usize));
                let h_spc = " ".repeat(max_lh_width - h.len() - ((-self.start_x_from_center(&7)) as usize));
                let i_spc = " ".repeat(max_lh_width - i.len() - ((-self.start_x_from_center(&8)) as usize));
                let j_spc = " ".repeat(max_lh_width - j.len() - ((-self.start_x_from_center(&9)) as usize));
                let k_spc = " ".repeat(max_lh_width - k.len() - ((-self.start_x_from_center(&10)) as usize));
                let l_spc = " ".repeat(max_lh_width - l.len() - ((-self.start_x_from_center(&11)) as usize));
                let dial_chs = format!("{l_spc}{l}  {a}{a_spc}\n")
                        + &format!("{k_spc}{k}    __    {b}{b_spc}\n")
                       + &format!("{j_spc}{j}    ╱  ╲    {c}{c_spc}\n")
                       + &format!("{i_spc}{i}    ╲__╱    {d}{d_spc}\n")
                        + &format!("{h_spc}{h}          {e}{e_spc}\n")
                            + &format!("{g_spc}{g}  {f}{f_spc}");
                dial_chs
            }

            todo!()
        }
    }
}

impl Dial {
    const KIND: &'static str = "dial";

    const DEFAULT_DIAL_COLOR: Color = Color::AQUA;
    const DEFAULT_LABEL_COLOR: Color = Color::GREY13;
    const DEFAULT_LABEL_SEL_COLOR: Color = Color::YELLOW;

    /// Create a new ArbSelector with the given drawing style and base drawing
    /// and the positions map. The positions map is a 2D array of letters where 'A' is the
    /// 1st position, 'B' is the 2nd position, etc.
    pub fn new_ultra_compact(ctx: &Context, labels: Vec<(usize, String)>) -> Self {
        Self::new_inner(ctx, labels, Spacing::UltraCompact)
    }

    pub fn new_compact(ctx: &Context, labels: Vec<(usize, String)>) -> Self {
        Self::new_inner(ctx, labels, Spacing::Compact)
    }

    pub fn new_semi_compact(ctx: &Context, labels: Vec<(usize, String)>) -> Self {
        Self::new_inner(ctx, labels, Spacing::SemiCompact)
    }

    pub fn new_spacious(ctx: &Context, labels: Vec<(usize, String)>) -> Self {
        Self::new_inner(ctx, labels, Spacing::Spacious)
    }

    fn new_inner(ctx: &Context, labels: Vec<(usize, String)>, spacing: Spacing) -> Self {
        let pane = spacing.dial_arb_selector(
            ctx,
            Self::DEFAULT_DIAL_COLOR,
            Self::DEFAULT_LABEL_COLOR,
            Style::new_const(Self::DEFAULT_LABEL_SEL_COLOR, Color::BLACK),
            labels.clone(),
        );
        pane.pane.set_kind(Self::KIND);
        Self {
            pane,
            dial_color: Rc::new(RefCell::new(Self::DEFAULT_DIAL_COLOR)),
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
            Self::DEFAULT_DIAL_COLOR,
            Self::DEFAULT_LABEL_COLOR,
            Style::new_const(Self::DEFAULT_LABEL_SEL_COLOR, Color::BLACK),
            self.labels.borrow().clone(),
        );
        pane.pane.set_kind(Self::KIND);
        self.pane = pane;
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Dial {}
