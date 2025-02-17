use crate::Element;

// TODO might make sense to use TinyVec here
//

/// DynVal represents a dynamic x or y screen position value which scales based on the
/// size of the parent element. The value is a fixed number of characters
/// (fixed) plus the flexible fraction of the parent element size (flex).
///
/// Additionally the DynVal can add the minimum or maximum of a set of other
/// SclVals. This is useful or Labels which depend on the size of a number of
/// other elements.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct DynVal {
    /// the multiplier is applied to the final value of the dynval
    pub mul: f64,
    /// fixed number of characters
    pub fixed: i32,
    /// flex of the total number of characters (1.0 = 100%)
    pub flex: f64,

    /// the DynVal adds all the provided SclVals
    pub plus: Vec<DynVal>,
    /// the DynVal adds the minimum value of these provided SclVals
    pub plus_min_of: Vec<DynVal>,
    /// the DynVal adds the maximum value of these provided SclVals
    pub plus_max_of: Vec<DynVal>,
}

impl Default for DynVal {
    fn default() -> Self {
        DynVal {
            mul: 1.0,
            fixed: 0,
            flex: 0.0,
            plus: Vec::new(),
            plus_min_of: Vec::new(),
            plus_max_of: Vec::new(),
        }
    }
}

impl From<i32> for DynVal {
    fn from(fixed: i32) -> Self {
        DynVal::new_fixed(fixed)
    }
}

impl From<usize> for DynVal {
    fn from(fixed: usize) -> Self {
        DynVal::new_fixed(fixed as i32)
    }
}

impl From<u16> for DynVal {
    fn from(fixed: u16) -> Self {
        DynVal::new_fixed(fixed as i32)
    }
}

impl From<f64> for DynVal {
    fn from(flex: f64) -> Self {
        DynVal::new_flex(flex)
    }
}

impl DynVal {
    pub const FULL: DynVal = DynVal {
        mul: 1.0,
        fixed: 0,
        flex: 1.0,
        plus: Vec::new(),
        plus_min_of: Vec::new(),
        plus_max_of: Vec::new(),
    };
    pub const HALF: DynVal = DynVal {
        mul: 1.0,
        fixed: 0,
        flex: 0.5,
        plus: Vec::new(),
        plus_min_of: Vec::new(),
        plus_max_of: Vec::new(),
    };
    pub const QUARTER: DynVal = DynVal {
        mul: 1.0,
        fixed: 0,
        flex: 0.25,
        plus: Vec::new(),
        plus_min_of: Vec::new(),
        plus_max_of: Vec::new(),
    };

    pub fn new<T: Into<DynVal>>(val: T) -> Self {
        val.into()
    }

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

    pub fn max_of(d1: DynVal, d2: DynVal) -> Self {
        DynVal {
            plus_max_of: vec![d1, d2],
            ..DynVal::default()
        }
    }

    pub fn min_of(d1: DynVal, d2: DynVal) -> Self {
        DynVal {
            plus_min_of: vec![d1, d2],
            ..DynVal::default()
        }
    }

    /// Create a new DynVal with a flex but which bounds at a minimum fixed value
    pub fn new_flex_with_min_fixed(flex: f64, min: i32) -> Self {
        DynVal {
            plus_max_of: vec![DynVal::new_fixed(min), DynVal::new_flex(flex)],
            ..DynVal::default()
        }
    }

    /// returns the DynVal which will be no larger than the provided max value
    pub fn with_max<D: Into<DynVal>>(self, max: D) -> Self {
        DynVal {
            plus_min_of: vec![self, max.into()],
            ..DynVal::default()
        }
    }

    /// returns the DynVal which will be no smaller than the provided min value
    pub fn with_min<D: Into<DynVal>>(self, min: D) -> Self {
        DynVal {
            plus_max_of: vec![self, min.into()],
            ..DynVal::default()
        }
    }

    /// convienence function to get the x position after the element
    pub fn x_after<D: Into<DynVal>>(el: &dyn Element, padding: D) -> Self {
        el.get_dyn_location_set()
            .get_dyn_end_x()
            .plus(padding.into())
    }

    /// convienence function to get the y position after the element
    pub fn y_after<D: Into<DynVal>>(el: &dyn Element, padding: D) -> Self {
        el.get_dyn_location_set()
            .get_dyn_end_y()
            .plus(padding.into())
    }

    /// convienence function to get the x position after the largest element in the x-direction
    pub fn x_after_all<D: Into<DynVal>>(els: Vec<&dyn Element>, padding: D) -> Self {
        let mut out = DynVal::default();
        for el in els.iter() {
            let el_x = el.get_dyn_location_set().get_dyn_end_x();
            out.plus_max_of.push(el_x);
        }
        out.plus(padding.into())
    }

    /// convienence function to get the y position after the largest element in the y-direction
    pub fn y_after_all<D: Into<DynVal>>(els: Vec<&dyn Element>, padding: D) -> Self {
        let mut out = DynVal::default();
        for el in els.iter() {
            let el_y = el.get_dyn_location_set().get_dyn_end_y();
            out.plus_max_of.push(el_y);
        }
        out.plus(padding.into())
    }

    /// Get the value from the absolute and relative parts  
    pub fn get_val(&self, max_size: u16) -> i32 {
        let flex = max_size as f64 * self.flex;
        let flex = f64::round(flex) as i32;

        let pre_multiplied = self.fixed
            + flex
            + self.sum_of_plusses(max_size)
            + self.min_from_plus_min_of(max_size)
            + self.max_from_plus_max_of(max_size);

        let multiplied = pre_multiplied as f64 * self.mul;
        f64::round(multiplied) as i32
    }

    /// get the bounds of this dynamic value
    pub fn get_bounds(&self) -> (i32, i32) {
        let min = self.get_val(0);
        let max = self.get_val(u16::MAX);
        (min, max)
    }

    /// get the flexible absolute value for the context provided
    /// this is the value of the flex portion of the DynVal
    /// without the fixed portion.
    pub fn get_flex_val_portion_for_ctx(&self, max_size: u16) -> i32 {
        let fixed_amount = self.get_val(0);
        let val = self.get_val(max_size);
        val - fixed_amount
    }

    pub fn is_flat(&self) -> bool {
        self.plus.is_empty() && self.plus_min_of.is_empty() && self.plus_max_of.is_empty()
    }

    pub fn flattened(self) -> DynVal {
        let mut out = self;
        out.flatten_internal();
        out
    }

    /// simplify the calculation of the DynVal
    /// recursive function. Add any plus values to the main value if they are flat
    pub fn flatten_internal(&mut self) {
        for i in 0..self.plus.len() {
            self.plus[i].flatten_internal();
        }
        for i in 0..self.plus_min_of.len() {
            self.plus_min_of[i].flatten_internal();
        }
        for i in 0..self.plus_max_of.len() {
            self.plus_max_of[i].flatten_internal();
        }

        let mut i = 0;
        while i < self.plus.len() {
            if self.plus[i].is_flat() {
                self.fixed += f64::round(self.plus[i].mul * self.plus[i].fixed as f64) as i32;
                self.flex += self.plus[i].mul * self.plus[i].flex;
                self.plus.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn neg(&self) -> DynVal {
        let mut out = self.clone();
        out.mul = -out.mul;
        out
    }

    pub fn mul(&self, mul: f64) -> DynVal {
        let mut out = self.clone();
        out.mul *= mul;
        out
    }

    pub fn div(&self, div: f64) -> DynVal {
        let mut out = self.clone();
        out.mul /= div;
        out
    }

    pub fn plus_in_place(&mut self, sv: DynVal) {
        self.plus.push(sv);
    }

    pub fn minus_in_place(&mut self, sv: DynVal) {
        self.plus.push(sv.neg());
    }

    /// returns a new DynVal which is the sum of the two SclVals
    /// without modifying the original DynVal provided
    pub fn plus(&self, sv: DynVal) -> DynVal {
        let mut out = self.clone();
        out.plus.push(sv);
        out
    }

    pub fn minus(&self, sv: DynVal) -> DynVal {
        let mut out = self.clone();
        out.plus.push(sv.neg());
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

    /// gets the min DynVal of the plusMinOF SclVals
    pub fn sum_of_plusses(&self, max_size: u16) -> i32 {
        self.plus.iter().fold(0, |acc, v| acc + v.get_val(max_size))
    }

    /// gets the min value of the plus_min_of SclVals, if there are no
    /// plus_min_of it returns 0
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

    /// gets the max value of the plus_max_of SclVals, if there are no
    /// plus_max_of it returns 0
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

        let sv = DynVal::FULL.minus(DynVal::new_fixed(1));
        assert_eq!(9, sv.get_val(10));
        assert_eq!(19, sv.get_val(20));
    }
}
