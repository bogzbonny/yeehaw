/*
package megafonts

import (
	"github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
)

// This font is down and grimey!
// no special characters, just letters and numbers you can type on the keyboard!
// watch out world!
//
//  - by bogz modded out from trad js-stick

var stickInputGroups = []FontInputGroup{
	{
		Glyphs: [][]rune{
			[]rune(`               __                                `),
			[]rune(`   | '' _/_/_ (|  o/ r  ' / \ \|/ _|_   ---    / `),
			[]rune(`   .    / /-  _|) /o C\   \ / /|\  |  ,     . /  `),
		},
		Chars:  []rune(` !"#$%&'()*+,-./`),
		Widths: []rune(`3236433222442423`),
	},
	{
		Glyphs: [][]rune{
			[]rune(` __  _   _  _       _   _  __  _   _               _  `),
			[]rune(`/ /\  |   ) _) |_| |_  |_   / (_) (_) . . / _ \ ? | a `),
			[]rune(`\/_/ _|_ /_ _)   |  _) |_| /  (_)  _) . , \ - / . |__ `),
		},
		Chars:  []rune(`0123456789:;<=>?@`),
		Widths: []rune(`54334443442222224`),
	},
	{
		Glyphs: [][]rune{
			[]rune(`      __   __  __   __  __  __       ___  ___          .  . .  .  _  `),
			[]rune(` /\  |__) /   |  \ |__ |__ / _  |__|  |    |  |__/ |   |\/| |\ | / \ `),
			[]rune(`/--\ |__) \__ |__/ |__ |   \__| |  | _|_ \_/  |  \ |__ |  | | \| \_/ `),
		},
		Chars:  []rune(`ABCDEFGHIJKLMNO`),
		Widths: []rune(`554544554554554`),
	},
	{
		Glyphs: [][]rune{
			[]rune(` __   __   __   __  ___                       __ `),
			[]rune(`|__) /  \ |__) |__   |  |  | \  / |  | \/ \ /  / `),
			[]rune(`|    \__X |  \  __|  |  \__/  \/  |/\| /\  |  /_ `),
		},
		Chars:  []rune(`PQRSTUVWXYZ`),
		Widths: []rune(`55554555343`),
	},
	{
		Glyphs: [][]rune{
			[]rune(` _    _            _   _      `),
			[]rune(`|  \   | /\    ' _|  |  |_ ^v `),
			[]rune(`|_  \ _|    __    |_ | _|     `),
		},
		Chars:  []rune(`[\]^_` + "`" + `{|}~`),
		Widths: []rune(`33333` + "2" + `4243`),
	},
}

// TODO options
// - text colour
// - background colour

func Stick() map[rune]yh.DrawChs2D {
	out := make(map[rune]yh.DrawChs2D)
	sty := tcell.StyleDefault
	for _, ig := range stickInputGroups {
		ig.AddGlyphsToMap(&out, sty)
	}
	return out
}
*/
