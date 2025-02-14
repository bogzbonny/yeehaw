use {
    crate::{Context, DynVal, Size},
    crossterm::style::Color as CrosstermColor,
    parking_lot::RwLock,
    rand::Rng,
    std::sync::Arc,
    std::time::Duration,
    //std::{cell::RefCell, rc::Rc},
};

/// The color store is a simple store for complex data within each color. This allows for
/// significantly less data to be cloned each time a color is cloned. This is a bit annoying to
/// work with as we have to pass the store around constantly - but the performance boost is
/// massive, especially visible with patterns.
#[derive(Clone, Default, Debug)]
#[allow(clippy::type_complexity)]
pub struct ColorStore {
    // NOTE the second bool is a store of if the gradient is effected by time
    pub pos_gradients: Arc<RwLock<Vec<(Vec<(DynVal, Color)>, bool)>>>,
    pub time_gradients: Arc<RwLock<Vec<Vec<(Duration, Color)>>>>, // no need for the bool, time gradients are ALWAYS time effected
    pub patterns: Arc<RwLock<Vec<(Vec<Vec<Color>>, bool)>>>,      //(Vec< (y) < Vec< (x) < Color>>>)
}

impl ColorStore {
    pub fn add_pattern(&self, pattern: Vec<Vec<Color>>) -> usize {
        let mut time_effected = false;
        for c in pattern.iter().flatten() {
            if c.is_time_effected(self) {
                time_effected = true;
                break;
            }
        }

        // attempt to find the pattern in the store before adding it
        for (i, p) in self.patterns.read().iter().enumerate() {
            if p.0 == pattern {
                return i;
            }
        }
        self.patterns.write().push((pattern, time_effected));
        self.patterns.read().len() - 1
    }
    pub fn add_pos_gradient(&self, gr: Vec<(DynVal, Color)>) -> usize {
        let mut time_effected = false;
        for (_, c) in gr.iter() {
            if c.is_time_effected(self) {
                time_effected = true;
                break;
            }
        }

        // attempt to find the gradient in the store before adding it
        for (i, g) in self.pos_gradients.read().iter().enumerate() {
            if g.0 == gr {
                return i;
            }
        }
        self.pos_gradients.write().push((gr, time_effected));
        self.pos_gradients.read().len() - 1
    }
    pub fn add_time_gradient(&self, gr: Vec<(Duration, Color)>) -> usize {
        // attempt to find the gradient in the store before adding it
        for (i, g) in self.time_gradients.read().iter().enumerate() {
            if g == &gr {
                return i;
            }
        }
        self.time_gradients.write().push(gr);
        self.time_gradients.read().len() - 1
    }

    pub fn is_pattern_time_effected(&self, id: usize) -> bool {
        if let Some((_, te)) = self.patterns.read().get(id) {
            return *te;
        }
        false
    }

    pub fn is_gradient_time_effected(&self, id: usize) -> bool {
        if let Some((_, te)) = self.pos_gradients.read().get(id) {
            return *te;
        }
        false
    }
}

// TODO color radial pinwheel (each angle has a different color)
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub enum Color {
    ANSI(CrosstermColor),
    Rgba(Rgba),
    Gradient(Gradient),
    RadialGradient(RadialGradient),
    TimeGradient(TimeGradient),
    Pattern(Pattern),
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

impl From<vt100_yh::Color> for Color {
    #[inline]
    fn from(value: vt100_yh::Color) -> Self {
        match value {
            vt100_yh::Color::Default => Self::ANSI(CrosstermColor::Reset),
            vt100_yh::Color::Idx(i) => Self::ANSI(CrosstermColor::AnsiValue(i)),
            vt100_yh::Color::Rgb(r, g, b) => Self::Rgba(Rgba::new(r, g, b)),
        }
    }
}

impl From<Rgba> for Color {
    fn from(c: Rgba) -> Self {
        Self::Rgba(c)
    }
}

impl From<Gradient> for Color {
    fn from(c: Gradient) -> Self {
        Self::Gradient(c)
    }
}

impl From<RadialGradient> for Color {
    fn from(c: RadialGradient) -> Self {
        Self::RadialGradient(c)
    }
}

impl From<TimeGradient> for Color {
    fn from(c: TimeGradient) -> Self {
        Self::TimeGradient(c)
    }
}

impl From<Pattern> for Color {
    fn from(c: Pattern) -> Self {
        Self::Pattern(c)
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

    // is the evaluation of the color effected by time
    pub fn is_time_effected(&self, cs: &ColorStore) -> bool {
        match self {
            Color::TimeGradient(_) => true,
            Color::Gradient(c) => c.is_time_effected(cs),
            Color::RadialGradient(c) => c.is_time_effected(cs),
            Color::Pattern(c) => c.is_time_effected(cs),
            _ => false,
        }
    }

    pub fn to_rgba(&self) -> Rgba {
        match self {
            Color::ANSI(c) => crossterm_to_rgb(*c).to_rgba(),
            Color::Rgba(c) => *c,
            _ => Rgba::new_with_alpha(0, 0, 0, 0),
        }
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

    #[allow(clippy::too_many_arguments)]
    /// blends two colors together with the given percentage of the other color
    pub fn blend(
        &self, cs: &ColorStore, dsl: &Duration, draw_size: &Size, x: u16, y: u16, other: Color,
        percent_other: f64,
    ) -> Color {
        match self {
            Color::ANSI(a) => {
                if a == &CrosstermColor::Reset {
                    return other.clone();
                }
                let c = crossterm_to_rgb(*a);
                c.blend(cs, dsl, draw_size, x, y, other, percent_other)
            }
            Color::Rgba(c) => match other {
                Color::ANSI(a) => {
                    if a == CrosstermColor::Reset {
                        return self.clone();
                    }
                    let oc = crossterm_to_rgb(a);
                    self.blend(cs, dsl, draw_size, x, y, oc, percent_other)
                }
                Color::Rgba(oc) => Color::Rgba(blend(*c, oc, percent_other)),
                Color::Gradient(gr) => {
                    let gr = gr.to_color(cs, dsl, draw_size, x, y);
                    self.clone()
                        .blend(cs, dsl, draw_size, x, y, gr, percent_other)
                }
                Color::TimeGradient(g) => {
                    let g = g.to_color(cs, dsl, draw_size, x, y);
                    self.clone()
                        .blend(cs, dsl, draw_size, x, y, g, percent_other)
                }
                Color::RadialGradient(rg) => {
                    let rg = rg.to_color(cs, dsl, draw_size, x, y);
                    self.clone()
                        .blend(cs, dsl, draw_size, x, y, rg, percent_other)
                }
                Color::Pattern(p) => {
                    let p = p.to_color(cs, dsl, x, y);
                    self.clone()
                        .blend(cs, dsl, draw_size, x, y, p, percent_other)
                }
            },
            Color::Gradient(gr) => {
                let gr = gr.to_color(cs, dsl, draw_size, x, y);
                gr.blend(cs, dsl, draw_size, x, y, other, percent_other)
            }
            Color::TimeGradient(gr) => {
                let gr = gr.to_color(cs, dsl, draw_size, x, y);
                gr.blend(cs, dsl, draw_size, x, y, other, percent_other)
            }
            Color::RadialGradient(gr) => {
                let gr = gr.to_color(cs, dsl, draw_size, x, y);
                gr.blend(cs, dsl, draw_size, x, y, other, percent_other)
            }
            Color::Pattern(p) => {
                let p = p.to_color(cs, dsl, x, y);
                p.blend(cs, dsl, draw_size, x, y, other, percent_other)
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

    pub fn darken(&self, store: &ColorStore) -> Self {
        match self {
            Color::ANSI(c) => Color::darken_ansi(c, store),
            Color::Rgba(c) => Color::Rgba(c.mul(0.5)),
            Color::Gradient(g) => Color::Gradient(
                g.apply_fn_to_colors(store, Box::new(move |store, c| c.darken(store))),
            ),
            Color::TimeGradient(g) => Color::TimeGradient(
                g.apply_fn_to_colors(store, Box::new(move |store, c| c.darken(store))),
            ),
            Color::RadialGradient(g) => Color::RadialGradient(
                g.apply_fn_to_colors(store, Box::new(move |store, c| c.darken(store))),
            ),
            Color::Pattern(p) => Color::Pattern(
                p.apply_fn_to_colors(store, Box::new(move |store, c| c.darken(store))),
            ),
        }
    }

    pub fn lighten(&self, store: &ColorStore) -> Self {
        match self {
            Color::ANSI(c) => Color::lighten_ansi(c, store),
            Color::Rgba(c) => Color::Rgba(c.mul(1.5)),
            Color::Gradient(g) => Color::Gradient(
                g.apply_fn_to_colors(store, Box::new(move |store, c| c.lighten(store))),
            ),
            Color::TimeGradient(g) => Color::TimeGradient(
                g.apply_fn_to_colors(store, Box::new(move |store, c| c.lighten(store))),
            ),
            Color::RadialGradient(g) => Color::RadialGradient(
                g.apply_fn_to_colors(store, Box::new(move |store, c| c.lighten(store))),
            ),
            Color::Pattern(p) => Color::Pattern(
                p.apply_fn_to_colors(store, Box::new(move |store, c| c.lighten(store))),
            ),
        }
    }

    pub fn darken_ansi(c: &CrosstermColor, store: &ColorStore) -> Color {
        match c {
            CrosstermColor::Red => Color::ANSI(CrosstermColor::DarkRed),
            CrosstermColor::Green => Color::ANSI(CrosstermColor::DarkGreen),
            CrosstermColor::Yellow => Color::ANSI(CrosstermColor::DarkYellow),
            CrosstermColor::Blue => Color::ANSI(CrosstermColor::DarkBlue),
            CrosstermColor::Magenta => Color::ANSI(CrosstermColor::DarkMagenta),
            CrosstermColor::Cyan => Color::ANSI(CrosstermColor::DarkCyan),
            CrosstermColor::Grey => Color::ANSI(CrosstermColor::DarkGrey),
            CrosstermColor::White => Color::ANSI(CrosstermColor::Grey),
            CrosstermColor::Rgb { r, g, b } => Color::new(*r, *g, *b).darken(store),
            _ => Color::ANSI(*c),
        }
    }

    pub fn lighten_ansi(c: &CrosstermColor, store: &ColorStore) -> Color {
        match c {
            CrosstermColor::DarkRed => Color::ANSI(CrosstermColor::Red),
            CrosstermColor::DarkGreen => Color::ANSI(CrosstermColor::Green),
            CrosstermColor::DarkYellow => Color::ANSI(CrosstermColor::Yellow),
            CrosstermColor::DarkBlue => Color::ANSI(CrosstermColor::Blue),
            CrosstermColor::DarkMagenta => Color::ANSI(CrosstermColor::Magenta),
            CrosstermColor::DarkCyan => Color::ANSI(CrosstermColor::Cyan),
            CrosstermColor::DarkGrey => Color::ANSI(CrosstermColor::Grey),
            CrosstermColor::Grey => Color::ANSI(CrosstermColor::White),
            CrosstermColor::Rgb { r, g, b } => Color::new(*r, *g, *b).lighten(store),
            _ => Color::ANSI(*c),
        }
    }

    /// considers the alpha of the self and blends with the previous color
    pub fn to_crossterm_color(
        &self, cs: &ColorStore, dsl: &Duration, draw_size: &Size, prev: Option<CrosstermColor>,
        x: u16, y: u16,
    ) -> CrosstermColor {
        match self {
            Color::ANSI(c) => *c,
            Color::Rgba(c) => c.to_crossterm_color(prev),
            Color::Gradient(gr) => gr
                .to_color(cs, dsl, draw_size, x, y)
                .to_crossterm_color(cs, dsl, draw_size, prev, x, y),
            Color::TimeGradient(g) => g
                .to_color(cs, dsl, draw_size, x, y)
                .to_crossterm_color(cs, dsl, draw_size, prev, x, y),
            Color::RadialGradient(rg) => rg
                .to_color(cs, dsl, draw_size, x, y)
                .to_crossterm_color(cs, dsl, draw_size, prev, x, y),
            Color::Pattern(p) => p
                .to_color(cs, dsl, x, y)
                .to_crossterm_color(cs, dsl, draw_size, prev, x, y),
        }
    }

    /// to color with the given context and position
    pub fn to_color(
        self, cs: &ColorStore, dsl: &Duration, draw_size: &Size, x: u16, y: u16,
    ) -> Color {
        match self {
            Color::ANSI(_) | Color::Rgba(_) => self,
            Color::Gradient(gr) => gr.to_color(cs, dsl, draw_size, x, y),
            Color::TimeGradient(g) => g.to_color(cs, dsl, draw_size, x, y),
            Color::RadialGradient(rg) => rg.to_color(cs, dsl, draw_size, x, y),
            Color::Pattern(p) => p.to_color(cs, dsl, x, y),
        }
    }

    /// set the offset of position dependent colors
    pub fn set_draw_size_if_unset(&mut self, s: Size) {
        match self {
            Color::Gradient(ref mut gr) => gr.set_draw_size_if_unset(s),
            Color::RadialGradient(ref mut rg) => rg.set_draw_size_if_unset(s),
            Color::Pattern(_) => {}
            Color::TimeGradient(_) => {}
            Color::Rgba(_) => {}
            Color::ANSI(_) => {}
        }
    }

    /// set the offset of position dependent colors
    pub fn add_to_offset(&mut self, x: i32, y: i32) {
        match self {
            Color::Gradient(ref mut gr) => gr.add_to_offset(x, y),
            Color::RadialGradient(ref mut rg) => rg.add_to_offset(x, y),
            Color::Pattern(ref mut p) => p.add_to_offset(x, y),
            Color::TimeGradient(_) => {}
            Color::Rgba(_) => {}
            Color::ANSI(_) => {}
        }
    }

    pub fn with_alpha(&self, store: &ColorStore, alpha: u8) -> Color {
        match self {
            Color::ANSI(a) => crossterm_to_rgb(*a).with_alpha(store, alpha),
            Color::Rgba(c) => {
                let mut c = *c;
                c.a = alpha;
                Color::Rgba(c)
            }
            Color::Gradient(g) => Color::Gradient(
                g.apply_fn_to_colors(store, Box::new(move |store, c| c.with_alpha(store, alpha))),
            ),
            Color::TimeGradient(g) => Color::TimeGradient(
                g.apply_fn_to_colors(store, Box::new(move |store, c| c.with_alpha(store, alpha))),
            ),
            Color::RadialGradient(g) => Color::RadialGradient(
                g.apply_fn_to_colors(store, Box::new(move |store, c| c.with_alpha(store, alpha))),
            ),
            Color::Pattern(p) => Color::Pattern(
                p.apply_fn_to_colors(store, Box::new(move |store, c| c.with_alpha(store, alpha))),
            ),
        }
    }

    pub fn overlay_color(&self, store: &ColorStore, overlay: Self) -> Self {
        match overlay {
            Color::Rgba(oc) => match self {
                Color::ANSI(a) => crossterm_to_rgb(*a).overlay_color(store, overlay),
                Color::Rgba(c) => Color::Rgba(c.overlay_color(oc)),
                Color::Gradient(g) => Color::Gradient(g.apply_fn_to_colors(
                    store,
                    Box::new(move |store, c| c.overlay_color(store, overlay.clone())),
                )),
                Color::TimeGradient(g) => Color::TimeGradient(g.apply_fn_to_colors(
                    store,
                    Box::new(move |store, c| c.overlay_color(store, overlay.clone())),
                )),
                Color::RadialGradient(g) => Color::RadialGradient(g.apply_fn_to_colors(
                    store,
                    Box::new(move |store, c| c.overlay_color(store, overlay.clone())),
                )),
                Color::Pattern(p) => Color::Pattern(p.apply_fn_to_colors(
                    store,
                    Box::new(move |store, c| c.overlay_color(store, overlay.clone())),
                )),
            },
            _ => overlay,
        }
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
    /// the draw size is the size to consider when drawing the gradient
    /// useful so that the gradient can be evaluated passively
    pub draw_size: Option<Size>,

    /// used to offset the gradient so that the gradient can be moved
    /// useful so that the gradient can be evaluated passively
    pub offset: (i32, i32),

    /// angle in degrees, starting from the positive-x-axis, moving clockwise
    pub angle_deg: f64,

    /// pos, color
    pub gradient_id: usize,
}

impl Gradient {
    pub fn new(ctx: &Context, grad: Vec<(DynVal, Color)>, mut angle_deg: f64) -> Self {
        if angle_deg < 0.0 {
            angle_deg += 360.0;
        } else if angle_deg >= 360.0 {
            angle_deg -= 360.0;
        }
        Gradient {
            draw_size: None,
            offset: (0, 0),
            angle_deg,
            gradient_id: ctx.color_store.add_pos_gradient(grad),
        }
    }

    pub fn new_x_grad_2_color(ctx: &Context, c1: Color, c2: Color) -> Self {
        let grad = vec![(DynVal::new_flex(0.), c1), (DynVal::FULL, c2)];
        Self::new(ctx, grad, 0.)
    }

    pub fn new_y_grad_2_color(ctx: &Context, c1: Color, c2: Color) -> Self {
        let grad = vec![(DynVal::new_flex(0.), c1), (DynVal::FULL, c2)];
        Self::new(ctx, grad, 90.)
    }

    /// length is the number of characters per color gradient
    pub fn new_x_grad_2_color_repeater(ctx: &Context, c1: Color, c2: Color, length: usize) -> Self {
        let grad = vec![
            (DynVal::new_fixed(0), c1.clone()),
            (DynVal::new_fixed(length as i32), c2),
            (DynVal::new_fixed(2 * length as i32), c1),
        ];
        Self::new(ctx, grad, 0.)
    }

    pub fn new_y_grad_2_color_repeater(ctx: &Context, c1: Color, c2: Color, length: usize) -> Self {
        let grad = vec![
            (DynVal::new_fixed(0), c1.clone()),
            (DynVal::new_fixed(length as i32), c2),
            (DynVal::new_fixed(2 * length as i32), c1),
        ];
        Self::new(ctx, grad, 90.)
    }

    /// length is the number of characters per color gradient
    pub fn new_x_grad_repeater(ctx: &Context, colors: Vec<Color>, length: usize) -> Self {
        Self::new_grad_repeater(ctx, colors, length, 0.)
    }

    pub fn new_y_grad_repeater(ctx: &Context, colors: Vec<Color>, length: usize) -> Self {
        Self::new_grad_repeater(ctx, colors, length, 90.)
    }

    /// length is the number of characters per color gradient
    pub fn new_grad_repeater(
        ctx: &Context, mut colors: Vec<Color>, length: usize, angle: f64,
    ) -> Self {
        if colors.is_empty() {
            return Gradient::default();
        }
        let mut v = Vec::with_capacity(colors.len() + 1);
        for (i, c) in colors.drain(..).enumerate() {
            v.push((DynVal::new_fixed((i * length) as i32), c));
        }
        v.push((DynVal::new_fixed((v.len() * length) as i32), v[0].1.clone()));
        Self::new(ctx, v, angle)
    }

    pub fn new_x_grad_repeater_time_loop(
        ctx: &Context, colors: Vec<Color>, length: usize, each_dur: Duration,
    ) -> (Self, Vec<TimeGradient>) {
        Self::new_grad_repeater_time_loop(ctx, colors, length, each_dur, 0.)
    }

    pub fn new_y_grad_repeater_time_loop(
        ctx: &Context, colors: Vec<Color>, length: usize, each_dur: Duration,
    ) -> (Self, Vec<TimeGradient>) {
        Self::new_grad_repeater_time_loop(ctx, colors, length, each_dur, 90.)
    }

    pub fn new_grad_repeater_time_loop(
        ctx: &Context, colors: Vec<Color>, length: usize, each_dur: Duration, angle: f64,
    ) -> (Self, Vec<TimeGradient>) {
        let mut out_tgs = Vec::with_capacity(colors.len());
        let mut time_colors = Vec::with_capacity(colors.len());
        for i in 0..colors.len() {
            let rotated_c = colors
                .clone()
                .iter()
                .cycle()
                .skip(i)
                .take(colors.len())
                .cloned()
                .collect();
            let tc = TimeGradient::new_loop(ctx, each_dur, rotated_c);
            out_tgs.push(tc.clone());
            time_colors.push(tc.into())
        }

        (
            Self::new_grad_repeater(ctx, time_colors, length, angle),
            out_tgs,
        )
    }

    pub fn x_grad_rainbow(ctx: &Context, length: usize) -> Self {
        Gradient::new_x_grad_repeater(
            ctx,
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

    pub fn y_grad_rainbow(ctx: &Context, length: usize) -> Self {
        Gradient::new_y_grad_repeater(
            ctx,
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

    pub fn x_grad_rainbow_time_loop(
        ctx: &Context, length: usize, each_dur: Duration,
    ) -> (Self, Vec<TimeGradient>) {
        Gradient::new_x_grad_repeater_time_loop(
            ctx,
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
            each_dur,
        )
    }

    pub fn y_grad_rainbow_time_loop(
        ctx: &Context, length: usize, each_dur: Duration,
    ) -> (Self, Vec<TimeGradient>) {
        Gradient::new_y_grad_repeater_time_loop(
            ctx,
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
            each_dur,
        )
    }

    pub fn with_angle(mut self, angle_deg: f64) -> Self {
        self.angle_deg = angle_deg;
        self
    }

    pub fn set_draw_size_if_unset(&mut self, s: Size) {
        if self.draw_size.is_none() {
            self.draw_size = Some(s);
        }
    }

    pub fn add_to_offset(&mut self, x: i32, y: i32) {
        self.offset.0 += x;
        self.offset.1 += y;
    }

    pub fn to_crossterm_color(
        &self, cs: &ColorStore, dsl: &Duration, draw_size: &Size, prev: Option<CrosstermColor>,
        x: u16, y: u16,
    ) -> CrosstermColor {
        self.to_color(cs, dsl, draw_size, x, y)
            .to_crossterm_color(cs, dsl, draw_size, prev, x, y)
    }

    pub fn len(&self, cs: &ColorStore) -> usize {
        let grs = cs.pos_gradients.read();
        let grad = grs.get(self.gradient_id);
        let Some(grad) = grad else {
            return 0;
        };
        grad.0.len()
    }

    pub fn get_grad(&self, cs: &ColorStore) -> Vec<(DynVal, Color)> {
        let grs = cs.pos_gradients.read();
        let grad = grs.get(self.gradient_id);
        let Some(grad) = grad else {
            return vec![];
        };
        grad.0.clone()
    }

    // is the evaluation of this gradient effected by time
    pub fn is_time_effected(&self, cs: &ColorStore) -> bool {
        cs.is_gradient_time_effected(self.gradient_id)
    }

    pub fn to_color(
        &self, cs: &ColorStore, dsl: &Duration, draw_size: &Size, x: u16, y: u16,
    ) -> Color {
        let grs = cs.pos_gradients.read();
        let grad = grs.get(self.gradient_id);
        let Some(grad) = grad else {
            return Color::TRANSPARENT;
        };

        if grad.0.is_empty() {
            return Color::TRANSPARENT;
        }

        let mut draw_size = *draw_size;
        let s = if let Some(s) = self.draw_size {
            draw_size = s;
            s
        } else {
            draw_size
        };

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
            let mut x_off = x as i32 - self.offset.0;
            x_off %= s.width as i32;
            x_off
        } else if self.angle_deg == 90. {
            let mut y_off = y as i32 - self.offset.1;
            y_off %= s.height as i32;
            y_off
        } else if self.angle_deg == 180. {
            let mut x_off = x as i32 - self.offset.0;
            x_off %= s.width as i32;
            -x_off
        } else if self.angle_deg == 270. {
            let mut y_off = y as i32 - self.offset.1;
            y_off %= s.height as i32;
            -y_off
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

            let x_off = x as i32 - self.offset.0;
            let y_off = y as i32 - self.offset.1;
            let x = x_off as f64;
            let y = y_off as f64;
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
        let max_pos = grad
            .0
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
        for ((p1, c1), (p2, c2)) in grad.0.windows(2).map(|w| (w[0].clone(), w[1].clone())) {
            if (p1.get_val(max_ctx_val) <= pos) && (pos < p2.get_val(max_ctx_val)) {
                start_clr = Some(c1.clone());
                end_clr = Some(c2.clone());
                start_pos = Some(p1.get_val(max_ctx_val));
                end_pos = Some(p2.get_val(max_ctx_val));
                break;
            }
        }
        let start_clr = start_clr.unwrap_or_else(|| grad.0[0].1.clone());
        let end_clr = end_clr.unwrap_or_else(|| grad.0[grad.0.len() - 1].1.clone());
        let start_pos = start_pos.unwrap_or_else(|| grad.0[0].0.get_val(max_ctx_val));
        let end_pos = end_pos.unwrap_or_else(|| grad.0[grad.0.len() - 1].0.get_val(max_ctx_val));
        let percent = (pos - start_pos) as f64 / (end_pos - start_pos) as f64;
        start_clr.blend(cs, dsl, &draw_size, x, y, end_clr, percent)
    }

    #[allow(clippy::type_complexity)]
    pub fn apply_fn_to_colors(
        &self, store: &ColorStore, f: Box<dyn Fn(&ColorStore, &Color) -> Color>,
    ) -> Self {
        let mod_gr = {
            let grs = store.pos_gradients.read();
            let gr = grs.get(self.gradient_id);
            let Some(gr) = gr else {
                return self.clone();
            };
            let mut mod_gr = gr.0.clone();
            for (_, c) in mod_gr.iter_mut() {
                *c = f(store, c);
            }
            mod_gr
        };
        let mut p = self.clone();
        let pattern_id = store.add_pos_gradient(mod_gr);
        p.gradient_id = pattern_id;
        p
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Default)]
pub struct RadialGradient {
    /// the draw size is the size to consider when drawing the gradient
    /// useful so that the gradient can be evaluated passively
    pub draw_size: Option<Size>,

    /// used to offset the gradient so that the gradient can be moved
    /// useful so that the gradient can be evaluated passively
    pub offset: (i32, i32),

    pub center: (DynVal, DynVal),
    /// x, y
    pub skew: (f64, f64),
    /// horizontal, vertical skew (as skew of (1., 1./0.55) seems to make a circle)
    //pub grad: Vec<(DynVal, Color)>,
    pub gradient_id: usize,
}

impl RadialGradient {
    pub fn new(
        ctx: &Context, grad: Vec<(DynVal, Color)>, center: (DynVal, DynVal), skew: (f64, f64),
    ) -> Self {
        RadialGradient {
            draw_size: None,
            offset: (0, 0),
            center,
            skew,
            gradient_id: ctx.color_store.add_pos_gradient(grad),
        }
    }

    /// creates a basic circle gradient with the given center, all colors are spaced evenly
    /// by the given distance 'each_dist'
    pub fn new_basic_circle(
        ctx: &Context, center: (DynVal, DynVal), each_dist: DynVal, colors: Vec<Color>,
    ) -> Self {
        let rgrad = colors
            .into_iter()
            .enumerate()
            .map(|(i, c)| (each_dist.clone().mul(i as f64), c))
            .collect();

        let id = ctx.color_store.add_pos_gradient(rgrad);

        RadialGradient {
            draw_size: None,
            offset: (0, 0),
            center,
            skew: (1., 1. / 0.55),
            gradient_id: id,
        }
    }

    /// creates a basic circle gradient with the given center, all colors are spaced evenly
    /// by the given distance 'each_dist'
    pub fn new_basic_circle_time_loop(
        ctx: &Context, center: (DynVal, DynVal), each_dur: Duration, each_dist: DynVal,
        colors: Vec<Color>,
    ) -> (Self, Vec<TimeGradient>) {
        let mut out_tgs = Vec::with_capacity(colors.len());
        let mut time_colors = Vec::with_capacity(colors.len());
        for i in 0..colors.len() {
            let rotated_c = colors
                .clone()
                .iter()
                .cycle()
                .skip(i)
                .take(colors.len())
                .cloned()
                .collect();
            let tc = TimeGradient::new_loop(ctx, each_dur, rotated_c);
            out_tgs.push(tc.clone());
            time_colors.push(tc.into())
        }

        (
            Self::new_basic_circle(ctx, center, each_dist, time_colors),
            out_tgs,
        )
    }

    pub fn len(&self, cs: &ColorStore) -> usize {
        let grs = cs.pos_gradients.read();
        let grad = grs.get(self.gradient_id);
        let Some(grad) = grad else {
            return 0;
        };
        grad.0.len()
    }

    pub fn get_grad(&self, cs: &ColorStore) -> Vec<(DynVal, Color)> {
        let grs = cs.pos_gradients.read();
        let grad = grs.get(self.gradient_id);
        let Some(grad) = grad else {
            return vec![];
        };
        grad.0.clone()
    }

    fn dist_from_center(
        x: f64, y: f64, center_x: f64, center_y: f64, skew_x: f64, skew_y: f64,
    ) -> f64 {
        let dx = x - center_x;
        let dy = y - center_y;
        let dx = skew_x * dx;
        let dy = skew_y * dy;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn set_draw_size_if_unset(&mut self, s: Size) {
        if self.draw_size.is_none() {
            self.draw_size = Some(s);
        }
    }

    pub fn add_to_offset(&mut self, x: i32, y: i32) {
        self.offset.0 += x;
        self.offset.1 += y;
    }

    // is the evaluation of this gradient effected by time
    pub fn is_time_effected(&self, cs: &ColorStore) -> bool {
        cs.is_gradient_time_effected(self.gradient_id)
    }

    pub fn to_color(
        &self, cs: &ColorStore, dsl: &Duration, draw_size: &Size, x: u16, y: u16,
    ) -> Color {
        let grs = cs.pos_gradients.read();
        let grad = grs.get(self.gradient_id);
        let Some(grad) = grad else {
            return Color::TRANSPARENT;
        };
        if grad.0.is_empty() {
            return Color::TRANSPARENT;
        }

        let mut draw_size = *draw_size;
        let s = if let Some(s) = self.draw_size {
            draw_size = s;
            s
        } else {
            draw_size
        };
        let x_off = x as f64 - self.offset.0 as f64;
        let y_off = y as f64 - self.offset.1 as f64;
        let (center_x, center_y) = (
            self.center.0.get_val(s.width) as f64,
            self.center.1.get_val(s.height) as f64,
        );
        let (skew_x, skew_y) = self.skew;
        let mut dist = Self::dist_from_center(x_off, y_off, center_x, center_y, skew_x, skew_y);

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
        for ((p1, c1), (p2, c2)) in grad.0.windows(2).map(|w| (w[0].clone(), w[1].clone())) {
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

        let start_clr = start_clr.unwrap_or_else(|| grad.0[0].1.clone());
        let end_clr = end_clr.unwrap_or_else(|| grad.0[grad.0.len() - 1].1.clone());
        let start_pos =
            start_pos.unwrap_or_else(|| grad.0[0].0.get_val(s.width.max(s.height)) as f64);
        let end_pos = end_pos
            .unwrap_or_else(|| grad.0[grad.0.len() - 1].0.get_val(s.width.max(s.height)) as f64);
        let percent = (dist - start_pos) / (end_pos - start_pos);
        start_clr.blend(cs, dsl, &draw_size, x, y, end_clr, percent)
    }

    #[allow(clippy::type_complexity)]
    pub fn apply_fn_to_colors(
        &self, store: &ColorStore, f: Box<dyn Fn(&ColorStore, &Color) -> Color>,
    ) -> Self {
        let mod_gr = {
            let grs = store.pos_gradients.read();
            let gr = grs.get(self.gradient_id);
            let Some(gr) = gr else {
                return self.clone();
            };
            let mut mod_gr = gr.0.clone();
            for (_, c) in mod_gr.iter_mut() {
                *c = f(store, c);
            }
            mod_gr
        };
        let mut p = self.clone();
        let pattern_id = store.add_pos_gradient(mod_gr);
        p.gradient_id = pattern_id;
        p
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Default)]
pub struct TimeGradient {
    /// The total time duration of the gradient
    pub total_dur: Duration,
    //pub grad: Vec<(Duration, Color)>,
    pub gradient_id: usize,
}

impl TimeGradient {
    pub fn new(ctx: &Context, total_dur: Duration, grad: Vec<(Duration, Color)>) -> Self {
        let id = ctx.color_store.add_time_gradient(grad.clone());
        TimeGradient {
            total_dur,
            gradient_id: id,
        }
    }

    /// creates a basic time gradient with multiple colors where each color lasts for the same
    /// duration
    pub fn new_loop(ctx: &Context, each_dur: Duration, colors: Vec<Color>) -> Self {
        if colors.is_empty() {
            return TimeGradient::default();
        }
        if colors.len() == 1 {
            return Self::new(ctx, each_dur, vec![(each_dur, colors[0].clone())]);
        }
        let total_dur = each_dur * colors.len() as u32;
        let mut grad = vec![];
        for (i, c) in colors.iter().enumerate() {
            grad.push((each_dur * i as u32, c.clone()));
        }
        grad.push((total_dur, colors[0].clone()));
        Self::new(ctx, total_dur, grad)
    }

    pub fn len(&self, cs: &ColorStore) -> usize {
        let tgs = cs.time_gradients.read();
        let grad = tgs.get(self.gradient_id);
        let Some(grad) = grad else {
            return 0;
        };
        grad.len()
    }

    pub fn get_grad(&self, cs: &ColorStore) -> Vec<(Duration, Color)> {
        let tgs = cs.time_gradients.read();
        let grad = tgs.get(self.gradient_id);
        let Some(grad) = grad else {
            return vec![];
        };
        grad.clone()
    }

    pub fn to_color(
        &self, cs: &ColorStore, dsl: &Duration, draw_size: &Size, x: u16, y: u16,
    ) -> Color {
        let tgs = cs.time_gradients.read();
        let grad = tgs.get(self.gradient_id);
        let Some(grad) = grad else {
            return Color::TRANSPARENT;
        };
        if grad.is_empty() {
            return Color::TRANSPARENT;
        }

        let mut d = *dsl;
        // calculate d so that it is within the range
        while d >= self.total_dur {
            d -= self.total_dur;
        }
        let mut start_clr: Option<Color> = None;
        let mut end_clr: Option<Color> = None;
        let mut start_time: Option<Duration> = None;
        let mut end_time: Option<Duration> = None;
        for ((t1, c1), (t2, c2)) in grad.windows(2).map(|w| (w[0].clone(), w[1].clone())) {
            if (t1 <= d) && (d < t2) {
                start_clr = Some(c1.clone());
                end_clr = Some(c2.clone());
                start_time = Some(t1);
                end_time = Some(t2);
                break;
            }
        }
        let start_clr = start_clr.unwrap_or_else(|| grad[0].1.clone());
        let end_clr = end_clr.unwrap_or_else(|| grad[grad.len() - 1].1.clone());
        let start_time = start_time.unwrap_or_else(|| grad[0].0);
        let end_time = end_time.unwrap_or_else(|| grad[grad.len() - 1].0);
        let percent = (d - start_time).as_secs_f64() / (end_time - start_time).as_secs_f64();
        start_clr.blend(cs, dsl, draw_size, x, y, end_clr, percent)
    }

    #[allow(clippy::type_complexity)]
    pub fn apply_fn_to_colors(
        &self, store: &ColorStore, f: Box<dyn Fn(&ColorStore, &Color) -> Color>,
    ) -> Self {
        let mod_gr = {
            let time_grs = store.time_gradients.read();
            let time_gr = time_grs.get(self.gradient_id);
            let Some(time_gr) = time_gr else {
                return self.clone();
            };
            let mut mod_gr = time_gr.clone();
            for (_, c) in mod_gr.iter_mut() {
                *c = f(store, c);
            }
            mod_gr
        };
        let mut p = self.clone();
        let time_gr_id = store.add_time_gradient(mod_gr);
        p.gradient_id = time_gr_id;
        p
    }
}

/// WARNING larger patterns render considerably more slowly
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Pattern {
    pub pattern_id: usize,
    pub offset: (i32, i32),
}

impl Pattern {
    pub fn new(ctx: &Context, pattern: Vec<Vec<Color>>) -> Self {
        let pattern_id = ctx.color_store.add_pattern(pattern.clone());
        Pattern {
            pattern_id,
            offset: (0, 0),
        }
    }

    // tile pattern with the given tile width and height
    pub fn new_tiles(
        ctx: &Context, tile_width: usize, tile_height: usize, tile1: Color, tile2: Color,
    ) -> Self {
        let mut pattern = Vec::with_capacity(tile_height * 2);
        for y in 0..tile_height * 2 {
            let mut row = Vec::with_capacity(tile_width * 2);
            for x in 0..tile_width * 2 {
                let left = x < tile_width;
                let top = y < tile_height;
                let c = match (left, top) {
                    (true, true) | (false, false) => tile1.clone(),
                    (true, false) | (false, true) => tile2.clone(),
                };
                row.push(c);
            }
            pattern.push(row);
        }
        let pattern_id = ctx.color_store.add_pattern(pattern.clone());

        Pattern {
            pattern_id,
            offset: (0, 0),
        }
    }

    // attempts to make a square tile pattern provided the width
    pub fn new_sqr_tiles(ctx: &Context, tile_width: usize, tile1: Color, tile2: Color) -> Self {
        // TODO use actual aspect ratio of the terminal if provided
        let tile_height = (tile_width as f64 * 0.55).round() as usize;
        Pattern::new_tiles(ctx, tile_width, tile_height, tile1, tile2)
    }

    pub fn add_to_offset(&mut self, x: i32, y: i32) {
        self.offset.0 += x;
        self.offset.1 += y;
    }

    pub fn get_pattern(&self, cs: &ColorStore) -> Vec<Vec<Color>> {
        let patterns = cs.patterns.read();
        let pattern = patterns.get(self.pattern_id);
        let Some(pattern) = pattern else {
            return vec![];
        };
        pattern.0.clone()
    }

    // is the evaluation of this gradient effected by time
    pub fn is_time_effected(&self, cs: &ColorStore) -> bool {
        cs.is_pattern_time_effected(self.pattern_id)
    }

    // get the color at the given x, y on the pattern, looping once the end is reached
    pub fn to_color(&self, cs: &ColorStore, _dsl: &Duration, x: u16, y: u16) -> Color {
        let patterns = cs.patterns.read();
        let pattern = patterns.get(self.pattern_id);
        let Some(pattern) = pattern else {
            debug!(
                "pattern not found: {}, patterns len {}",
                self.pattern_id,
                patterns.len()
            );
            return Color::TRANSPARENT;
        };
        if pattern.0.is_empty() {
            debug!("pattern is empty: {}", self.pattern_id);
            return Color::TRANSPARENT;
        }
        let x = (x as i32 - self.offset.0) as usize;
        let y = (y as i32 - self.offset.1) as usize;
        let x = x % pattern.0[0].len();
        let y = y % pattern.0.len();
        pattern.0[y][x].clone()
    }

    #[allow(clippy::type_complexity)]
    pub fn apply_fn_to_colors(
        &self, store: &ColorStore, f: Box<dyn Fn(&ColorStore, &Color) -> Color>,
    ) -> Self {
        let mod_pattern = {
            let patterns = store.patterns.read();
            let pattern = patterns.get(self.pattern_id);
            let Some(pattern) = pattern else {
                return self.clone();
            };
            let mut mod_pattern = pattern.0.clone();
            for cs in mod_pattern.iter_mut() {
                for c in cs.iter_mut() {
                    *c = f(store, c);
                }
            }
            mod_pattern
        };
        let mut p = self.clone();
        let pattern_id = store.add_pattern(mod_pattern);
        p.pattern_id = pattern_id;
        p
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

pub fn blend(c1: Rgba, c2: Rgba, perc_c2: f64) -> Rgba {
    let c1_a_perc = c1.a as f64 / 255.0;
    let c2_a_perc = c2.a as f64 / 255.0;
    let a_blend_perc = c1_a_perc + c2_a_perc - (c1_a_perc * c2_a_perc);

    let perc_c1 = 1. - perc_c2;

    let r = ((c1.r as f64 * perc_c1 * c1_a_perc + c2.r as f64 * perc_c2 * c2_a_perc) / a_blend_perc)
        as u8;
    let g = ((c1.g as f64 * perc_c1 * c1_a_perc + c2.g as f64 * perc_c2 * c2_a_perc) / a_blend_perc)
        as u8;
    let b = ((c1.b as f64 * perc_c1 * c1_a_perc + c2.b as f64 * perc_c2 * c2_a_perc) / a_blend_perc)
        as u8;
    let a = ((c1.a as f64 * perc_c1 * c1_a_perc + c2.a as f64 * perc_c2 * c2_a_perc) / a_blend_perc)
        as u8;
    Rgba::new_with_alpha(r, g, b, a)
}

/*
/// This is a different blend function that takes into account the alpha of the colors
/// and mixes in the opposite color for each alpha.
pub fn blend2(c1: Rgba, c2: Rgba, perc_c2: f64) -> Rgba {
    let c1_a_perc = c1.a as f64 / 255.0;
    let c2_a_perc = c2.a as f64 / 255.0;
    let a_blend_perc = c1_a_perc + c2_a_perc - (c1_a_perc * c2_a_perc);
    //let a_blend_perc = (c1_a_perc + c2_a_perc) / 2.;

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
*/

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
        if a == 0 {
            return match prev {
                Some(prev) => prev,
                _ => CrosstermColor::Reset,
            };
        }
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
    pub const LIGHT_YELLOW3:    Color = Color::new(255, 255, 100);

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

pub fn crossterm_to_rgb(ct: CrosstermColor) -> Color {
    match ct {
        CrosstermColor::Reset => Color::ANSI(ct),
        CrosstermColor::Black => Color::BLACK,
        CrosstermColor::DarkGrey => Color::DARK_GREY,
        CrosstermColor::Red => Color::RED,
        CrosstermColor::DarkRed => Color::DARK_RED,
        CrosstermColor::Green => Color::GREEN,
        CrosstermColor::DarkGreen => Color::DARK_GREEN,
        CrosstermColor::Yellow => Color::YELLOW,
        CrosstermColor::DarkYellow => Color::OLIVE,
        CrosstermColor::Blue => Color::BLUE,
        CrosstermColor::DarkBlue => Color::DARK_BLUE,
        CrosstermColor::Magenta => Color::MAGENTA,
        CrosstermColor::DarkMagenta => Color::DARK_MAGENTA,
        CrosstermColor::Cyan => Color::CYAN,
        CrosstermColor::DarkCyan => Color::DARK_CYAN,
        CrosstermColor::White => Color::WHITE,
        CrosstermColor::Grey => Color::GREY,
        CrosstermColor::Rgb { r, g, b } => Color::new(r, g, b),
        CrosstermColor::AnsiValue(ansi) => ansi_to_rgb_color(ansi),
    }
}

// converts ansi color to rgb color
pub fn ansi_to_rgb_color(ansi: u8) -> Color {
    match ansi {
        0 => Color::new(0, 0, 0),
        1 => Color::new(128, 0, 0),
        2 => Color::new(0, 128, 0),
        3 => Color::new(128, 128, 0),
        4 => Color::new(0, 0, 128),
        5 => Color::new(128, 0, 128),
        6 => Color::new(0, 128, 128),
        7 => Color::new(192, 192, 192),
        8 => Color::new(128, 128, 128),
        9 => Color::new(255, 0, 0),
        10 => Color::new(0, 255, 0),
        11 => Color::new(255, 255, 0),
        12 => Color::new(0, 0, 255),
        13 => Color::new(255, 0, 255),
        14 => Color::new(0, 255, 255),
        15 => Color::new(255, 255, 255),
        16 => Color::new(0, 0, 0),
        17 => Color::new(0, 0, 95),
        18 => Color::new(0, 0, 135),
        19 => Color::new(0, 0, 175),
        20 => Color::new(0, 0, 215),
        21 => Color::new(0, 0, 255),
        22 => Color::new(0, 95, 0),
        23 => Color::new(0, 95, 95),
        24 => Color::new(0, 95, 135),
        25 => Color::new(0, 95, 175),
        26 => Color::new(0, 95, 215),
        27 => Color::new(0, 95, 255),
        28 => Color::new(0, 135, 0),
        29 => Color::new(0, 135, 95),
        30 => Color::new(0, 135, 135),
        31 => Color::new(0, 135, 175),
        32 => Color::new(0, 135, 215),
        33 => Color::new(0, 135, 255),
        34 => Color::new(0, 175, 0),
        35 => Color::new(0, 175, 95),
        36 => Color::new(0, 175, 135),
        37 => Color::new(0, 175, 175),
        38 => Color::new(0, 175, 215),
        39 => Color::new(0, 175, 255),
        40 => Color::new(0, 215, 0),
        41 => Color::new(0, 215, 95),
        42 => Color::new(0, 215, 135),
        43 => Color::new(0, 215, 175),
        44 => Color::new(0, 215, 215),
        45 => Color::new(0, 215, 255),
        46 => Color::new(0, 255, 0),
        47 => Color::new(0, 255, 95),
        48 => Color::new(0, 255, 135),
        49 => Color::new(0, 255, 175),
        50 => Color::new(0, 255, 215),
        51 => Color::new(0, 255, 255),
        52 => Color::new(95, 0, 0),
        53 => Color::new(95, 0, 95),
        54 => Color::new(95, 0, 135),
        55 => Color::new(95, 0, 175),
        56 => Color::new(95, 0, 215),
        57 => Color::new(95, 0, 255),
        58 => Color::new(95, 95, 0),
        59 => Color::new(95, 95, 95),
        60 => Color::new(95, 95, 135),
        61 => Color::new(95, 95, 175),
        62 => Color::new(95, 95, 215),
        63 => Color::new(95, 95, 255),
        64 => Color::new(95, 135, 0),
        65 => Color::new(95, 135, 95),
        66 => Color::new(95, 135, 135),
        67 => Color::new(95, 135, 175),
        68 => Color::new(95, 135, 215),
        69 => Color::new(95, 135, 255),
        70 => Color::new(95, 175, 0),
        71 => Color::new(95, 175, 95),
        72 => Color::new(95, 175, 135),
        73 => Color::new(95, 175, 175),
        74 => Color::new(95, 175, 215),
        75 => Color::new(95, 175, 255),
        76 => Color::new(95, 215, 0),
        77 => Color::new(95, 215, 95),
        78 => Color::new(95, 215, 135),
        79 => Color::new(95, 215, 175),
        80 => Color::new(95, 215, 215),
        81 => Color::new(95, 215, 255),
        82 => Color::new(95, 255, 0),
        83 => Color::new(95, 255, 95),
        84 => Color::new(95, 255, 135),
        85 => Color::new(95, 255, 175),
        86 => Color::new(95, 255, 215),
        87 => Color::new(95, 255, 255),
        88 => Color::new(135, 0, 0),
        89 => Color::new(135, 0, 95),
        90 => Color::new(135, 0, 135),
        91 => Color::new(135, 0, 175),
        92 => Color::new(135, 0, 215),
        93 => Color::new(135, 0, 255),
        94 => Color::new(135, 95, 0),
        95 => Color::new(135, 95, 95),
        96 => Color::new(135, 95, 135),
        97 => Color::new(135, 95, 175),
        98 => Color::new(135, 95, 215),
        99 => Color::new(135, 95, 255),
        100 => Color::new(135, 135, 0),
        101 => Color::new(135, 135, 95),
        102 => Color::new(135, 135, 135),
        103 => Color::new(135, 135, 175),
        104 => Color::new(135, 135, 215),
        105 => Color::new(135, 135, 255),
        106 => Color::new(135, 175, 0),
        107 => Color::new(135, 175, 95),
        108 => Color::new(135, 175, 135),
        109 => Color::new(135, 175, 175),
        110 => Color::new(135, 175, 215),
        111 => Color::new(135, 175, 255),
        112 => Color::new(135, 215, 0),
        113 => Color::new(135, 215, 95),
        114 => Color::new(135, 215, 135),
        115 => Color::new(135, 215, 175),
        116 => Color::new(135, 215, 215),
        117 => Color::new(135, 215, 255),
        118 => Color::new(135, 255, 0),
        119 => Color::new(135, 255, 95),
        120 => Color::new(135, 255, 135),
        121 => Color::new(135, 255, 175),
        122 => Color::new(135, 255, 215),
        123 => Color::new(135, 255, 255),
        124 => Color::new(175, 0, 0),
        125 => Color::new(175, 0, 95),
        126 => Color::new(175, 0, 135),
        127 => Color::new(175, 0, 175),
        128 => Color::new(175, 0, 215),
        129 => Color::new(175, 0, 255),
        130 => Color::new(175, 95, 0),
        131 => Color::new(175, 95, 95),
        132 => Color::new(175, 95, 135),
        133 => Color::new(175, 95, 175),
        134 => Color::new(175, 95, 215),
        135 => Color::new(175, 95, 255),
        136 => Color::new(175, 135, 0),
        137 => Color::new(175, 135, 95),
        138 => Color::new(175, 135, 135),
        139 => Color::new(175, 135, 175),
        140 => Color::new(175, 135, 215),
        141 => Color::new(175, 135, 255),
        142 => Color::new(175, 175, 0),
        143 => Color::new(175, 175, 95),
        144 => Color::new(175, 175, 135),
        145 => Color::new(175, 175, 175),
        146 => Color::new(175, 175, 215),
        147 => Color::new(175, 175, 255),
        148 => Color::new(175, 215, 0),
        149 => Color::new(175, 215, 95),
        150 => Color::new(175, 215, 135),
        151 => Color::new(175, 215, 175),
        152 => Color::new(175, 215, 215),
        153 => Color::new(175, 215, 255),
        154 => Color::new(175, 255, 0),
        155 => Color::new(175, 255, 95),
        156 => Color::new(175, 255, 135),
        157 => Color::new(175, 255, 175),
        158 => Color::new(175, 255, 215),
        159 => Color::new(175, 255, 255),
        160 => Color::new(215, 0, 0),
        161 => Color::new(215, 0, 95),
        162 => Color::new(215, 0, 135),
        163 => Color::new(215, 0, 175),
        164 => Color::new(215, 0, 215),
        165 => Color::new(215, 0, 255),
        166 => Color::new(215, 95, 0),
        167 => Color::new(215, 95, 95),
        168 => Color::new(215, 95, 135),
        169 => Color::new(215, 95, 175),
        170 => Color::new(215, 95, 215),
        171 => Color::new(215, 95, 255),
        172 => Color::new(215, 135, 0),
        173 => Color::new(215, 135, 95),
        174 => Color::new(215, 135, 135),
        175 => Color::new(215, 135, 175),
        176 => Color::new(215, 135, 215),
        177 => Color::new(215, 135, 255),
        178 => Color::new(215, 175, 0),
        179 => Color::new(215, 175, 95),
        180 => Color::new(215, 175, 135),
        181 => Color::new(215, 175, 175),
        182 => Color::new(215, 175, 215),
        183 => Color::new(215, 175, 255),
        184 => Color::new(215, 215, 0),
        185 => Color::new(215, 215, 95),
        186 => Color::new(215, 215, 135),
        187 => Color::new(215, 215, 175),
        188 => Color::new(215, 215, 215),
        189 => Color::new(215, 215, 255),
        190 => Color::new(215, 255, 0),
        191 => Color::new(215, 255, 95),
        192 => Color::new(215, 255, 135),
        193 => Color::new(215, 255, 175),
        194 => Color::new(215, 255, 215),
        195 => Color::new(215, 255, 255),
        196 => Color::new(255, 0, 0),
        197 => Color::new(255, 0, 95),
        198 => Color::new(255, 0, 135),
        199 => Color::new(255, 0, 175),
        200 => Color::new(255, 0, 215),
        201 => Color::new(255, 0, 255),
        202 => Color::new(255, 95, 0),
        203 => Color::new(255, 95, 95),
        204 => Color::new(255, 95, 135),
        205 => Color::new(255, 95, 175),
        206 => Color::new(255, 95, 215),
        207 => Color::new(255, 95, 255),
        208 => Color::new(255, 135, 0),
        209 => Color::new(255, 135, 95),
        210 => Color::new(255, 135, 135),
        211 => Color::new(255, 135, 175),
        212 => Color::new(255, 135, 215),
        213 => Color::new(255, 135, 255),
        214 => Color::new(255, 175, 0),
        215 => Color::new(255, 175, 95),
        216 => Color::new(255, 175, 135),
        217 => Color::new(255, 175, 175),
        218 => Color::new(255, 175, 215),
        219 => Color::new(255, 175, 255),
        220 => Color::new(255, 215, 0),
        221 => Color::new(255, 215, 95),
        222 => Color::new(255, 215, 135),
        223 => Color::new(255, 215, 175),
        224 => Color::new(255, 215, 215),
        225 => Color::new(255, 215, 255),
        226 => Color::new(255, 255, 0),
        227 => Color::new(255, 255, 95),
        228 => Color::new(255, 255, 135),
        229 => Color::new(255, 255, 175),
        230 => Color::new(255, 255, 215),
        231 => Color::new(255, 255, 255),
        232 => Color::new(8, 8, 8),
        233 => Color::new(18, 18, 18),
        234 => Color::new(28, 28, 28),
        235 => Color::new(38, 38, 38),
        236 => Color::new(48, 48, 48),
        237 => Color::new(58, 58, 58),
        238 => Color::new(68, 68, 68),
        239 => Color::new(78, 78, 78),
        240 => Color::new(88, 88, 88),
        241 => Color::new(98, 98, 98),
        242 => Color::new(108, 108, 108),
        243 => Color::new(118, 118, 118),
        244 => Color::new(128, 128, 128),
        245 => Color::new(138, 138, 138),
        246 => Color::new(148, 148, 148),
        247 => Color::new(158, 158, 158),
        248 => Color::new(168, 168, 168),
        249 => Color::new(178, 178, 178),
        250 => Color::new(188, 188, 188),
        251 => Color::new(198, 198, 198),
        252 => Color::new(208, 208, 208),
        253 => Color::new(218, 218, 218),
        254 => Color::new(228, 228, 228),
        255 => Color::new(238, 238, 238),
    }
}
