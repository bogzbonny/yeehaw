use crate::{Context, Location};

// SclVal represents a X or Y screen position value which scales based on the
// size of the parent widget. The value is a static number of characters
// (static) plus the fraction of the parent widget size (fraction x size).
//
// Additionally the SclVal can add the minimum or maximum of a set of other
// SclVals. This is useful or Labels which depend on the size of a number of
// other elements.

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct SclVal {
    fixed: usize,  // fixed number of characters
    fraction: f64, // fraction of the parent widget size number of characters

    plus: Vec<SclVal>,        // The SclVal Adds all the provided SclVals
    minus: Vec<SclVal>,       // The SclVal Subtracts all the provided SclVals
    plus_min_of: Vec<SclVal>, // The SclVal Adds the minimum value of these provided SclVals
    plus_max_of: Vec<SclVal>, // The SclVal Adds the maximum value of these provided SclVals
}

impl SclVal {
    pub fn new_fixed(fixed: usize) -> SclVal {
        SclVal {
            fixed,
            ..SclVal::default()
        }
    }

    pub fn new_frac(fraction: f64) -> SclVal {
        SclVal {
            fraction,
            ..SclVal::default()
        }
    }

    pub fn new_abs_and_rel(abs: usize, rel: f64) -> SclVal {
        SclVal {
            fixed: abs,
            fraction: rel,
            ..SclVal::default()
        }
    }

    // Get the value from the absolute and relative psvts
    pub fn get_val(&self, max_size: usize) -> usize {
        let f = max_size as f64 * self.fraction;
        let rounded = (f + 0.5) as usize; // round the float to the nearest int
        self.fixed
            + rounded
            + self.min_from_plus_min_of(max_size)
            + self.max_from_plus_max_of(max_size)
            + self.sum_of_plusses(max_size)
            - self.sum_of_minuses(max_size)
    }

    pub fn plus(mut self, sv: SclVal) -> SclVal {
        self.plus.push(sv);
        self
    }

    pub fn minus(mut self, sv: SclVal) -> SclVal {
        self.minus.push(sv);
        self
    }

    pub fn plus_fixed(mut self, fixed: usize) -> SclVal {
        self.plus.push(SclVal::new_fixed(fixed));
        self
    }

    pub fn minus_fixed(mut self, fixed: usize) -> SclVal {
        self.minus.push(SclVal::new_fixed(fixed));
        self
    }

    pub fn plus_frac(mut self, fraction: f64) -> SclVal {
        self.plus.push(SclVal::new_frac(fraction));
        self
    }

    pub fn minus_frac(mut self, fraction: f64) -> SclVal {
        self.minus.push(SclVal::new_frac(fraction));
        self
    }

    pub fn plus_min_of(mut self, sv: SclVal) -> SclVal {
        self.plus_min_of.push(sv);
        self
    }

    pub fn plus_max_of(mut self, sv: SclVal) -> SclVal {
        self.plus_max_of.push(sv);
        self
    }

    // gets the min SclVal of the plusMinOF SclVals
    pub fn sum_of_plusses(&self, max_size: usize) -> usize {
        self.plus.iter().fold(0, |acc, v| acc + v.get_val(max_size))
    }

    pub fn sum_of_minuses(&self, max_size: usize) -> usize {
        self.minus
            .iter()
            .fold(0, |acc, v| acc + v.get_val(max_size))
    }

    // gets the min value of the plus_min_of SclVals, if there are no
    // plus_min_of it returns 0
    pub fn min_from_plus_min_of(&self, max_size: usize) -> usize {
        let mut out = 0;
        for k in self.plus_min_of.iter() {
            let v = k.get_val(max_size);
            if v < out {
                out = v;
            }
        }
        out
    }

    // gets the max value of the plus_max_of SclVals, if there are no
    // plus_max_of it returns 0
    pub fn max_from_plus_max_of(&self, max_size: usize) -> usize {
        let mut out = 0;
        for k in self.plus_max_of.iter() {
            let v = k.get_val(max_size);
            if v > out {
                out = v;
            }
        }
        out
    }
}

// ------------------------------------

#[derive(Default)]
pub struct SclLocation {
    pub start_x: SclVal,
    pub end_x: SclVal,
    pub start_y: SclVal,
    pub end_y: SclVal,
}

impl SclLocation {
    pub fn new(start_x: SclVal, end_x: SclVal, start_y: SclVal, end_y: SclVal) -> SclLocation {
        SclLocation {
            start_x,
            end_x,
            start_y,
            end_y,
        }
    }

    pub fn height(&self, p_ctx: &Context) -> usize {
        self.end_y.get_val(p_ctx.get_height() as usize)
            - self.start_y.get_val(p_ctx.get_height() as usize)
            + 1
    }

    pub fn width(&self, p_ctx: &Context) -> usize {
        self.end_x.get_val(p_ctx.get_width() as usize)
            - self.start_x.get_val(p_ctx.get_width() as usize)
            + 1
    }

    //pub fn get_location(&self) -> Location {
    //    let w = self.get_width() as i32;
    //    let h = self.get_height() as i32;
    //    let x1 = self.loc_x.get_val(self.p_ctx.get_width().into()) as i32;
    //    let y1 = self.loc_y.get_val(self.p_ctx.get_height().into()) as i32;
    //    let x2 = x1 + w - 1;
    //    let y2 = y1 + h - 1;
    //    Location::new(x1, x2, y1, y2)
    //}
    pub fn get_location_for_context(&self, p_ctx: &Context) -> Location {
        Location::new(
            self.start_x.get_val(p_ctx.get_width() as usize) as i32,
            self.end_x.get_val(p_ctx.get_width() as usize) as i32,
            self.start_y.get_val(p_ctx.get_height() as usize) as i32,
            self.end_y.get_val(p_ctx.get_height() as usize) as i32,
        )
    }
}

#[cfg(test)]
pub mod scl_val_tests {
    use super::*;

    #[test]
    fn test_scl_val() {
        let sv = SclVal::new_fixed(1);
        assert_eq!(1, sv.get_val(10));

        let sv = SclVal::new_frac(0.5);
        assert_eq!(5, sv.get_val(10));

        let sv = SclVal::new_frac(0.5).plus(SclVal::new_fixed(1));
        assert_eq!(6, sv.get_val(10));
    }
}
