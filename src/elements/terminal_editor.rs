use {
    crate::{
        widgets::{Label, TextBox},
        Context, DrawCh, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponse, EventResponses, Parent, ParentPane, Priority, ReceivableEventChanges,
        TerminalPane, ZIndex,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    portable_pty::CommandBuilder,
    std::{cell::RefCell, rc::Rc},
};

// TODO remove or kill exit_on_return
// TODO nicer top bar during non-editing / styles
// TODO implement/test editor missing case

// TODO make into a selectible widget once widget is refactored into element

/// NOTE this is not the most secure thing as it uses temp files to store the text
/// but it is the easiest way to get a text editor in a terminal
/// this editor should not be used for passwords
// displays the size
#[derive(Clone)]
pub struct TermEditorPane {
    pub pane: ParentPane,
    pub editor: Option<String>,
    pub title: Rc<RefCell<String>>, // title for the textbox also used for tempfile suffix
    pub text: Rc<RefCell<Option<String>>>,
    pub tempfile: Rc<RefCell<Option<tempfile::NamedTempFile>>>,

    // if the tempfile was just created (and thus the text is empty)
    pub just_created: Rc<RefCell<bool>>,

    // XXX isn't used
    // if set to true, this element will destruct on save
    // otherwise the element is replaced with the text
    // until it is next opened.
    pub exit_on_return: Rc<RefCell<bool>>,

    pub clicked_down: Rc<RefCell<bool>>, // activated when mouse is clicked down while over button

    pub text_changed_hook: Rc<RefCell<TextChangedHook>>,
}

pub type TextChangedHook = Box<dyn FnMut(Context, String) -> EventResponses>;

impl TermEditorPane {
    pub const KIND: &'static str = "term_editor_pane";

    pub fn new<S: Into<String>>(ctx: &Context, title: S) -> Self {
        let editor: Option<String> = std::env::var("EDITOR").ok();
        let pane = ParentPane::new(ctx, Self::KIND);

        let out = Self {
            pane,
            editor,
            title: Rc::new(RefCell::new(title.into())),
            text: Rc::new(RefCell::new(None)),
            tempfile: Rc::new(RefCell::new(None)),
            just_created: Rc::new(RefCell::new(true)),
            exit_on_return: Rc::new(RefCell::new(false)),
            clicked_down: Rc::new(RefCell::new(false)),
            text_changed_hook: Rc::new(RefCell::new(Box::new(|_, _| EventResponses::default()))),
        };
        let _ = out.open_editor(ctx); // ignore resp
        out
    }

    pub fn open_editor(&self, ctx: &Context) -> EventResponse {
        let text = self.text.borrow().clone();
        match self.editor {
            Some(ref editor) => {
                let mut cmd = CommandBuilder::new(editor);

                let prefix = format!("{}_", self.title.borrow());
                let tempfile = tempfile::Builder::new()
                    .prefix(prefix.as_str())
                    .tempfile()
                    .unwrap();
                // set the tempfile contents to the text
                if let Some(text) = text {
                    std::fs::write(tempfile.path(), text).unwrap();
                }

                let tempfile_path = tempfile.path().to_str().unwrap().to_string();
                cmd.arg(tempfile_path.clone());
                self.tempfile.replace(Some(tempfile));
                self.just_created.replace(true);

                if let Ok(cwd) = std::env::current_dir() {
                    cmd.cwd(cwd);
                }
                let term = TerminalPane::new_with_builder(ctx, cmd);
                self.pane.add_element(Box::new(term))
            }
            None => {
                let text = text.unwrap_or_else(|| {
                    "No editor found (please set your $EDITOR environment var)".to_string()
                });
                let tb = TextBox::new(ctx, text)
                    .with_width(DynVal::new_flex(1.))
                    .with_height(DynVal::new_flex(1.))
                    .with_no_wordwrap()
                    .at(DynVal::new_fixed(0), DynVal::new_fixed(0));

                self.pane.add_element(Box::new(tb))
            }
        }
    }

    pub fn with_text_changed_hook(self, hook: TextChangedHook) -> Self {
        *self.text_changed_hook.borrow_mut() = hook;
        self
    }

    pub fn set_text_changed_hook(&self, hook: TextChangedHook) {
        *self.text_changed_hook.borrow_mut() = hook;
    }

    pub fn with_text(self, text: String) -> Self {
        *self.text.borrow_mut() = Some(text);
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

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.pane.set_at(loc_x, loc_y);
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
        if self.tempfile.borrow().is_none() {
            // activate the editor on click
            let clicked_down = *self.clicked_down.borrow();
            if let Event::Mouse(me) = ev {
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, EventResponses::default());
                    }
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        self.pane.clear_elements();
                        let resp = self.open_editor(ctx);
                        return (true, resp.into());
                    }
                    _ => {}
                }
            }
            *self.clicked_down.borrow_mut() = false;
        }

        let (captured, resps) = self.pane.receive_event(ctx, ev.clone());

        (captured, resps)
    }
    fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let out = self.pane.drawing(ctx);

        // check for changes to the tempfile each draw
        if let Some(tempfile) = self.tempfile.borrow().as_ref() {
            let tempfile_path = tempfile.path().to_str().unwrap().to_string();
            let Ok(file_contents) = std::fs::read_to_string(tempfile_path) else {
                return out;
            };
            let old_text = self.text.borrow().clone();

            if old_text.as_deref() != Some(file_contents.as_str()) {
                let is_empty = file_contents.is_empty();
                if !*self.just_created.borrow() {
                    self.text.replace(Some(file_contents.clone()));
                    self.text_changed_hook.borrow_mut()(ctx.clone(), file_contents);
                }
                if *self.just_created.borrow() && !is_empty {
                    *self.just_created.borrow_mut() = false;
                }
            }
        }

        if !self.pane.has_elements() {
            debug!("pane has no elements");
            //if !*self.exit_on_return.borrow() {
            //*resp = EventResponse::None;
            //} else {
            self.tempfile.borrow_mut().take();
            let text = self.text.borrow().clone().unwrap_or_default();
            let label = Label::new(ctx, &text).at(DynVal::new_fixed(0), DynVal::new_fixed(0));
            self.pane.add_element(Box::new(label));
            //*resp = EventResponse::NewElement(Box::new(label), None);
            //}
        }
        out
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
