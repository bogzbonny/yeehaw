use rand::Rng;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Debug, Eq)]
pub enum Color {
    ANSI(crossterm::style::Color),
    Rgba(Rgba),
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

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Color {
        Color::Rgba(Rgba::new(r, g, b))
    }

    pub const fn new_with_alpha(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::Rgba(Rgba::new_with_alpha(r, g, b, a))
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

    // considers the alpha of the self and blends with the previous colour
    pub fn to_crossterm_color(
        &self, prev: Option<crossterm::style::Color>,
    ) -> crossterm::style::Color {
        match self {
            Color::ANSI(c) => *c,
            Color::Rgba(c) => c.to_crossterm_color(prev),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8, // alpha
}

impl Default for Rgba {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    pub const fn new_with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // returns a tuple of the rgb values
    pub fn to_tuple(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    // considers the alpha of the self and blends with the previous colour
    pub fn to_crossterm_color(
        &self, prev: Option<crossterm::style::Color>,
    ) -> crossterm::style::Color {
        let (r, g, b) = self.to_tuple();
        let a = self.a as f64 / 255.0;
        let prev = prev.unwrap_or(crossterm::style::Color::Rgb { r: 0, g: 0, b: 0 });
        let (pr, pg, pb) = match prev {
            crossterm::style::Color::Rgb { r, g, b } => (r, g, b),
            _ => (0, 0, 0),
        };
        let r = (r as f64 * a + pr as f64 * (1.0 - a)) as u8;
        let g = (g as f64 * a + pg as f64 * (1.0 - a)) as u8;
        let b = (b as f64 * a + pb as f64 * (1.0 - a)) as u8;
        crossterm::style::Color::Rgb { r, g, b }
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
    pub const GREY11:        Color = Color::new(110, 110, 115);
    pub const GREY13:        Color = Color::new(130, 130, 135);
    pub const GREY14:        Color = Color::new(140, 140, 145);
    pub const GREY15:        Color = Color::new(150, 150, 155);

    pub const DIM_YELLOW:       Color = Color::new(200, 200, 100);
    pub const ORANGE2:          Color = Color::new(200, 100, 50);
    pub const YELLOW2:          Color = Color::new(220, 220, 0);
    pub const PALLID_BLUE:      Color = Color::new(90, 110, 190);
    pub const LIGHT_YELLOW2:    Color = Color::new(255, 255, 200);


    // css named colours
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
