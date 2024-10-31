use {
    crate::{
        widgets::TextBox, Context, DrawCh, DrawChPos, DrawChs2D, DynLocationSet, DynVal, Element,
        ElementID, Event, EventResponses, Parent, ParentPane, Priority, ReceivableEventChanges,
        SortingHat, Style, TerminalPane, ZIndex,
    },
    portable_pty::CommandBuilder,
    std::{cell::RefCell, rc::Rc},
};

// displays the size
#[derive(Clone)]
pub struct TermEditorPane {
    pub pane: ParentPane,
    pub editor: Option<String>,
    pub text: Rc<RefCell<String>>,
}

impl TermEditorPane {
    pub const KIND: &'static str = "term_editor_pane";

    pub fn new(hat: &SortingHat, _ctx: &Context) -> Self {
        let editor: Option<String> = std::env::var("EDITOR").ok();
        let pane = ParentPane::new(hat, Self::KIND);

        let out = Self {
            pane,
            editor,
            text: Rc::new(RefCell::new(String::new())),
        };
        //out.open_editor();
        out
    }

    //pub fn open_editor(&self) {
    //    match self.editor {
    //        Some(ref editor) => {
    //            let mut cmd = CommandBuilder::new(editor);
    //            if let Ok(cwd) = std::env::current_dir() {
    //                cmd.cwd(cwd);
    //            }
    //            let term = TerminalPane::new_with_builder(
    //                self.pane.hat.clone(),
    //                self.pane.ctx.clone(),
    //                cmd,
    //            );
    //            self.pane.add_element(Box::new(term));
    //        }
    //        None => {
    //            let tb = TextBox::new(self.pane.hat.clone(), self.pane.ctx.clone());

    //            self.pane.add_element(Box::new(term));
    //        }
    //    }
    //}

    pub fn with_text(self, text: String) -> Self {
        *self.text.borrow_mut() = text;
        self
    }

    pub fn with_height(self, h: DynVal) -> Self {
        self.pane.set_dyn_height(h);
        self
    }

    pub fn with_default_ch(self, ch: DrawCh) -> Self {
        self.pane.pane.set_default_ch(ch);
        self
    }

    pub fn with_width(self, w: DynVal) -> Self {
        self.pane.set_dyn_width(w);
        self
    }

    pub fn with_z(self, z: ZIndex) -> Self {
        self.pane.set_z(z);
        self
    }
}

impl Element for TermEditorPane {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
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
