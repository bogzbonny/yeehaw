use crate::{element::RelocationRequest, Size};

// ZIndex is the z-index or position in the z-dimension of the element
// The lower the z-index, further toward the front the element is
// (0 is the front, 1 is behind 0, etc.)
pub type ZIndex = i32;

// Location holds the primary location of an element in the context of
// its parent element
#[derive(Clone, Default, Debug)]
pub struct Location {
    pub start_x: i32,
    pub end_x: i32,
    pub start_y: i32,
    pub end_y: i32,
}

impl Location {
    pub fn new(start_x: i32, end_x: i32, start_y: i32, end_y: i32) -> Location {
        Location {
            start_x,
            end_x,
            start_y,
            end_y,
        }
    }

    // Height returns the height of the Location
    // NOTE: might require screen input at some point
    pub fn height(&self) -> i32 {
        // NOTE: 1 must be added to get the proper height as the end and start
        // values are inclusive.
        // eg, 25 - 0 = 25. but since cell 25 would belong to the given pane, the
        // actual width would be 26.
        self.end_y - self.start_y + 1
    }

    // Width returns the width of the Location
    // NOTE: might require screen input at some point
    pub fn width(&self) -> i32 {
        self.end_x - self.start_x + 1
    }

    pub fn set_width(&mut self, width: usize) {
        self.end_x = self.start_x + width as i32 - 1;
    }

    // X returns the start and end x values of the Location
    pub fn x(&self) -> (i32, i32) {
        (self.start_x, self.end_x)
    }

    // Y returns the start and end y values of the Location
    pub fn y(&self) -> (i32, i32) {
        (self.start_y, self.end_y)
    }

    // ContainsLocation checks if the given location falls in the primary
    // location of an element
    pub fn contains_location(&self, location: &Location) -> bool {
        if location.start_x >= self.start_x
            && location.end_x <= self.end_x
            && location.start_y >= self.start_y
            && location.end_y <= self.end_y
        {
            return true;
        }
        false
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        if x >= self.start_x && x <= self.end_x && y >= self.start_y && y <= self.end_y {
            return true;
        }
        false
    }

    // Relocate increments the Location by the given values
    pub fn relocate(&mut self, rr: RelocationRequest) {
        self.start_x += rr.left;
        self.end_x += rr.right;
        self.start_y += rr.up;
        self.end_y += rr.down;
    }

    // GetSize returns the size of the Location
    pub fn get_size(&self) -> Size {
        Size::new(self.width() as u16, self.height() as u16)
    }

    pub fn adjust_mouse_event(
        &self, ev: &crossterm::event::MouseEvent,
    ) -> crossterm::event::MouseEvent {
        let (x_adj, y_adj) = (ev.column, ev.row);
        let start_x = if self.start_x < 0 { 0 } else { self.start_x as u16 };
        let start_y = if self.start_y < 0 { 0 } else { self.start_y as u16 };
        let x_adj = x_adj.saturating_sub(start_x);
        let y_adj = y_adj.saturating_sub(start_y);
        let mut ev = *ev; // copy
        ev.column = x_adj;
        ev.row = y_adj;
        ev
    }

    pub fn adjust_location_by(&mut self, x: i32, y: i32) {
        self.start_x += x;
        self.end_x += x;
        self.start_y += y;
        self.end_y += y;
    }
}

// LocationSet holds the primary location as well as the extra
// locations of an element. In addition it holds a ZIndex which all
// locations are said to exist at.
#[derive(Clone)]
pub struct LocationSet {
    pub l: Location,

    // Extra locations are locations that are not within the primary location
    // but are still considered to be part of the element.
    // This is useful for elements that do not have a rectangular shape (ie, the
    // menu element)
    pub extra: Vec<Location>,

    pub z: ZIndex,
}

impl Default for LocationSet {
    fn default() -> LocationSet {
        LocationSet {
            l: Location::default(),
            extra: Vec::new(),
            z: 255, // far back
        }
    }
}

impl LocationSet {
    pub fn new(l: Location, extra: Vec<Location>, z: ZIndex) -> LocationSet {
        LocationSet { l, extra, z }
    }

    pub fn with_location(mut self, l: Location) -> LocationSet {
        self.l = l;
        self
    }

    pub fn with_extra(mut self, extra: Vec<Location>) -> LocationSet {
        self.extra = extra;
        self
    }

    pub fn with_z(mut self, z: ZIndex) -> LocationSet {
        self.z = z;
        self
    }

    // Contains checks if the given location falls in the primary
    // or extra location of an element
    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.contains_within_primary(x, y) || self.contains_within_extra(x, y)
    }

    pub fn contains_within_primary(&self, x: i32, y: i32) -> bool {
        if self.l.contains_point(x, y) {
            return true;
        }
        false
    }

    pub fn contains_within_extra(&self, x: i32, y: i32) -> bool {
        for eloc in self.extra.iter() {
            if eloc.contains_point(x, y) {
                return true;
            }
        }
        false
    }

    // Relocate increments the Location by the given values
    pub fn relocate(&mut self, rr: RelocationRequest) {
        self.l.relocate(rr);
        for loc in self.extra.iter_mut() {
            loc.relocate(rr);
        }
    }

    // returns None is the point is not contained by the LocationSet
    pub fn get_z_index_for_point(&self, x: i32, y: i32) -> Option<ZIndex> {
        if self.l.contains_point(x, y) {
            return Some(self.z);
        }

        for eloc in self.extra.iter() {
            if eloc.contains_point(x, y) {
                return Some(self.z);
            }
        }
        None
    }

    pub fn set_extra_locations(&mut self, extra: Vec<Location>) {
        self.extra = extra;
    }

    pub fn push_extra_locations(&mut self, extra: Location) {
        self.extra.push(extra);
    }

    pub fn adjust_locations_by(&mut self, x: i32, y: i32) {
        self.l.adjust_location_by(x, y);
        for loc in self.extra.iter_mut() {
            loc.adjust_location_by(x, y);
        }
    }
}
