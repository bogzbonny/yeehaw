use {
    crate::{DynLocation, Event, Loc, Size, SortingHat},
    std::collections::HashMap,
    tokio::sync::mpsc::Sender,
};

/// Context is a struct which contains information about the current context of a
/// given element.
/// The context of an element is passed to the element during key function calls
/// where the element may need to know its size
///
/// Additionally, metadata may be addended to the context to pass additional
/// arbitrary information.
#[derive(Clone, Debug)]
pub struct Context {
    /// The size of the element, during initialization of an element this size will be unknown and
    /// will be set to a size of (0,0). During all subsiquent calls to the element, the size should
    /// be known.
    pub s: Size,
    pub dur_since_launch: std::time::Duration,
    /// the visible region of the element for very large elements, this may be a subset of the
    /// entire element
    pub visible_region: Option<Loc>,
    //                      key , value
    pub metadata: HashMap<String, Vec<u8>>,
    pub parent_ctx: Option<Box<Context>>,
    pub hat: SortingHat,
    pub ev_tx: Sender<Event>,
}

/// Default implementation for Context, use only as a dummy value
impl Default for Context {
    fn default() -> Self {
        let (ev_tx, _) = tokio::sync::mpsc::channel::<Event>(1);
        Context {
            s: Size::default(),
            dur_since_launch: std::time::Duration::default(),
            visible_region: None,
            metadata: HashMap::new(),
            parent_ctx: None,
            hat: SortingHat::default(),
            ev_tx,
        }
    }
}

impl Context {
    // TODO return error
    pub fn new_context_for_screen_no_dur(hat: &SortingHat, ev_tx: Sender<Event>) -> Context {
        let (xmax, ymax) = crossterm::terminal::size().expect("no terminal size");
        Context {
            s: Size::new(xmax, ymax),
            dur_since_launch: std::time::Duration::default(),
            visible_region: None,
            metadata: HashMap::new(),
            parent_ctx: None,
            hat: hat.clone(),
            ev_tx,
        }
    }

    // TODO return error
    pub fn new_context_for_screen(
        launch_instant: std::time::Instant, hat: &SortingHat, ev_tx: Sender<Event>,
    ) -> Context {
        let (xmax, ymax) = crossterm::terminal::size().expect("no terminal size");
        Context {
            s: Size::new(xmax, ymax),
            dur_since_launch: launch_instant.elapsed(),
            visible_region: None,
            metadata: HashMap::new(),
            parent_ctx: None,
            hat: hat.clone(),
            ev_tx,
        }
    }

    /// create a new context for a child element
    /// provided its location and a higher level context
    pub fn child_context(&self, child_loc: &DynLocation) -> Context {
        let size = child_loc.get_size(self);
        let visible_region = if let Some(mut vr) = self.visible_region {
            // make the visible region relative to the el
            let el_x = child_loc.get_start_x(self) as u16;
            let el_y = child_loc.get_start_y(self) as u16;
            vr.start_x = vr.start_x.saturating_sub(el_x);
            vr.end_x = vr.end_x.saturating_sub(el_x);
            vr.start_y = vr.start_y.saturating_sub(el_y);
            vr.end_y = vr.end_y.saturating_sub(el_y);
            Some(vr)
        } else {
            None
        };
        Context {
            s: size,
            dur_since_launch: self.dur_since_launch,
            visible_region,
            metadata: self.metadata.clone(),
            parent_ctx: Some(Box::new(self.clone())),
            hat: self.hat.clone(),
            ev_tx: self.ev_tx.clone(),
        }
    }

    /// create a new context for initialization of a child element where the size of the child is
    /// not yet known
    pub fn child_init_context(&self) -> Context {
        Context {
            s: Size::default(),
            dur_since_launch: self.dur_since_launch,
            visible_region: None,
            metadata: self.metadata.clone(),
            parent_ctx: Some(Box::new(self.clone())),
            hat: self.hat.clone(),
            ev_tx: self.ev_tx.clone(),
        }
    }

    pub fn must_get_parent_context(&self) -> &Context {
        self.parent_ctx
            .as_ref()
            .expect("all contexts besides the first (TUI) context must have a parent")
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
