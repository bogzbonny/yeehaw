use {
    crate::{Context, DynVal, Size},
    crossterm::style::Color as CrosstermColor,
    rand::Rng,
    std::time::Duration,
};

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub enum Color {
    ANSI(CrosstermColor),
    Rgba(Rgba),
    Gradient(Gradient),
    TimeGradient(TimeGradient),
    RadialGradient(RadialGradient),
}

impl Default for Color {
    fn default() -> Self {
        Color::Rgba(Rgba::new(0, 0, 0))
    }
}

impl From<ratatui::style::Color> for Color {
    fn from(c: ratatui::style::Color) -> Self {
        Self::ANSI(c.into())
    }
}

impl From<CrosstermColor> for Color {
    fn from(c: CrosstermColor) -> Self {
        Self::ANSI(c)
    }
}

impl From<vt100::Color> for Color {
    #[inline]
    fn from(value: vt100::Color) -> Self {
        match value {
            vt100::Color::Default => Self::ANSI(CrosstermColor::Reset),
            vt100::Color::Idx(i) => Self::ANSI(CrosstermColor::AnsiValue(i)),
            vt100::Color::Rgb(r, g, b) => Self::Rgba(Rgba::new(r, g, b)),
        }
    }
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Color {
        Color::Rgba(Rgba::new(r, g, b))
    }

    pub const fn new_with_alpha(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::Rgba(Rgba::new_with_alpha(r, g, b, a))
    }

    pub fn new_from_hsva(h: f64, s: f64, v: f64, a: f64) -> Color {
        let (r, g, b) = Self::hsv_to_rgb(h, s, v);
        Color::Rgba(Rgba::new_with_alpha(r, g, b, (a * 255.0) as u8))
    }

    pub fn new_from_hsv(h: f64, s: f64, v: f64) -> Color {
        let (r, g, b) = Self::hsv_to_rgb(h, s, v);
        Color::Rgba(Rgba::new(r, g, b))
    }

    pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
        let c = v * s;
        let x = c * (1. - ((h / 60.).rem_euclid(2.) - 1.).abs());
        let m = v - c;
        let (r, g, b) = match h {
            h if h < 60. => (c, x, 0.),
            h if h < 120. => (x, c, 0.),
            h if h < 180. => (0., c, x),
            h if h < 240. => (0., x, c),
            h if h < 300. => (x, 0., c),
            _ => (c, 0., x),
        };
        (
            ((r + m) * 255.) as u8,
            ((g + m) * 255.) as u8,
            ((b + m) * 255.) as u8,
        )
    }

    pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
        let r = r as f64 / 255.;
        let g = g as f64 / 255.;
        let b = b as f64 / 255.;
        let cmax = r.max(g).max(b);
        let cmin = r.min(g).min(b);
        let delta = cmax - cmin;
        let h = if delta == 0. {
            0.
        } else if cmax == r {
            60. * (((g - b) / delta) % 6.)
        } else if cmax == g {
            60. * ((b - r) / delta + 2.)
        } else {
            60. * ((r - g) / delta + 4.)
        };
        let s = if cmax == 0. { 0. } else { delta / cmax };
        let v = cmax;
        (h, s, v)
    }

    /// blends two colors together with the given percentage of the other color
    pub fn blend(
        &self, s: Size, dur_since_launch: Duration, x: u16, y: u16, other: Color,
        percent_other: f64, blend_kind: BlendKind,
    ) -> Color {
        match self {
            Color::ANSI(_) => {
                if percent_other < 0.5 {
                    self.clone()
                } else {
                    other
                }
            }
            Color::Rgba(c) => match other {
                Color::ANSI(_) => {
                    if percent_other < 0.5 {
                        self.clone()
                    } else {
                        other
                    }
                }
                Color::Rgba(oc) => Color::Rgba(blend_kind.blend(*c, oc, percent_other)),
                Color::Gradient(gr) => {
                    let gr = gr.to_color(s, dur_since_launch, x, y);
                    self.clone()
                        .blend(s, dur_since_launch, x, y, gr, percent_other, blend_kind)
                }
                Color::TimeGradient(tg) => {
                    let tg = tg.to_color(s, dur_since_launch, x, y);
                    self.clone()
                        .blend(s, dur_since_launch, x, y, tg, percent_other, blend_kind)
                }
                Color::RadialGradient(rg) => {
                    let rg = rg.to_color(s, dur_since_launch, x, y);
                    self.clone()
                        .blend(s, dur_since_launch, x, y, rg, percent_other, blend_kind)
                }
            },
            Color::Gradient(gr) => {
                let gr = gr.to_color(s, dur_since_launch, x, y);
                gr.blend(s, dur_since_launch, x, y, other, percent_other, blend_kind)
            }
            Color::TimeGradient(gr) => {
                let gr = gr.to_color(s, dur_since_launch, x, y);
                gr.blend(s, dur_since_launch, x, y, other, percent_other, blend_kind)
            }
            Color::RadialGradient(gr) => {
                let gr = gr.to_color(s, dur_since_launch, x, y);
                gr.blend(s, dur_since_launch, x, y, other, percent_other, blend_kind)
            }
        }
    }

    pub fn random_light() -> Self {
        let r: u8 = rand::thread_rng().gen_range(150..=255);
        let g: u8 = rand::thread_rng().gen_range(150..=255);
        let b: u8 = rand::thread_rng().gen_range(150..=255);
        Self::new(r, g, b)
    }

    pub fn random_dark() -> Self {
        let r: u8 = rand::thread_rng().gen_range(0..150);
        let g: u8 = rand::thread_rng().gen_range(0..150);
        let b: u8 = rand::thread_rng().gen_range(0..150);
        Self::new(r, g, b)
    }

    pub fn darken(&self) -> Self {
        match self {
            Color::ANSI(c) => Color::darken_ansi(c),
            Color::Rgba(c) => Color::Rgba(c.mul(0.5)),
            Color::Gradient(gr) => {
                let mut grad = vec![];
                for (x, c) in &gr.grad {
                    grad.push((x.clone(), c.darken()));
                }
                Color::Gradient(Gradient {
                    grad,
                    angle_deg: gr.angle_deg,
                })
            }
            Color::TimeGradient(tg) => {
                let mut points = vec![];
                for (dur, c) in &tg.points {
                    points.push((*dur, c.darken()));
                }
                Color::TimeGradient(TimeGradient::new(tg.total_dur, points))
            }
            Color::RadialGradient(rg) => {
                let mut grad = vec![];
                for (x, c) in &rg.grad {
                    grad.push((x.clone(), c.darken()));
                }
                Color::RadialGradient(RadialGradient {
                    center: rg.center.clone(),
                    skew: rg.skew,
                    grad,
                })
            }
        }
    }

    pub fn lighten(&self) -> Self {
        match self {
            Color::ANSI(c) => Color::lighten_ansi(c),
            Color::Rgba(c) => Color::Rgba(c.mul(1.5)),
            Color::Gradient(gr) => {
                let mut grad = vec![];
                for (x, c) in &gr.grad {
                    grad.push((x.clone(), c.lighten()));
                }
                Color::Gradient(Gradient {
                    grad,
                    angle_deg: gr.angle_deg,
                })
            }
            Color::TimeGradient(tg) => {
                let mut points = vec![];
                for (dur, c) in &tg.points {
                    points.push((*dur, c.lighten()));
                }
                Color::TimeGradient(TimeGradient::new(tg.total_dur, points))
            }
            Color::RadialGradient(rg) => {
                let mut grad = vec![];
                for (x, c) in &rg.grad {
                    grad.push((x.clone(), c.lighten()));
                }
                Color::RadialGradient(RadialGradient {
                    center: rg.center.clone(),
                    skew: rg.skew,
                    grad,
                })
            }
        }
    }

    pub fn darken_ansi(c: &CrosstermColor) -> Color {
        match c {
            CrosstermColor::Red => Color::ANSI(CrosstermColor::DarkRed),
            CrosstermColor::Green => Color::ANSI(CrosstermColor::DarkGreen),
            CrosstermColor::Yellow => Color::ANSI(CrosstermColor::DarkYellow),
            CrosstermColor::Blue => Color::ANSI(CrosstermColor::DarkBlue),
            CrosstermColor::Magenta => Color::ANSI(CrosstermColor::DarkMagenta),
            CrosstermColor::Cyan => Color::ANSI(CrosstermColor::DarkCyan),
            CrosstermColor::Grey => Color::ANSI(CrosstermColor::DarkGrey),
            CrosstermColor::White => Color::ANSI(CrosstermColor::Grey),
            CrosstermColor::Rgb { r, g, b } => Color::new(*r, *g, *b).darken(),
            _ => Color::ANSI(*c),
        }
    }

    pub fn lighten_ansi(c: &CrosstermColor) -> Color {
        match c {
            CrosstermColor::DarkRed => Color::ANSI(CrosstermColor::Red),
            CrosstermColor::DarkGreen => Color::ANSI(CrosstermColor::Green),
            CrosstermColor::DarkYellow => Color::ANSI(CrosstermColor::Yellow),
            CrosstermColor::DarkBlue => Color::ANSI(CrosstermColor::Blue),
            CrosstermColor::DarkMagenta => Color::ANSI(CrosstermColor::Magenta),
            CrosstermColor::DarkCyan => Color::ANSI(CrosstermColor::Cyan),
            CrosstermColor::DarkGrey => Color::ANSI(CrosstermColor::Grey),
            CrosstermColor::Grey => Color::ANSI(CrosstermColor::White),
            CrosstermColor::Rgb { r, g, b } => Color::new(*r, *g, *b).lighten(),
            _ => Color::ANSI(*c),
        }
    }

    /// considers the alpha of the self and blends with the previous color
    pub fn to_crossterm_color(
        &self, ctx: &Context, prev: Option<CrosstermColor>, x: u16, y: u16,
    ) -> CrosstermColor {
        match self {
            Color::ANSI(c) => *c,
            Color::Rgba(c) => c.to_crossterm_color(prev),
            Color::Gradient(gr) => gr
                .to_color(ctx.size, ctx.dur_since_launch, x, y)
                .to_crossterm_color(ctx, prev, x, y),
            Color::TimeGradient(tg) => tg
                .to_color(ctx.size, ctx.dur_since_launch, x, y)
                .to_crossterm_color(ctx, prev, x, y),
            Color::RadialGradient(rg) => rg
                .to_color(ctx.size, ctx.dur_since_launch, x, y)
                .to_crossterm_color(ctx, prev, x, y),
        }
    }

    /// to color with the given context and position
    pub fn to_color(self, ctx: &Context, x: u16, y: u16) -> Color {
        match self {
            Color::ANSI(_) | Color::Rgba(_) => self,
            Color::Gradient(gr) => gr.to_color(ctx.size, ctx.dur_since_launch, x, y),
            Color::TimeGradient(tg) => tg.to_color(ctx.size, ctx.dur_since_launch, x, y),
            Color::RadialGradient(rg) => rg.to_color(ctx.size, ctx.dur_since_launch, x, y),
        }
    }

    /// to color with the given context and position
    pub fn update_color(&mut self, s: Size, dur_since_launch: Duration, x: u16, y: u16) {
        match &self {
            Color::Gradient(gr) => *self = gr.clone().to_color(s, dur_since_launch, x, y),
            Color::TimeGradient(tg) => *self = tg.clone().to_color(s, dur_since_launch, x, y),
            Color::RadialGradient(rg) => *self = rg.clone().to_color(s, dur_since_launch, x, y),
            _ => {}
        }
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        match self {
            Color::Rgba(c) => c.a = alpha,
            Color::Gradient(gr) => {
                for (_, c) in &mut gr.grad {
                    c.set_alpha(alpha);
                }
            }
            Color::TimeGradient(tg) => {
                for (_, c) in &mut tg.points {
                    c.set_alpha(alpha);
                }
            }
            Color::RadialGradient(rg) => {
                for (_, c) in &mut rg.grad {
                    c.set_alpha(alpha);
                }
            }
            _ => {}
        }
    }

    pub fn with_alpha(mut self, alpha: u8) -> Self {
        self.set_alpha(alpha);
        self
    }

    pub fn overlay_color(&self, overlay: Self) -> Self {
        match overlay {
            Color::Rgba(oc) => match self {
                Color::ANSI(_) => overlay,
                Color::Rgba(c) => Color::Rgba(c.overlay_color(oc)),
                Color::Gradient(gr) => {
                    let mut grad = vec![];
                    for (x, c) in &gr.grad {
                        grad.push((x.clone(), c.overlay_color(overlay.clone())));
                    }
                    Color::Gradient(Gradient {
                        grad,
                        angle_deg: gr.angle_deg,
                    })
                }
                Color::TimeGradient(tg) => {
                    let mut points = vec![];
                    for (dur, c) in &tg.points {
                        points.push((*dur, c.overlay_color(overlay.clone())));
                    }
                    Color::TimeGradient(TimeGradient::new(tg.total_dur, points))
                }
                Color::RadialGradient(rg) => {
                    let mut grad = vec![];
                    for (x, c) in &rg.grad {
                        grad.push((x.clone(), c.overlay_color(overlay.clone())));
                    }
                    Color::RadialGradient(RadialGradient {
                        center: rg.center.clone(),
                        skew: rg.skew,
                        grad,
                    })
                }
            },
            _ => overlay,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct RadialGradient {
    pub center: (DynVal, DynVal),
    /// x, y
    pub skew: (f64, f64),
    /// horizontal, vertical skew (as skew of (1., 1./0.55) seems to make a circle)
    pub grad: Vec<(DynVal, Color)>,
}

impl RadialGradient {
    fn dist_from_center(
        x: f64, y: f64, center_x: f64, center_y: f64, skew_x: f64, skew_y: f64,
    ) -> f64 {
        let dx = x - center_x;
        let dy = y - center_y;
        let dx = skew_x * dx;
        let dy = skew_y * dy;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn to_color(&self, s: Size, dur_since_launch: Duration, x: u16, y: u16) -> Color {
        if self.grad.is_empty() {
            return Color::TRANSPARENT;
        }

        let x = x as f64;
        let y = y as f64;
        let (center_x, center_y) = (
            self.center.0.get_val(s.width) as f64,
            self.center.1.get_val(s.height) as f64,
        );
        let (skew_x, skew_y) = self.skew;
        let mut dist = Self::dist_from_center(x, y, center_x, center_y, skew_x, skew_y);

        // choose the furthest corner as the max distance
        let max_dist1 = Self::dist_from_center(0., 0., center_x, center_y, skew_x, skew_y);
        let max_dist2 = Self::dist_from_center(
            (s.width - 1) as f64,
            (s.height - 1) as f64,
            center_x,
            center_y,
            skew_x,
            skew_y,
        );
        let max_dist3 = Self::dist_from_center(
            0.,
            (s.height - 1) as f64,
            center_x,
            center_y,
            skew_x,
            skew_y,
        );
        let max_dist4 =
            Self::dist_from_center((s.width - 1) as f64, 0., center_x, center_y, skew_x, skew_y);
        let max_dist = max_dist1.max(max_dist2).max(max_dist3).max(max_dist4);

        // loop the pos if it is outside the maximum value
        while dist < 0. {
            dist += max_dist;
        }
        while dist > max_dist {
            dist -= max_dist;
        }

        let mut start_clr: Option<Color> = None;
        let mut end_clr: Option<Color> = None;
        let mut start_pos: Option<f64> = None;
        let mut end_pos: Option<f64> = None;
        for ((p1, c1), (p2, c2)) in self.grad.windows(2).map(|w| (w[0].clone(), w[1].clone())) {
            if (p1.get_val(s.width.max(s.height)) as f64 <= dist)
                && (dist < p2.get_val(s.width.max(s.height)) as f64)
            {
                start_clr = Some(c1.clone());
                end_clr = Some(c2.clone());
                start_pos = Some(p1.get_val(s.width.max(s.height)) as f64);
                end_pos = Some(p2.get_val(s.width.max(s.height)) as f64);
                break;
            }
        }

        let start_clr = start_clr.unwrap_or_else(|| self.grad[0].1.clone());
        let end_clr = end_clr.unwrap_or_else(|| self.grad[self.grad.len() - 1].1.clone());
        let start_pos =
            start_pos.unwrap_or_else(|| self.grad[0].0.get_val(s.width.max(s.height)) as f64);
        let end_pos = end_pos.unwrap_or_else(|| {
            self.grad[self.grad.len() - 1]
                .0
                .get_val(s.width.max(s.height)) as f64
        });
        let percent = (dist - start_pos) / (end_pos - start_pos);
        start_clr.blend(
            s,
            dur_since_launch,
            x as u16,
            y as u16,
            end_clr,
            percent,
            BlendKind::Blend1,
        )

        //// loop the pos if it is outside the maximum value
        //let max_pos = self.grad.last().expect("TODO??").0.get_val(max_ctx_val);
        //while pos < 0 {
        //    pos += max_pos;
        //}
        //while pos > max_pos {
        //    pos -= max_pos;
        //}

        //// find the two colors to blend
        //let mut start_clr: Option<Color> = None;
        //let mut end_clr: Option<Color> = None;
        //let mut start_pos: Option<i32> = None;
        //let mut end_pos: Option<i32> = None;
        //for ((p1, c1), (p2, c2)) in self.grad.windows(2).map(|w| (w[0].clone(), w[1].clone())) {
        //    if (p1.get_val(max_ctx_val) <= pos) && (pos < p2.get_val(max_ctx_val)) {
        //        start_clr = Some(c1.clone());
        //        end_clr = Some(c2.clone());
        //        start_pos = Some(p1.get_val(max_ctx_val));
        //        end_pos = Some(p2.get_val(max_ctx_val));
        //        break;
        //    }
        //}
        //let start_clr = start_clr.unwrap_or_else(|| self.grad[0].1.clone());
        //let end_clr = end_clr.unwrap_or_else(|| self.grad[self.grad.len() - 1].1.clone());
        //let start_pos = start_pos.unwrap_or_else(|| self.grad[0].0.get_val(max_ctx_val));
        //let end_pos =
        //    end_pos.unwrap_or_else(|| self.grad[self.grad.len() - 1].0.get_val(max_ctx_val));
        //let percent = (pos - start_pos) as f64 / (end_pos - start_pos) as f64;
        //start_clr.blend(s, dur_since_launch, x, y, end_clr, percent, BlendKind::Blend1)
    }
}

/// a gradient along a line with a given angle
/// the angle is in degrees, starting from the positive-x-axis, moving clockwise
/// the gradient position is a DynVal which is adjusted based on the width and height of the context
/// for angles 0, 90, 180, 270, the gradient is a simple linear gradient
/// for other angles, the gradient is a linear gradient along the line defined by the angle and the length
/// of the DynVal will be determined by the distance from the line
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Gradient {
    /// pos, color
    pub grad: Vec<(DynVal, Color)>,
    /// angle in degrees, starting from the positive-x-axis, moving clockwise
    pub angle_deg: f64,
}

impl Gradient {
    pub fn new(grad: Vec<(DynVal, Color)>, angle_deg: f64) -> Self {
        if angle_deg < 0.0 {
            let angle_deg = 360.0 + angle_deg;
            Gradient { grad, angle_deg }
        } else if angle_deg >= 360.0 {
            let angle_deg = angle_deg - 360.0;
            Gradient { grad, angle_deg }
        } else {
            Gradient { grad, angle_deg }
        }
    }

    pub fn new_x_grad_2_color(c1: Color, c2: Color) -> Self {
        let grad = vec![(DynVal::new_flex(0.), c1), (DynVal::FULL, c2)];
        Gradient {
            grad,
            angle_deg: 0.,
        }
    }

    pub fn new_y_grad_2_color(c1: Color, c2: Color) -> Self {
        let grad = vec![(DynVal::new_flex(0.), c1), (DynVal::FULL, c2)];
        Gradient {
            grad,
            angle_deg: 90.,
        }
    }

    /// length is the number of characters per color gradient
    pub fn new_x_grad_2_color_repeater(c1: Color, c2: Color, length: usize) -> Self {
        let grad = vec![
            (DynVal::new_fixed(0), c1.clone()),
            (DynVal::new_fixed(length as i32), c2),
            (DynVal::new_fixed(2 * length as i32), c1),
        ];
        Gradient {
            grad,
            angle_deg: 0.,
        }
    }

    pub fn new_y_grad_2_color_repeater(c1: Color, c2: Color, length: usize) -> Self {
        let grad = vec![
            (DynVal::new_fixed(0), c1.clone()),
            (DynVal::new_fixed(length as i32), c2),
            (DynVal::new_fixed(2 * length as i32), c1),
        ];
        Gradient {
            grad,
            angle_deg: 90.,
        }
    }

    /// length is the number of characters per color gradient
    pub fn new_x_grad_repeater(mut colors: Vec<Color>, length: usize) -> Self {
        if colors.is_empty() {
            return Gradient::default();
        }
        let mut v = Vec::with_capacity(colors.len() + 1);
        for (i, c) in colors.drain(..).enumerate() {
            v.push((DynVal::new_fixed((i * length) as i32), c));
        }
        v.push((DynVal::new_fixed((v.len() * length) as i32), v[0].1.clone()));
        Gradient {
            grad: v,
            angle_deg: 0.,
        }
    }

    pub fn new_y_grad_repeater(mut colors: Vec<Color>, length: usize) -> Self {
        if colors.is_empty() {
            return Gradient::default();
        }
        let mut v = Vec::with_capacity(colors.len() + 1);
        for (i, c) in colors.drain(..).enumerate() {
            v.push((DynVal::new_fixed((i * length) as i32), c));
        }
        v.push((DynVal::new_fixed((v.len() * length) as i32), v[0].1.clone()));
        Gradient {
            grad: v,
            angle_deg: 90.,
        }
    }

    pub fn x_grad_rainbow(length: usize) -> Self {
        Gradient::new_x_grad_repeater(
            vec![
                Color::VIOLET,
                Color::INDIGO,
                Color::BLUE,
                Color::GREEN,
                Color::YELLOW,
                Color::ORANGE,
                Color::RED,
            ],
            length,
        )
    }

    pub fn y_grad_rainbow(length: usize) -> Self {
        Gradient::new_y_grad_repeater(
            vec![
                Color::VIOLET,
                Color::INDIGO,
                Color::BLUE,
                Color::GREEN,
                Color::YELLOW,
                Color::ORANGE,
                Color::RED,
            ],
            length,
        )
    }

    pub fn to_crossterm_color(
        &self, ctx: &Context, prev: Option<CrosstermColor>, x: u16, y: u16,
    ) -> CrosstermColor {
        self.to_color(ctx.size, ctx.dur_since_launch, x, y)
            .to_crossterm_color(ctx, prev, x, y)
    }

    pub fn to_color(&self, s: Size, dur_since_launch: Duration, mut x: u16, mut y: u16) -> Color {
        if self.grad.is_empty() {
            return Color::TRANSPARENT;
        }

        // determine the maximum value of the context (used for computing the DynVal)
        let angle_rad = self.angle_deg * std::f64::consts::PI / 180.0;
        let max_ctx_val = if self.angle_deg == 0. || self.angle_deg == 180. {
            s.width
        } else if self.angle_deg == 90. || self.angle_deg == 270. {
            s.height
        } else {
            let width1 = s.width as f64;
            let height1 = s.height as f64;
            let angle_rad = angle_rad.abs();
            let width2 = height1 * (std::f64::consts::PI / 2. - angle_rad).tan();
            let height2 = width1 * angle_rad.tan();
            let (w, h) = if height2 < height1 { (width1, height2) } else { (width2, height1) };
            //let w = w * 4. / 3.;
            //let h = h * 3. / 4.;
            //let w = width1;
            //let h = height1;
            (w * w + h * h).sqrt().round() as u16
        };

        // determine the position on the gradient of the given x, y
        let mut pos = if self.angle_deg == 0. {
            x %= s.width;
            x as i32
        } else if self.angle_deg == 90. {
            y %= s.height;
            y as i32
        } else if self.angle_deg == 180. {
            x %= s.width;
            -(x as i32)
        } else if self.angle_deg == 270. {
            y %= s.height;
            -(y as i32)
        } else {
            //            x
            //   ┌───────────────┐
            //        xb      xa
            //   ┌─────────┐┌────┐
            //   ┌───────────────────
            //   │╲A       ╱╲A   │
            //   │ ╲      ╱╲╱╲   │
            //  posb╲    ╱    ╲  │y
            //   │   ╲╱╲╱      ╲ │
            //   │    ╲╱╲      ╱╲│
            //   │     ╲╱      ╲╱
            //   │      ╲      ╱
            //   │  posa ╲    ╱
            //   │        ╲╱╲╱
            //   │         ╲╱
            //

            let x = x as f64;
            let y = y as f64;
            #[allow(non_snake_case)]
            let sin_A = angle_rad.sin();
            #[allow(non_snake_case)]
            let cos_A = angle_rad.cos();
            #[allow(non_snake_case)]
            let tan_A = angle_rad.tan();
            let pos_a = y / sin_A;
            let xa = y / tan_A;
            let xb = x - xa;
            let pos_b = xb * cos_A;
            let pos = pos_a + pos_b;
            pos.round() as i32
        };

        // loop the pos if it is outside the maximum value
        let max_pos = self
            .grad
            .last()
            .expect("should not be empty")
            .0
            .get_val(max_ctx_val);
        while pos < 0 {
            pos += max_pos;
        }
        while pos > max_pos {
            pos -= max_pos;
        }

        // find the two colors to blend
        let mut start_clr: Option<Color> = None;
        let mut end_clr: Option<Color> = None;
        let mut start_pos: Option<i32> = None;
        let mut end_pos: Option<i32> = None;
        for ((p1, c1), (p2, c2)) in self.grad.windows(2).map(|w| (w[0].clone(), w[1].clone())) {
            if (p1.get_val(max_ctx_val) <= pos) && (pos < p2.get_val(max_ctx_val)) {
                start_clr = Some(c1.clone());
                end_clr = Some(c2.clone());
                start_pos = Some(p1.get_val(max_ctx_val));
                end_pos = Some(p2.get_val(max_ctx_val));
                break;
            }
        }
        let start_clr = start_clr.unwrap_or_else(|| self.grad[0].1.clone());
        let end_clr = end_clr.unwrap_or_else(|| self.grad[self.grad.len() - 1].1.clone());
        let start_pos = start_pos.unwrap_or_else(|| self.grad[0].0.get_val(max_ctx_val));
        let end_pos =
            end_pos.unwrap_or_else(|| self.grad[self.grad.len() - 1].0.get_val(max_ctx_val));
        let percent = (pos - start_pos) as f64 / (end_pos - start_pos) as f64;
        start_clr.blend(
            s,
            dur_since_launch,
            x,
            y,
            end_clr,
            percent,
            BlendKind::Blend1,
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct TimeGradient {
    /// The total time duration of the gradient
    pub total_dur: Duration,
    pub points: Vec<(Duration, Color)>,
}

impl TimeGradient {
    pub fn new(total_dur: Duration, points: Vec<(Duration, Color)>) -> Self {
        TimeGradient { total_dur, points }
    }

    pub fn to_color(&self, s: Size, dur_since_launch: Duration, x: u16, y: u16) -> Color {
        if self.points.is_empty() {
            return Color::TRANSPARENT;
        }

        let mut d = dur_since_launch;
        // calculate d so that it is within the range
        while d >= self.total_dur {
            d -= self.total_dur;
        }
        let mut start_clr: Option<Color> = None;
        let mut end_clr: Option<Color> = None;
        let mut start_time: Option<Duration> = None;
        let mut end_time: Option<Duration> = None;
        for ((t1, c1), (t2, c2)) in self.points.windows(2).map(|w| (w[0].clone(), w[1].clone())) {
            if (t1 <= d) && (d < t2) {
                start_clr = Some(c1.clone());
                end_clr = Some(c2.clone());
                start_time = Some(t1);
                end_time = Some(t2);
                break;
            }
        }
        let start_clr = start_clr.unwrap_or_else(|| self.points[0].1.clone());
        let end_clr = end_clr.unwrap_or_else(|| self.points[self.points.len() - 1].1.clone());
        let start_time = start_time.unwrap_or_else(|| self.points[0].0);
        let end_time = end_time.unwrap_or_else(|| self.points[self.points.len() - 1].0);
        let percent = (d - start_time).as_secs_f64() / (end_time - start_time).as_secs_f64();
        start_clr.blend(
            s,
            dur_since_launch,
            x,
            y,
            end_clr,
            percent,
            BlendKind::Blend1,
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    /// alpha
    pub a: u8,
}

impl Default for Rgba {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendKind {
    Blend1,
    Blend2,
}

impl BlendKind {
    pub fn blend(&self, c1: Rgba, c2: Rgba, perc_other: f64) -> Rgba {
        match self {
            BlendKind::Blend1 => Self::blend1(c1, c2, perc_other),
            BlendKind::Blend2 => Self::blend2(c1, c2, perc_other),
        }
    }

    pub fn blend1(c1: Rgba, c2: Rgba, perc_c2: f64) -> Rgba {
        let c1_a_perc = c1.a as f64 / 255.0;
        let c2_a_perc = c2.a as f64 / 255.0;
        let a_blend_perc = c1_a_perc + c2_a_perc - (c1_a_perc * c2_a_perc);

        let perc_c1 = 1. - perc_c2;

        let r = ((c1.r as f64 * perc_c1 * c1_a_perc + c2.r as f64 * perc_c2 * c2_a_perc)
            / a_blend_perc) as u8;
        let g = ((c1.g as f64 * perc_c1 * c1_a_perc + c2.g as f64 * perc_c2 * c2_a_perc)
            / a_blend_perc) as u8;
        let b = ((c1.b as f64 * perc_c1 * c1_a_perc + c2.b as f64 * perc_c2 * c2_a_perc)
            / a_blend_perc) as u8;

        let a = (a_blend_perc * 255.0) as u8;
        Rgba::new_with_alpha(r, g, b, a)
    }

    /// This is a different blend function that takes into account the alpha of the colors
    /// and mixes in the opposite color for each alpha.
    pub fn blend2(c1: Rgba, c2: Rgba, perc_c2: f64) -> Rgba {
        let c1_a_perc = c1.a as f64 / 255.0;
        let c2_a_perc = c2.a as f64 / 255.0;
        let a_blend_perc = c1_a_perc + c2_a_perc - (c1_a_perc * c2_a_perc);

        let perc_c1 = 1. - perc_c2;

        let r = ((c1.r as f64 * perc_c1 * c1_a_perc
            + c1.r as f64 * perc_c2 * (1. - c2_a_perc)
            + c2.r as f64 * perc_c2 * c2_a_perc
            + c2.r as f64 * perc_c1 * (1. - c1_a_perc))
            / a_blend_perc) as u8;
        let g = ((c1.g as f64 * perc_c1 * c1_a_perc
            + c1.g as f64 * perc_c2 * (1. - c2_a_perc)
            + c2.g as f64 * perc_c2 * c2_a_perc
            + c2.g as f64 * perc_c1 * (1. - c1_a_perc))
            / a_blend_perc) as u8;
        let b = ((c1.b as f64 * perc_c1 * c1_a_perc
            + c1.b as f64 * perc_c2 * (1. - c2_a_perc)
            + c2.b as f64 * perc_c2 * c2_a_perc
            + c2.b as f64 * perc_c1 * (1. - c1_a_perc))
            / a_blend_perc) as u8;

        let a = (a_blend_perc * 255.0) as u8;
        Rgba::new_with_alpha(r, g, b, a)
    }
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    pub const fn new_with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// returns a tuple of the rgb values
    pub fn to_tuple(&self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    /// Multiply the color by a scalar amount
    pub fn mul(&self, amount: f64) -> Self {
        let r = (self.r as f64 * amount) as u8;
        let g = (self.g as f64 * amount) as u8;
        let b = (self.b as f64 * amount) as u8;
        Self::new(r, g, b)
    }

    /// considers the alpha of the self and blends with the previous color
    pub fn to_crossterm_color(&self, prev: Option<CrosstermColor>) -> CrosstermColor {
        let (r, g, b, a) = self.to_tuple();
        let a = a as f64 / 255.0;
        let prev = prev.unwrap_or(CrosstermColor::Rgb { r: 0, g: 0, b: 0 });
        let (pr, pg, pb) = match prev {
            CrosstermColor::Rgb { r, g, b } => (r, g, b),
            _ => (0, 0, 0),
        };
        let r = (r as f64 * a + pr as f64 * (1.0 - a)) as u8;
        let g = (g as f64 * a + pg as f64 * (1.0 - a)) as u8;
        let b = (b as f64 * a + pb as f64 * (1.0 - a)) as u8;
        CrosstermColor::Rgb { r, g, b }
    }

    pub fn overlay_color(&self, overlay: Self) -> Self {
        let (r, g, b, a) = overlay.to_tuple();
        let a = a as f64 / 255.0;
        let (under_r, under_g, under_b, under_a) = self.to_tuple();
        let r = (r as f64 * a + under_r as f64 * (1.0 - a)) as u8;
        let g = (g as f64 * a + under_g as f64 * (1.0 - a)) as u8;
        let b = (b as f64 * a + under_b as f64 * (1.0 - a)) as u8;
        Self {
            r,
            g,
            b,
            a: under_a,
        }
    }
}

#[rustfmt::skip]
impl Color {
    pub const TRANSPARENT:         Color = Color::new_with_alpha(0, 0, 0, 0);

    pub const GREY1:         Color = Color::new(10, 10, 10);
    pub const GREY2:         Color = Color::new(20, 20, 20);
    pub const GREY3:         Color = Color::new(30, 30, 32);
    pub const GREY4:         Color = Color::new(40, 40, 42);
    pub const GREY5:         Color = Color::new(50, 50, 55);
    pub const GREY6:         Color = Color::new(60, 60, 65);
    pub const GREY7:         Color = Color::new(70, 70, 75);
    pub const GREY8:         Color = Color::new(80, 80, 85);
    pub const GREY9:         Color = Color::new(90, 90, 95);
    pub const GREY10:        Color = Color::new(100, 100, 105);
    pub const GREY11:        Color = Color::new(110, 110, 115);
    pub const GREY12:        Color = Color::new(120, 120, 125);
    pub const GREY13:        Color = Color::new(130, 130, 135);
    pub const GREY14:        Color = Color::new(140, 140, 145);
    pub const GREY15:        Color = Color::new(150, 150, 155);
    pub const GREY16:        Color = Color::new(160, 160, 165);
    pub const GREY17:        Color = Color::new(170, 170, 175);
    pub const GREY18:        Color = Color::new(180, 180, 185);
    pub const GREY19:        Color = Color::new(190, 190, 195);
    pub const GREY20:        Color = Color::new(200, 200, 205);
    pub const GREY21:        Color = Color::new(210, 210, 215);
    pub const GREY22:        Color = Color::new(220, 220, 225);
    pub const GREY23:        Color = Color::new(230, 230, 235);
    pub const GREY24:        Color = Color::new(240, 240, 245);
    pub const GREY25:        Color = Color::new(250, 250, 255);

    pub const DIM_YELLOW:       Color = Color::new(200, 200, 100);
    pub const ORANGE2:          Color = Color::new(200, 100, 50);
    pub const YELLOW2:          Color = Color::new(220, 220, 0);
    pub const PALLID_BLUE:      Color = Color::new(90, 110, 190);
    pub const LIGHT_YELLOW2:    Color = Color::new(255, 255, 200);

    /// css named colours
    pub const MAROON:                  Color = Color::new(128, 0, 0);
    pub const DARK_RED:                Color = Color::new(139, 0, 0);
    pub const BROWN:                   Color = Color::new(165, 42, 42);
    pub const FIREBRICK:               Color = Color::new(178, 34, 34);
    pub const CRIMSON:                 Color = Color::new(220, 20, 60);
    pub const RED:                     Color = Color::new(255, 0, 0);
    pub const TOMATO:                  Color = Color::new(255, 99, 71);
    pub const CORAL:                   Color = Color::new(255, 127, 80);
    pub const INDIAN_RED:              Color = Color::new(205, 92, 92);
    pub const LIGHT_CORAL:             Color = Color::new(240, 128, 128);
    pub const DARK_SALMON:             Color = Color::new(233, 150, 122);
    pub const SALMON:                  Color = Color::new(250, 128, 114);
    pub const LIGHT_SALMON:            Color = Color::new(255, 160, 122);
    pub const ORANGE_RED:              Color = Color::new(255, 69, 0);
    pub const DARK_ORANGE:             Color = Color::new(255, 140, 0);
    pub const ORANGE:                  Color = Color::new(255, 165, 0);
    pub const GOLD:                    Color = Color::new(255, 215, 0);
    pub const DARK_GOLDEN_ROD:         Color = Color::new(184, 134, 11);
    pub const GOLDEN_ROD:              Color = Color::new(218, 165, 32);
    pub const PALE_GOLDEN_ROD:         Color = Color::new(238, 232, 170);
    pub const DARK_KHAKI:              Color = Color::new(189, 183, 107);
    pub const KHAKI:                   Color = Color::new(240, 230, 140);
    pub const OLIVE:                   Color = Color::new(128, 128, 0);
    pub const YELLOW:                  Color = Color::new(255, 255, 0);
    pub const YELLOW_GREEN:            Color = Color::new(154, 205, 50);
    pub const DARK_OLIVE_GREEN:        Color = Color::new(85, 107, 47);
    pub const OLIVE_DRAB:              Color = Color::new(107, 142, 35);
    pub const LAWN_GREEN:              Color = Color::new(124, 252, 0);
    pub const CHARTREUSE:              Color = Color::new(127, 255, 0);
    pub const GREEN_YELLOW:            Color = Color::new(173, 255, 47);
    pub const DARK_GREEN:              Color = Color::new(0, 100, 0);
    pub const GREEN:                   Color = Color::new(0, 128, 0);
    pub const FOREST_GREEN:            Color = Color::new(34, 139, 34);
    pub const LIME:                    Color = Color::new(0, 255, 0);
    pub const LIME_GREEN:              Color = Color::new(50, 205, 50);
    pub const LIGHT_GREEN:             Color = Color::new(144, 238, 144);
    pub const PALE_GREEN:              Color = Color::new(152, 251, 152);
    pub const DARK_SEA_GREEN:          Color = Color::new(143, 188, 143);
    pub const MEDIUM_SPRING_GREEN:     Color = Color::new(0, 250, 154);
    pub const SPRING_GREEN:            Color = Color::new(0, 255, 127);
    pub const SEA_GREEN:               Color = Color::new(46, 139, 87);
    pub const MEDIUM_AQUA_MARINE:      Color = Color::new(102, 205, 170);
    pub const MEDIUM_SEA_GREEN:        Color = Color::new(60, 179, 113);
    pub const LIGHT_SEA_GREEN:         Color = Color::new(32, 178, 170);
    pub const DARK_SLATE_GRAY:         Color = Color::new(47, 79, 79);
    pub const TEAL:                    Color = Color::new(0, 128, 128);
    pub const DARK_CYAN:               Color = Color::new(0, 139, 139);
    pub const AQUA:                    Color = Color::new(0, 255, 255);
    pub const CYAN:                    Color = Color::new(0, 255, 255);
    pub const LIGHT_CYAN:              Color = Color::new(224, 255, 255);
    pub const DARK_TURQUOISE:          Color = Color::new(0, 206, 209);
    pub const TURQUOISE:               Color = Color::new(64, 224, 208);
    pub const MEDIUM_TURQUOISE:        Color = Color::new(72, 209, 204);
    pub const PALE_TURQUOISE:          Color = Color::new(175, 238, 238);
    pub const AQUA_MARINE:             Color = Color::new(127, 255, 212);
    pub const POWDER_BLUE:             Color = Color::new(176, 224, 230);
    pub const CADET_BLUE:              Color = Color::new(95, 158, 160);
    pub const STEEL_BLUE:              Color = Color::new(70, 130, 180);
    pub const CORNFLOWER_BLUE:         Color = Color::new(100, 149, 237);
    pub const DEEP_SKY_BLUE:           Color = Color::new(0, 191, 255);
    pub const DODGER_BLUE:             Color = Color::new(30, 144, 255);
    pub const LIGHT_BLUE:              Color = Color::new(173, 216, 230);
    pub const SKY_BLUE:                Color = Color::new(135, 206, 235);
    pub const LIGHT_SKY_BLUE:          Color = Color::new(135, 206, 250);
    pub const MIDNIGHT_BLUE:           Color = Color::new(25, 25, 112);
    pub const NAVY:                    Color = Color::new(0, 0, 128);
    pub const DARK_BLUE:               Color = Color::new(0, 0, 139);
    pub const MEDIUM_BLUE:             Color = Color::new(0, 0, 205);
    pub const BLUE:                    Color = Color::new(0, 0, 255);
    pub const ROYAL_BLUE:              Color = Color::new(65, 105, 225);
    pub const BLUE_VIOLET:             Color = Color::new(138, 43, 226);
    pub const INDIGO:                  Color = Color::new(75, 0, 130);
    pub const DARK_SLATE_BLUE:         Color = Color::new(72, 61, 139);
    pub const SLATE_BLUE:              Color = Color::new(106, 90, 205);
    pub const MEDIUM_SLATE_BLUE:       Color = Color::new(123, 104, 238);
    pub const MEDIUM_PURPLE:           Color = Color::new(147, 112, 219);
    pub const DARK_MAGENTA:            Color = Color::new(139, 0, 139);
    pub const DARK_VIOLET:             Color = Color::new(148, 0, 211);
    pub const DARK_ORCHID:             Color = Color::new(153, 50, 204);
    pub const MEDIUM_ORCHID:           Color = Color::new(186, 85, 211);
    pub const PURPLE:                  Color = Color::new(128, 0, 128);
    pub const THISTLE:                 Color = Color::new(216, 191, 216);
    pub const PLUM:                    Color = Color::new(221, 160, 221);
    pub const VIOLET:                  Color = Color::new(238, 130, 238);
    pub const MAGENTA:                 Color = Color::new(255, 0, 255);
    pub const FUCHSIA:                 Color = Color::new(255, 0, 255);
    pub const ORCHID:                  Color = Color::new(218, 112, 214);
    pub const MEDIUM_VIOLET_RED:       Color = Color::new(199, 21, 133);
    pub const PALE_VIOLET_RED:         Color = Color::new(219, 112, 147);
    pub const DEEP_PINK:               Color = Color::new(255, 20, 147);
    pub const HOT_PINK:                Color = Color::new(255, 105, 180);
    pub const LIGHT_PINK:              Color = Color::new(255, 182, 193);
    pub const PINK:                    Color = Color::new(255, 192, 203);
    pub const ANTIQUE_WHITE:           Color = Color::new(250, 235, 215);
    pub const BEIGE:                   Color = Color::new(245, 245, 220);
    pub const BISQUE:                  Color = Color::new(255, 228, 196);
    pub const BLANCHED_ALMOND:         Color = Color::new(255, 235, 205);
    pub const WHEAT:                   Color = Color::new(245, 222, 179);
    pub const CORN_SILK:               Color = Color::new(255, 248, 220);
    pub const LEMON_CHIFFON:           Color = Color::new(255, 250, 205);
    pub const LIGHT_GOLDEN_ROD_YELLOW: Color = Color::new(250, 250, 210);
    pub const LIGHT_YELLOW:            Color = Color::new(255, 255, 224);
    pub const SADDLE_BROWN:            Color = Color::new(139, 69, 19);
    pub const SIENNA:                  Color = Color::new(160, 82, 45);
    pub const CHOCOLATE:               Color = Color::new(210, 105, 30);
    pub const PERU:                    Color = Color::new(205, 133, 63);
    pub const SANDY_BROWN:             Color = Color::new(244, 164, 96);
    pub const BURLY_WOOD:              Color = Color::new(222, 184, 135);
    pub const TAN:                     Color = Color::new(210, 180, 140);
    pub const ROSY_BROWN:              Color = Color::new(188, 143, 143);
    pub const MOCCASIN:                Color = Color::new(255, 228, 181);
    pub const NAVAJO_WHITE:            Color = Color::new(255, 222, 173);
    pub const PEACH_PUFF:              Color = Color::new(255, 218, 185);
    pub const MISTY_ROSE:              Color = Color::new(255, 228, 225);
    pub const LAVENDER_BLUSH:          Color = Color::new(255, 240, 245);
    pub const LINEN:                   Color = Color::new(250, 240, 230);
    pub const OLD_LACE:                Color = Color::new(253, 245, 230);
    pub const PAPAYA_WHIP:             Color = Color::new(255, 239, 213);
    pub const SEA_SHELL:               Color = Color::new(255, 245, 238);
    pub const MINT_CREAM:              Color = Color::new(245, 255, 250);
    pub const SLATE_GRAY:              Color = Color::new(112, 128, 144);
    pub const LIGHT_SLATE_GRAY:        Color = Color::new(119, 136, 153);
    pub const LIGHT_STEEL_BLUE:        Color = Color::new(176, 196, 222);
    pub const LAVENDER:                Color = Color::new(230, 230, 250);
    pub const FLORAL_WHITE:            Color = Color::new(255, 250, 240);
    pub const ALICE_BLUE:              Color = Color::new(240, 248, 255);
    pub const GHOST_WHITE:             Color = Color::new(248, 248, 255);
    pub const HONEYDEW:                Color = Color::new(240, 255, 240);
    pub const IVORY:                   Color = Color::new(255, 255, 240);
    pub const AZURE:                   Color = Color::new(240, 255, 255);
    pub const SNOW:                    Color = Color::new(255, 250, 250);
    pub const BLACK:                   Color = Color::new(0, 0, 0);
    pub const DIM_GRAY:                Color = Color::new(105, 105, 105);
    pub const DIM_GREY:                Color = Color::new(105, 105, 105);
    pub const GRAY:                    Color = Color::new(128, 128, 128);
    pub const GREY:                    Color = Color::new(128, 128, 128);
    pub const DARK_GRAY:               Color = Color::new(169, 169, 169);
    pub const DARK_GREY:               Color = Color::new(169, 169, 169);
    pub const SILVER:                  Color = Color::new(192, 192, 192);
    pub const LIGHT_GRAY:              Color = Color::new(211, 211, 211);
    pub const LIGHT_GREY:              Color = Color::new(211, 211, 211);
    pub const GAINSBORO:               Color = Color::new(220, 220, 220);
    pub const WHITE_SMOKE:             Color = Color::new(245, 245, 245);
    pub const WHITE:                   Color = Color::new(255, 255, 255);

    pub fn from_name(name: &str) -> Color {
        // normalize the name
        let name = name.to_lowercase();
        let name = name.replace(' ', "_");

        match name.as_str() {
            "maroon"                  => Self::MAROON,
            "dark_red"                => Self::DARK_RED,
            "brown"                   => Self::BROWN,
            "firebrick"               => Self::FIREBRICK,
            "crimson"                 => Self::CRIMSON,
            "red"                     => Self::RED,
            "tomato"                  => Self::TOMATO,
            "coral"                   => Self::CORAL,
            "indian_red"              => Self::INDIAN_RED,
            "light_coral"             => Self::LIGHT_CORAL,
            "dark_salmon"             => Self::DARK_SALMON,
            "salmon"                  => Self::SALMON,
            "light_salmon"            => Self::LIGHT_SALMON,
            "orange_red"              => Self::ORANGE_RED,
            "dark_orange"             => Self::DARK_ORANGE,
            "orange"                  => Self::ORANGE,
            "gold"                    => Self::GOLD,
            "dark_golden_rod"         => Self::DARK_GOLDEN_ROD,
            "golden_rod"              => Self::GOLDEN_ROD,
            "pale_golden_rod"         => Self::PALE_GOLDEN_ROD,
            "dark_khaki"              => Self::DARK_KHAKI,
            "khaki"                   => Self::KHAKI,
            "olive"                   => Self::OLIVE,
            "yellow"                  => Self::YELLOW,
            "yellow_green"            => Self::YELLOW_GREEN,
            "dark_olive_green"        => Self::DARK_OLIVE_GREEN,
            "olive_drab"              => Self::OLIVE_DRAB,
            "lawn_green"              => Self::LAWN_GREEN,
            "chartreuse"              => Self::CHARTREUSE,
            "green_yellow"            => Self::GREEN_YELLOW,
            "dark_green"              => Self::DARK_GREEN,
            "green"                   => Self::GREEN,
            "forest_green"            => Self::FOREST_GREEN,
            "lime"                    => Self::LIME,
            "lime_green"              => Self::LIME_GREEN,
            "light_green"             => Self::LIGHT_GREEN,
            "pale_green"              => Self::PALE_GREEN,
            "dark_sea_green"          => Self::DARK_SEA_GREEN,
            "medium_spring_green"     => Self::MEDIUM_SPRING_GREEN,
            "spring_green"            => Self::SPRING_GREEN,
            "sea_green"               => Self::SEA_GREEN,
            "medium_aqua_marine"      => Self::MEDIUM_AQUA_MARINE,
            "medium_sea_green"        => Self::MEDIUM_SEA_GREEN,
            "light_sea_green"         => Self::LIGHT_SEA_GREEN,
            "dark_slate_gray"         => Self::DARK_SLATE_GRAY,
            "teal"                    => Self::TEAL,
            "dark_cyan"               => Self::DARK_CYAN,
            "aqua"                    => Self::AQUA,
            "cyan"                    => Self::CYAN,
            "light_cyan"              => Self::LIGHT_CYAN,
            "dark_turquoise"          => Self::DARK_TURQUOISE,
            "turquoise"               => Self::TURQUOISE,
            "medium_turquoise"        => Self::MEDIUM_TURQUOISE,
            "pale_turquoise"          => Self::PALE_TURQUOISE,
            "aqua_marine"             => Self::AQUA_MARINE,
            "powder_blue"             => Self::POWDER_BLUE,
            "cadet_blue"              => Self::CADET_BLUE,
            "steel_blue"              => Self::STEEL_BLUE,
            "cornflower_blue"         => Self::CORNFLOWER_BLUE,
            "deep_sky_blue"           => Self::DEEP_SKY_BLUE,
            "dodger_blue"             => Self::DODGER_BLUE,
            "light_blue"              => Self::LIGHT_BLUE,
            "sky_blue"                => Self::SKY_BLUE,
            "light_sky_blue"          => Self::LIGHT_SKY_BLUE,
            "midnight_blue"           => Self::MIDNIGHT_BLUE,
            "navy"                    => Self::NAVY,
            "dark_blue"               => Self::DARK_BLUE,
            "medium_blue"             => Self::MEDIUM_BLUE,
            "blue"                    => Self::BLUE,
            "royal_blue"              => Self::ROYAL_BLUE,
            "blue_violet"             => Self::BLUE_VIOLET,
            "indigo"                  => Self::INDIGO,
            "dark_slate_blue"         => Self::DARK_SLATE_BLUE,
            "slate_blue"              => Self::SLATE_BLUE,
            "medium_slate_blue"       => Self::MEDIUM_SLATE_BLUE,
            "medium_purple"           => Self::MEDIUM_PURPLE,
            "dark_magenta"            => Self::DARK_MAGENTA,
            "dark_violet"             => Self::DARK_VIOLET,
            "dark_orchid"             => Self::DARK_ORCHID,
            "medium_orchid"           => Self::MEDIUM_ORCHID,
            "purple"                  => Self::PURPLE,
            "thistle"                 => Self::THISTLE,
            "plum"                    => Self::PLUM,
            "violet"                  => Self::VIOLET,
            "magenta"                 => Self::MAGENTA,
            "fuchsia"                 => Self::FUCHSIA,
            "orchid"                  => Self::ORCHID,
            "medium_violet_red"       => Self::MEDIUM_VIOLET_RED,
            "pale_violet_red"         => Self::PALE_VIOLET_RED,
            "deep_pink"               => Self::DEEP_PINK,
            "hot_pink"                => Self::HOT_PINK,
            "light_pink"              => Self::LIGHT_PINK,
            "pink"                    => Self::PINK,
            "antique_white"           => Self::ANTIQUE_WHITE,
            "beige"                   => Self::BEIGE,
            "bisque"                  => Self::BISQUE,
            "blanched_almond"         => Self::BLANCHED_ALMOND,
            "wheat"                   => Self::WHEAT,
            "corn_silk"               => Self::CORN_SILK,
            "lemon_chiffon"           => Self::LEMON_CHIFFON,
            "light_golden_rod_yellow" => Self::LIGHT_GOLDEN_ROD_YELLOW,
            "light_yellow"            => Self::LIGHT_YELLOW,
            "saddle_brown"            => Self::SADDLE_BROWN,
            "sienna"                  => Self::SIENNA,
            "chocolate"               => Self::CHOCOLATE,
            "peru"                    => Self::PERU,
            "sandy_brown"             => Self::SANDY_BROWN,
            "burly_wood"              => Self::BURLY_WOOD,
            "tan"                     => Self::TAN,
            "rosy_brown"              => Self::ROSY_BROWN,
            "moccasin"                => Self::MOCCASIN,
            "navajo_white"            => Self::NAVAJO_WHITE,
            "peach_puff"              => Self::PEACH_PUFF,
            "misty_rose"              => Self::MISTY_ROSE,
            "lavender_blush"          => Self::LAVENDER_BLUSH,
            "linen"                   => Self::LINEN,
            "old_lace"                => Self::OLD_LACE,
            "papaya_whip"             => Self::PAPAYA_WHIP,
            "sea_shell"               => Self::SEA_SHELL,
            "mint_cream"              => Self::MINT_CREAM,
            "slate_gray"              => Self::SLATE_GRAY,
            "light_slate_gray"        => Self::LIGHT_SLATE_GRAY,
            "light_steel_blue"        => Self::LIGHT_STEEL_BLUE,
            "lavender"                => Self::LAVENDER,
            "floral_white"            => Self::FLORAL_WHITE,
            "alice_blue"              => Self::ALICE_BLUE,
            "ghost_white"             => Self::GHOST_WHITE,
            "honeydew"                => Self::HONEYDEW,
            "ivory"                   => Self::IVORY,
            "azure"                   => Self::AZURE,
            "snow"                    => Self::SNOW,
            "black"                   => Self::BLACK,
            "space_gray"              => Self::DIM_GRAY,
            "space_grey"              => Self::DIM_GREY,
            "gray"                    => Self::GRAY,
            "grey"                    => Self::GREY,
            "dark_gray"               => Self::DARK_GRAY,
            "dark_grey"               => Self::DARK_GREY,
            "silver"                  => Self::SILVER,
            "light_gray"              => Self::LIGHT_GRAY,
            "light_grey"              => Self::LIGHT_GREY,
            "gainsboro"               => Self::GAINSBORO,
            "white_smoke"             => Self::WHITE_SMOKE,
            "white"                   => Self::WHITE,
            _                         => Self::WHITE,
        }

    }

}
