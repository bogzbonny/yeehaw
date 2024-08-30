use std::rc::Rc;

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
    pub fraction: f64, // fraction of the size number of characters (1 = 100)

    // NOTE Rc is used such that locations can dynamicly reference each other
    plus: Vec<Rc<SclVal>>,        // the SclVal adds all the provided SclVals
    minus: Vec<Rc<SclVal>>,       // the SclVal subtracts all the provided SclVals
    plus_min_of: Vec<Rc<SclVal>>, // the SclVal adds the minimum value of these provided SclVals
    plus_max_of: Vec<Rc<SclVal>>, // the SclVal adds the maximum value of these provided SclVals
}

impl SclVal {
    pub fn new_fixed(fixed: i32) -> SclVal {
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

    pub fn new_abs_and_rel(abs: i32, rel: f64) -> SclVal {
        SclVal {
            fixed: abs,
            fraction: rel,
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

    pub fn plus_in_place<S: Into<Rc<SclVal>>>(&mut self, sv: S) {
        self.plus.push(sv.into());
    }

    pub fn minus_in_place<S: Into<Rc<SclVal>>>(&mut self, sv: S) {
        self.minus.push(sv.into());
    }

    pub fn plus<S: Into<Rc<SclVal>>>(mut self, sv: S) -> SclVal {
        self.plus.push(sv.into());
        self
    }

    pub fn minus<S: Into<Rc<SclVal>>>(mut self, sv: S) -> SclVal {
        self.minus.push(sv.into());
        self
    }

    pub fn plus_fixed(mut self, fixed: i32) -> SclVal {
        self.plus.push(Rc::new(SclVal::new_fixed(fixed)));
        self
    }

    pub fn minus_fixed(mut self, fixed: i32) -> SclVal {
        self.minus.push(Rc::new(SclVal::new_fixed(fixed)));
        self
    }

    pub fn plus_frac(mut self, fraction: f64) -> SclVal {
        self.plus.push(Rc::new(SclVal::new_frac(fraction)));
        self
    }

    pub fn minus_frac(mut self, fraction: f64) -> SclVal {
        self.minus.push(Rc::new(SclVal::new_frac(fraction)));
        self
    }

    pub fn plus_min_of<S: Into<Rc<SclVal>>>(mut self, sv: S) -> SclVal {
        self.plus_min_of.push(sv.into());
        self
    }

    pub fn plus_max_of<S: Into<Rc<SclVal>>>(mut self, sv: S) -> SclVal {
        self.plus_max_of.push(sv.into());
        self
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

        let sv = SclVal::new_frac(0.5).plus(Rc::new(SclVal::new_fixed(1)));
        assert_eq!(6, sv.get_val(10));
    }
}
