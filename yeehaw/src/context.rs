use {
    // crate::ColorContext,
    crate::{ColorStore, Event, SortingHat},
    std::collections::HashMap,
    tokio::sync::mpsc::Sender,
};

/// Context is a struct which contains information about the current context of a
/// given element.
/// The context of an element is passed to the element during key function calls
/// where the element may need to know information relavent to its rendering.
///
/// Additionally, metadata may be added to a context to pass additional
/// arbitrary information between elements.
#[derive(Clone, Debug)]
pub struct Context {
    pub dur_since_launch: std::time::Duration,
    //                      key , value
    pub metadata: HashMap<String, Vec<u8>>,
    pub hat: SortingHat,
    pub ev_tx: Sender<Event>,
    pub color_store: ColorStore,
}

///// Default implementation for Context, use only as a dummy value
//impl Default for Context {
//    fn default() -> Self {
//        let (ev_tx, _) = tokio::sync::mpsc::channel::<Event>(1);
//        Context {
//            size: Size::default(),
//            dur_since_launch: std::time::Duration::default(),
//            visible_region: None,
//            metadata: HashMap::new(),
//            parent_ctx: None,
//            hat: SortingHat::default(),
//            ev_tx,
//            color_store: ColorStore::default(),
//        }
//    }
//}

impl Context {
    pub fn new_context_no_dur(
        hat: &SortingHat, ev_tx: Sender<Event>, color_store: &ColorStore,
    ) -> Context {
        Context {
            dur_since_launch: std::time::Duration::default(),
            metadata: HashMap::new(),
            hat: hat.clone(),
            ev_tx,
            color_store: color_store.clone(),
        }
    }

    pub fn new_context(
        launch_instant: std::time::Instant, hat: &SortingHat, ev_tx: Sender<Event>,
        color_store: &ColorStore,
    ) -> Context {
        Context {
            dur_since_launch: launch_instant.elapsed(),
            metadata: HashMap::new(),
            hat: hat.clone(),
            ev_tx,
            color_store: color_store.clone(),
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

    //pub fn get_color_context(&self) -> ColorContext {
    //    ColorContext {
    //        store: self.color_store.clone(),
    //        size: self.size,
    //        dur_since_launch: self.dur_since_launch,
    //    }
    //}
}
