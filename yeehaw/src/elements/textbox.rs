use {
    super::{
        HorizontalSBPositions, HorizontalScrollbar, Selectability, VerticalSBPositions,
        VerticalScrollbar,
    },
    crate::{
        elements::menu::{MenuItem, MenuPath, MenuStyle},
        Keyboard as KB, *,
    },
    crossterm::event::{KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

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
    pub fn new<S: Into<String>>(ctx: &Context, text: S) -> Self {
        let text = text.into();
        let s = Size::get_text_size(&text);
        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_dyn_width(DynVal::new_fixed(s.width as i32))
            .with_dyn_height(DynVal::new_fixed(s.height as i32))
            .with_styles(TextBoxInner::STYLE);
        let inner = TextBoxInner::new(ctx, text);
        pane.pane.add_element(Box::new(inner.clone()));
        let tb = TextBox {
            pane,
            inner: Rc::new(RefCell::new(inner)),
            x_scrollbar: Rc::new(RefCell::new(None)),
            y_scrollbar: Rc::new(RefCell::new(None)),
            line_number_tb: Rc::new(RefCell::new(None)),
        };

        let tb_ = tb.clone();
        tb.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                let sel = tb_.pane.get_selectability();
                *tb_.inner.borrow().selectedness.borrow_mut() = sel;
                *tb_.inner.borrow().current_sty.borrow_mut() = tb_.pane.get_current_style();
                if sel != Selectability::Selected {
                    *tb_.inner.borrow().visual_mode.borrow_mut() = false;
                }
            }));
        tb
    }

    // XXX integrate
    /*
    pub fn to_widgets(mut self, ctx: &Context) -> crate::widgets::Widget {
        let (x, y) = (self.pane.get_dyn_start_x(), self.pane.get_dyn_start_y());
        let (h, w) = (self.pane.get_dyn_height(), self.pane.get_dyn_width());
        let mut out: Vec<Box<dyn crate::widgets::Widget>> = vec![];

        let ln_tb = if *self.line_numbered.borrow() {
            // determine the width of the line numbers textbox

            // create the line numbers textbox
            let (lns, lnw) = self.get_line_numbers(ctx);
            let ln_tb = TextBoxInner::new(ctx, lns)
                .at(x.clone(), y.clone())
                .with_width(DynVal::new_fixed(lnw as i32))
                .with_height(h.clone())
                .with_no_wordwrap()
                .non_editable();
            ln_tb.set_selectability(ctx, Selectability::Unselectable);
            //.with_selectability(Selectability::Unselectable);
            out.push(Box::new(ln_tb.clone()));

            // reduce the width of the main textbox
            self.pane.set_dyn_start_x(x.clone().plus_fixed(lnw as i32));
            self.pane.set_dyn_width(w.clone().minus_fixed(lnw as i32));

            self.line_number_tb = Rc::new(RefCell::new(Some(ln_tb.clone())));
            Some(ln_tb)
        } else {
            None
        };
        out.push(Box::new(self.clone()));

        // add corner decor
        let y_sb_op = *self.y_scrollbar_op.borrow();
        let x_sb_op = *self.x_scrollbar_op.borrow();
        let no_y_sb = matches!(y_sb_op, VerticalSBPositions::None);
        let no_x_sb = matches!(x_sb_op, HorizontalSBPositions::None);
        if !no_y_sb && !no_x_sb {
            let cd = Label::new(ctx, &(self.corner_decor.borrow().ch.to_string()))
                .with_style(ctx, self.corner_decor.borrow().style.clone());
            let (cd_x, cd_y) = match (y_sb_op, x_sb_op) {
                (VerticalSBPositions::ToTheLeft, HorizontalSBPositions::Above) => {
                    (x.clone().minus_fixed(1), y.clone().minus_fixed(1))
                }
                (VerticalSBPositions::ToTheLeft, HorizontalSBPositions::Below) => {
                    (x.clone().minus_fixed(1), y.clone().plus(h.clone()))
                }
                (VerticalSBPositions::ToTheRight, HorizontalSBPositions::Above) => {
                    (x.clone().plus(w.clone()), y.clone().minus_fixed(1))
                }
                (VerticalSBPositions::ToTheRight, HorizontalSBPositions::Below) => {
                    (x.clone().plus(w.clone()), y.clone().plus(h.clone()))
                }
                _ => panic!("impossible"),
            };
            let cd = cd.at(cd_x, cd_y);
            out.push(Box::new(cd));
        }

        if !no_y_sb {
            let x2 = match y_sb_op {
                VerticalSBPositions::ToTheLeft => x.clone().minus_fixed(1),
                VerticalSBPositions::ToTheRight => x.clone().plus(w.clone()),
                _ => panic!("impossible"),
            };
            let vsb = VerticalScrollbar::new(ctx, h.clone(), self.pane.content_height())
                .at(x2, y.clone())
                .with_styles(Self::STYLE_SCROLLBAR);

            match ln_tb {
                Some(ln_tb) => {
                    let pane_ = self.pane.clone();
                    let ln_tb_wb = ln_tb.pane.clone();
                    let hook = Box::new(move |ctx, y| {
                        pane_.set_content_y_offset(&ctx, y);
                        ln_tb_wb.set_content_y_offset(&ctx, y);
                    });
                    *vsb.position_changed_hook.borrow_mut() = Some(hook);
                }
                None => {
                    let pane_ = self.pane.clone();
                    let hook = Box::new(move |ctx, y| pane_.set_content_y_offset(&ctx, y));
                    *vsb.position_changed_hook.borrow_mut() = Some(hook);
                }
            }

            *self.y_scrollbar.borrow_mut() = Some(vsb.clone());
            out.push(Box::new(vsb));
        }

        if !no_x_sb {
            let y2 = match x_sb_op {
                HorizontalSBPositions::Above => y.clone().minus_fixed(1),
                HorizontalSBPositions::Below => y.clone().plus(h),
                _ => panic!("impossible"),
            };

            let hsb = HorizontalScrollbar::new(ctx, w, self.x_new_domain_chs())
                .at(x, y2)
                .with_styles(Self::STYLE_SCROLLBAR);

            let pane_ = self.pane.clone();
            let hook = Box::new(move |ctx, x| pane_.set_content_x_offset(&ctx, x));
            *hsb.position_changed_hook.borrow_mut() = Some(hook);
            *self.x_scrollbar.borrow_mut() = Some(hsb.clone());

            out.push(Box::new(hsb));
        }

        let _ = self.drawing(ctx); // to set the pane content
        crate::widgets::Widget(out)
    }
    */

    pub fn with_scrollbars(self, ctx: &Context) -> Self {
        self.set_x_scrollbar_inner(ctx, HorizontalSBPositions::Below);
        self.set_y_scrollbar_inner(ctx, VerticalSBPositions::ToTheRight);
        self
    }

    pub fn with_left_scrollbar(self, ctx: &Context) -> Self {
        self.set_y_scrollbar_inner(ctx, VerticalSBPositions::ToTheLeft);
        self
    }

    pub fn with_right_scrollbar(self, ctx: &Context) -> Self {
        self.set_y_scrollbar_inner(ctx, VerticalSBPositions::ToTheRight);
        self
    }

    fn set_y_scrollbar_inner(&self, ctx: &Context, pos: VerticalSBPositions) {
        let height = DynVal::full();
        let content_height = self.inner.borrow().pane.content_height();

        // accounts for the other scrollbar
        let inner_start_y = self.inner.borrow().pane.get_dyn_start_y();

        let sb = VerticalScrollbar::new(ctx, height, content_height).without_keyboard_events();
        match pos {
            VerticalSBPositions::ToTheLeft => {
                sb.set_at(1.into(), inner_start_y);
                self.inner.borrow().pane.set_start_x(1.into());
            }
            VerticalSBPositions::ToTheRight => {
                sb.set_at(DynVal::full().minus_fixed(1), inner_start_y);
                self.inner
                    .borrow()
                    .pane
                    .set_end_x(DynVal::full().minus_fixed(1));
            }
            VerticalSBPositions::None => {
                return;
            }
        }

        // wire the scrollbar to the listbox
        let pane_ = self.inner.borrow().pane.clone();
        let hook = Box::new(move |ctx, y| pane_.set_content_y_offset(&ctx, y));
        *sb.position_changed_hook.borrow_mut() = Some(hook);
        *self.y_scrollbar.borrow_mut() = Some(sb.clone());
        self.pane.pane.add_element(Box::new(sb.clone()));
        self.inner.borrow().y_scrollbar.replace(Some(sb));
    }

    pub fn with_top_scrollbar(self, ctx: &Context) -> Self {
        self.set_x_scrollbar_inner(ctx, HorizontalSBPositions::Above);
        self
    }

    pub fn with_lower_scrollbar(self, ctx: &Context) -> Self {
        self.set_x_scrollbar_inner(ctx, HorizontalSBPositions::Below);
        self
    }

    fn set_x_scrollbar_inner(&self, ctx: &Context, pos: HorizontalSBPositions) {
        let width = DynVal::full();
        let content_width = self.inner.borrow().pane.content_height();

        // accounts for the other scrollbar
        let mut inner_start_x = self.inner.borrow().pane.get_dyn_start_x();
        if let Some(ln_tb) = &*self.inner.borrow().line_number_tb.borrow() {
            inner_start_x = ln_tb.pane.get_dyn_start_x();
        }

        let sb = HorizontalScrollbar::new(ctx, width, content_width).without_keyboard_events();
        match pos {
            HorizontalSBPositions::Above => {
                sb.set_at(inner_start_x, 0.into());
                self.inner.borrow().pane.set_start_y(1.into());
                if let Some(ln_tb) = &*self.inner.borrow().line_number_tb.borrow() {
                    ln_tb.pane.set_start_y(1.into());
                }
            }
            HorizontalSBPositions::Below => {
                sb.set_at(inner_start_x, DynVal::full().minus_fixed(1));
                self.inner
                    .borrow()
                    .pane
                    .set_end_y(DynVal::full().minus_fixed(1));
                if let Some(ln_tb) = &*self.inner.borrow().line_number_tb.borrow() {
                    ln_tb.pane.set_end_y(DynVal::full().minus_fixed(1));
                }
            }
            HorizontalSBPositions::None => {
                return;
            }
        }

        // wire the scrollbar to the listbox
        let pane_ = self.inner.borrow().pane.clone();
        let hook = Box::new(move |ctx, x| pane_.set_content_x_offset(&ctx, x));
        *sb.position_changed_hook.borrow_mut() = Some(hook);
        *self.x_scrollbar.borrow_mut() = Some(sb.clone());
        self.pane.pane.add_element(Box::new(sb.clone()));
        self.inner.borrow().x_scrollbar.replace(Some(sb));
    }

    pub fn with_right_click_menu(self, rcm: Option<RightClickMenu>) -> Self {
        *self.inner.borrow().right_click_menu.borrow_mut() = rcm;
        self
    }

    pub fn with_text_when_empty<S: Into<String>>(self, text: S) -> Self {
        *self.inner.borrow().text_when_empty.borrow_mut() = text.into();
        self
    }

    pub fn with_text_when_empty_fg(self, fg: Color) -> Self {
        *self.inner.borrow().text_when_empty_fg.borrow_mut() = fg;
        self
    }

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
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
    pub fn with_size(self, width: DynVal, height: DynVal) -> Self {
        self.pane.set_dyn_width(width);
        self.pane.set_dyn_height(height);
        self
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.pane.set_at(loc_x, loc_y);
        self
    }

    pub fn set_at(&self, loc_x: DynVal, loc_y: DynVal) {
        self.pane.set_at(loc_x, loc_y);
    }

    pub fn with_ch_cursor(self) -> Self {
        *self.inner.borrow().ch_cursor.borrow_mut() = true;
        self
    }

    pub fn with_no_ch_cursor(self) -> Self {
        *self.inner.borrow().ch_cursor.borrow_mut() = false;
        self
    }

    pub fn editable(self) -> Self {
        self.inner.borrow().set_editable();
        self
    }

    pub fn non_editable(self) -> Self {
        self.inner.borrow().set_non_editable();
        self
    }

    pub fn with_wordwrap(self) -> Self {
        *self.inner.borrow().wordwrap.borrow_mut() = true;
        self
    }

    pub fn with_no_wordwrap(self) -> Self {
        *self.inner.borrow().wordwrap.borrow_mut() = false;
        self
    }

    pub fn with_line_numbers(self, ctx: &Context) -> Self {
        let start_x = self.inner.borrow().pane.get_dyn_start_x();
        let start_y = self.inner.borrow().pane.get_dyn_start_y();

        // determine the width of the line numbers textbox
        let (lns, lnw) = self.inner.borrow().get_line_numbers(ctx);

        // create the line numbers textbox
        let ln_tb = TextBoxInner::new(ctx, lns)
            .at(start_x.clone(), start_y)
            .with_width(DynVal::new_fixed(lnw as i32))
            .with_height(DynVal::full())
            .with_no_wordwrap()
            .non_editable();
        //ln_tb.pane.set_selectability(Selectability::Unselectable);
        self.pane.pane.add_element(Box::new(ln_tb.clone()));

        let new_inner_start_x = start_x.plus_fixed(lnw as i32);

        // reduce the width of the main textbox
        self.inner.borrow().pane.set_start_x(new_inner_start_x);
        *self.inner.borrow().line_number_tb.borrow_mut() = Some(ln_tb.clone());
        self
    }

    // TODO not that important and annoying to calculate if the tb has t
    // a line numbers element
    //pub fn with_no_line_numbers(self) -> Self {
    //    *self.inner.borrow().line_numbered.borrow_mut() = false;
    //    self
    //}

    pub fn with_position_style_hook(
        self, hook: Box<dyn FnMut(Context, usize, Style) -> Style>,
    ) -> Self {
        *self.inner.borrow().position_style_hook.borrow_mut() = Some(hook);
        self
    }

    pub fn set_position_style_hook(
        &mut self, hook: Box<dyn FnMut(Context, usize, Style) -> Style>,
    ) {
        *self.inner.borrow().position_style_hook.borrow_mut() = Some(hook);
    }

    pub fn with_cursor_changed_hook(
        self, hook: Box<dyn FnMut(Context, usize) -> EventResponses>,
    ) -> Self {
        *self.inner.borrow().cursor_changed_hook.borrow_mut() = Some(hook);
        self
    }

    pub fn set_cursor_changed_hook(
        &mut self, hook: Box<dyn FnMut(Context, usize) -> EventResponses>,
    ) {
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
        self
    }

    pub fn with_corner_decor(self, decor: DrawCh) -> Self {
        *self.inner.borrow().corner_decor.borrow_mut() = decor;
        self
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

    //fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
    //    *self.inner.borrow().current_sty.borrow_mut() = self.pane.get_current_style();
    //    self.pane.drawing(ctx)
    //}
}

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct TextBoxInner {
    pub pane: Pane,
    pub current_sty: Rc<RefCell<Style>>,
    pub selectedness: Rc<RefCell<Selectability>>,
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
    ///                                                              abs_pos
    pub cursor_changed_hook: Rc<RefCell<Option<Box<dyn FnMut(Context, usize) -> EventResponses>>>>,

    pub x_scrollbar: Rc<RefCell<Option<HorizontalScrollbar>>>,
    pub y_scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
    pub line_number_tb: Rc<RefCell<Option<TextBoxInner>>>,

    /// for when there are two scrollbars
    pub corner_decor: Rc<RefCell<DrawCh>>,
    pub right_click_menu: Rc<RefCell<Option<RightClickMenu>>>,
}

impl TextBoxInner {
    const KIND: &'static str = "textbox_inner";

    const STYLE: SelStyles = SelStyles {
        selected_style: Style::new_const(Color::BLACK, Color::WHITE),
        ready_style: Style::new_const(Color::BLACK, Color::GREY13),
        unselectable_style: Style::new_const(Color::BLACK, Color::GREY15),
    };

    const DEFAULT_CURSOR_STYLE: Style = Style::new_const(Color::WHITE, Color::BLUE);

    /// for textboxes which are editable
    pub fn editable_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KeyPossibility::Chars.into(), Priority::Focused),
            (KB::KEY_BACKSPACE.into(), Priority::Focused),
            (KB::KEY_ENTER.into(), Priority::Focused),
            (KB::KEY_SHIFT_ENTER.into(), Priority::Focused),
            (KB::KEY_LEFT.into(), Priority::Focused),
            (KB::KEY_RIGHT.into(), Priority::Focused),
            (KB::KEY_UP.into(), Priority::Focused),
            (KB::KEY_DOWN.into(), Priority::Focused),
            (KB::KEY_SHIFT_LEFT.into(), Priority::Focused),
            (KB::KEY_SHIFT_RIGHT.into(), Priority::Focused),
            (KB::KEY_SHIFT_UP.into(), Priority::Focused),
            (KB::KEY_SHIFT_DOWN.into(), Priority::Focused),
        ])
    }

    /// non-editable textboxes can still scroll
    pub fn non_editable_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KB::KEY_LEFT.into(), Priority::Focused),
            (KB::KEY_RIGHT.into(), Priority::Focused),
            (KB::KEY_UP.into(), Priority::Focused),
            (KB::KEY_DOWN.into(), Priority::Focused),
            (KB::KEY_H.into(), Priority::Focused),
            (KB::KEY_J.into(), Priority::Focused),
            (KB::KEY_K.into(), Priority::Focused),
            (KB::KEY_L.into(), Priority::Focused),
        ])
    }

    pub fn new<S: Into<String>>(ctx: &Context, text: S) -> Self {
        let text = text.into();
        let pane = Pane::new(ctx, Self::KIND)
            .with_dyn_width(DynVal::full())
            .with_dyn_height(DynVal::full())
            .with_self_receivable_events(Self::editable_receivable_events());

        let tb = TextBoxInner {
            pane,
            current_sty: Rc::new(RefCell::new(Style::default())),
            selectedness: Rc::new(RefCell::new(Selectability::Ready)),
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

        let tb1 = tb.clone();
        let tb2 = tb.clone();
        let tb3 = tb.clone();
        let rcm = RightClickMenu::new(ctx, MenuStyle::default()).with_menu_items(
            ctx,
            vec![
                MenuItem::new(ctx, MenuPath("Cut".to_string())).with_click_fn(Some(Box::new(
                    move |ctx| tb1.cut_to_clipboard(&ctx).unwrap(),
                ))),
                MenuItem::new(ctx, MenuPath("Copy".to_string())).with_click_fn(Some(Box::new(
                    move |_ctx| {
                        tb2.copy_to_clipboard().unwrap();
                        EventResponses::default()
                    },
                ))),
                MenuItem::new(ctx, MenuPath("Paste".to_string())).with_click_fn(Some(Box::new(
                    move |ctx| tb3.paste_from_clipboard(&ctx).unwrap(),
                ))),
            ],
        );
        *tb.right_click_menu.borrow_mut() = Some(rcm);

        let _ = tb.drawing(ctx); // to set the pane content
        tb
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.pane.set_at(loc_x, loc_y);
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

    pub fn get_text(&self) -> String {
        self.text.borrow().iter().collect()
    }

    pub fn set_text(&self, text: String) {
        *self.text.borrow_mut() = text.chars().collect();
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

    pub fn set_cursor_pos(&self, ctx: &Context, new_abs_pos: usize) -> EventResponses {
        *self.cursor_pos.borrow_mut() = new_abs_pos;
        if let Some(hook) = &mut *self.cursor_changed_hook.borrow_mut() {
            hook(ctx.clone(), new_abs_pos)
        } else {
            EventResponses::default()
        }
    }

    pub fn incr_cursor_pos(&self, ctx: &Context, pos_change: isize) -> EventResponses {
        let new_pos = (self.get_cursor_pos() as isize + pos_change).max(0) as usize;
        self.set_cursor_pos(ctx, new_pos)
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
        }

        // get the largest amount of digits in the line numbers from the string
        let line_num_width = max_line_num.to_string().chars().count();

        let mut s = String::new();
        let mut true_line_num = 1;
        for (i, wr_ch) in wr_chs.chs.iter().enumerate() {
            if wr_ch.ch == '\n' || i == 0 {
                if wr_ch.abs_pos.is_some() || i == 0 {
                    s += &format!("{:line_num_width$} ", true_line_num);
                    true_line_num += 1;
                }
                s += "\n";
            }
        }
        (s, line_num_width + 1) // +1 for the extra space after the digits
    }

    /// takes into account the width of the line numbers textbox
    pub fn x_new_domain_chs(&self) -> usize {
        let lnw = if let Some(ln_tb) = self.line_number_tb.borrow().as_ref() {
            ln_tb.pane.content_width()
        } else {
            0
        };
        self.pane.content_width() + lnw
    }

    /// NOTE the resp is sent in to potentially modify the offsets from numbers tb
    pub fn correct_offsets(&self, ctx: &Context, w: &WrChs) -> EventResponse {
        let (x, y) = w.cursor_x_and_y(self.get_cursor_pos());
        let (x, y) = (x.unwrap_or(0), y.unwrap_or(0));
        self.pane.correct_offsets_to_view_position(ctx, x, y);

        let y_offset = self.pane.get_content_y_offset();
        let x_offset = self.pane.get_content_x_offset();

        // update the scrollbars/line numbers textbox
        if let Some(sb) = self.y_scrollbar.borrow().as_ref() {
            sb.external_change(y_offset, self.pane.content_height());
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
            sb.external_change(x_offset, self.x_new_domain_chs());
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
        self.set_cursor_pos(ctx, start_pos);

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

    pub fn copy_to_clipboard(&self) -> Result<(), Error> {
        let text = self.visual_selected_text();
        arboard::Clipboard::new()?.set_text(text)?;
        Ok(())
    }

    pub fn cut_to_clipboard(&self, ctx: &Context) -> Result<EventResponses, Error> {
        self.copy_to_clipboard()?;
        Ok(self.delete_visual_selection(ctx))
    }

    pub fn paste_from_clipboard(&self, ctx: &Context) -> Result<EventResponses, Error> {
        let mut resps = self.delete_visual_selection(ctx);

        let cliptext = arboard::Clipboard::new()?.get_text()?;
        if cliptext.is_empty() {
            return Ok(resps);
        }
        let cliprunes = cliptext.chars().collect::<Vec<char>>();
        let mut rs = self.text.borrow().clone();
        let cur_pos = self.get_cursor_pos();
        rs.splice(cur_pos..cur_pos, cliprunes.iter().cloned());
        *self.text.borrow_mut() = rs;

        self.incr_cursor_pos(ctx, cliprunes.len() as isize);
        let w = self.get_wrapped(ctx);
        self.pane.set_content_from_string(w.wrapped_string()); // See NOTE-1

        let resp = self.correct_offsets(ctx, &w);
        resps.push(resp);

        if let Some(hook) = &mut *self.text_changed_hook.borrow_mut() {
            resps.extend(hook(ctx.clone(), self.get_text()));
        }
        Ok(resps)
    }

    pub fn receive_key_event(&self, ctx: &Context, ev: Vec<KeyEvent>) -> (bool, EventResponses) {
        if *self.selectedness.borrow() != Selectability::Selected || ev.is_empty() {
            return (false, EventResponses::default());
        }

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

            // update the scrollbars
            let y_offset = self.pane.get_content_y_offset();
            let x_offset = self.pane.get_content_x_offset();
            if let Some(sb) = self.y_scrollbar.borrow().as_ref() {
                sb.external_change(y_offset, self.pane.content_height());
            }
            if let Some(sb) = self.x_scrollbar.borrow().as_ref() {
                sb.external_change(x_offset, self.x_new_domain_chs());
            }
            return (true, EventResponses::default());
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
                    self.incr_cursor_pos(ctx, -1);
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
                    self.incr_cursor_pos(ctx, 1);
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
                    self.set_cursor_pos(ctx, new_pos);
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
                    self.set_cursor_pos(ctx, new_pos);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if ev[0] == KB::KEY_LEFT => {
                if cursor_pos > 0 && cursor_pos <= self.text.borrow().len() {
                    // do not move left if at the beginning of a line
                    if self.text.borrow()[cursor_pos - 1] != '\n' {
                        self.incr_cursor_pos(ctx, -1);
                        let w = self.get_wrapped(ctx);
                        resps = self.correct_offsets(ctx, &w).into();
                    }
                }
            }

            _ if ev[0] == KB::KEY_RIGHT => {
                // don't allow moving to the next line
                if cursor_pos < self.text.borrow().len() && self.text.borrow()[cursor_pos] != '\n' {
                    self.incr_cursor_pos(ctx, 1);
                    let w = self.get_wrapped(ctx);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if ev[0] == KB::KEY_UP => {
                let w = self.get_wrapped(ctx);
                if let Some(new_pos) = w.get_cursor_above_position(cursor_pos) {
                    self.set_cursor_pos(ctx, new_pos);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if ev[0] == KB::KEY_DOWN => {
                let w = self.get_wrapped(ctx);
                if let Some(new_pos) = w.get_cursor_below_position(cursor_pos) {
                    self.set_cursor_pos(ctx, new_pos);
                    resps = self.correct_offsets(ctx, &w).into();
                }
            }

            _ if *self.editable.borrow() && KeyPossibility::Chars.matches_key(&ev[0]) => {
                let r: String = format!("{}", ev[0].code);
                let mut rs = self.text.borrow().clone();
                rs.splice(cursor_pos..cursor_pos, r.chars());
                *self.text.borrow_mut() = rs;
                self.incr_cursor_pos(ctx, 1);
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

            _ if *self.editable.borrow() && (ev[0] == KB::KEY_BACKSPACE) => {
                if visual_mode {
                    resps = self.delete_visual_selection(ctx);
                } else if cursor_pos > 0 {
                    let mut rs = self.text.borrow().clone();
                    rs.remove(cursor_pos - 1);
                    self.incr_cursor_pos(ctx, -1);
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
                self.incr_cursor_pos(ctx, 1);
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
            if let Some(resp) = rcm.create_menu_if_right_click(ev) {
                return (true, resp);
            }
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
                self.set_cursor_pos(ctx, new_pos);
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
                self.set_cursor_pos(ctx, new_pos);
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
                self.set_cursor_pos(ctx, new_pos);
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
                self.set_cursor_pos(ctx, new_pos);
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
                self.set_cursor_pos(ctx, new_pos);
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
                self.set_cursor_pos(ctx, new_pos);
                let resp = self.correct_offsets(ctx, &w);
                return (true, resp.into());
            }
            MouseEventKind::ScrollDown
                if ev.modifiers == KeyModifiers::NONE && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_down(ctx);
                return (true, EventResponses::default());
            }

            MouseEventKind::ScrollUp
                if ev.modifiers == KeyModifiers::NONE && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_up(ctx);
                return (true, EventResponses::default());
            }
            MouseEventKind::ScrollLeft
                if ev.modifiers == KeyModifiers::NONE && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_left(ctx);
                return (true, EventResponses::default());
            }
            MouseEventKind::ScrollDown
                if ev.modifiers == KeyModifiers::SHIFT && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_left(ctx);
                return (true, EventResponses::default());
            }
            MouseEventKind::ScrollRight
                if ev.modifiers == KeyModifiers::NONE && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_right(ctx);
                return (true, EventResponses::default());
            }
            MouseEventKind::ScrollUp
                if ev.modifiers == KeyModifiers::SHIFT && selectedness == Selectability::Ready =>
            {
                self.pane.scroll_right(ctx);
                return (true, EventResponses::default());
            }

            MouseEventKind::Moved | MouseEventKind::Up(_) => {
                *self.mouse_dragging.borrow_mut() = false;
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
                    self.set_cursor_pos(ctx, new_pos);
                    let resp = self.correct_offsets(ctx, &w);
                    return (true, resp.into());
                }
                return (true, EventResponses::default());
            }
            _ => {}
        }

        (false, EventResponses::default())
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

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let w = self.get_wrapped(ctx);
        let wrapped = w.wrapped_string();

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
            return self.pane.drawing(ctx);
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
                let sty = hook(ctx.clone(), wr_ch.abs_pos.unwrap(), existing_sty);
                self.pane
                    .content
                    .borrow_mut()
                    .change_style_at_xy(wr_ch.x_pos, wr_ch.y_pos, sty);
            }
        }

        // set cursor style
        if *self.selectedness.borrow() == Selectability::Selected && *self.ch_cursor.borrow() {
            let (cur_x, cur_y) = w.cursor_x_and_y(self.get_cursor_pos());
            if let (Some(cur_x), Some(cur_y)) = (cur_x, cur_y) {
                self.pane.content.borrow_mut().change_style_at_xy(
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
                    self.pane.content.borrow_mut().change_style_at_xy(
                        cur_x,
                        cur_y,
                        self.cursor_style.borrow().clone(),
                    );
                }
            }
        }

        self.pane.drawing(ctx)
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
