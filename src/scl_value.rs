// SclVal represents a X or Y screen position value which scales based on the
// size of the parent widget. The value is a static number of characters
// (static) plus the fraction of the parent widget size (fraction x size).
//
// Additionally the SclVal can add the minimum or maximum of a set of other
// SclVals. This is useful or Labels which depend on the size of a number of
// other elements.

#[derive(Clone, Debug, Default)]
pub struct SclVal {
    pub fixed: i32,    // fixed number of characters
    pub fraction: f64, // fraction of the total number of characters (1.0 = 100%)

    // NOTE Rc is used such that locations can dynamicly reference each other
    plus: Vec<SclVal>,        // the SclVal adds all the provided SclVals
    minus: Vec<SclVal>,       // the SclVal subtracts all the provided SclVals
    plus_min_of: Vec<SclVal>, // the SclVal adds the minimum value of these provided SclVals
    plus_max_of: Vec<SclVal>, // the SclVal adds the maximum value of these provided SclVals
}

impl SclVal {
    pub fn new_fixed(fixed: i32) -> Self {
        SclVal {
            fixed,
            ..SclVal::default()
        }
    }

    pub fn new_frac(fraction: f64) -> Self {
        SclVal {
            fraction,
            ..SclVal::default()
        }
    }

    // Create a new SclVal with a fraction but which bounds at a minimum static value
    pub fn new_frac_with_min(fraction: f64, min: i32) -> Self {
        SclVal {
            plus_max_of: vec![SclVal::new_fixed(min), SclVal::new_frac(fraction)],
            ..SclVal::default()
        }
    }

    // Get the value from the absolute and relative psvts
    pub fn get_val(&self, max_size: u16) -> i32 {
        let f = max_size as f64 * self.fraction;
        let rounded = (f + 0.5) as i32; // round the float to the nearest int

        (self.fixed
            + rounded
            + self.min_from_plus_min_of(max_size)
            + self.max_from_plus_max_of(max_size)
            + self.sum_of_plusses(max_size))
        .saturating_sub(self.sum_of_minuses(max_size))
    }

    pub fn plus_in_place(&mut self, sv: SclVal) {
        self.plus.push(sv);
    }

    pub fn minus_in_place(&mut self, sv: SclVal) {
        self.minus.push(sv);
    }

    // returns a new SclVal which is the sum of the two SclVals
    // without modifying the original SclVal provided
    pub fn plus(&self, sv: SclVal) -> SclVal {
        let mut out = self.clone();
        out.plus.push(sv);
        out
    }

    pub fn minus(&self, sv: SclVal) -> SclVal {
        let mut out = self.clone();
        out.minus.push(sv);
        out
    }

    pub fn plus_fixed(&self, fixed: i32) -> SclVal {
        self.plus(SclVal::new_fixed(fixed))
    }

    pub fn minus_fixed(&self, fixed: i32) -> SclVal {
        self.minus(SclVal::new_fixed(fixed))
    }

    pub fn plus_frac(&self, fraction: f64) -> SclVal {
        self.plus(SclVal::new_frac(fraction))
    }

    pub fn minus_frac(&self, fraction: f64) -> SclVal {
        self.minus(SclVal::new_frac(fraction))
    }

    pub fn plus_min_of(&self, sv: SclVal) -> SclVal {
        let mut out = self.clone();
        out.plus_min_of.push(sv);
        out
    }

    pub fn plus_max_of(&self, sv: SclVal) -> SclVal {
        let mut out = self.clone();
        out.plus_max_of.push(sv);
        out
    }

    // gets the min SclVal of the plusMinOF SclVals
    pub fn sum_of_plusses(&self, max_size: u16) -> i32 {
        self.plus.iter().fold(0, |acc, v| acc + v.get_val(max_size))
    }

    pub fn sum_of_minuses(&self, max_size: u16) -> i32 {
        self.minus
            .iter()
            .fold(0, |acc, v| acc + v.get_val(max_size))
    }

    // gets the min value of the plus_min_of SclVals, if there are no
    // plus_min_of it returns 0
    pub fn min_from_plus_min_of(&self, max_size: u16) -> i32 {
        let mut out = None;
        for k in self.plus_min_of.iter() {
            let v = k.get_val(max_size);
            match out {
                Some(o) => {
                    if v < o {
                        out = Some(v);
                    }
                }
                None => {
                    out = Some(v);
                }
            }
        }
        out.unwrap_or(0)
    }

    // gets the max value of the plus_max_of SclVals, if there are no
    // plus_max_of it returns 0
    pub fn max_from_plus_max_of(&self, max_size: u16) -> i32 {
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
