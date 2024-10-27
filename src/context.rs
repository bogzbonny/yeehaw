use {
    crate::{Loc, Size},
    std::collections::HashMap,
};

// Context is a struct which contains information about the current context of a
// given element.
// The context of an element is passed to the element during key function calls
// where the element may need to know its size
//
// Additionally, metadata may be addended to the context to pass additional
// arbitrary information.
#[derive(Default, Clone, Debug)]
pub struct Context {
    pub s: Size,
    pub dur_since_launch: std::time::Duration,
    pub visible_region: Option<Loc>, // the visible region of the element
    //                      key , value
    pub metadata: HashMap<String, Vec<u8>>,
}

impl Context {
    pub fn new(s: Size, dur_since_launch: std::time::Duration) -> Context {
        Context {
            s,
            dur_since_launch,
            visible_region: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_visible_region(mut self, vr: Option<Loc>) -> Self {
        self.visible_region = vr;
        self
    }

    pub fn with_size(mut self, s: Size) -> Self {
        self.s = s;
        self
    }

    pub fn with_height(mut self, h: u16) -> Self {
        self.s.height = h;
        self
    }

    pub fn with_width(mut self, w: u16) -> Self {
        self.s.width = w;
        self
    }

    // TODO return error
    pub fn new_context_for_screen_no_dur() -> Context {
        let (xmax, ymax) = crossterm::terminal::size().unwrap();
        Context {
            s: Size::new(xmax, ymax),
            dur_since_launch: std::time::Duration::default(),
            visible_region: None,
            metadata: HashMap::new(),
        }
    }

    // TODO return error
    pub fn new_context_for_screen(launch_instant: std::time::Instant) -> Context {
        let (xmax, ymax) = crossterm::terminal::size().unwrap();
        Context {
            s: Size::new(xmax, ymax),
            dur_since_launch: launch_instant.elapsed(),
            visible_region: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, md: Vec<u8>) -> Self {
        self.metadata.insert(key, md);
        self
    }

    pub fn clear_metadata(&mut self) {
        self.metadata.clear();
    }

    pub fn get_metadata(&self, key: &str) -> Option<Vec<u8>> {
        self.metadata.get(key).cloned()
    }

    pub fn get_width(&self) -> u16 {
        self.s.width
    }

    pub fn get_height(&self) -> u16 {
        self.s.height
    }
}
