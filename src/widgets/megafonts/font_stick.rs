use {
    super::FontInputGroup,
    crate::{DrawChs2D, Style},
    std::collections::HashMap,
};

// This font is down and grimey!
// no special characters, just letters and numbers you can type on the keyboard!
// watch out world!
//
// Credit: original author of Stick Letters Joan Stark.
// This version has been modified by bogz.

// TODO options
// - text colour
// - background colour

#[rustfmt::skip]
pub fn stick() -> HashMap<char, DrawChs2D> {
    let mut font_input_group = vec![
    FontInputGroup{
        glyphs: vec![
        r#"               __                                "#.chars().collect(),
        r#"   | '' _/_/_ (|  o/ r  ' / \ \|/ _|_   ---    / "#.chars().collect(),
        r#"   .    / /-  _|) /o C\   \ / /|\  |  ,     . /  "#.chars().collect(),
        ],
        widths: vec![3,2,3,6,4,3,3,2,2,2,4,4,2,4,2,3],
        chars: r##" !"#$%&'()*+,-./"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#" __  _   _  _       _   _  __  _   _               _  "#.chars().collect(),
        r#"/ /\  |   ) _) |_| |_  |_   / (_) (_) . . / _ \ ? | a "#.chars().collect(),
        r#"\/_/ _|_ /_ _)   |  _) |_| /  (_)  _) . , \ - / . |__ "#.chars().collect(),
        ],
        widths: vec![5,4,3,3,4,4,4,3,4,4,2,2,2,2,2,2,4],
        chars: r##"0123456789:;<=>?@"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#"      __   __  __   __  __  __       ___  ___          .  . .  .  _  "#.chars().collect(),
        r#" /\  |__) /   |  \ |__ |__ / _  |__|  |    |  |__/ |   |\/| |\ | / \ "#.chars().collect(),
        r#"/--\ |__) \__ |__/ |__ |   \__| |  | _|_ \_/  |  \ |__ |  | | \| \_/ "#.chars().collect(),
        ],
        widths: vec![5,5,4,5,4,4,5,5,4,5,5,4,5,5,4],
        chars: r##"ABCDEFGHIJKLMNO"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#" __   __   __   __  ___                       __ "#.chars().collect(),
        r#"|__) /  \ |__) |__   |  |  | \  / |  | \/ \ /  / "#.chars().collect(),
        r#"|    \__X |  \  __|  |  \__/  \/  |/\| /\  |  /_ "#.chars().collect(),
        ],
        widths: vec![5,5,5,5,4,5,5,5,3,4,3],
        chars: r##"PQRSTUVWXYZ"##.chars().collect(),
    }, 
    FontInputGroup{
        glyphs: vec![
        r#" _    _            _   _      "#.chars().collect(),
        r#"|  \   | /\    ' _|  |  |_ ^v "#.chars().collect(),
        r#"|_  \ _|    __    |_ | _|     "#.chars().collect(),
        ],
        widths: vec![3,3,3,3,3,2,4,2,4,3],
        chars: r##"[\]^_`{|}~"##.chars().collect(),
    }
    ];
    let mut out = HashMap::new();
    for ig in font_input_group.iter_mut() {
        ig.add_glyphs_to_map(&mut out, Style::default());
    }
    out
}
