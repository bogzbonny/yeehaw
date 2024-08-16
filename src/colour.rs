use rand::Rng;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct RgbColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColour {
    pub const fn new(r: u8, g: u8, b: u8) -> RgbColour {
        RgbColour { r, g, b }
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

    // returns a tuple of the rgb values
    pub fn to_tuple(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    //pub fn encode(&self) -> Result<Vec<u8>, Error> {
    //    let v = bincode::encode_to_vec(self, BINCODE_CONFIG)?;
    //    Ok(v)
    //}

    //pub fn decode(bytes: &[u8]) -> Result<RgbColour, Error> {
    //    let (v, _) = bincode::decode_from_slice(bytes, BINCODE_CONFIG)?;
    //    Ok(v)
    //}
}

#[rustfmt::skip]
impl RgbColour {
    pub const GREY1:         RgbColour = RgbColour::new(10, 10, 10);
    pub const GREY2:         RgbColour = RgbColour::new(20, 20, 20);
    pub const GREY3:         RgbColour = RgbColour::new(30, 30, 32);
    pub const GREY4:         RgbColour = RgbColour::new(40, 40, 42);
    pub const GREY5:         RgbColour = RgbColour::new(50, 50, 55);
    pub const GREY6:         RgbColour = RgbColour::new(60, 60, 65);
    pub const GREY7:         RgbColour = RgbColour::new(70, 70, 75);
    pub const GREY8:         RgbColour = RgbColour::new(80, 80, 85);
    pub const GREY9:         RgbColour = RgbColour::new(90, 90, 95);
    pub const GREY11:        RgbColour = RgbColour::new(110, 110, 115);
    pub const GREY13:        RgbColour = RgbColour::new(130, 130, 135);

    // TODO remove these colours only use the css ones
    pub const DIM_YELLOW:    RgbColour = RgbColour::new(200, 200, 100);
    pub const ORANGE2:       RgbColour = RgbColour::new(200, 100, 50);
    pub const YELLOW2:       RgbColour = RgbColour::new(220, 220, 0);
    pub const PALLID_BLUE:   RgbColour = RgbColour::new(90, 110, 190);

    pub const LIGHT_YELLOW2:    RgbColour = RgbColour::new(255, 255, 200);


    // css named colours
    pub const MAROON:                  RgbColour = RgbColour::new(128, 0, 0);
    pub const DARK_RED:                RgbColour = RgbColour::new(139, 0, 0);
    pub const BROWN:                   RgbColour = RgbColour::new(165, 42, 42);
    pub const FIREBRICK:               RgbColour = RgbColour::new(178, 34, 34);
    pub const CRIMSON:                 RgbColour = RgbColour::new(220, 20, 60);
    pub const RED:                     RgbColour = RgbColour::new(255, 0, 0);
    pub const TOMATO:                  RgbColour = RgbColour::new(255, 99, 71);
    pub const CORAL:                   RgbColour = RgbColour::new(255, 127, 80);
    pub const INDIAN_RED:              RgbColour = RgbColour::new(205, 92, 92);
    pub const LIGHT_CORAL:             RgbColour = RgbColour::new(240, 128, 128);
    pub const DARK_SALMON:             RgbColour = RgbColour::new(233, 150, 122);
    pub const SALMON:                  RgbColour = RgbColour::new(250, 128, 114);
    pub const LIGHT_SALMON:            RgbColour = RgbColour::new(255, 160, 122);
    pub const ORANGE_RED:              RgbColour = RgbColour::new(255, 69, 0);
    pub const DARK_ORANGE:             RgbColour = RgbColour::new(255, 140, 0);
    pub const ORANGE:                  RgbColour = RgbColour::new(255, 165, 0);
    pub const GOLD:                    RgbColour = RgbColour::new(255, 215, 0);
    pub const DARK_GOLDEN_ROD:         RgbColour = RgbColour::new(184, 134, 11);
    pub const GOLDEN_ROD:              RgbColour = RgbColour::new(218, 165, 32);
    pub const PALE_GOLDEN_ROD:         RgbColour = RgbColour::new(238, 232, 170);
    pub const DARK_KHAKI:              RgbColour = RgbColour::new(189, 183, 107);
    pub const KHAKI:                   RgbColour = RgbColour::new(240, 230, 140);
    pub const OLIVE:                   RgbColour = RgbColour::new(128, 128, 0);
    pub const YELLOW:                  RgbColour = RgbColour::new(255, 255, 0);
    pub const YELLOW_GREEN:            RgbColour = RgbColour::new(154, 205, 50);
    pub const DARK_OLIVE_GREEN:        RgbColour = RgbColour::new(85, 107, 47);
    pub const OLIVE_DRAB:              RgbColour = RgbColour::new(107, 142, 35);
    pub const LAWN_GREEN:              RgbColour = RgbColour::new(124, 252, 0);
    pub const CHARTREUSE:              RgbColour = RgbColour::new(127, 255, 0);
    pub const GREEN_YELLOW:            RgbColour = RgbColour::new(173, 255, 47);
    pub const DARK_GREEN:              RgbColour = RgbColour::new(0, 100, 0);
    pub const GREEN:                   RgbColour = RgbColour::new(0, 128, 0);
    pub const FOREST_GREEN:            RgbColour = RgbColour::new(34, 139, 34);
    pub const LIME:                    RgbColour = RgbColour::new(0, 255, 0);
    pub const LIME_GREEN:              RgbColour = RgbColour::new(50, 205, 50);
    pub const LIGHT_GREEN:             RgbColour = RgbColour::new(144, 238, 144);
    pub const PALE_GREEN:              RgbColour = RgbColour::new(152, 251, 152);
    pub const DARK_SEA_GREEN:          RgbColour = RgbColour::new(143, 188, 143);
    pub const MEDIUM_SPRING_GREEN:     RgbColour = RgbColour::new(0, 250, 154);
    pub const SPRING_GREEN:            RgbColour = RgbColour::new(0, 255, 127);
    pub const SEA_GREEN:               RgbColour = RgbColour::new(46, 139, 87);
    pub const MEDIUM_AQUA_MARINE:      RgbColour = RgbColour::new(102, 205, 170);
    pub const MEDIUM_SEA_GREEN:        RgbColour = RgbColour::new(60, 179, 113);
    pub const LIGHT_SEA_GREEN:         RgbColour = RgbColour::new(32, 178, 170);
    pub const DARK_SLATE_GRAY:         RgbColour = RgbColour::new(47, 79, 79);
    pub const TEAL:                    RgbColour = RgbColour::new(0, 128, 128);
    pub const DARK_CYAN:               RgbColour = RgbColour::new(0, 139, 139);
    pub const AQUA:                    RgbColour = RgbColour::new(0, 255, 255);
    pub const CYAN:                    RgbColour = RgbColour::new(0, 255, 255);
    pub const LIGHT_CYAN:              RgbColour = RgbColour::new(224, 255, 255);
    pub const DARK_TURQUOISE:          RgbColour = RgbColour::new(0, 206, 209);
    pub const TURQUOISE:               RgbColour = RgbColour::new(64, 224, 208);
    pub const MEDIUM_TURQUOISE:        RgbColour = RgbColour::new(72, 209, 204);
    pub const PALE_TURQUOISE:          RgbColour = RgbColour::new(175, 238, 238);
    pub const AQUA_MARINE:             RgbColour = RgbColour::new(127, 255, 212);
    pub const POWDER_BLUE:             RgbColour = RgbColour::new(176, 224, 230);
    pub const CADET_BLUE:              RgbColour = RgbColour::new(95, 158, 160);
    pub const STEEL_BLUE:              RgbColour = RgbColour::new(70, 130, 180);
    pub const CORNFLOWER_BLUE:         RgbColour = RgbColour::new(100, 149, 237);
    pub const DEEP_SKY_BLUE:           RgbColour = RgbColour::new(0, 191, 255);
    pub const DODGER_BLUE:             RgbColour = RgbColour::new(30, 144, 255);
    pub const LIGHT_BLUE:              RgbColour = RgbColour::new(173, 216, 230);
    pub const SKY_BLUE:                RgbColour = RgbColour::new(135, 206, 235);
    pub const LIGHT_SKY_BLUE:          RgbColour = RgbColour::new(135, 206, 250);
    pub const MIDNIGHT_BLUE:           RgbColour = RgbColour::new(25, 25, 112);
    pub const NAVY:                    RgbColour = RgbColour::new(0, 0, 128);
    pub const DARK_BLUE:               RgbColour = RgbColour::new(0, 0, 139);
    pub const MEDIUM_BLUE:             RgbColour = RgbColour::new(0, 0, 205);
    pub const BLUE:                    RgbColour = RgbColour::new(0, 0, 255);
    pub const ROYAL_BLUE:              RgbColour = RgbColour::new(65, 105, 225);
    pub const BLUE_VIOLET:             RgbColour = RgbColour::new(138, 43, 226);
    pub const INDIGO:                  RgbColour = RgbColour::new(75, 0, 130);
    pub const DARK_SLATE_BLUE:         RgbColour = RgbColour::new(72, 61, 139);
    pub const SLATE_BLUE:              RgbColour = RgbColour::new(106, 90, 205);
    pub const MEDIUM_SLATE_BLUE:       RgbColour = RgbColour::new(123, 104, 238);
    pub const MEDIUM_PURPLE:           RgbColour = RgbColour::new(147, 112, 219);
    pub const DARK_MAGENTA:            RgbColour = RgbColour::new(139, 0, 139);
    pub const DARK_VIOLET:             RgbColour = RgbColour::new(148, 0, 211);
    pub const DARK_ORCHID:             RgbColour = RgbColour::new(153, 50, 204);
    pub const MEDIUM_ORCHID:           RgbColour = RgbColour::new(186, 85, 211);
    pub const PURPLE:                  RgbColour = RgbColour::new(128, 0, 128);
    pub const THISTLE:                 RgbColour = RgbColour::new(216, 191, 216);
    pub const PLUM:                    RgbColour = RgbColour::new(221, 160, 221);
    pub const VIOLET:                  RgbColour = RgbColour::new(238, 130, 238);
    pub const MAGENTA:                 RgbColour = RgbColour::new(255, 0, 255);
    pub const FUCHSIA:                 RgbColour = RgbColour::new(255, 0, 255);
    pub const ORCHID:                  RgbColour = RgbColour::new(218, 112, 214);
    pub const MEDIUM_VIOLET_RED:       RgbColour = RgbColour::new(199, 21, 133);
    pub const PALE_VIOLET_RED:         RgbColour = RgbColour::new(219, 112, 147);
    pub const DEEP_PINK:               RgbColour = RgbColour::new(255, 20, 147);
    pub const HOT_PINK:                RgbColour = RgbColour::new(255, 105, 180);
    pub const LIGHT_PINK:              RgbColour = RgbColour::new(255, 182, 193);
    pub const PINK:                    RgbColour = RgbColour::new(255, 192, 203);
    pub const ANTIQUE_WHITE:           RgbColour = RgbColour::new(250, 235, 215);
    pub const BEIGE:                   RgbColour = RgbColour::new(245, 245, 220);
    pub const BISQUE:                  RgbColour = RgbColour::new(255, 228, 196);
    pub const BLANCHED_ALMOND:         RgbColour = RgbColour::new(255, 235, 205);
    pub const WHEAT:                   RgbColour = RgbColour::new(245, 222, 179);
    pub const CORN_SILK:               RgbColour = RgbColour::new(255, 248, 220);
    pub const LEMON_CHIFFON:           RgbColour = RgbColour::new(255, 250, 205);
    pub const LIGHT_GOLDEN_ROD_YELLOW: RgbColour = RgbColour::new(250, 250, 210);
    pub const LIGHT_YELLOW:            RgbColour = RgbColour::new(255, 255, 224);
    pub const SADDLE_BROWN:            RgbColour = RgbColour::new(139, 69, 19);
    pub const SIENNA:                  RgbColour = RgbColour::new(160, 82, 45);
    pub const CHOCOLATE:               RgbColour = RgbColour::new(210, 105, 30);
    pub const PERU:                    RgbColour = RgbColour::new(205, 133, 63);
    pub const SANDY_BROWN:             RgbColour = RgbColour::new(244, 164, 96);
    pub const BURLY_WOOD:              RgbColour = RgbColour::new(222, 184, 135);
    pub const TAN:                     RgbColour = RgbColour::new(210, 180, 140);
    pub const ROSY_BROWN:              RgbColour = RgbColour::new(188, 143, 143);
    pub const MOCCASIN:                RgbColour = RgbColour::new(255, 228, 181);
    pub const NAVAJO_WHITE:            RgbColour = RgbColour::new(255, 222, 173);
    pub const PEACH_PUFF:              RgbColour = RgbColour::new(255, 218, 185);
    pub const MISTY_ROSE:              RgbColour = RgbColour::new(255, 228, 225);
    pub const LAVENDER_BLUSH:          RgbColour = RgbColour::new(255, 240, 245);
    pub const LINEN:                   RgbColour = RgbColour::new(250, 240, 230);
    pub const OLD_LACE:                RgbColour = RgbColour::new(253, 245, 230);
    pub const PAPAYA_WHIP:             RgbColour = RgbColour::new(255, 239, 213);
    pub const SEA_SHELL:               RgbColour = RgbColour::new(255, 245, 238);
    pub const MINT_CREAM:              RgbColour = RgbColour::new(245, 255, 250);
    pub const SLATE_GRAY:              RgbColour = RgbColour::new(112, 128, 144);
    pub const LIGHT_SLATE_GRAY:        RgbColour = RgbColour::new(119, 136, 153);
    pub const LIGHT_STEEL_BLUE:        RgbColour = RgbColour::new(176, 196, 222);
    pub const LAVENDER:                RgbColour = RgbColour::new(230, 230, 250);
    pub const FLORAL_WHITE:            RgbColour = RgbColour::new(255, 250, 240);
    pub const ALICE_BLUE:              RgbColour = RgbColour::new(240, 248, 255);
    pub const GHOST_WHITE:             RgbColour = RgbColour::new(248, 248, 255);
    pub const HONEYDEW:                RgbColour = RgbColour::new(240, 255, 240);
    pub const IVORY:                   RgbColour = RgbColour::new(255, 255, 240);
    pub const AZURE:                   RgbColour = RgbColour::new(240, 255, 255);
    pub const SNOW:                    RgbColour = RgbColour::new(255, 250, 250);
    pub const BLACK:                   RgbColour = RgbColour::new(0, 0, 0);
    pub const DIM_GRAY:                RgbColour = RgbColour::new(105, 105, 105);
    pub const DIM_GREY:                RgbColour = RgbColour::new(105, 105, 105);
    pub const GRAY:                    RgbColour = RgbColour::new(128, 128, 128);
    pub const GREY:                    RgbColour = RgbColour::new(128, 128, 128);
    pub const DARK_GRAY:               RgbColour = RgbColour::new(169, 169, 169);
    pub const DARK_GREY:               RgbColour = RgbColour::new(169, 169, 169);
    pub const SILVER:                  RgbColour = RgbColour::new(192, 192, 192);
    pub const LIGHT_GRAY:              RgbColour = RgbColour::new(211, 211, 211);
    pub const LIGHT_GREY:              RgbColour = RgbColour::new(211, 211, 211);
    pub const GAINSBORO:               RgbColour = RgbColour::new(220, 220, 220);
    pub const WHITE_SMOKE:             RgbColour = RgbColour::new(245, 245, 245);
    pub const WHITE:                   RgbColour = RgbColour::new(255, 255, 255);

    pub fn from_name(name: &str) -> RgbColour {
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
            "space_gray"                => Self::DIM_GRAY,
            "space_grey"                => Self::DIM_GREY,
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

impl From<RgbColour> for crossterm::style::Color {
    fn from(item: RgbColour) -> Self {
        Self::Rgb {
            r: item.r,
            g: item.g,
            b: item.b,
        }
    }
}

//impl From<RgbColour> for plotters::style::RGBColor {
//    fn from(item: RgbColour) -> Self {
//        Self(item.r, item.g, item.b)
//    }
//}

// used for zell custom colour additions
#[derive(Copy, Clone)]
pub struct Colouring {
    pub fg: Option<RgbColour>,
    pub bg: Option<RgbColour>,
}

// The colour scheme defines the background
// and forground colours for the main zell 2D visualization:
// [ norm  ] [ norm  ] [ norm  ] [ norm  ]
// [ norm  ] [ alt   ] [ norm  ] [ alt   ]
// [ norm  ] [ norm  ] [ norm  ] [ norm  ]
// [ norm  ] [ alt   ] [ norm  ] [ alt   ]
#[derive(Copy, Clone, Default, Debug)]
pub struct ColourScheme {
    //          (fg       , bg       ),
    pub normal: (RgbColour, RgbColour),
    pub alt: (RgbColour, RgbColour), // alternating
    pub sel: (RgbColour, RgbColour), // selected
}

impl ColourScheme {
    pub const fn new(
        fg: RgbColour, bg: RgbColour, alt_bg: RgbColour, sel_fg: RgbColour, sel_bg: RgbColour,
    ) -> ColourScheme {
        ColourScheme {
            normal: (fg, bg),
            alt: (fg, alt_bg),
            sel: (sel_fg, sel_bg),
        }
    }
}
