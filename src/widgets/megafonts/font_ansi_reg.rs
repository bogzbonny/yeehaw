use {
    super::{FontInputGroup, Megafont},
    crate::Style,
    std::collections::HashMap,
};

// trad ANSI regular megafont modified/extended by bogz

// TODO add more glyphs
// TODO options:
// - shadow colour
// - text colour
// - background colour
// - bg character

#[rustfmt::skip]
pub fn ansi_regular_ex() -> Megafont {
    let mut font_input_group = vec![
    FontInputGroup{
        glyphs: vec![
        r#"       ██ █ █  ██  ██  ▄▄▄█▄▄▄ ██  ██    ██    █  ██ ██                                ██ "#.chars().collect(),
        r#"       ██     ████████ ██ █       ██     ██      ██   ██ ▄ ██ ▄   ██                  ██  "#.chars().collect(),
        r#"       ██      ██  ██  ███████   ██   ████████   ██   ██  ████  ██████    █████      ██   "#.chars().collect(),
        r#"              ████████    █ ██  ██    ██  ██     ██   ██ ▀ ██ ▀   ██                ██    "#.chars().collect(),
        r#"       ██      ██  ██  ▀▀▀█▀▀▀ ██  ██ ██████      ██ ██                ▄█       ██ ██     "#.chars().collect(),
        ],
        widths: vec![7,3,4,9,8,7,9,2,4,4,7,7,3,6,3,7],
        chars: r##" !"#$%&'()*+,-./"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#" ██████   ██ ██████  ██████  ██   ██ ███████  ██████ ███████  █████   █████  "#.chars().collect(),
        r#"██  ████ ███      ██      ██ ██   ██ ██      ██           ██ ██   ██ ██   ██ "#.chars().collect(),
        r#"██ ██ ██  ██  █████   █████  ███████ ███████ ███████     ██   █████   ██████ "#.chars().collect(),
        r#"████  ██  ██ ██           ██      ██      ██ ██    ██   ██   ██   ██      ██ "#.chars().collect(),
        r#" ██████   ██ ███████ ██████       ██ ███████  ██████    ██    █████   █████  "#.chars().collect(),
        ],
        widths: vec![9, 4, 8, 8, 8, 8, 8, 8, 8, 8],
        chars: r##"0123456789"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#"        ██       ██   █████   ██████  "#.chars().collect(),
        r#"██ ██  ██  █████  ██      ██ ██    ██ "#.chars().collect(),
        r#"      ██           ██  ▄███  ██ ██ ██ "#.chars().collect(),
        r#"██ ▄█  ██  █████  ██   ▀▀    ██  █ ██ "#.chars().collect(),
        r#"   ▀    ██       ██    ██     █  ███  "#.chars().collect(),
        ],
        widths: vec![3, 3, 5, 6, 5, 7, 9],
        chars: r##":;<=>?@"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#" █████  ██████   ██████ ██████  ███████ ███████  ██████  ██   ██ ██      ██ ██   ██ ██      ███    ███ ███    ██ "#.chars().collect(),
        r#"██   ██ ██   ██ ██      ██   ██ ██      ██      ██       ██   ██ ██      ██ ██  ██  ██      ████  ████ ████   ██ "#.chars().collect(),
        r#"███████ ██████  ██      ██   ██ █████   █████   ██   ███ ███████ ██      ██ █████   ██      ██ ████ ██ ██ ██  ██ "#.chars().collect(),
        r#"██   ██ ██   ██ ██      ██   ██ ██      ██      ██    ██ ██   ██ ██ ██   ██ ██  ██  ██      ██  ██  ██ ██  ██ ██ "#.chars().collect(),
        r#"██   ██ ██████   ██████ ██████  ███████ ██       ██████  ██   ██ ██  █████  ██   ██ ███████ ██      ██ ██   ████ "#.chars().collect(),
        ],
        widths: vec![8,8,8,8,8,8,9,8,3,8,8,8,11,10],
        chars: r##"ABCDEFGHIJKLMN"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#" ██████  ██████   ██████  ██████  ███████ ████████ ██    ██ ██    ██ ██     ██ ██   ██ ██    ██ ███████ "#.chars().collect(),
        r#"██    ██ ██   ██ ██    ██ ██   ██ ██         ██    ██    ██ ██    ██ ██     ██  ██ ██   ██  ██     ███  "#.chars().collect(),
        r#"██    ██ ██████  ██    ██ ██████  ███████    ██    ██    ██ ██    ██ ██  █  ██   ███     ████     ███   "#.chars().collect(),
        r#"██    ██ ██      ██▄██▄██ ██   ██      ██    ██    ██    ██  ██  ██  ██ ███ ██  ██ ██     ██     ███    "#.chars().collect(),
        r#" ██████  ██       ▀▀▀██▀  ██   ██ ███████    ██     ██████    ████    ███ ███  ██   ██    ██    ███████ "#.chars().collect(),
        ],
        widths: vec![9,8,9,8,8,9,9,9,10,8,9,8],
        chars: r##"OPQRSTUVWXYZ"##.chars().collect(),
    }, 
    FontInputGroup{
        glyphs: vec![
        r#"███ ███  ███          "#.chars().collect(),
        r#"██   ██ ██ ██         "#.chars().collect(),
        r#"██   ██               "#.chars().collect(),
        r#"██   ██               "#.chars().collect(),
        r#"███ ███       ███████ "#.chars().collect(),
        ],
        widths: vec![4, 4, 6, 8],
        chars: r##"[]^_"##.chars().collect(),
    }
    ];
    let mut chs = HashMap::new();
    for ig in font_input_group.iter_mut() {
        ig.add_glyphs_to_map(&mut chs, Style::default_const());
    }
    Megafont::new(chs)
}
