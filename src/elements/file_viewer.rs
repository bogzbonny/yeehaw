use {
    crate::{
        widgets::TextBox, Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Parent, Priority, ReceivableEventChanges, SelfReceivableEvents, WidgetPane,
    },
    std::path::PathBuf,
    std::{cell::RefCell, rc::Rc},
};

// displays the size
#[derive(Clone)]
pub struct FileViewerPane {
    pub pane: WidgetPane,
}

impl FileViewerPane {
    pub fn new(ctx: &Context, file_path: PathBuf) -> FileViewerPane {
        let content = std::fs::read_to_string(file_path).unwrap();

        let pane = WidgetPane::new(ctx);
        let tb = TextBox::new(ctx, content)
            .with_width(DynVal::new_flex(1.))
            .with_height(DynVal::new_flex(1.))
            .with_right_scrollbar()
            .with_lower_scrollbar()
            .editable()
            .with_no_wordwrap()
            .at(DynVal::new_fixed(0), DynVal::new_fixed(0))
            .to_widgets(ctx);
        pane.add_widgets(tb);

        FileViewerPane { pane }
    }
}

impl Element for FileViewerPane {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> SelfReceivableEvents {
        self.pane.receivable()
    }
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.pane.receive_event(ctx, ev.clone())
    }
    fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.pane.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_parent(&self, up: Box<dyn Parent>) {
        self.pane.set_parent(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.pane.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.pane.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.pane.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.pane.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.pane.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.pane.get_visible()
    }
}
