use {
    crate::*,
    crossterm::event::{MouseButton, MouseEventKind},
    portable_pty::CommandBuilder,
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
    pub editor_is_open: Rc<RefCell<bool>>,

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
        Self::new_with_custom_editor(
            ctx,
            title,
            editor,
            "No editor found (please set your $EDITOR environment var)",
        )
    }

    /// use this if you want to specify a mandatory editor or use an alternative
    /// environment variable
    ///
    /// NOTE the caller of this function is expected to check for the editor
    /// and provide a None if the editor is not found. Otherwise the editor
    /// will error on first call. See the `new` function for expected behaviour.
    pub fn new_with_custom_editor<S: Into<String>, T: Into<String>>(
        ctx: &Context, title: S, mut editor: Option<String>, editor_missing_text: T,
    ) -> Self {
        // ensure that the editor is found
        let mut set_none = false;
        if let Some(ref editor) = editor {
            if !std::path::Path::new(editor).exists() {
                set_none = true;
            }
        }
        if set_none {
            editor = None;
        }

        let pane = ParentPane::new(ctx, Self::KIND);

        let non_editing_textbox = TextBox::new(ctx, "")
            .with_width(DynVal::FULL)
            .with_height(DynVal::FULL)
            .with_wordwrap(ctx)
            .non_editable(ctx)
            .with_right_click_menu(None)
            .at(DynVal::new_fixed(0), DynVal::new_fixed(0));

        Self {
            pane,
            editor,
            editor_not_found_text: Rc::new(RefCell::new(editor_missing_text.into())),
            title: Rc::new(RefCell::new(title.into())),
            text: Rc::new(RefCell::new(None)),
            tempfile: Rc::new(RefCell::new(None)),
            editor_is_open: Rc::new(RefCell::new(false)),
            non_editing_textbox: Rc::new(RefCell::new(non_editing_textbox)),
            just_created: Rc::new(RefCell::new(true)),
            clicked_down: Rc::new(RefCell::new(false)),
            text_changed_hook: Rc::new(RefCell::new(Box::new(|_, _| EventResponses::default()))),
        }
    }

    #[must_use]
    pub fn open_editor(&self, ctx: &Context) -> EventResponse {
        let text = self.text.borrow().clone();
        self.editor_is_open.replace(true);
        match self.editor {
            Some(ref editor) => {
                let mut cmd = CommandBuilder::new(editor);

                let prefix = format!("{}_", self.title.borrow());
                let Ok(tempfile) = tempfile::Builder::new().prefix(prefix.as_str()).tempfile()
                else {
                    log_err!("Could not create tempfile");
                    return EventResponse::None;
                };
                if let Some(text) = text {
                    // set the tempfile contents to the text
                    if let Err(e) = std::fs::write(tempfile.path(), text) {
                        log_err!("Could not write to tempfile: {}", e);
                        return EventResponse::None;
                    }
                }

                let tempfile_path = tempfile
                    .path()
                    .to_str()
                    .expect("tempfile to_str is None")
                    .to_string();
                cmd.arg(tempfile_path.clone());
                self.tempfile.replace(Some(tempfile));
                self.just_created.replace(true);

                if let Ok(cwd) = std::env::current_dir() {
                    cmd.cwd(cwd);
                }
                match TerminalPane::new_with_builder(ctx, cmd) {
                    Ok(term) => {
                        //term.reset_prev_draw(); // XXX remove
                        self.pane.add_element(Box::new(term))
                    }
                    Err(e) => {
                        log_err!("Could not open terminal: {}", e);
                        EventResponse::None
                    }
                }
            }
            None => {
                let el = ParentPaneOfSelectable::new(ctx)
                    .with_dyn_height(DynVal::FULL)
                    .with_dyn_width(DynVal::FULL)
                    .with_focused(ctx);

                let title_label = Label::new(ctx, &self.title.borrow());
                let _ = el.add_element(Box::new(title_label));

                let start_text = self.editor_not_found_text.borrow().clone();
                let tb = TextBox::new(ctx, "")
                    .with_text_when_empty(start_text)
                    .with_width(DynVal::FULL)
                    .with_height(DynVal::FULL.minus(2.into()))
                    .with_right_scrollbar(ctx)
                    .with_line_numbers(ctx)
                    .at(0, 2);
                tb.pane.select();

                let tb_ = tb.clone();
                let self_ = self.clone();
                let ctx_ = ctx.clone();
                let save_fn = Box::new(move |_, _| {
                    let t = tb_.get_text();
                    self_.text.replace(Some(t.clone()));
                    self_.text_changed_hook.borrow_mut()(ctx_.clone(), t);
                    EventResponses::default()
                });

                let btn_save =
                    Button::new(ctx, "save", save_fn).at(DynVal::FULL.minus(7.into()), 0);
                let _ = el.add_element(Box::new(btn_save));

                let _ = el.add_element(Box::new(tb));
                self.pane.add_element(Box::new(el))
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

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
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
        if !*self.editor_is_open.borrow() {
            // activate the editor on click
            let clicked_down = *self.clicked_down.borrow();
            if let Event::Mouse(me) = ev {
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        //return (true, EventResponses::default());
                    }
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        let mut resps = EventResponses::default();
                        let resp = self.pane.clear_elements();
                        resps.push(resp);
                        let resp = self.open_editor(ctx);
                        resps.push(resp);
                        return (true, resps);
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }
            } else {
                *self.clicked_down.borrow_mut() = false;
            }
        }

        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());

        if !self.pane.has_elements() {
            self.tempfile.borrow_mut().take();
            self.editor_is_open.replace(false);
            let text = self.text.borrow().clone().unwrap_or_default();
            debug!("text: {}", text);
            self.non_editing_textbox.borrow().set_text(text);
            let non_editing_textbox = self.non_editing_textbox.borrow().clone();
            non_editing_textbox.set_dirty();

            let resp = self.pane.add_element(Box::new(non_editing_textbox));
            resps.push(resp);
        }

        (captured, resps)
    }
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        let out = self.pane.drawing(ctx, force_update);

        // TODO maybe do this somewhere else? on a different thread?
        // check for changes to the tempfile each draw
        if let Some(tempfile) = self.tempfile.borrow().as_ref() {
            let tempfile_path = tempfile
                .path()
                .to_str()
                .expect("tempfile to_str is None")
                .to_string();
            let Ok(file_contents) = std::fs::read_to_string(tempfile_path) else {
                return out;
            };
            let old_text = self.text.borrow().clone();
            let new_text = file_contents.trim_end_matches('\n').to_string();

            if old_text.as_deref() != Some(&new_text) {
                debug!("old_text: {:?}", old_text);
                debug!("new_text: {}", new_text);
                let is_empty = new_text.is_empty();
                if !*self.just_created.borrow() {
                    self.text.replace(Some(new_text.clone()));
                    debug!("text changed to: {}", new_text);
                    self.text_changed_hook.borrow_mut()(ctx.clone(), new_text);
                }
                if *self.just_created.borrow() && !is_empty {
                    *self.just_created.borrow_mut() = false;
                }
            }
        }

        out
    }
}
