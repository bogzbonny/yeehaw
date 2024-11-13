use {
    crate::*,
    crossterm::event::{MouseButton, MouseEventKind},
    portable_pty::CommandBuilder,
    std::{cell::RefCell, rc::Rc},
};

/// TODO implement/test editor missing case
/// save / revert buttons

/// TODO make into a selectible widget once widget is refactored into element

/// NOTE this is not the most secure thing as it uses temp files to store the text
/// but it is the easiest way to get a text editor in a terminal
/// this editor should not be used for passwords.
#[derive(Clone)]
pub struct TermEditorPane {
    pub pane: ParentPane,
    pub editor: Option<String>,
    /// the /opt/homebrew/bin/nvim environment variable
    pub editor_not_found_text: Rc<RefCell<String>>,
    /// text to display if the editor is not found
    pub title: Rc<RefCell<String>>,
    /// title for the textbox also used for tempfile suffix
    pub text: Rc<RefCell<Option<String>>>,

    pub tempfile: Rc<RefCell<Option<tempfile::NamedTempFile>>>,

    pub non_editing_textbox: Rc<RefCell<TextBox>>,
    /// the textbox when not being edited for viewing the text

    /// if the tempfile was just created (and thus the text is empty)
    pub just_created: Rc<RefCell<bool>>,

    pub clicked_down: Rc<RefCell<bool>>,
    /// activated when mouse is clicked down while over button
    pub text_changed_hook: Rc<RefCell<TextChangedHook>>,
}

pub type TextChangedHook = Box<dyn FnMut(Context, String) -> EventResponses>;

impl TermEditorPane {
    pub const KIND: &'static str = "term_editor_pane";

    pub fn new<S: Into<String>>(ctx: &Context, title: S) -> Self {
        let editor: Option<String> = std::env::var("EDITOR").ok();
        Self::new_with_custom_editor(ctx, title, editor)
    }

    /// use this if you want to specify a mandatory editor or use an alternative
    /// environment variable
    pub fn new_with_custom_editor<S: Into<String>>(
        ctx: &Context, title: S, editor: Option<String>,
    ) -> Self {
        let pane = ParentPane::new(ctx, Self::KIND);

        let non_editing_textbox = TextBox::new(ctx, "")
            .with_width(DynVal::full())
            .with_height(DynVal::full())
            .with_wordwrap()
            .non_editable()
            .with_right_click_menu(None)
            .at(DynVal::new_fixed(0), DynVal::new_fixed(0));

        Self {
            pane,
            editor,
            editor_not_found_text: Rc::new(RefCell::new(
                "No editor found (please set your $EDITOR environment var)".into(),
            )),
            title: Rc::new(RefCell::new(title.into())),
            text: Rc::new(RefCell::new(None)),
            tempfile: Rc::new(RefCell::new(None)),
            non_editing_textbox: Rc::new(RefCell::new(non_editing_textbox)),
            just_created: Rc::new(RefCell::new(true)),
            clicked_down: Rc::new(RefCell::new(false)),
            text_changed_hook: Rc::new(RefCell::new(Box::new(|_, _| EventResponses::default()))),
        }
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
                if let Some(text) = text {
                    // set the tempfile contents to the text
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
                let start_text = self.editor_not_found_text.borrow().clone();
                let tb = TextBox::new(ctx, "")
                    .with_text_when_empty(start_text)
                    .with_width(DynVal::full())
                    .with_height(DynVal::full())
                    .with_no_wordwrap()
                    .at(DynVal::new_fixed(0), DynVal::new_fixed(0));

                use crate::widgets::Widget;
                tb.set_selectability(ctx, Selectability::Selected);

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

    pub fn with_non_editing_textbox(self, tb: TextBox) -> Self {
        self.non_editing_textbox.replace(tb);
        self
    }

    pub fn set_non_editing_textbox(&self, tb: TextBox) {
        self.non_editing_textbox.replace(tb);
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

#[yeehaw_derive::impl_element_from(pane)]
impl Element for TermEditorPane {
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

        if !self.pane.has_elements() {
            debug!("pane has no elements");
            self.tempfile.borrow_mut().take();
            let text = self.text.borrow().clone().unwrap_or_default();
            self.non_editing_textbox.borrow().set_text(text);
            let non_editing_textbox = self.non_editing_textbox.borrow().clone();

            self.pane.add_element(Box::new(non_editing_textbox));
        }

        (captured, resps)
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

        out
    }
}
