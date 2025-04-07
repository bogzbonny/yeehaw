use crate::{DynLocation, Loc, Size};

/// The draw region is passed to elements during key function calls where the element may need to
/// know information relavent to its rendering.
#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub struct DrawRegion {
    /// The size of the element, during initialization of an element this size will be unknown and
    /// will be set to a size of (0,0). During all subsiquent calls to the element, the size should
    /// be known.
    pub size: Size,

    /// the visible region of the element for very large elements, this may be a subset of the
    /// entire element
    pub visible_region: Option<Loc>,
}

impl DrawRegion {
    pub fn new_for_screen() -> Self {
        let (xmax, ymax) = crossterm::terminal::size().expect("no terminal size");
        DrawRegion {
            size: Size::new(xmax, ymax),
            visible_region: None,
        }
    }

    /// Creates a new DrawRegion with a large size (1000x1000) for testing purposes.
    /// This is particularly useful for scenarios where a large drawing area is needed
    /// without the constraints of a visible region.
    /// used for testing
    pub fn new_large() -> Self {
        DrawRegion {
            size: Size::new(1000, 1000),
            visible_region: None,
        }
    }

    /// create a new context for a child element
    /// provided its location and a higher level context
    pub fn child_region(&self, child_loc: &DynLocation) -> Self {
        let size = child_loc.get_size(self);
        let visible_region = if let Some(mut vr) = self.visible_region {
            // make the visible region relative to the el
            // NOTE because this is a saturating_sub, if the visible region is outside the start
            // bounds of the child, the visible region will be bounded to 0
            let start_x = child_loc.get_start_x(self) as u16;
            let start_y = child_loc.get_start_y(self) as u16;
            vr.start_x = vr.start_x.saturating_sub(start_x);
            vr.end_x = vr.end_x.saturating_sub(start_x);
            vr.start_y = vr.start_y.saturating_sub(start_y);
            vr.end_y = vr.end_y.saturating_sub(start_y);
            Some(vr)
        } else {
            None
        };
        Self {
            size,
            visible_region,
        }
    }

    pub fn with_visible_region(mut self, vr: Option<Loc>) -> Self {
        self.visible_region = vr;
        self
    }

    pub fn with_size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn with_height(mut self, h: u16) -> Self {
        self.size.height = h;
        self
    }

    pub fn with_width(mut self, w: u16) -> Self {
        self.size.width = w;
        self
    }

    pub fn get_width(&self) -> u16 {
        self.size.width
    }

    pub fn get_height(&self) -> u16 {
        self.size.height
    }
}
