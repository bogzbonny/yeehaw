/*
package widgets

import yh "keybase.io/nwmod/nwmod/yeehaw"

// SclVal represents a X or Y screen position value which scales based on the
// size of the parent widget. The value is a static number of characters
// (static) plus the fraction of the parent widget size (fraction x size).
//
// Additionally the SclVal can add the minimum or maximum of a set of other
// SclVals. This is useful or Labels which depend on the size of a number of
// other elements.
type SclVal struct {
	static   int     // static number of characters
	fraction float64 // fraction of the parent widget size number of characters

	plus      []SclVal // The SclVal Adds all the provided SclVals
	minus     []SclVal // The SclVal Subtracts all the provided SclVals
	plusMinOF []SclVal // The SclVal Adds the minimum value of these provided SclVals
	plusMaxOF []SclVal // The SclVal Adds the maximum value of these provided SclVals
}

func NewStatic(abs int) SclVal {
	return SclVal{static: abs}
}

func NewFrac(rel float64) SclVal {
	return SclVal{fraction: rel}
}

func NewAbsAndRel(abs int, rel float64) SclVal {
	return SclVal{static: abs, fraction: rel}
}

// Get the value from the absolute and relative psvts
func (sv *SclVal) GetVal(maxSize int) int {
	f := float64(maxSize) * sv.fraction
	rnd := int(f + 0.5) // round the float to the nesvest int
	return sv.static + rnd +
		sv.minFromPlusMinOf(maxSize) + sv.maxFromPlusMaxOf(maxSize) +
		sv.sumOfPlusses(maxSize) - sv.sumOfMinuses(maxSize)
}

// -------------------------

func (sv SclVal) Plus(sv2 SclVal) SclVal {
	out := sv
	out.plus = append(out.plus, sv2)
	return out
}

func (sc SclVal) Minus(sv2 SclVal) SclVal {
	out := sc
	out.minus = append(out.minus, sv2)
	return out
}

//func (sv SclVal) PlusStatic(static int) SclVal {
//    out := sv
//    out.static += static
//    return out
//}

//func (sv SclVal) MinusStatic(static int) SclVal {
//    out := sv
//    out.static -= static
//    return out
//}

func (sv SclVal) PlusStatic(static int) SclVal {
	out := sv
	out.plus = append(out.plus, NewStatic(static))
	return out
}

func (sv SclVal) MinusStatic(static int) SclVal {
	out := sv
	out.minus = append(out.minus, NewStatic(static))
	return out
}

func (sv SclVal) PlusFrac(frac float64) SclVal {
	out := sv
	out.plus = append(out.plus, NewFrac(frac))
	return out
}

func (sv SclVal) MinusFrac(frac float64) SclVal {
	out := sv
	out.minus = append(out.minus, NewFrac(frac))
	return out
}

func (sv SclVal) PlusMinOf(svs ...SclVal) SclVal {
	out := sv
	out.plusMinOF = append(out.plusMinOF, svs...)
	return out
}

func (sv SclVal) PlusMaxOf(svs ...SclVal) SclVal {
	out := sv
	out.plusMaxOF = append(out.plusMaxOF, svs...)
	return out
}

// gets the min SclVal of the plusMinOF SclVals
func (sv SclVal) sumOfPlusses(maxSize int) int {
	sum := 0
	for _, v := range sv.plus {
		sum += v.GetVal(maxSize)
	}
	return sum
}

func (sv SclVal) sumOfMinuses(maxSize int) int {
	sum := 0
	for _, v := range sv.minus {
		sum += v.GetVal(maxSize)
	}
	return sum
}

// gets the min SclVal of the plusMinOF SclVals
func (sv SclVal) minFromPlusMinOf(maxSize int) int {
	if len(sv.plusMinOF) == 0 {
		return 0
	}
	min := sv.plusMinOF[0].GetVal(maxSize)
	for _, v := range sv.plusMinOF {
		vv := v.GetVal(maxSize)
		if vv < min {
			min = vv
		}
	}
	return min
}

// gets the max SclVal of the plusMaxOF SclVals
func (sv SclVal) maxFromPlusMaxOf(maxSize int) int {
	if len(sv.plusMaxOF) == 0 {
		return 0
	}
	max := sv.plusMaxOF[0].GetVal(maxSize)
	for _, v := range sv.plusMaxOF {
		vv := v.GetVal(maxSize)
		if vv > max {
			max = vv
		}
	}
	return max
}

// ------------------------------------
type SclLocation struct {
	StartX SclVal
	EndX   SclVal
	StartY SclVal
	EndY   SclVal
}

func NewSclLocation(startX, endX, startY, endY SclVal) SclLocation {
	return SclLocation{startX, endX, startY, endY}
}

func (l SclLocation) Height(pCtx yh.Context) int {
	return l.EndY.GetVal(pCtx.GetHeight()) - l.StartY.GetVal(pCtx.GetHeight()) + 1
}

func (l SclLocation) Width(pCtx yh.Context) int {
	return l.EndX.GetVal(pCtx.GetWidth()) - l.StartX.GetVal(pCtx.GetWidth()) + 1
}
*/
