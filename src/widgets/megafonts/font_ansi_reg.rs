//use crate::widgets::megafonts::FontInputGroup;

// trad ANSI regular megafont modified/extended by bogz

//pub static ANSI_REG_EX_INPUT_GROUPS: Vec<FontInputGroup> = vec![
//    FontInputGroup((
//        vec![
//        r#"       ██ █ █  ██  ██  ▄▄▄█▄▄▄ ██  ██    ██    █  ██ ██                                ██ "#.chars().collect(),
//        r#"       ██     ████████ ██ █       ██     ██      ██   ██ ▄ ██ ▄   ██                  ██  "#.chars().collect(),
//        r#"       ██      ██  ██  ███████   ██   ████████   ██   ██  ████  ██████    █████      ██   "#.chars().collect(),
//        r#"              ████████    █ ██  ██    ██  ██     ██   ██ ▀ ██ ▀   ██                ██    "#.chars().collect(),
//        r#"       ██      ██  ██  ▀▀▀█▀▀▀ ██  ██ ██████      ██ ██                ▄█       ██ ██     "#.chars().collect(),
//        ],
//        r##" !"#$%&'()*+,-./"##.chars().collect(),
//        vec![7,3,4,9,8,7,9,2,4,4,7,7,3,6,3,7],
//    ))
//];

/*

// trad ANSI regular megafont modified/extended by bogz

var ANSIRegExInputGroups = []FontInputGroup{
    {Glyphs: [][]rune{
        []rune(`       ██ █ █  ██  ██  ▄▄▄█▄▄▄ ██  ██    ██    █  ██ ██                                ██ `),
        []rune(`       ██     ████████ ██ █       ██     ██      ██   ██ ▄ ██ ▄   ██                  ██  `),
        []rune(`       ██      ██  ██  ███████   ██   ████████   ██   ██  ████  ██████    █████      ██   `),
        []rune(`              ████████    █ ██  ██    ██  ██     ██   ██ ▀ ██ ▀   ██                ██    `),
        []rune(`       ██      ██  ██  ▀▀▀█▀▀▀ ██  ██ ██████      ██ ██                ▄█       ██ ██     `),
    },
        Chars:  []rune(` !"#$%&'()*+,-./`),
        Widths: []rune(`7349879244773637`),
    },
    {Glyphs: [][]rune{
        []rune(` ██████   ██ ██████  ██████  ██   ██ ███████  ██████ ███████  █████   █████  `),
        []rune(`██  ████ ███      ██      ██ ██   ██ ██      ██           ██ ██   ██ ██   ██ `),
        []rune(`██ ██ ██  ██  █████   █████  ███████ ███████ ███████     ██   █████   ██████ `),
        []rune(`████  ██  ██ ██           ██      ██      ██ ██    ██   ██   ██   ██      ██ `),
        []rune(` ██████   ██ ███████ ██████       ██ ███████  ██████    ██    █████   █████  `),
    },
        Chars:  []rune(`0123456789`),
        Widths: []rune(`9488888888`),
    },
    {Glyphs: [][]rune{
        []rune(`        ██       ██   █████   ██████  `),
        []rune(`██ ██  ██  █████  ██      ██ ██    ██ `),
        []rune(`      ██           ██  ▄███  ██ ██ ██ `),
        []rune(`██ ▄█  ██  █████  ██   ▀▀    ██  █ ██ `),
        []rune(`   ▀    ██       ██    ██     █  ███  `),
    },
        Chars:  []rune(`:;<=>?@`),
        Widths: []rune(`3356579`),
    },
    {Glyphs: [][]rune{
        []rune(` █████  ██████   ██████ ██████  ███████ ███████  ██████  ██   ██ ██      ██ ██   ██ ██      ███    ███ ███    ██ `),
        []rune(`██   ██ ██   ██ ██      ██   ██ ██      ██      ██       ██   ██ ██      ██ ██  ██  ██      ████  ████ ████   ██ `),
        []rune(`███████ ██████  ██      ██   ██ █████   █████   ██   ███ ███████ ██      ██ █████   ██      ██ ████ ██ ██ ██  ██ `),
        []rune(`██   ██ ██   ██ ██      ██   ██ ██      ██      ██    ██ ██   ██ ██ ██   ██ ██  ██  ██      ██  ██  ██ ██  ██ ██ `),
        []rune(`██   ██ ██████   ██████ ██████  ███████ ██       ██████  ██   ██ ██  █████  ██   ██ ███████ ██      ██ ██   ████ `),
    },
        Chars:  []rune(`ABCDEFGHIJKLMN`),
        Widths: []rune(`888888983888BA`),
    },
    {Glyphs: [][]rune{
        []rune(` ██████  ██████   ██████  ██████  ███████ ████████ ██    ██ ██    ██ ██     ██ ██   ██ ██    ██ ███████ `),
        []rune(`██    ██ ██   ██ ██    ██ ██   ██ ██         ██    ██    ██ ██    ██ ██     ██  ██ ██   ██  ██     ███  `),
        []rune(`██    ██ ██████  ██    ██ ██████  ███████    ██    ██    ██ ██    ██ ██  █  ██   ███     ████     ███   `),
        []rune(`██    ██ ██      ██▄██▄██ ██   ██      ██    ██    ██    ██  ██  ██  ██ ███ ██  ██ ██     ██     ███    `),
        []rune(` ██████  ██       ▀▀▀██▀  ██   ██ ███████    ██     ██████    ████    ███ ███  ██   ██    ██    ███████ `),
    },
        Chars:  []rune(`OPQRSTUVWXYZ`),
        Widths: []rune(`98988999A898`),
    },

    // TODO add more glyphs
    {Glyphs: [][]rune{
        []rune(`███ ███  ███          `),
        []rune(`██   ██ ██ ██         `),
        []rune(`██   ██               `),
        []rune(`██   ██               `),
        []rune(`███ ███       ███████ `),
    },
        Chars:  []rune(`[]^_`),
        Widths: []rune(`4468`),
    },
}

// TODO options:
// - shadow colour
// - text colour
// - background colour
// - bg character

func ANSIRegularEx() map[rune]yh.DrawChs2D {
    out := make(map[rune]yh.DrawChs2D)
    sty := tcell.StyleDefault
    for _, ig := range ANSIRegExInputGroups {
        ig.AddGlyphsToMap(&out, sty)
    }
    return out
}
*/
