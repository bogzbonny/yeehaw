use crate::element::RelocationRequest;
use std::ops::Deref;

// ZIndex is the z-index or position in the z-dimension of the element
// The lower the z-index, further toward the front the element is
// (0 is the front, 1 is behind 0, etc.)
#[derive(Clone, Copy, Default)]
pub struct ZIndex(pub i32);

impl Deref for ZIndex {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Location holds the primary location of an element in the context of
// its parent element
#[derive(Clone, Default)]
pub struct Location {
    pub start_x: i32,
    pub end_x: i32,
    pub start_y: i32,
    pub end_y: i32,
    pub z: ZIndex,
}

impl Location {
    pub fn new(start_x: i32, end_x: i32, start_y: i32, end_y: i32, z: ZIndex) -> Location {
        Location {
            start_x,
            end_x,
            start_y,
            end_y,
            z,
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

    // Relocate increments the Location by the given values
    pub fn relocate(&mut self, rr: RelocationRequest) {
        self.start_x += rr.left;
        self.end_x += rr.right;
        self.start_y += rr.up;
        self.end_y += rr.down;
    }

    // GetSize returns the size of the Location
    pub fn get_size(&self) -> Size {
        Size::new(self.width(), self.height())
    }

    // TODO for crossterm
    //pub fn adjust_mouse_event(&self, ev: &tcell::EventMouse) -> tcell::EventMouse {
    //    let (x_adj, y_adj) = ev.position();
    //    let x_adj = x_adj - self.start_x;
    //    let y_adj = y_adj - self.start_y;
    //    tcell::EventMouse::new(x_adj, y_adj, ev.buttons(), ev.modifiers())
    //}

    pub fn adjust_location_by(&mut self, x: i32, y: i32) {
        self.start_x += x;
        self.end_x += x;
        self.start_y += y;
        self.end_y += y;
    }
}

// Locations holds the primary location as well as the extra
// locations of an element
#[derive(Default)]
pub struct Locations {
    l: Location,
    // Extra locations are locations that are not within the primary location
    // but are still considered to be part of the element.
    // This is useful for elements that do not have a rectangular shape (ie, the
    // menu element)
    extra: Vec<Location>,
}

impl From<Location> for Locations {
    fn from(l: Location) -> Self {
        Locations {
            l,
            extra: Vec::new(),
        }
    }
}

impl Locations {
    pub fn new(l: Location, extra: Vec<Location>) -> Locations {
        Locations { l, extra }
    }

    // Contains checks if the given location falls in the primary
    // or extra location of an element
    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.contains_within_primary(x, y) || self.contains_within_extra(x, y)
    }

    pub fn contains_within_primary(&self, x: i32, y: i32) -> bool {
        if x >= self.l.start_x && x <= self.l.end_x && y >= self.l.start_y && y <= self.l.end_y {
            return true;
        }
        false
    }

    pub fn contains_within_extra(&self, x: i32, y: i32) -> bool {
        for loc in self.extra.iter() {
            if x >= loc.start_x && x <= loc.end_x && y >= loc.start_y && y <= loc.end_y {
                return true;
            }
        }
        false
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

// Size holds the width and height of an element
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub fn new(width: u16, height: u16) -> Size {
        Size { width, height }
    }
}

pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}
