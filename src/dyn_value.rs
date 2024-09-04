// DynVal represents a X or Y screen position value which scales based on the
// size of the parent widget. The value is a fixed number of characters
// (fixed) plus the flex of the parent widget size (flex x size).
//
// Additionally the DynVal can add the minimum or maximum of a set of other
// SclVals. This is useful or Labels which depend on the size of a number of
// other elements.

// TODO multiplier...
//  - then can get rid of minus field
//  - int or float multiplier?? int is nice for fixed, float is nice for flex

#[derive(Clone, Debug, Default)]
pub struct DynVal {
    pub fixed: i32, // fixed number of characters
    pub flex: f64,  // flex of the total number of characters (1.0 = 100%)

    // NOTE Rc is used such that locations can dynamicly reference each other
    plus: Vec<DynVal>,        // the DynVal adds all the provided SclVals
    minus: Vec<DynVal>,       // the DynVal subtracts all the provided SclVals
    plus_min_of: Vec<DynVal>, // the DynVal adds the minimum value of these provided SclVals
    plus_max_of: Vec<DynVal>, // the DynVal adds the maximum value of these provided SclVals
}

impl DynVal {
    pub fn new_fixed(fixed: i32) -> Self {
        DynVal {
            fixed,
            ..DynVal::default()
        }
    }

    pub fn new_flex(flex: f64) -> Self {
        DynVal {
            flex,
            ..DynVal::default()
        }
    }

    // Create a new DynVal with a flex but which bounds at a minimum fixed value
    pub fn new_flex_with_min_fixed(flex: f64, min: i32) -> Self {
        DynVal {
            plus_max_of: vec![DynVal::new_fixed(min), DynVal::new_flex(flex)],
            ..DynVal::default()
        }
    }

    // Create a new DynVal with a flex but which bounds at a maximum fixed value
    pub fn new_flex_with_max_fixed(flex: f64, max: i32) -> Self {
        DynVal {
            plus_min_of: vec![DynVal::new_fixed(max), DynVal::new_flex(flex)],
            ..DynVal::default()
        }
    }

    // Create a new DynVal with a flex but which bounds at minimum and maximum fixed values
    pub fn new_flex_with_min_and_max_fixed(flex: f64, min: i32, max: i32) -> Self {
        DynVal {
            fixed: min,
            plus_min_of: vec![DynVal::new_fixed(max - min), DynVal::new_flex(flex)],
            ..DynVal::default()
        }
    }

    // Get the value from the absolute and relative psvts
    pub fn get_val(&self, max_size: u16) -> i32 {
        let f = max_size as f64 * self.flex;
        let rounded = (f + 0.5) as i32; // round the float to the nearest int

        (self.fixed
            + rounded
            + self.min_from_plus_min_of(max_size)
            + self.max_from_plus_max_of(max_size)
            + self.sum_of_plusses(max_size))
        .saturating_sub(self.sum_of_minuses(max_size))
    }

    pub fn plus_in_place(&mut self, sv: DynVal) {
        self.plus.push(sv);
    }

    pub fn minus_in_place(&mut self, sv: DynVal) {
        self.minus.push(sv);
    }

    // returns a new DynVal which is the sum of the two SclVals
    // without modifying the original DynVal provided
    pub fn plus(&self, sv: DynVal) -> DynVal {
        let mut out = self.clone();
        out.plus.push(sv);
        out
    }

    pub fn minus(&self, sv: DynVal) -> DynVal {
        let mut out = self.clone();
        out.minus.push(sv);
        out
    }

    pub fn plus_fixed(&self, fixed: i32) -> DynVal {
        self.plus(DynVal::new_fixed(fixed))
    }

    pub fn minus_fixed(&self, fixed: i32) -> DynVal {
        self.minus(DynVal::new_fixed(fixed))
    }

    pub fn plus_flex(&self, flex: f64) -> DynVal {
        self.plus(DynVal::new_flex(flex))
    }

    pub fn minus_flex(&self, flex: f64) -> DynVal {
        self.minus(DynVal::new_flex(flex))
    }

    pub fn plus_min_of(&self, sv: DynVal) -> DynVal {
        let mut out = self.clone();
        out.plus_min_of.push(sv);
        out
    }

    pub fn plus_max_of(&self, sv: DynVal) -> DynVal {
        let mut out = self.clone();
        out.plus_max_of.push(sv);
        out
    }

    // gets the min DynVal of the plusMinOF SclVals
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
pub mod dyn_val_tests {
    use super::*;

    #[test]
    fn test_dyn_val() {
        let sv = DynVal::new_fixed(1);
        assert_eq!(1, sv.get_val(10));

        let sv = DynVal::new_flex(0.5);
        assert_eq!(5, sv.get_val(10));

        let sv = DynVal::new_flex(0.5).plus(DynVal::new_fixed(1));
        assert_eq!(6, sv.get_val(10));
    }
}
