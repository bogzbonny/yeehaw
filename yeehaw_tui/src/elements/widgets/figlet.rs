use {crate::*, figlet_rs::FIGfont};

/// TODO click function

#[derive(Clone)]
pub struct FigletText {
    pub pane: Pane,
}

impl FigletText {
    const KIND: &'static str = "figlet";

    pub fn new(ctx: &Context, text: &str, font: FIGfont) -> Self {
        let Some(fig_text) = font.convert(text) else {
            return FigletText {
                pane: Pane::new(ctx, Self::KIND),
            };
        };

        let text = format!("{}", fig_text);
        let text = text.trim_end_matches('\n'); // remove the last newline
        let content = DrawChs2D::from_string(text.to_string(), Style::default());
        let size = content.size();

        let pane = Pane::new(ctx, Self::KIND)
            .with_dyn_width(DynVal::new_fixed(size.width as i32))
            .with_dyn_height(DynVal::new_fixed(size.height as i32));
        pane.set_content(content);
        FigletText { pane }
    }

    // ----------------------------------------------
    // decorators

    pub fn with_min_height(self) -> Self {
        self.pane.get_content_mut().trim_bottom_whitespace();
        let h = self.pane.content_height();
        self.pane.set_dyn_height(DynVal::new_fixed(h as i32));
        self
    }

    pub fn with_style(self, sty: Style) -> Self {
        self.pane.set_style(sty.clone());
        self.pane.set_content_style(sty);
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for FigletText {}
