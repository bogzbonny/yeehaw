use {
    crate::{
        elements::menu::{MenuItem, MenuPath, MenuStyle},
        Keyboard as KB, *,
    },
    crossterm::event::{KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
};

// TODO cache WrChs for efficiency. Also get_wrapped should return a Ref<WrChs>

// TODO better multiline cursor movement
// retain greater cursor position between lines, ex:
//    123456789<cursor, starting position>
//    1234<cursor after moving down>
//    123456789<cursor, after moving down again>

#[derive(Clone)]
pub struct TextBox {
    pub pane: SelectablePane,
    pub inner: Rc<RefCell<TextBoxInner>>,
    pub x_scrollbar: Rc<RefCell<Option<HorizontalScrollbar>>>,
    pub y_scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
    pub line_number_tb: Rc<RefCell<Option<TextBoxInner>>>,
}

impl TextBox {
    const KIND: &'static str = "textbox";
    pub fn new<S: Into<String>>(init_ctx: &Context, text: S) -> Self {
        let text = text.into();
        let s = Size::get_text_size(&text);
        let pane = SelectablePane::new(init_ctx, Self::KIND)
            .with_dyn_width(DynVal::new_fixed(s.width as i32))
            .with_dyn_height(DynVal::new_fixed(s.height as i32))
            .with_styles(TextBoxInner::STYLE);
        let inner = TextBoxInner::new(init_ctx, text);

        pane.pane.add_element(Box::new(inner.clone()));
        let tb = TextBox {
            pane,
            inner: Rc::new(RefCell::new(inner)),
            x_scrollbar: Rc::new(RefCell::new(None)),
            y_scrollbar: Rc::new(RefCell::new(None)),
            line_number_tb: Rc::new(RefCell::new(None)),
        };

        *tb.inner.borrow().current_sty.borrow_mut() = tb.pane.get_current_style();

        let tb_ = tb.clone();

        tb.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                tb_.post_hook_for_set_selectability();
            }));

        let _ = tb.drawing(&init_ctx.child_context(&tb.pane.get_dyn_location()), true); // to set the pane content
        tb
    }

    pub fn post_hook_for_set_selectability(&self) {
        let sel = self.pane.get_selectability();
        *self.inner.borrow().selectedness.borrow_mut() = sel;
        *self.inner.borrow().current_sty.borrow_mut() = self.pane.get_current_style();
        *self.inner.borrow().is_dirty.borrow_mut() = true;
        if sel != Selectability::Selected {
            *self.inner.borrow().visual_mode.borrow_mut() = false;
        }
    }

    pub fn set_dirty(&self) {
        *self.inner.borrow().is_dirty.borrow_mut() = true;
    }

    pub fn with_scrollbars(self, init_ctx: &Context) -> Self {
        self.set_x_scrollbar_inner(init_ctx, HorizontalSBPositions::Below);
        self.set_y_scrollbar_inner(init_ctx, VerticalSBPositions::ToTheRight);
        self
    }

    pub fn with_left_scrollbar(self, init_ctx: &Context) -> Self {
        self.set_y_scrollbar_inner(init_ctx, VerticalSBPositions::ToTheLeft);
        self
    }

    pub fn with_right_scrollbar(self, init_ctx: &Context) -> Self {
        self.set_y_scrollbar_inner(init_ctx, VerticalSBPositions::ToTheRight);
        self
    }

    fn set_y_scrollbar_inner(&self, init_ctx: &Context, pos: VerticalSBPositions) {
        let content_height = self.inner.borrow().pane.content_height();
        let content_size = self.inner.borrow().pane.content_size();

        // accounts for the other scrollbar
        let inner_start_y = self.inner.borrow().pane.get_dyn_start_y();

        let sb = VerticalScrollbar::new(init_ctx, DynVal::FULL, content_size, content_height)
            .without_keyboard_events();
        if self.x_scrollbar.borrow().is_some() {
            sb.pane.set_end_y(DynVal::FULL.minus_fixed(1));
        }
        match pos {
            VerticalSBPositions::ToTheLeft => {
                sb.set_at(0.into(), inner_start_y);
                if let Some(x_sb) = &*self.x_scrollbar.borrow() {
                    x_sb.pane.set_start_x(1);
                }
                self.inner.borrow().pane.set_start_x(1);
            }
            VerticalSBPositions::ToTheRight => {
                sb.set_at(DynVal::FULL.minus_fixed(1), inner_start_y);
                self.inner
                    .borrow()
                    .pane
                    .set_end_x(DynVal::FULL.minus_fixed(1));
                if let Some(x_sb) = &*self.x_scrollbar.borrow() {
                    x_sb.pane.set_end_x(DynVal::FULL.minus_fixed(1));
                }
            }
            VerticalSBPositions::None => {
                return;
            }
        }
        let size = init_ctx
            .child_context(&self.inner.borrow().pane.get_dyn_location())
            .size;
        sb.set_scrollable_view_size(size);
        if let Some(x_sb) = &*self.x_scrollbar.borrow() {
            x_sb.set_scrollable_view_size(size);
        }

        // wire the scrollbar to the textbox
        let pane_ = self.inner.borrow().pane.clone();
        let is_dirty = self.inner.borrow().is_dirty.clone();
        let hook = Box::new(move |init_ctx, y| {
            pane_.set_content_y_offset(&init_ctx, y);
            is_dirty.replace(true);
        });
        *sb.position_changed_hook.borrow_mut() = Some(hook);
        *self.y_scrollbar.borrow_mut() = Some(sb.clone());
        self.pane.pane.add_element(Box::new(sb.clone()));
        self.inner.borrow().y_scrollbar.replace(Some(sb));

        if let VerticalSBPositions::ToTheLeft = pos {
            self.reset_line_numbers(init_ctx);
        }
        self.set_corner_decor(init_ctx);
        self.reset_sb_sizes(init_ctx);
    }

    pub fn with_top_scrollbar(self, init_ctx: &Context) -> Self {
        self.set_x_scrollbar_inner(init_ctx, HorizontalSBPositions::Above);
        self
    }

    pub fn with_bottom_scrollbar(self, init_ctx: &Context) -> Self {
        self.set_x_scrollbar_inner(init_ctx, HorizontalSBPositions::Below);
        self
    }

    fn set_x_scrollbar_inner(&self, init_ctx: &Context, pos: HorizontalSBPositions) {
        let content_width = self.inner.borrow().pane.content_width();
        let content_size = self.inner.borrow().pane.content_size();

        // accounts for the other scrollbar
        let inner_start_x = if let Some(ln_tb) = &*self.inner.borrow().line_number_tb.borrow() {
            ln_tb.pane.get_dyn_start_x()
        } else {
            self.inner.borrow().pane.get_dyn_start_x()
        };

        let sb = HorizontalScrollbar::new(init_ctx, DynVal::FULL, content_size, content_width)
            .without_keyboard_events();
        if self.x_scrollbar.borrow().is_some() {
            sb.pane.set_end_y(DynVal::FULL.minus_fixed(1));
        }
        match pos {
            HorizontalSBPositions::Above => {
                sb.set_at(inner_start_x, 0.into());
                self.inner.borrow().pane.set_start_y(1);
                if let Some(ln_tb) = &*self.inner.borrow().line_number_tb.borrow() {
                    ln_tb.pane.set_start_y(1);
                }
                if let Some(y_sb) = &*self.y_scrollbar.borrow() {
                    y_sb.pane.set_start_y(1);
                }
            }
            HorizontalSBPositions::Below => {
                sb.set_at(inner_start_x, DynVal::FULL.minus_fixed(1));
                self.inner
                    .borrow()
                    .pane
                    .set_end_y(DynVal::FULL.minus_fixed(1));
                if let Some(ln_tb) = &*self.inner.borrow().line_number_tb.borrow() {
                    ln_tb.pane.set_end_y(DynVal::FULL.minus_fixed(1));
                }
                if let Some(y_sb) = &*self.y_scrollbar.borrow() {
                    y_sb.pane.set_end_y(DynVal::FULL.minus_fixed(1));
                }
            }
            HorizontalSBPositions::None => {
                return;
            }
        }

        // wire the scrollbar to the textbox
        let pane_ = self.inner.borrow().pane.clone();
        let is_dirty = self.inner.borrow().is_dirty.clone();
        let hook = Box::new(move |init_ctx, x| {
            pane_.set_content_x_offset(&init_ctx, x);
            is_dirty.replace(true);
        });
        *sb.position_changed_hook.borrow_mut() = Some(hook);
        *self.x_scrollbar.borrow_mut() = Some(sb.clone());
        self.pane.pane.add_element(Box::new(sb.clone()));
        self.inner.borrow().x_scrollbar.replace(Some(sb));
        self.set_corner_decor(init_ctx);
        self.reset_sb_sizes(init_ctx);
    }

    pub fn reset_sb_sizes(&self, init_ctx: &Context) {
        let inner_ctx = init_ctx
            .must_get_parent_context()
            .child_context(&self.pane.get_dyn_location())
            .child_context(&self.inner.borrow().pane.get_dyn_location());
        if let Some(y_sb) = &*self.y_scrollbar.borrow() {
            y_sb.set_scrollable_view_size(inner_ctx.size);
            *y_sb.scrollable_view_chs.borrow_mut() =
                DynVal::new_fixed(inner_ctx.size.height as i32);
        }
        if let Some(x_sb) = &*self.x_scrollbar.borrow() {
            x_sb.set_scrollable_view_size(inner_ctx.size);
            *x_sb.scrollable_view_chs.borrow_mut() = DynVal::new_fixed(inner_ctx.size.width as i32);
        }

        // correct offsets
        let _ = self.inner.borrow().correct_ln_and_sbs(&inner_ctx);
        let _ = self.drawing(&init_ctx.child_context(&self.pane.get_dyn_location()), true); // to set the pane content
        self.set_dirty();
    }

    pub fn with_line_numbers(self, init_ctx: &Context) -> Self {
        self.set_line_numbers(init_ctx);
        self
    }

    pub fn reset_line_numbers(&self, init_ctx: &Context) {
        let ln_id = self
            .inner
            .borrow()
            .line_number_tb
            .borrow()
            .as_ref()
            .map(|ln_tb| ln_tb.pane.id());
        if let Some(ln_id) = ln_id {
            self.pane.pane.remove_element(&ln_id);
            self.set_line_numbers(init_ctx);
        }
    }

    pub fn set_line_numbers(&self, init_ctx: &Context) {
        let start_x = self.inner.borrow().pane.get_dyn_start_x();
        let start_y = self.inner.borrow().pane.get_dyn_start_y();
        let end_y = self.inner.borrow().pane.get_dyn_end_y();

        // determine the width of the line numbers textbox
        let (lns, lnw) = self.inner.borrow().get_line_numbers(init_ctx);

        // create the line numbers textbox
        let ln_tb = TextBoxInner::new(init_ctx, lns)
            .at(start_x.clone(), start_y)
            .with_width(DynVal::new_fixed(lnw as i32))
            .with_no_wordwrap()
            .non_editable()
            .non_navigable();

        ln_tb.pane.set_end_y(end_y);

        //*ln_tb.current_sty.borrow_mut() = self.pane.get_current_style();
        *ln_tb.current_sty.borrow_mut() = TextBoxInner::LINE_NUMBERS_STYLE;

        *ln_tb.selectedness.borrow_mut() = Selectability::Unselectable;
        self.pane.pane.add_element(Box::new(ln_tb.clone()));

        let new_inner_start_x = start_x.plus_fixed(lnw as i32);

        // reduce the width of the main textbox
        self.inner.borrow().pane.set_start_x(new_inner_start_x);
        *self.inner.borrow().line_number_tb.borrow_mut() = Some(ln_tb.clone());
        self.reset_sb_sizes(init_ctx);
    }

    pub fn set_corner_decor(&self, init_ctx: &Context) {
        // add corner decor
        if let (Some(x_sb), Some(y_sb)) = (&*self.x_scrollbar.borrow(), &*self.y_scrollbar.borrow())
        {
            let corner_decor = self.inner.borrow().corner_decor.borrow().clone();
            let cd = Label::new(init_ctx, &(corner_decor.ch.to_string()))
                .with_style(corner_decor.style.clone());

            let cd_y = x_sb.pane.get_dyn_start_y();
            let cd_x = y_sb.pane.get_dyn_start_x();
            let cd = cd.at(cd_x, cd_y);
            self.pane.pane.add_element(Box::new(cd));
        }
    }

    // TODO create this function eventually
    // not that important and annoying to calculate if the tb has a line numbers element
    //pub fn with_no_line_numbers(self) -> Self {
    //    *self.inner.borrow().line_numbered.borrow_mut() = false;
    //    self
    //}

    pub fn with_right_click_menu(self, rcm: Option<RightClickMenu>) -> Self {
        *self.inner.borrow().right_click_menu.borrow_mut() = rcm;
        self
    }

    pub fn with_text_when_empty<S: Into<String>>(self, text: S) -> Self {
        *self.inner.borrow().text_when_empty.borrow_mut() = text.into();
        self.set_dirty();
        self
    }

    pub fn set_text_when_empty(&self, text: String) {
        *self.inner.borrow().text_when_empty.borrow_mut() = text;
        self.set_dirty();
    }

    pub fn with_text_when_empty_fg(self, fg: Color) -> Self {
        *self.inner.borrow().text_when_empty_fg.borrow_mut() = fg;
        self.set_dirty();
        self
    }

    pub fn with_styles(self, styles: SelStyles) -> Self {
        let curr_sty = self.pane.get_current_style();
        if let Some(ln_tb) = &*self.inner.borrow().line_number_tb.borrow() {
            *ln_tb.current_sty.borrow_mut() = curr_sty;
        }
        *self.inner.borrow().current_sty.borrow_mut() = self.pane.get_current_style();
        self.pane.set_styles(styles);
        self.set_dirty();
        self
    }

    pub fn with_width(self, width: DynVal) -> Self {
        self.pane.set_dyn_width(width);
        self.set_dirty();
        self
    }
    pub fn with_height(self, height: DynVal) -> Self {
        self.pane.set_dyn_height(height);
        self.set_dirty();
        self
    }
    pub fn with_size(self, width: DynVal, height: DynVal) -> Self {
        self.pane.set_dyn_width(width);
        self.pane.set_dyn_height(height);
        self.set_dirty();
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self.set_dirty();
        self
    }

    pub fn set_at(&self, loc_x: DynVal, loc_y: DynVal) {
        self.pane.set_at(loc_x, loc_y);
        self.set_dirty();
    }

    pub fn with_ch_cursor(self) -> Self {
        *self.inner.borrow().ch_cursor.borrow_mut() = true;
        self.set_dirty();
        self
    }

    pub fn with_no_ch_cursor(self) -> Self {
        *self.inner.borrow().ch_cursor.borrow_mut() = false;
        self.set_dirty();
        self
    }

    pub fn editable(self, init_ctx: &Context) -> Self {
        self.inner.borrow().set_editable();
        self.reset_sb_sizes(init_ctx);
        self
    }

    pub fn non_editable(self, init_ctx: &Context) -> Self {
        self.inner.borrow().set_non_editable();
        self.reset_sb_sizes(init_ctx);
        self
    }

    pub fn with_wordwrap(self, init_ctx: &Context) -> Self {
        *self.inner.borrow().wordwrap.borrow_mut() = true;
        self.reset_sb_sizes(init_ctx);
        self
    }

    pub fn with_no_wordwrap(self, init_ctx: &Context) -> Self {
        *self.inner.borrow().wordwrap.borrow_mut() = false;
        self.reset_sb_sizes(init_ctx);
        self
    }

    pub fn with_position_style_hook(
        self, hook: Box<dyn FnMut(Context, usize, Style) -> Style>,
    ) -> Self {
        *self.inner.borrow().position_style_hook.borrow_mut() = Some(hook);
        self.set_dirty();
        self
    }

    pub fn set_position_style_hook(
        &mut self, hook: Box<dyn FnMut(Context, usize, Style) -> Style>,
    ) {
        *self.inner.borrow().position_style_hook.borrow_mut() = Some(hook);
    }

    pub fn with_cursor_changed_hook(self, hook: CursorChangedHook) -> Self {
        *self.inner.borrow().cursor_changed_hook.borrow_mut() = Some(hook);
        self
    }

    pub fn set_cursor_changed_hook(&mut self, hook: CursorChangedHook) {
        *self.inner.borrow().cursor_changed_hook.borrow_mut() = Some(hook);
    }

    pub fn with_text_changed_hook(
        self, hook: Box<dyn FnMut(Context, String) -> EventResponses>,
    ) -> Self {
        *self.inner.borrow().text_changed_hook.borrow_mut() = Some(hook);
        self
    }

    pub fn set_text_changed_hook(
        &mut self, hook: Box<dyn FnMut(Context, String) -> EventResponses>,
    ) {
        *self.inner.borrow().text_changed_hook.borrow_mut() = Some(hook);
    }

    pub fn with_cursor_style(self, style: Style) -> Self {
        *self.inner.borrow().cursor_style.borrow_mut() = style;
        self.set_dirty();
        self
    }

    pub fn with_corner_decor(self, decor: DrawCh) -> Self {
        *self.inner.borrow().corner_decor.borrow_mut() = decor;
        self.set_dirty();
        self
    }

    pub fn get_text(&self) -> String {
        self.inner.borrow().get_text()
    }

    pub fn set_text(&self, text: String) {
        self.inner.borrow().set_text(text);
        self.set_dirty();
    }

    pub fn set_cursor_pos(&self, pos: usize) {
        self.inner.borrow().set_cursor_pos(pos);
        self.set_dirty();
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for TextBox {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        if self.pane.get_selectability() == Selectability::Unselectable {
            return (false, EventResponses::default());
        }
        self.pane.receive_event(ctx, ev)
    }
}

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct TextBoxInner {
    pub pane: Pane,
    pub current_sty: Rc<RefCell<Style>>,
    pub selectedness: Rc<RefCell<Selectability>>,

    /// whether or not the content is dirty and needs updating
    pub is_dirty: Rc<RefCell<bool>>,

    /// used for redrawing logic
    pub last_size: Rc<RefCell<Size>>,

    /// the last size given to the sbs
    pub last_size_for_sbs: Rc<RefCell<Size>>,

    pub text: Rc<RefCell<Vec<char>>>,
    pub text_when_empty: Rc<RefCell<String>>,
    /// greyed out text when the textbox is empty
    pub text_when_empty_fg: Rc<RefCell<Color>>,
    pub ch_cursor: Rc<RefCell<bool>>,
    /// whether or not this tb has a ch cursor
    pub editable: Rc<RefCell<bool>>,
    /// whether or not this tb can be edited
    pub wordwrap: Rc<RefCell<bool>>,
    /// whether or not there are lefthand line numbers
    pub cursor_pos: Rc<RefCell<usize>>,
    /// cursor absolute position in the text
    pub cursor_style: Rc<RefCell<Style>>,
    pub visual_mode: Rc<RefCell<bool>>,
    /// whether or not the cursor is visual selecting
    pub mouse_dragging: Rc<RefCell<bool>>,
    /// if the mouse is currently dragging
    pub visual_mode_start_pos: Rc<RefCell<usize>>,
    /// the start position of the visual select
    pub text_changed_hook: Rc<RefCell<Option<Box<dyn FnMut(Context, String) -> EventResponses>>>>,

    /// When this hook is non-nil each characters style will be determineda via this hook.
    /// This is intended to be used if the caller of the textbox wishes granular control
    /// over the text styling.
    ///                                                              abs_pos, existing
    pub position_style_hook: Rc<RefCell<Option<Box<dyn FnMut(Context, usize, Style) -> Style>>>>,

    /// this hook is called each time the cursor moves
    pub cursor_changed_hook: Rc<RefCell<Option<CursorChangedHook>>>,

    pub x_scrollbar: Rc<RefCell<Option<HorizontalScrollbar>>>,
    pub y_scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
    pub line_number_tb: Rc<RefCell<Option<TextBoxInner>>>,

    /// for when there are two scrollbars
    pub corner_decor: Rc<RefCell<DrawCh>>,
    pub right_click_menu: Rc<RefCell<Option<RightClickMenu>>>,
}

/// The hook is called when the cursor position changes, the hook is passed the absolute position of the cursor.
pub type CursorChangedHook = Box<dyn FnMut(usize) -> EventResponses>;

impl TextBoxInner {
    const KIND: &'static str = "textbox_inner";

    const STYLE: SelStyles = SelStyles {
        selected_style: Style::new_const(Color::BLACK, Color::WHITE),
        ready_style: Style::new_const(Color::BLACK, Color::GREY17),
        unselectable_style: Style::new_const(Color::BLACK, Color::GREY15),
    };

    const LINE_NUMBERS_STYLE: Style = Style::new_const(Color::BLACK, Color::GREY13);

    const DEFAULT_CURSOR_STYLE: Style = Style::new_const(Color::WHITE, Color::BLUE);

    /// for textboxes which are editable
    pub fn editable_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KeyPossibility::Chars.into()),
            (KB::KEY_BACKSPACE.into()),
            (KB::KEY_ENTER.into()),
            (KB::KEY_SHIFT_ENTER.into()),
            (KB::KEY_LEFT.into()),
            (KB::KEY_RIGHT.into()),
            (KB::KEY_UP.into()),
            (KB::KEY_DOWN.into()),
            (KB::KEY_SHIFT_LEFT.into()),
            (KB::KEY_SHIFT_RIGHT.into()),
            (KB::KEY_SHIFT_UP.into()),
            (KB::KEY_SHIFT_DOWN.into()),
        ])
    }

    /// non-editable textboxes can still scroll
    pub fn non_editable_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KB::KEY_LEFT.into()),
            (KB::KEY_RIGHT.into()),
            (KB::KEY_UP.into()),
            (KB::KEY_DOWN.into()),
            (KB::KEY_H.into()),
            (KB::KEY_J.into()),
            (KB::KEY_K.into()),
            (KB::KEY_L.into()),
        ])
    }

    pub fn new<S: Into<String>>(ctx: &Context, text: S) -> Self {
        let text = text.into();
        let pane = Pane::new(ctx, Self::KIND)
            .with_dyn_width(DynVal::FULL)
            .with_dyn_height(DynVal::FULL)
            .with_self_receivable_events(Self::editable_receivable_events())
            .with_focused(true);

        let tb = TextBoxInner {
            pane,
            current_sty: Rc::new(RefCell::new(Style::default())),
            selectedness: Rc::new(RefCell::new(Selectability::Ready)),
            is_dirty: Rc::new(RefCell::new(true)),
            last_size: Rc::new(RefCell::new(Size::default())),
            last_size_for_sbs: Rc::new(RefCell::new(Size::default())),
            text: Rc::new(RefCell::new(text.chars().collect())),
            text_when_empty: Rc::new(RefCell::new("enter text...".to_string())),
            text_when_empty_fg: Rc::new(RefCell::new(Color::GREY6)),
            wordwrap: Rc::new(RefCell::new(true)),
            ch_cursor: Rc::new(RefCell::new(true)),
            editable: Rc::new(RefCell::new(true)),
            cursor_pos: Rc::new(RefCell::new(0)),
            cursor_style: Rc::new(RefCell::new(Self::DEFAULT_CURSOR_STYLE)),
            visual_mode: Rc::new(RefCell::new(false)),
            mouse_dragging: Rc::new(RefCell::new(false)),
            visual_mode_start_pos: Rc::new(RefCell::new(0)),
            text_changed_hook: Rc::new(RefCell::new(None)),
            position_style_hook: Rc::new(RefCell::new(None)),
            cursor_changed_hook: Rc::new(RefCell::new(None)),
            x_scrollbar: Rc::new(RefCell::new(None)),
            y_scrollbar: Rc::new(RefCell::new(None)),
            line_number_tb: Rc::new(RefCell::new(None)),
            corner_decor: Rc::new(RefCell::new(DrawCh::new('⁙', Style::default_const()))),
            right_click_menu: Rc::new(RefCell::new(None)),
        };

        let (tb1, tb2, tb3) = (tb.clone(), tb.clone(), tb.clone());
        let rcm = RightClickMenu::new(ctx, MenuStyle::default()).with_menu_items(
            ctx,
            vec![
                MenuItem::new(ctx, MenuPath("Cut".to_string())).with_click_fn(Some(Box::new(
                    move |ctx| {
                        tb1.is_dirty.replace(true);
                        tb1.cut_to_clipboard(&ctx)
                    },
                ))),
                MenuItem::new(ctx, MenuPath("Copy".to_string())).with_click_fn(Some(Box::new(
                    move |_ctx| {
                        tb2.is_dirty.replace(true);
                        tb2.copy_to_clipboard();
                        EventResponses::default()
                    },
                ))),
                MenuItem::new(ctx, MenuPath("Paste".to_string())).with_click_fn(Some(Box::new(
                    move |ctx| {
                        tb3.is_dirty.replace(true);
                        tb3.paste_from_clipboard(&ctx)
                    },
                ))),
            ],
        );
        *tb.right_click_menu.borrow_mut() = Some(rcm);

        tb
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    pub fn with_width(self, width: DynVal) -> Self {
        self.pane.set_dyn_width(width);
        self
    }

    pub fn with_height(self, height: DynVal) -> Self {
        self.pane.set_dyn_height(height);
        self
    }

    pub fn with_no_wordwrap(self) -> Self {
        *self.wordwrap.borrow_mut() = false;
        self
    }

    pub fn with_wordwrap(self) -> Self {
        *self.wordwrap.borrow_mut() = true;
        self
    }

    pub fn set_editable(&self) {
        *self.editable.borrow_mut() = true;
        self.pane
            .set_self_receivable_events(TextBoxInner::editable_receivable_events());
    }

    pub fn set_non_editable(&self) {
        *self.editable.borrow_mut() = false;
        self.pane
            .set_self_receivable_events(TextBoxInner::non_editable_receivable_events());
    }

    pub fn editable(self) -> Self {
        self.set_editable();
        self
    }

    pub fn non_editable(self) -> Self {
        self.set_non_editable();
        self
    }

    pub fn non_navigable(self) -> Self {
        self.pane
            .set_self_receivable_events(SelfReceivableEvents(vec![]));
        self
    }

    pub fn get_text(&self) -> String {
        self.text.borrow().iter().collect()
    }

    pub fn set_text(&self, text: String) {
        *self.text.borrow_mut() = text.chars().collect();
        self.is_dirty.replace(true);
    }

    // ---------------------------------------------------------

    pub fn get_cursor_pos(&self) -> usize {
        let cur_pos = *self.cursor_pos.borrow();
        // NOTE the cursor can be placed at the end of the text
        // hence the position is the length
        if cur_pos > self.text.borrow().len() {
            self.text.borrow().len()
        } else {
            cur_pos
        }
    }

    pub fn get_visual_mode_start_pos(&self) -> usize {
        let pos = *self.visual_mode_start_pos.borrow();
        if pos >= self.text.borrow().len() {
            self.text.borrow().len() - 1
        } else {
            pos
        }
    }

    pub fn set_cursor_pos(&self, new_abs_pos: usize) -> EventResponses {
        *self.cursor_pos.borrow_mut() = new_abs_pos;
        if let Some(hook) = &mut *self.cursor_changed_hook.borrow_mut() {
            hook(new_abs_pos)
        } else {
            EventResponses::default()
        }
    }

    pub fn incr_cursor_pos(&self, pos_change: isize) -> EventResponses {
        let new_pos = (self.get_cursor_pos() as isize + pos_change).max(0) as usize;
        self.set_cursor_pos(new_pos)
    }

    /// returns the wrapped characters of the text
    pub fn get_wrapped(&self, ctx: &Context) -> WrChs {
        let mut rs = self.text.borrow().clone();
        rs.push(' '); // add the space for the final possible position
        let mut chs = vec![];
        let mut max_x = 0;
        let (mut x, mut y) = (0, 0); // working x and y position in the textbox
        for (abs_pos, r) in rs.iter().enumerate() {
            if *self.wordwrap.borrow() && x == self.pane.get_width(ctx) {
                y += 1;
                x = 0;
                if x > max_x {
                    max_x = x;
                }
                chs.push(WrCh::new('\n', None, x, y));
            }

            if *r == '\n' {
                // If ch_cursor without wordwrap, add an extra space to the end of
                // the line so that the cursor can be placed there. Without this
                // extra space, placing the cursor at the end of the largest line
                // will panic.
                if *self.ch_cursor.borrow() && !*self.wordwrap.borrow() {
                    if x > max_x {
                        max_x = x;
                    }
                    chs.push(WrCh::new(' ', None, x, y));
                }

                // the "newline character" exists as an extra space at
                // the end of the line
                if x > max_x {
                    max_x = x;
                }
                chs.push(WrCh::new('\n', Some(abs_pos), x, y));

                // move the working position to the beginning of the next line
                y += 1;
                x = 0;
            } else {
                if x > max_x {
                    max_x = x;
                }
                chs.push(WrCh::new(*r, Some(abs_pos), x, y));
                x += 1;
            }
        }
        WrChs { chs, max_x }
    }

    /// returns the formatted line numbers of the textbox
    /// line numbers are right justified
    pub fn get_line_numbers(&self, ctx: &Context) -> (String, usize) {
        let wr_chs = self.get_wrapped(ctx);

        // get the max line number
        let mut max_line_num = 0;
        for (i, wr_ch) in wr_chs.chs.iter().enumerate() {
            if (wr_ch.ch == '\n' && wr_ch.abs_pos.is_some()) || i == 0 {
                max_line_num += 1;
            }
            //if wr_ch.y_pos + 1 > max_line_num {
            //    max_line_num = wr_ch.y_pos + 1;
            //}
        }

        // get the largest amount of digits in the line numbers from the string
        let line_num_width = max_line_num.to_string().chars().count();

        let mut s = String::new();
        let mut true_line_num = 1;

        for (i, wr_ch) in wr_chs.chs.iter().enumerate() {
            // NOTE this i=0 case needs to be seperate from the newline case
            // for when the first character is a newline we need print two line numbers
            if i == 0 {
                s += &format!("{:line_num_width$} ", true_line_num);
                true_line_num += 1;
                s += "\n";
            }
            if wr_ch.ch == '\n' {
                if wr_ch.abs_pos.is_some() {
                    s += &format!("{:line_num_width$} ", true_line_num);
                    true_line_num += 1;
                }
                s += "\n";
            }
        }

        (s, line_num_width + 1) // +1 for the extra space after the digits
    }

    /// NOTE the resp is sent in to potentially modify the offsets from numbers tb
    pub fn correct_offsets(&self, ctx: &Context, w: &WrChs) -> EventResponse {
        let (x, y) = w.cursor_x_and_y(self.get_cursor_pos());
        let (x, y) = (x.unwrap_or(0), y.unwrap_or(0));
        self.pane.correct_offsets_to_view_position(ctx, x, y);
        self.correct_ln_and_sbs(ctx)
    }

    /// correct the line number (if applicable) and scrollbar positions
    pub fn correct_ln_and_sbs(&self, ctx: &Context) -> EventResponse {
        let y_offset = self.pane.get_content_y_offset();
        let x_offset = self.pane.get_content_x_offset();

        let update_size = if *self.last_size_for_sbs.borrow() != ctx.size {
            *self.last_size_for_sbs.borrow_mut() = ctx.size;
            self.is_dirty.replace(true);
            true
        } else {
            false
        };

        // update the scrollbars/line numbers textbox
        if let Some(sb) = self.y_scrollbar.borrow().as_ref() {
            if update_size {
                sb.set_scrollable_view_size(ctx.size);
                *sb.scrollable_view_chs.borrow_mut() = DynVal::new_fixed(ctx.size.height as i32);
            }

            sb.external_change(y_offset, self.pane.content_height(), ctx.size);
        }
        let resp = EventResponse::default();
        if let Some(ln_tb) = self.line_number_tb.borrow().as_ref() {
            let (lns, lnw) = self.get_line_numbers(ctx);
            let last_lnw = ln_tb.pane.get_width(ctx);
            if lnw != last_lnw {
                let ln_start_x = ln_tb.pane.get_dyn_start_x();
                let tb_start_x = ln_start_x.plus_fixed(lnw as i32);
                self.pane.set_start_x(tb_start_x);
                ln_tb.pane.set_dyn_width(DynVal::new_fixed(lnw as i32))
            }
            ln_tb.set_text(lns);
            ln_tb.pane.set_content_y_offset(ctx, y_offset);
        }
        if let Some(sb) = self.x_scrollbar.borrow().as_ref() {
            if update_size {
                sb.set_scrollable_view_size(ctx.size);
                *sb.scrollable_view_chs.borrow_mut() = DynVal::new_fixed(ctx.size.width as i32);
            }
            sb.external_change(x_offset, self.pane.content_width(), ctx.size);
        }
        resp
    }

    /// get the start and end position of the visually selected text
    pub fn visual_selection_pos(&self) -> Option<(usize, usize)> {
        let mut cur_pos = self.get_cursor_pos();
        if *self.visual_mode.borrow() {
            let start_pos = self.get_visual_mode_start_pos();
            if cur_pos >= self.text.borrow().len() {
                cur_pos = self.text.borrow().len() - 1;
            }
            if start_pos < cur_pos {
                Some((start_pos, cur_pos))
            } else {
                Some((cur_pos, start_pos))
            }
        } else {
            if cur_pos >= self.text.borrow().len() {
                return None;
            }
            Some((cur_pos, cur_pos))
        }
    }

    pub fn visual_selected_text(&self) -> String {
        let text = self.text.borrow();
        let Some((start_pos, end_pos)) = self.visual_selection_pos() else {
            return String::new();
        };
        if !*self.visual_mode.borrow() {
            return text[start_pos].to_string();
        }
        text[start_pos..=end_pos].iter().collect()
    }

    pub fn delete_visual_selection(&self, ctx: &Context) -> EventResponses {
        if !*self.visual_mode.borrow() {
            return EventResponses::default();
        }

        // delete everything in the visual selection
        let mut rs = self.text.borrow().clone();

        let Some((start_pos, end_pos)) = self.visual_selection_pos() else {
            return EventResponses::default();
        };
        rs.drain(start_pos..=end_pos);
        self.set_cursor_pos(start_pos);

        *self.text.borrow_mut() = rs;
        *self.visual_mode.borrow_mut() = false;
        let w = self.get_wrapped(ctx);
        self.pane.set_content_from_string(w.wrapped_string());
        let resp = self.correct_offsets(ctx, &w);
        let mut resps = if let Some(hook) = &mut *self.text_changed_hook.borrow_mut() {
            hook(ctx.clone(), self.get_text())
        } else {
            EventResponses::default()
        };
        resps.push(resp);
        resps
    }

    pub fn copy_to_clipboard(&self) {
        let text = self.visual_selected_text();
        let Ok(mut cb) = arboard::Clipboard::new() else {
            log_err!("failed to get clipboard");
            return;
        };
        if let Err(e) = cb.set_text(text) {
            log_err!("failed to set text to clipboard: {}", e);
        }
    }

    pub fn cut_to_clipboard(&self, ctx: &Context) -> EventResponses {
        self.copy_to_clipboard();
        self.delete_visual_selection(ctx)
    }

    pub fn paste_from_clipboard(&self, ctx: &Context) -> EventResponses {
        let mut resps = self.delete_visual_selection(ctx);

        let Ok(mut cb) = arboard::Clipboard::new() else {
            log_err!("failed to get clipboard");
            return EventResponses::default();
        };
        let Ok(cliptext) = cb.get_text() else {
            log_err!("failed to get text from clipboard");
            return EventResponses::default();
        };
        if cliptext.is_empty() {
            return resps;
        }
        let cliprunes = cliptext.chars().collect::<Vec<char>>();
        let mut rs = self.text.borrow().clone();
        let cur_pos = self.get_cursor_pos();
        rs.splice(cur_pos..cur_pos, cliprunes.iter().cloned());
        *self.text.borrow_mut() = rs;

        self.incr_cursor_pos(cliprunes.len() as isize);
        let w = self.get_wrapped(ctx);
        self.pane.set_content_from_string(w.wrapped_string()); // See NOTE-1

        let resp = self.correct_offsets(ctx, &w);
        resps.push(resp);

        if let Some(hook) = &mut *self.text_changed_hook.borrow_mut() {
            resps.extend(hook(ctx.clone(), self.get_text()));
        }
        resps
    }

    pub fn receive_key_event(&self, ctx: &Context, ev: Vec<KeyEvent>) -> (bool, EventResponses) {
        if *self.selectedness.borrow() != Selectability::Selected || ev.is_empty() {
            return (false, EventResponses::default());
        }

        self.is_dirty.replace(true);

        if !*self.ch_cursor.borrow() {
            match true {
                _ if ev[0] == KB::KEY_LEFT || ev[0] == KB::KEY_H => {
                    self.pane.scroll_left(ctx);
                }
                _ if ev[0] == KB::KEY_RIGHT || ev[0] == KB::KEY_L => {
                    self.pane.scroll_right(ctx);
                }
                _ if ev[0] == KB::KEY_DOWN || ev[0] == KB::KEY_J => {
                    self.pane.scroll_down(ctx);
                }
                _ if ev[0] == KB::KEY_UP || ev[0] == KB::KEY_K => {
                    self.pane.scroll_up(ctx);
                }
                _ => {}
            }

            return (true, self.correct_ln_and_sbs(ctx).into());
        }

        let mut visual_mode_event = false;
        let visual_mode = *self.visual_mode.borrow();
        let cursor_pos = self.get_cursor_pos();

        let mut resps = EventResponses::default();
        match true {
            _ if ev[0] == KB::KEY_SHIFT_LEFT => {
                visual_mode_event = true;
                if !visual_mode {
                    *self.visual_mode.borrow_mut() = true;
                    *self.visual_mode_start_pos.borrow_mut() = cursor_pos;
                }
                if cursor_pos > 0 {
                    self.incr_cursor_pos(-1);
                    let w = self.get_wrapped(ctx);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if ev[0] == KB::KEY_SHIFT_RIGHT => {
                visual_mode_event = true;
                if !visual_mode {
                    *self.visual_mode.borrow_mut() = true;
                    *self.visual_mode_start_pos.borrow_mut() = cursor_pos;
                }
                if cursor_pos < self.text.borrow().len() {
                    self.incr_cursor_pos(1);
                    let w = self.get_wrapped(ctx);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if ev[0] == KB::KEY_SHIFT_UP => {
                visual_mode_event = true;
                if !visual_mode {
                    *self.visual_mode.borrow_mut() = true;
                    *self.visual_mode_start_pos.borrow_mut() = cursor_pos;
                }
                let w = self.get_wrapped(ctx);
                if let Some(new_pos) = w.get_cursor_above_position(cursor_pos) {
                    self.set_cursor_pos(new_pos);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if ev[0] == KB::KEY_SHIFT_DOWN => {
                visual_mode_event = true;
                if !visual_mode {
                    *self.visual_mode.borrow_mut() = true;
                    *self.visual_mode_start_pos.borrow_mut() = cursor_pos;
                }
                let w = self.get_wrapped(ctx);
                if let Some(new_pos) = w.get_cursor_below_position(cursor_pos) {
                    self.set_cursor_pos(new_pos);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if ev[0] == KB::KEY_LEFT || (!*self.editable.borrow() && ev[0] == KB::KEY_H) => {
                if cursor_pos > 0 && cursor_pos <= self.text.borrow().len() {
                    // do not move left if at the beginning of a line
                    if self.text.borrow()[cursor_pos - 1] != '\n' {
                        self.incr_cursor_pos(-1);
                        let w = self.get_wrapped(ctx);
                        resps = self.correct_offsets(ctx, &w).into();
                    }
                }
            }

            _ if ev[0] == KB::KEY_RIGHT || (!*self.editable.borrow() && ev[0] == KB::KEY_L) => {
                // don't allow moving to the next line
                if cursor_pos < self.text.borrow().len() && self.text.borrow()[cursor_pos] != '\n' {
                    self.incr_cursor_pos(1);
                    let w = self.get_wrapped(ctx);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if ev[0] == KB::KEY_UP || (!*self.editable.borrow() && ev[0] == KB::KEY_K) => {
                let w = self.get_wrapped(ctx);
                if let Some(new_pos) = w.get_cursor_above_position(cursor_pos) {
                    self.set_cursor_pos(new_pos);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if ev[0] == KB::KEY_DOWN || (!*self.editable.borrow() && ev[0] == KB::KEY_J) => {
                let w = self.get_wrapped(ctx);
                if let Some(new_pos) = w.get_cursor_below_position(cursor_pos) {
                    self.set_cursor_pos(new_pos);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if *self.editable.borrow() && KeyPossibility::Chars.matches_key(&ev[0]) => {
                if let crossterm::event::KeyCode::Char(r) = ev[0].code {
                    let mut rs = self.text.borrow().clone();
                    rs.insert(cursor_pos, r);
                    *self.text.borrow_mut() = rs;
                    self.incr_cursor_pos(1);
                    let w = self.get_wrapped(ctx);

                    // NOTE-1: must call SetContentFromString to update the content
                    // before updating the offset or else the offset amount may not
                    // exist in the content and the widget pane will reject the new
                    // offset
                    self.pane.set_content_from_string(w.wrapped_string());
                    let resp = self.correct_offsets(ctx, &w);

                    if let Some(hook) = &mut *self.text_changed_hook.borrow_mut() {
                        resps = hook(ctx.clone(), self.get_text());
                    }
                    resps.push(resp);
                }
            }

            _ if *self.editable.borrow() && (ev[0] == KB::KEY_BACKSPACE) => {
                if visual_mode {
                    resps = self.delete_visual_selection(ctx);
                } else if cursor_pos > 0 {
                    let mut rs = self.text.borrow().clone();
                    rs.remove(cursor_pos - 1);
                    self.incr_cursor_pos(-1);
                    *self.text.borrow_mut() = rs;
                    let w = self.get_wrapped(ctx);
                    self.pane.set_content_from_string(w.wrapped_string()); // See NOTE-1
                    let resp = self.correct_offsets(ctx, &w);
                    if let Some(hook) = &mut *self.text_changed_hook.borrow_mut() {
                        resps = hook(ctx.clone(), self.get_text());
                    }
                    resps.push(resp);
                }
            }

            _ if *self.editable.borrow() && ev[0] == KB::KEY_ENTER => {
                let mut rs = self.text.borrow().clone();
                rs.splice(cursor_pos..cursor_pos, std::iter::once('\n'));
                *self.text.borrow_mut() = rs;
                self.incr_cursor_pos(1);
                let w = self.get_wrapped(ctx);
                self.pane.set_content_from_string(w.wrapped_string()); // See NOTE-1
                let resp = self.correct_offsets(ctx, &w);
                if let Some(hook) = &mut *self.text_changed_hook.borrow_mut() {
                    resps = hook(ctx.clone(), self.get_text());
                }
                resps.push(resp);
            }

            _ => {}
        }

        if *self.visual_mode.borrow() && !visual_mode_event {
            *self.visual_mode.borrow_mut() = false;
        }

        (true, resps)
    }

    pub fn receive_mouse_event(&self, ctx: &Context, ev: MouseEvent) -> (bool, EventResponses) {
        // handle right click
        if let Some(rcm) = &*self.right_click_menu.borrow() {
            if let Some(resps) = rcm.create_menu_if_right_click(ev) {
                return (true, resps);
            }
        }

        if !matches!(ev.kind, MouseEventKind::Moved) {
            self.is_dirty.replace(true);
        }

        let selectedness = *self.selectedness.borrow();
        let cursor_pos = self.get_cursor_pos();
        match ev.kind {
            MouseEventKind::ScrollDown
                if ev.modifiers == KeyModifiers::NONE
                    && selectedness == Selectability::Selected =>
            {
                let w = self.get_wrapped(ctx);
                let Some(new_pos) = w.get_cursor_below_position(cursor_pos) else {
                    return (true, EventResponses::default());
                };
                self.set_cursor_pos(new_pos);
                let resp = self.correct_offsets(ctx, &w);
                return (true, resp.into());
            }
            MouseEventKind::ScrollUp
                if ev.modifiers == KeyModifiers::NONE
                    && selectedness == Selectability::Selected =>
            {
                let w = self.get_wrapped(ctx);
                let Some(new_pos) = w.get_cursor_above_position(cursor_pos) else {
                    return (true, EventResponses::default());
                };
                self.set_cursor_pos(new_pos);
                let resp = self.correct_offsets(ctx, &w);
                return (true, resp.into());
            }
            MouseEventKind::ScrollLeft
                if ev.modifiers == KeyModifiers::NONE
                    && selectedness == Selectability::Selected =>
            {
                let w = self.get_wrapped(ctx);
                let Some(new_pos) = w.get_cursor_left_position(cursor_pos) else {
                    return (true, EventResponses::default());
                };
                self.set_cursor_pos(new_pos);
                let resp = self.correct_offsets(ctx, &w);
                return (true, resp.into());
            }
            MouseEventKind::ScrollDown
                if ev.modifiers == KeyModifiers::SHIFT
                    && selectedness == Selectability::Selected =>
            {
                let w = self.get_wrapped(ctx);
                let Some(new_pos) = w.get_cursor_left_position(cursor_pos) else {
                    return (true, EventResponses::default());
                };
                self.set_cursor_pos(new_pos);
                let resp = self.correct_offsets(ctx, &w);
                return (true, resp.into());
            }
            MouseEventKind::ScrollRight
                if ev.modifiers == KeyModifiers::NONE
                    && selectedness == Selectability::Selected =>
            {
                let w = self.get_wrapped(ctx);
                let Some(new_pos) = w.get_cursor_right_position(cursor_pos) else {
                    return (true, EventResponses::default());
                };
                self.set_cursor_pos(new_pos);
                let resp = self.correct_offsets(ctx, &w);
                return (true, resp.into());
            }
            MouseEventKind::ScrollUp
                if ev.modifiers == KeyModifiers::SHIFT
                    && selectedness == Selectability::Selected =>
            {
                let w = self.get_wrapped(ctx);
                let Some(new_pos) = w.get_cursor_right_position(cursor_pos) else {
                    return (true, EventResponses::default());
                };
                self.set_cursor_pos(new_pos);
                let resp = self.correct_offsets(ctx, &w);
                return (true, resp.into());
            }
            MouseEventKind::ScrollDown
                if ev.modifiers == KeyModifiers::NONE && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_down(ctx);
                return (true, self.correct_ln_and_sbs(ctx).into());
            }
            MouseEventKind::ScrollUp
                if ev.modifiers == KeyModifiers::NONE && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_up(ctx);
                return (true, self.correct_ln_and_sbs(ctx).into());
            }
            MouseEventKind::ScrollLeft
                if ev.modifiers == KeyModifiers::NONE && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_left(ctx);
                return (true, self.correct_ln_and_sbs(ctx).into());
            }
            MouseEventKind::ScrollDown
                if ev.modifiers == KeyModifiers::SHIFT && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_left(ctx);
                return (true, self.correct_ln_and_sbs(ctx).into());
            }
            MouseEventKind::ScrollRight
                if ev.modifiers == KeyModifiers::NONE && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_right(ctx);
                return (true, self.correct_ln_and_sbs(ctx).into());
            }
            MouseEventKind::ScrollUp
                if ev.modifiers == KeyModifiers::SHIFT && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_right(ctx);
                return (true, self.correct_ln_and_sbs(ctx).into());
            }

            MouseEventKind::Moved | MouseEventKind::Up(_) => {
                *self.mouse_dragging.borrow_mut() = false;

                // we need to capture the click, needed for selectability mechanism
                return (true, EventResponses::default());
            }

            // set the cursor to the mouse position on primary click
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left)
                if selectedness == Selectability::Ready =>
            {
                let w = self.get_wrapped(ctx);
                let resp = self.correct_offsets(ctx, &w);
                return (true, resp.into());
            }

            // set the cursor to the mouse position on primary click
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left)
                if selectedness == Selectability::Selected =>
            {
                let x = ev.column as usize + self.pane.get_content_x_offset();
                let y = ev.row as usize + self.pane.get_content_y_offset();
                let w = self.get_wrapped(ctx);

                let mouse_dragging = *self.mouse_dragging.borrow();
                let visual_mode = *self.visual_mode.borrow();
                if mouse_dragging && !visual_mode {
                    *self.visual_mode.borrow_mut() = true;
                    *self.visual_mode_start_pos.borrow_mut() = cursor_pos;
                }
                if !mouse_dragging && visual_mode {
                    *self.visual_mode.borrow_mut() = false;
                }
                *self.mouse_dragging.borrow_mut() = true;
                if let Some(new_pos) = w.get_nearest_valid_cursor_from_position(x, y) {
                    self.set_cursor_pos(new_pos);
                    let resp = self.correct_offsets(ctx, &w);
                    return (true, resp.into());
                }
                return (true, EventResponses::default());
            }
            _ => {}
        }

        (false, EventResponses::default())
    }

    /// updates the content of the textbox
    pub fn update_content(&self, ctx: &Context) {
        let w = self.get_wrapped(ctx);
        let wrapped = w.wrapped_string();
        self.correct_ln_and_sbs(ctx);

        let curr_sty = self.current_sty.borrow().clone();
        let mut sty = curr_sty.clone();
        if wrapped.len() == 1
            && *self.ch_cursor.borrow()
            && *self.selectedness.borrow() != Selectability::Selected
        {
            // set greyed out text
            let text = self.text_when_empty.borrow();
            sty.set_fg(self.text_when_empty_fg.borrow().clone());
            self.pane
                .set_content_from_string_with_style(ctx, &text, sty);
            return;
        } else {
            self.pane
                .set_content_from_string_with_style(ctx, &wrapped, sty);
        }

        // set styles from hooks if applicable
        if let Some(hook) = &mut *self.position_style_hook.borrow_mut() {
            for wr_ch in w.chs.iter() {
                let existing_sty = curr_sty.clone();
                if wr_ch.abs_pos.is_none() {
                    continue;
                }
                if let Some(abs_pos) = wr_ch.abs_pos {
                    let sty = hook(ctx.clone(), abs_pos, existing_sty);
                    self.pane
                        .get_content_mut()
                        .change_style_at_xy(wr_ch.x_pos, wr_ch.y_pos, sty);
                }
            }
        }

        // set cursor style
        if *self.selectedness.borrow() == Selectability::Selected && *self.ch_cursor.borrow() {
            let (cur_x, cur_y) = w.cursor_x_and_y(self.get_cursor_pos());
            if let (Some(cur_x), Some(cur_y)) = (cur_x, cur_y) {
                self.pane.get_content_mut().change_style_at_xy(
                    cur_x,
                    cur_y,
                    self.cursor_style.borrow().clone(),
                );
            }
        }
        if *self.visual_mode.borrow() {
            let start_pos = self.get_visual_mode_start_pos();
            let cur_pos = self.get_cursor_pos();

            let start = if start_pos < cur_pos { start_pos } else { cur_pos };
            let end = if start_pos < cur_pos { cur_pos } else { start_pos };
            for i in start..=end {
                if let (Some(cur_x), Some(cur_y)) = w.cursor_x_and_y(i) {
                    self.pane.get_content_mut().change_style_at_xy(
                        cur_x,
                        cur_y,
                        self.cursor_style.borrow().clone(),
                    );
                }
            }
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for TextBoxInner {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::KeyCombo(ke) => self.receive_key_event(ctx, ke),
            Event::Mouse(me) => self.receive_mouse_event(ctx, me),
            _ => (false, EventResponses::default()),
        }
    }

    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        if self.is_dirty.replace(false) || *self.last_size.borrow() != ctx.size || force_update {
            self.update_content(ctx);
            self.last_size.replace(ctx.size);
        }
        self.pane.drawing(ctx, force_update)
    }
}

/// wrapped character
#[derive(Clone, Default)]
pub struct WrCh {
    /// the character
    ch: char,

    /// absolute position in the text
    /// If this character is a NOT a part of the text and only introduced
    /// due to line wrapping, the absPos will be None (and ch='\n')
    abs_pos: Option<usize>,

    /// x position in the line
    x_pos: usize,
    /// y position of the line
    y_pos: usize,
}

impl WrCh {
    pub fn new(ch: char, abs_pos: Option<usize>, x_pos: usize, y_pos: usize) -> Self {
        WrCh {
            ch,
            abs_pos,
            x_pos,
            y_pos,
        }
    }
}

/// wrapped characters
#[derive(Clone, Default)]
pub struct WrChs {
    chs: Vec<WrCh>,
    /// the maximum x position within the wrapped characters
    max_x: usize,
}

impl WrChs {
    pub fn wrapped_string(&self) -> String {
        self.chs.iter().map(|wr_ch| wr_ch.ch).collect()
    }

    /// gets the cursor x and y position in the wrapped text
    /// from the absolute cursor position provided.
    pub fn cursor_x_and_y(&self, cur_abs: usize) -> (Option<usize>, Option<usize>) {
        self.chs
            .iter()
            .find(|wr_ch| wr_ch.abs_pos == Some(cur_abs))
            .map(|wr_ch| (Some(wr_ch.x_pos), Some(wr_ch.y_pos)))
            .unwrap_or_default()
    }

    /// gets the line at the given y position
    pub fn get_line(&self, y: usize) -> Vec<char> {
        self.chs
            .iter()
            .filter(|wr_ch| wr_ch.y_pos == y)
            .map(|wr_ch| wr_ch.ch)
            .collect()
    }

    /// maximum y position in the wrapped text
    pub fn max_y(&self) -> usize {
        self.chs.last().cloned().unwrap_or_default().y_pos
    }

    pub fn max_x(&self) -> usize {
        self.max_x
    }

    /// determine the cursor position above the current cursor position
    ///                                                         cur_abs
    pub fn get_cursor_above_position(&self, cur_abs: usize) -> Option<usize> {
        let (cur_i, cur_x, cur_y) = self
            .chs
            .iter()
            .enumerate()
            .find(|(_, wr_ch)| wr_ch.abs_pos == Some(cur_abs))
            .map(|(i, wr_ch)| (i, wr_ch.x_pos, wr_ch.y_pos))?;

        if cur_y == 0 {
            return None;
        }
        self.chs
            .iter()
            .take(cur_i)
            .rev()
            .find(|wr_ch| wr_ch.y_pos == cur_y - 1 && wr_ch.x_pos <= cur_x)
            .map(|wr_ch| wr_ch.abs_pos)
            .unwrap_or(None)
    }

    /// determine the cursor position below the current cursor position.
    ///                                                         cur_abs
    pub fn get_cursor_below_position(&self, cur_abs: usize) -> Option<usize> {
        let (cur_i, cur_x, cur_y) = self
            .chs
            .iter()
            .enumerate()
            .find(|(_, wr_ch)| wr_ch.abs_pos == Some(cur_abs))
            .map(|(i, wr_ch)| (i, wr_ch.x_pos, wr_ch.y_pos))?;

        if cur_y == self.max_y() {
            return None;
        }

        // move forwards in the wrapped text until we find the character with a y position one
        // greater than the current cursor position and with the maximum x position less than or
        // equal to the current cursor x position.
        self.chs
            .iter()
            .skip(cur_i)
            .take_while(|wr_ch| wr_ch.y_pos <= cur_y + 1) // optimization 
            .filter(|wr_ch| wr_ch.y_pos == cur_y + 1 && wr_ch.x_pos <= cur_x)
            .last()
            .map(|wr_ch| wr_ch.abs_pos)
            .unwrap_or(None)
    }

    //                                                        cur_abs
    pub fn get_cursor_left_position(&self, cur_abs: usize) -> Option<usize> {
        let (cur_i, cur_x, cur_y) = self
            .chs
            .iter()
            .enumerate()
            .find(|(_, wr_ch)| wr_ch.abs_pos == Some(cur_abs))
            .map(|(i, wr_ch)| (i, wr_ch.x_pos, wr_ch.y_pos))?;

        if cur_x == 0 {
            return None;
        }
        self.chs
            .iter()
            .take(cur_i)
            .rev()
            .find(|wr_ch| wr_ch.y_pos == cur_y && wr_ch.x_pos < cur_x)
            .map(|wr_ch| wr_ch.abs_pos)
            .unwrap_or(None)
    }

    pub fn get_cursor_right_position(&self, cur_abs: usize) -> Option<usize> {
        let (cur_i, cur_x, cur_y) = self
            .chs
            .iter()
            .enumerate()
            .find(|(_, wr_ch)| wr_ch.abs_pos == Some(cur_abs))
            .map(|(i, wr_ch)| (i, wr_ch.x_pos, wr_ch.y_pos))?;

        if cur_x >= self.max_x {
            return None;
        }
        self.chs
            .iter()
            .skip(cur_i)
            .find(|wr_ch| wr_ch.y_pos == cur_y && wr_ch.x_pos > cur_x)
            .map(|wr_ch| wr_ch.abs_pos)
            .unwrap_or(None)
    }

    pub fn get_nearest_valid_cursor_from_position(&self, x: usize, y: usize) -> Option<usize> {
        let mut nearest_abs = None; // nearest absolute position with the same y position
        let mut nearest_abs_y_pos = None; // Y position of the nearest absolute position
        let mut nearest_abs_x_pos = None; // X position of the nearest absolute position
        for wr_ch in &self.chs {
            if wr_ch.abs_pos.is_none() {
                continue;
            }
            if wr_ch.y_pos == y && wr_ch.x_pos == x {
                return wr_ch.abs_pos;
            }

            let y_diff = (wr_ch.y_pos as isize - y as isize).abs();
            let nearest_y_diff = match nearest_abs_y_pos {
                Some(nearest_abs_y_pos) => (nearest_abs_y_pos as isize - y as isize).abs(),
                None => y_diff,
            };

            let x_diff = (wr_ch.x_pos as isize - x as isize).abs();
            let nearest_x_diff = match nearest_abs_x_pos {
                Some(nearest_abs_x_pos) => (nearest_abs_x_pos as isize - x as isize).abs(),
                None => x_diff,
            };

            if y_diff < nearest_y_diff || (y_diff == nearest_y_diff && x_diff < nearest_x_diff) {
                nearest_abs_y_pos = Some(wr_ch.y_pos);
                nearest_abs_x_pos = Some(wr_ch.x_pos);
                nearest_abs = wr_ch.abs_pos;
            }
        }
        nearest_abs
    }
}
