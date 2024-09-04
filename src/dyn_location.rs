use crate::{Context, DynVal};

// ZIndex is the z-index or position in the z-dimension of the element
// The lower the z-index, further toward the front the element is
// (0 is the front, 1 is behind 0, etc.)
pub type ZIndex = i32;

#[derive(Default, Debug, Clone)]
pub struct DynLocation {
    pub start_x: DynVal, // inclusive
    pub end_x: DynVal,   // exclusive
    pub start_y: DynVal, // inclusive
    pub end_y: DynVal,   // exclusive
}

impl DynLocation {
    pub fn new(start_x: DynVal, end_x: DynVal, start_y: DynVal, end_y: DynVal) -> DynLocation {
        DynLocation {
            start_x,
            end_x,
            start_y,
            end_y,
        }
    }

    pub fn new_fixed(start_x: i32, end_x: i32, start_y: i32, end_y: i32) -> DynLocation {
        DynLocation {
            start_x: DynVal::new_fixed(start_x),
            end_x: DynVal::new_fixed(end_x),
            start_y: DynVal::new_fixed(start_y),
            end_y: DynVal::new_fixed(end_y),
        }
    }

    pub fn height(&self, ctx: &Context) -> usize {
        let out = self.end_y.get_val(ctx.get_height()) - self.start_y.get_val(ctx.get_height());
        if out < 0 {
            0
        } else {
            out as usize
        }
    }

    pub fn width(&self, ctx: &Context) -> usize {
        let out = self.end_x.get_val(ctx.get_width()) - self.start_x.get_val(ctx.get_width());
        if out < 0 {
            0
        } else {
            out as usize
        }
    }

    pub fn get_dyn_height(&self) -> DynVal {
        self.end_y.clone().minus(self.start_y.clone())
    }

    pub fn get_dyn_width(&self) -> DynVal {
        self.end_x.clone().minus(self.start_x.clone())
    }

    pub fn set_dyn_width(&mut self, width: DynVal) {
        self.end_x = self.start_x.clone().plus(width);
    }

    pub fn set_dyn_height(&mut self, height: DynVal) {
        self.end_y = self.start_y.clone().plus(height);
    }

    pub fn set_start_x(&mut self, start_x: DynVal) {
        self.start_x = start_x;
    }

    pub fn set_start_y(&mut self, start_y: DynVal) {
        self.start_y = start_y;
    }

    pub fn set_end_x(&mut self, end_x: DynVal) {
        self.end_x = end_x;
    }

    pub fn set_end_y(&mut self, end_y: DynVal) {
        self.end_y = end_y;
    }

    // X returns the start and end x values of the Location
    pub fn get_start_x(&self, ctx: &Context) -> i32 {
        self.start_x.get_val(ctx.get_width())
    }
    pub fn get_start_y(&self, ctx: &Context) -> i32 {
        self.start_y.get_val(ctx.get_height())
    }
    // X returns the start and end x values of the Location
    pub fn get_end_x(&self, ctx: &Context) -> i32 {
        self.end_x.get_val(ctx.get_width())
    }
    pub fn get_end_y(&self, ctx: &Context) -> i32 {
        self.end_y.get_val(ctx.get_height())
    }
    pub fn x(&self, ctx: &Context) -> (i32, i32) {
        (
            self.start_x.get_val(ctx.get_width()),
            self.end_x.get_val(ctx.get_width()),
        )
    }

    // Y returns the start and end y values of the Location
    pub fn y(&self, ctx: &Context) -> (i32, i32) {
        (
            self.start_y.get_val(ctx.get_height()),
            self.end_y.get_val(ctx.get_height()),
        )
    }

    pub fn contains_point(&self, ctx: &Context, x: i32, y: i32) -> bool {
        let (start_x, end_x) = self.x(ctx);
        let (start_y, end_y) = self.y(ctx);
        if x >= start_x && x < end_x && y >= start_y && y < end_y {
            return true;
        }
        false
    }

    // GetSize returns the size of the Location
    pub fn get_size(&self, ctx: &Context) -> Size {
        Size::new(self.width(ctx) as u16, self.height(ctx) as u16)
    }

    pub fn adjust_mouse_event(
        &self, ctx: &Context, ev: &crossterm::event::MouseEvent,
    ) -> crossterm::event::MouseEvent {
        let (x_adj, y_adj) = (ev.column, ev.row);
        let start_x = self.get_start_x(ctx);
        let start_y = self.get_start_y(ctx);
        let start_x = if start_x < 0 { 0 } else { start_x as u16 };
        let start_y = if start_y < 0 { 0 } else { start_y as u16 };
        let x_adj = x_adj.saturating_sub(start_x);
        let y_adj = y_adj.saturating_sub(start_y);
        let mut ev = *ev; // copy
        ev.column = x_adj;
        ev.row = y_adj;
        ev
    }

    pub fn adjust_location_by(&mut self, x: DynVal, y: DynVal) {
        self.start_x.plus_in_place(x.clone());
        self.end_x.plus_in_place(x);
        self.start_y.plus_in_place(y.clone());
        self.end_y.plus_in_place(y);
    }

    pub fn adjust_location_by_x(&mut self, x: DynVal) {
        self.start_x.plus_in_place(x.clone());
        self.end_x.plus_in_place(x);
    }

    pub fn adjust_location_by_y(&mut self, y: DynVal) {
        self.start_y.plus_in_place(y.clone());
        self.end_y.plus_in_place(y);
    }
}

// DynLocationSet holds the primary location as well as the extra
// locations of an element. In addition it holds a ZIndex which all
// locations are said to exist at.
#[derive(Clone, Debug)]
pub struct DynLocationSet {
    pub l: DynLocation,

    // Extra locations are locations that are not within the primary location
    // but are still considered to be part of the element.
    // This is useful for elements that do not have a rectangular shape (ie, the
    // menu element)
    pub extra: Vec<DynLocation>,

    pub z: ZIndex,
}

impl Default for DynLocationSet {
    fn default() -> DynLocationSet {
        DynLocationSet {
            l: DynLocation::default(),
            extra: Vec::new(),
            z: 255, // far back
        }
    }
}

impl DynLocationSet {
    pub fn new(l: DynLocation, extra: Vec<DynLocation>, z: ZIndex) -> DynLocationSet {
        DynLocationSet { l, extra, z }
    }

    pub fn with_location(mut self, l: DynLocation) -> DynLocationSet {
        self.l = l;
        self
    }

    pub fn with_extra(mut self, extra: Vec<DynLocation>) -> DynLocationSet {
        self.extra = extra;
        self
    }

    pub fn set_extra(&mut self, extra: Vec<DynLocation>) {
        self.extra = extra;
    }

    pub fn with_z(mut self, z: ZIndex) -> DynLocationSet {
        self.z = z;
        self
    }

    pub fn set_z(&mut self, z: ZIndex) {
        self.z = z;
    }

    // convenience function to set the width of the primary location
    pub fn set_dyn_width(&mut self, width: DynVal) {
        self.l.set_dyn_width(width);
    }

    // convenience function to set the height of the primary location
    pub fn set_dyn_height(&mut self, height: DynVal) {
        self.l.set_dyn_height(height);
    }

    // convenience function to set the start x of the primary location
    pub fn set_start_x(&mut self, start_x: DynVal) {
        self.l.set_start_x(start_x);
    }

    // convenience function to set the start y of the primary location
    pub fn set_start_y(&mut self, start_y: DynVal) {
        self.l.set_start_y(start_y);
    }

    // convenience function to set the end x of the primary location
    pub fn set_end_x(&mut self, end_x: DynVal) {
        self.l.set_end_x(end_x);
    }

    // convenience function to set the end y of the primary location
    pub fn set_end_y(&mut self, end_y: DynVal) {
        self.l.set_end_y(end_y);
    }

    // convenience function to get the start x of the primary location
    pub fn get_start_x(&self, ctx: &Context) -> i32 {
        self.l.get_start_x(ctx)
    }

    // convenience function to get the start y of the primary location
    pub fn get_start_y(&self, ctx: &Context) -> i32 {
        self.l.get_start_y(ctx)
    }

    // convenience function to get the end x of the primary location
    pub fn get_end_x(&self, ctx: &Context) -> i32 {
        self.l.get_end_x(ctx)
    }

    // convenience function to get the end y of the primary location
    pub fn get_end_y(&self, ctx: &Context) -> i32 {
        self.l.get_end_y(ctx)
    }

    // convenience function to get the width of the primary location
    pub fn get_width_val(&self, ctx: &Context) -> usize {
        self.l.width(ctx)
    }

    // convenience function to get the height of the primary location
    pub fn get_height_val(&self, ctx: &Context) -> usize {
        self.l.height(ctx)
    }

    pub fn get_dyn_start_x(&self) -> DynVal {
        self.l.start_x.clone()
    }

    pub fn get_dyn_start_y(&self) -> DynVal {
        self.l.start_y.clone()
    }

    pub fn get_dyn_end_x(&self) -> DynVal {
        self.l.end_x.clone()
    }

    pub fn get_dyn_end_y(&self) -> DynVal {
        self.l.end_y.clone()
    }

    // convenience function to get the height of the primary location
    pub fn get_dyn_height(&self) -> DynVal {
        self.l.get_dyn_height()
    }

    // convenience function to get the width of the primary location
    pub fn get_dyn_width(&self) -> DynVal {
        self.l.get_dyn_width()
    }

    // Contains checks if the given location falls in the primary
    // or extra location of an element
    pub fn contains(&self, ctx: &Context, x: i32, y: i32) -> bool {
        self.contains_within_primary(ctx, x, y) || self.contains_within_extra(ctx, x, y)
    }

    pub fn contains_within_primary(&self, ctx: &Context, x: i32, y: i32) -> bool {
        if self.l.contains_point(ctx, x, y) {
            return true;
        }
        false
    }

    pub fn contains_within_extra(&self, ctx: &Context, x: i32, y: i32) -> bool {
        for eloc in self.extra.iter() {
            if eloc.contains_point(ctx, x, y) {
                return true;
            }
        }
        false
    }

    // returns None is the point is not contained by the DynLocationSet
    pub fn get_z_index_for_point(&self, ctx: &Context, x: i32, y: i32) -> Option<ZIndex> {
        if self.l.contains_point(ctx, x, y) {
            return Some(self.z);
        }

        for eloc in self.extra.iter() {
            if eloc.contains_point(ctx, x, y) {
                return Some(self.z);
            }
        }
        None
    }

    pub fn set_extra_locations(&mut self, extra: Vec<DynLocation>) {
        self.extra = extra;
    }

    pub fn push_extra_locations(&mut self, extra: DynLocation) {
        self.extra.push(extra);
    }

    pub fn adjust_locations_by(&mut self, x: DynVal, y: DynVal) {
        for loc in self.extra.iter_mut() {
            loc.adjust_location_by(x.clone(), y.clone());
        }
        self.l.adjust_location_by(x, y);
    }
}

// --------------------------------------------------
// Size holds the width and height of an element
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub fn new(width: u16, height: u16) -> Size {
        Size { width, height }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Default, Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}
