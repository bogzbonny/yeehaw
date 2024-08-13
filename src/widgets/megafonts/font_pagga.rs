use {
    super::FontInputGroup,
    crate::{DrawChs2D, Style},
    std::collections::HashMap,
};

// Credit: original author of Stick Letters Sam Hocevar <sam@hocevar.net>
// This version has been modified by bogz.

// TODO options:
// - shadow colour
// - text colour
// - background colour
// - bg character

#[rustfmt::skip]
pub fn pagga() -> HashMap<char, DrawChs2D> {
    let mut font_input_group = vec![
    FontInputGroup{
        glyphs: vec![
        r#"░░░█░▀░▀░▄█▄█▄░▄█▀░▀░█░▄▀░░▀░▄▀░░▀▄░░▄░▄░░░░░░░░░░░░░░░░░░█░"#.chars().collect(),
        r#"░░░▀░░░░░▄█▄█▄░▀██░▄▀░░▄█▀░░░█░░░░█░░▄█▄░▄█▄░░░░░▄▄▄░░░░▄▀░░"#.chars().collect(),
        r#"░░░▀░░░░░░▀░▀░░▀▀░░▀░▀░░▀▀░░░░▀░░▀░░░▄▀▄░░▀░░▄▀░░░░░░▀░░▀░░░"#.chars().collect(),
        ],
        widths: vec![3,2,4,6,4,4,4,2,4,4,4,4,4,4,3,4],
        chars: r##" !"#$%&'()*+,-./"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#"▄▀▄░▀█░░▀▀▄░▀▀█░█░█░█▀▀░▄▀▀░▀▀█░▄▀▄░▄▀▄░░░░░░░▄▀░░░░░▀▄░░▀▀█░▄▀▄░"#.chars().collect(),
        r#"█/█░░█░░▄▀░░░▀▄░░▀█░▀▀▄░█▀▄░▄▀░░▄▀▄░░▀█░▀░░▀░▀▄░░▀▀▀░░▄▀░░▀░░█a▀░"#.chars().collect(),
        r#"░▀░░▀▀▀░▀▀▀░▀▀░░░░▀░▀▀░░░▀░░▀░░░░▀░░▀▀░░▀░▄▀░░░▀░▀▀▀░▀░░░░▀░░░▀░░"#.chars().collect(),
        ],
        widths: vec![4,4,4,4,4,4,4,4,4,4,2,3,4,4,4,4,4],
        chars: r##"0123456789:;<=>?@"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#"█▀█░█▀▄░█▀▀░█▀▄░█▀▀░█▀▀░█▀▀░█░█░▀█▀░▀▀█░█░█░█░░░█▄█░█▀█░█▀█░"#.chars().collect(),
        r#"█▀█░█▀▄░█░░░█░█░█▀▀░█▀▀░█░█░█▀█░░█░░░░█░█▀▄░█░░░█░█░█░█░█░█░"#.chars().collect(),
        r#"▀░▀░▀▀░░▀▀▀░▀▀░░▀▀▀░▀░░░▀▀▀░▀░▀░▀▀▀░▀▀░░▀░▀░▀▀▀░▀░▀░▀░▀░▀▀▀░"#.chars().collect(),
        ],
        widths: vec![4,4,4,4,4,4,4,4,4,4,4,4,4,4,4],
        chars: r##"ABCDEFGHIJKLMNO"##.chars().collect(),
    },
    FontInputGroup{
        glyphs: vec![
        r#"█▀█░▄▀▄░█▀▄░█▀▀░▀█▀░█░█░█░█░█░█░█░█░█░█░▀▀█░"#.chars().collect(),
        r#"█▀▀░█\█░█▀▄░▀▀█░░█░░█░█░▀▄▀░█▄█░▄▀▄░░█░░▄▀░░"#.chars().collect(),
        r#"▀░░░░▀\░▀░▀░▀▀▀░░▀░░▀▀▀░░▀░░▀░▀░▀░▀░░▀░░▀▀▀░"#.chars().collect(),
        ],
        widths: vec![4,4,4,4,4,4,4,4,4,4,4],
        chars: r##"PQRSTUVWXYZ"##.chars().collect(),
    }, 
    FontInputGroup{
        glyphs: vec![
        r#"█▀░░█░░░▀█░░▄▀▄░░░░░▀▄░░█▀░░█░░▀█░░░░░░░░"#.chars().collect(),
        r#"█░░░░▀▄░░█░░░░░░░░░░░░░▀▄░░░█░░░▄▀░░▄▀▄▀░"#.chars().collect(),
        r#"▀▀░░░░▀░▀▀░░░░░░▀▀▀░░░░░▀▀░░▀░░▀▀░░░░░░░░"#.chars().collect(),
        ],
        widths: vec![4,4,4,4,4,3,5,3,5,5],
        chars: r##"[\]^_`{|}~"##.chars().collect(),
    }
    ];
    let mut out = HashMap::new();
    for ig in font_input_group.iter_mut() {
        ig.add_glyphs_to_map(&mut out, Style::default());
    }
    out
}
