use {
    super::{
        common, HorizontalSBPositions, HorizontalScrollbar, Label, SclVal, Selectability,
        VerticalSBPositions, VerticalScrollbar, WBStyles, Widget, WidgetBase, Widgets,
    },
    crate::{
        element::RelocationRequest, Context, DrawCh, DrawChPos, Element, ElementID, Error, Event,
        EventResponse, EventResponses, KeyPossibility, Keyboard as KB, Priority,
        ReceivableEventChanges, RgbColour, SortingHat, Style, UpwardPropagator,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// TODO better multiline cursor movement
// retain greater cursor position between lines, ex:
//    123456789<cursor, starting position>
//    1234<cursor after moving down>
//    123456789<cursor, after moving down again>

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct TextBox {
    pub base: WidgetBase,
    pub text: Rc<RefCell<Vec<char>>>,
    pub ch_cursor: Rc<RefCell<bool>>, // whether or not this tb has a ch cursor
    pub editable: Rc<RefCell<bool>>,  // whether or not this tb can be edited
    pub wordwrap: Rc<RefCell<bool>>,  // whether or not to wrap text
    pub line_numbered: Rc<RefCell<bool>>, // whether or not there are lefthand line numbers
    pub cursor_pos: Rc<RefCell<usize>>, // cursor absolute position in the text
    pub cursor_style: Rc<RefCell<Style>>,
    pub visual_mode: Rc<RefCell<bool>>, // whether or not the cursor is visual selecting
    pub mouse_dragging: Rc<RefCell<bool>>, // if the mouse is currently dragging
    pub visual_mode_start_pos: Rc<RefCell<usize>>, // the start position of the visual select

    pub text_changed_hook: Rc<RefCell<Option<Box<dyn FnMut(Context, String) -> EventResponses>>>>,

    // When this hook is non-nil each characters style will be determineda via this hook.
    // This is intended to be used if the caller of the textbox wishes granular control
    // over the text styling.
    //                                                              abs_pos, existing
    pub position_style_hook: Rc<RefCell<Option<Box<dyn FnMut(Context, usize, Style) -> Style>>>>,

    // this hook is called each time the cursor moves
    //                                                              abs_pos
    pub cursor_changed_hook: Rc<RefCell<Option<Box<dyn FnMut(Context, usize) -> EventResponses>>>>,

    pub x_scrollbar_op: Rc<RefCell<HorizontalSBPositions>>,
    pub y_scrollbar_op: Rc<RefCell<VerticalSBPositions>>,
    pub x_scrollbar: Rc<RefCell<Option<HorizontalScrollbar>>>,
    pub y_scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,

    pub line_number_tb: Rc<RefCell<Option<TextBox>>>,

    // for when there are two scrollbars
    pub corner_decor: Rc<RefCell<DrawCh>>,
    // XXX TODO
    //pub right_click_menu: Rc<RefCell<Option<RightClickMenuTemplate>>>,
}

impl TextBox {
    const KIND: &'static str = "widget_textbox";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new()
            .with_bg(RgbColour::WHITE)
            .with_fg(RgbColour::BLACK),
        ready_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::BLACK),
        unselectable_style: Style::new()
            .with_bg(RgbColour::GREY15)
            .with_fg(RgbColour::BLACK),
    };

    const STYLE_SCROLLBAR: WBStyles = WBStyles {
        selected_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::WHITE),
        ready_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::WHITE),
        unselectable_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::WHITE),
    };

    const DEFAULT_CURSOR_STYLE: Style = Style::new().with_bg(RgbColour::BLUE);
    const DEFAULT_RIGHT_CLICK_MENU_STYLE: Style = Style::new().with_bg(RgbColour::LIME);

    // for textboxes which are editable
    pub fn editable_receivable_events() -> Vec<Event> {
        vec![
            KeyPossibility::Runes.into(),
            KB::KEY_ENTER.into(),
            KB::KEY_SHIFT_ENTER.into(),
            KB::KEY_LEFT.into(),
            KB::KEY_RIGHT.into(),
            KB::KEY_UP.into(),
            KB::KEY_DOWN.into(),
            KB::KEY_SHIFT_LEFT.into(),
            KB::KEY_SHIFT_RIGHT.into(),
            KB::KEY_SHIFT_UP.into(),
            KB::KEY_SHIFT_DOWN.into(),
        ]
    }

    // non-editable textboxes can still scroll
    pub fn non_editable_receivable_events() -> Vec<Event> {
        vec![
            KB::KEY_LEFT.into(),
            KB::KEY_RIGHT.into(),
            KB::KEY_UP.into(),
            KB::KEY_DOWN.into(),
            KB::KEY_H.into(),
            KB::KEY_J.into(),
            KB::KEY_K.into(),
            KB::KEY_L.into(),
        ]
    }

    pub fn new(hat: &SortingHat, ctx: &Context, text: String) -> Self {
        let (width, height) = common::get_text_size(&text);
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            SclVal::new_fixed(width),
            SclVal::new_fixed(height),
            Self::STYLE,
            Self::editable_receivable_events(),
        );

        let tb = TextBox {
            base: wb,
            text: Rc::new(RefCell::new(text.chars().collect())),
            wordwrap: Rc::new(RefCell::new(true)),
            line_numbered: Rc::new(RefCell::new(false)),
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
            x_scrollbar_op: Rc::new(RefCell::new(HorizontalSBPositions::None)),
            y_scrollbar_op: Rc::new(RefCell::new(VerticalSBPositions::None)),
            x_scrollbar: Rc::new(RefCell::new(None)),
            y_scrollbar: Rc::new(RefCell::new(None)),
            line_number_tb: Rc::new(RefCell::new(None)),
            corner_decor: Rc::new(RefCell::new(DrawCh::new('⁙', false, Style::default()))),
        };

        // XXX TODO
        //rcm := els.NewRightClickMenuTemplate(DefaultRightClickMenuStyle)
        //rcm.SetMenuItems([]*els.MenuItem{
        //    els.NewMenuItem("Cut", true,
        //        func(_ yh.Context) yh.EventResponse {
        //            resp, _ := tb.CutToClipboard()
        //            return resp
        //        },
        //    ),
        //    els.NewMenuItem("Copy", true,
        //        func(_ yh.Context) yh.EventResponse {
        //            _ = tb.CopyToClipboard()
        //            return yh.NewEventResponse()
        //        },
        //    ),
        //    els.NewMenuItem("Paste", true,
        //        func(_ yh.Context) yh.EventResponse {
        //            resp, _ := tb.PasteFromClipboard()
        //            return resp
        //        },
        //    ),
        //})
        //tb.RightClickMenu = rcm

        let _ = tb.drawing(ctx); // to set the base content
        tb
    }

    // ---------------------------------------------------------
    // Decorators

    pub fn with_left_scrollbar(self) -> Self {
        *self.y_scrollbar_op.borrow_mut() = VerticalSBPositions::ToTheLeft;
        self
    }

    pub fn with_right_scrollbar(self) -> Self {
        *self.y_scrollbar_op.borrow_mut() = VerticalSBPositions::ToTheRight;
        self
    }

    pub fn with_top_scrollbar(self) -> Self {
        *self.x_scrollbar_op.borrow_mut() = HorizontalSBPositions::Above;
        self
    }

    pub fn with_lower_scrollbar(self) -> Self {
        *self.x_scrollbar_op.borrow_mut() = HorizontalSBPositions::Below;
        self
    }

    // XXX TODO
    //pub fn with_right_click_menu(mut self) -> Self {
    //self.right_click_menu = rcm;
    //    self
    //}

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn with_width(self, width: SclVal) -> Self {
        self.base.set_attr_scl_width(width);
        self
    }
    pub fn with_height(self, height: SclVal) -> Self {
        self.base.set_attr_scl_height(height);
        self
    }
    pub fn with_size(self, width: SclVal, height: SclVal) -> Self {
        self.base.set_attr_scl_width(width);
        self.base.set_attr_scl_height(height);
        self
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn with_ch_cursor(self) -> Self {
        *self.ch_cursor.borrow_mut() = true;
        self
    }

    pub fn with_no_ch_cursor(self) -> Self {
        *self.ch_cursor.borrow_mut() = false;
        self
    }

    pub fn editable(self) -> Self {
        *self.editable.borrow_mut() = true;

        let evs = Self::editable_receivable_events()
            .drain(..)
            .map(|ev| (ev, Priority::FOCUSED))
            .collect();
        self.base.set_receivable_events(evs);
        self
    }

    pub fn non_editable(self) -> Self {
        *self.editable.borrow_mut() = false;
        let evs = Self::non_editable_receivable_events()
            .drain(..)
            .map(|ev| (ev, Priority::FOCUSED))
            .collect();
        self.base.set_receivable_events(evs);
        self
    }

    pub fn with_wordwrap(self) -> Self {
        *self.wordwrap.borrow_mut() = true;
        self
    }

    pub fn with_no_wordwrap(self) -> Self {
        *self.wordwrap.borrow_mut() = false;
        self
    }

    pub fn with_line_numbers(self) -> Self {
        *self.line_numbered.borrow_mut() = true;
        self
    }

    pub fn with_no_line_numbers(self) -> Self {
        *self.line_numbered.borrow_mut() = false;
        self
    }

    pub fn with_position_style_hook(
        self, hook: Box<dyn FnMut(Context, usize, Style) -> Style>,
    ) -> Self {
        *self.position_style_hook.borrow_mut() = Some(hook);
        self
    }

    pub fn set_position_style_hook(
        &mut self, hook: Box<dyn FnMut(Context, usize, Style) -> Style>,
    ) {
        *self.position_style_hook.borrow_mut() = Some(hook);
    }

    pub fn with_cursor_changed_hook(
        self, hook: Box<dyn FnMut(Context, usize) -> EventResponses>,
    ) -> Self {
        *self.cursor_changed_hook.borrow_mut() = Some(hook);
        self
    }

    pub fn set_cursor_changed_hook(
        &mut self, hook: Box<dyn FnMut(Context, usize) -> EventResponses>,
    ) {
        *self.cursor_changed_hook.borrow_mut() = Some(hook);
    }

    pub fn with_text_changed_hook(
        self, hook: Box<dyn FnMut(Context, String) -> EventResponses>,
    ) -> Self {
        *self.text_changed_hook.borrow_mut() = Some(hook);
        self
    }

    pub fn set_text_changed_hook(
        &mut self, hook: Box<dyn FnMut(Context, String) -> EventResponses>,
    ) {
        *self.text_changed_hook.borrow_mut() = Some(hook);
    }

    pub fn with_cursor_style(self, style: Style) -> Self {
        *self.cursor_style.borrow_mut() = style;
        self
    }

    pub fn with_corner_decor(self, decor: DrawCh) -> Self {
        *self.corner_decor.borrow_mut() = decor;
        self
    }

    pub fn to_widgets(self, hat: &SortingHat, ctx: &Context) -> Widgets {
        let (x, y) = (
            self.base.get_attr_scl_loc_x(),
            self.base.get_attr_scl_loc_y(),
        );
        let (h, w) = (
            self.base.get_attr_scl_height(),
            self.base.get_attr_scl_width(),
        );
        let mut out = vec![];

        let mut ln_tb = if *self.line_numbered.borrow() {
            // determine the width of the line numbers textbox

            // create the line numbers textbox
            let (lns, lnw) = self.get_line_numbers(ctx);
            let mut ln_tb = TextBox::new(hat, ctx, lns)
                .at(x, y)
                .with_width(SclVal::new_fixed(lnw))
                .with_height(h)
                .with_no_wordwrap()
                .non_editable()
                .with_selectability(Selectability::Unselectable);
            out.push(ln_tb.clone());

            // reduce the width of the main textbox
            self.base.set_attr_scl_width(w.minus_fixed(lnw));
            self.base.set_attr_scl_loc_x(x.plus_fixed(lnw));

            self.line_number_tb = Rc::new(RefCell::new(Some(ln_tb.clone())));
            ln_tb
        };
        out.push(self.clone());

        // add corner decor
        let y_sb_op = *self.y_scrollbar_op.borrow();
        let x_sb_op = *self.x_scrollbar_op.borrow();
        let no_y_sb = matches!(y_sb_op, VerticalSBPositions::None);
        let no_x_sb = matches!(x_sb_op, HorizontalSBPositions::None);
        if !no_y_sb && !no_x_sb {
            let cd = Label::new(hat, ctx, &(self.corner_decor.borrow().ch.to_string()))
                .with_style(ctx, self.corner_decor.borrow().style);
            let (cd_x, cd_y) = match (y_sb_op, x_sb_op) {
                (VerticalSBPositions::ToTheLeft, HorizontalSBPositions::Above) => {
                    (x.minus_fixed(1), y.minus_fixed(1))
                }
                (VerticalSBPositions::ToTheLeft, HorizontalSBPositions::Below) => {
                    (x.minus_fixed(1), y.plus(h))
                }
                (VerticalSBPositions::ToTheRight, HorizontalSBPositions::Above) => {
                    (x.plus(w), y.minus_fixed(1))
                }
                (VerticalSBPositions::ToTheRight, HorizontalSBPositions::Below) => {
                    (x.plus(w), y.plus(h))
                }
                _ => panic!("impossible"),
            };
            cd.at(cd_x, cd_y);
            out.push(cd);
        }

        if !no_y_sb {
            let x2 = match y_sb_op {
                VerticalSBPositions::ToTheLeft => x.minus_fixed(1),
                VerticalSBPositions::ToTheRight => x.plus(w),
                _ => panic!("impossible"),
            };
            let vsb = VerticalScrollbar::new(hat, h, self.base.content_height())
                .at(x2, y)
                .with_styles(Self::STYLE_SCROLLBAR);

            match ln_tb {
                Some(ln_tb) => {
                    let wb_ = self.base.clone();
                    let ln_tb_wb = ln_tb.base.clone();
                    let hook = Box::new(move |ctx, y| {
                        wb_.set_content_y_offset(&ctx, y);
                        ln_tb_wb.set_content_y_offset(&ctx, y);
                    });
                    *vsb.position_changed_hook.borrow_mut() = Some(hook);
                }
                None => {
                    let wb_ = self.base.clone();
                    let hook = Box::new(move |ctx, y| wb_.set_content_y_offset(&ctx, y));
                    *vsb.position_changed_hook.borrow_mut() = Some(hook);
                }
            }

            *self.y_scrollbar.borrow_mut() = Some(vsb);
            out.push(vsb);
        }

        if !no_x_sb {
            let y2 = match x_sb_op {
                HorizontalSBPositions::Above => y.minus_fixed(1),
                HorizontalSBPositions::Below => y.plus(h),
                _ => panic!("impossible"),
            };

            let hsb = HorizontalScrollbar::new(hat, w, self.base.content_width())
                .at(x, y2)
                .with_styles(Self::STYLE_SCROLLBAR);

            let wb_ = self.base.clone();
            let hook = Box::new(move |ctx, x| wb_.set_content_x_offset(&ctx, x));
            *hsb.position_changed_hook.borrow_mut() = Some(hook);

            out.push(hsb);
        }

        let _ = drawing(ctx); // to set the base content
        Widgets(out)
    }

    pub fn get_text(&self) -> String {
        self.text.borrow().iter().collect()
    }

    pub fn set_text(&self, text: String) {
        *self.text.borrow_mut() = text.chars().collect();
    }

    // ---------------------------------------------------------

    pub fn get_cursor_pos(&self) -> usize {
        *self.cursor_pos.borrow()
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
        let new_pos = (*self.cursor_pos.borrow() as isize + pos_change).max(0) as usize;
        self.set_cursor_pos(ctx, new_pos)
    }

    // returns the wrapped characters of the text
    pub fn get_wrapped(&self, ctx: &Context) -> WrChs {
        let mut rs = self.text.borrow().clone();
        rs.push(' '); // add the space for the final possible position
        let mut chs = vec![];
        let mut max_x = 0;
        let (mut x, mut y) = (0, 0); // working x and y position in the textbox
        for (abs_pos, r) in rs.iter().enumerate() {
            if *self.wordwrap.borrow() && x == self.base.get_width(ctx) {
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

    // returns the formatted line numbers of the textbox
    // line numbers are right justified
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

    // NOTE the resp is sent in to potentially modify the offsets from numbers tb
    pub fn correct_offsets(&self, ctx: &Context, w: WrChs) -> EventResponse {
        let (x, y) = w.cursor_x_and_y(*self.cursor_pos.borrow());
        let (x, y) = (x.unwrap_or(0), y.unwrap_or(0));
        self.base.correct_offsets_to_view_position(ctx, x, y);

        let y_offset = *self.base.sp.content_view_offset_y.borrow();
        let x_offset = *self.base.sp.content_view_offset_x.borrow();

        // update the scrollbars/line numbers textbox
        if let Some(sb) = self.y_scrollbar.borrow().as_ref() {
            sb.external_change(&ctx, y_offset, self.base.content_height());
        }
        let mut resp = EventResponse::default();
        if let Some(ln_tb) = self.line_number_tb.borrow().as_ref() {
            let (lns, lnw) = self.get_line_numbers(ctx);
            let last_lnw = ln_tb.base.get_width(ctx);
            if lnw != last_lnw {
                let diff_lnw = lnw - last_lnw;
                let new_tb_width = self.base.get_attr_scl_width().minus_fixed(diff_lnw);
                self.base.set_attr_scl_width(new_tb_width);
                resp.set_relocation(RelocationRequest::new_left(diff_lnw as i32));
            }
            ln_tb.set_text(lns);
            ln_tb.base.set_attr_scl_width(SclVal::new_fixed(lnw));
            ln_tb.base.set_content_y_offset(ctx, y_offset);
        }
        if let Some(sb) = self.x_scrollbar.borrow().as_ref() {
            sb.external_change(&ctx, x_offset, self.base.content_width());
        }
        resp
    }

    pub fn visual_selected_text(&self) -> String {
        let text = self.text.borrow();
        if !*self.visual_mode.borrow() {
            return text[*self.cursor_pos.borrow()].to_string();
        }
        let start_pos = *self.visual_mode_start_pos.borrow();
        let cur_pos = *self.cursor_pos.borrow();
        if start_pos < cur_pos {
            return text[start_pos..cur_pos + 1].iter().collect();
        }
        self.text.borrow()[cur_pos..start_pos + 1].iter().collect()
    }

    pub fn delete_visual_selection(&self, ctx: &Context) -> EventResponses {
        if !*self.visual_mode.borrow() {
            return EventResponses::default();
        }

        // delete everything in the visual selection
        let mut rs = self.text.borrow().clone();
        if *self.visual_mode_start_pos.borrow() < *self.cursor_pos.borrow() {
            rs.drain(*self.visual_mode_start_pos.borrow()..*self.cursor_pos.borrow() + 1);
            self.set_cursor_pos(ctx, *self.visual_mode_start_pos.borrow());
        } else {
            rs.drain(*self.cursor_pos.borrow()..*self.visual_mode_start_pos.borrow() + 1);
            // (leave the cursor at the start of the visual selection)
        }
        *self.text.borrow_mut() = rs;
        *self.visual_mode.borrow_mut() = false;
        let w = self.get_wrapped(ctx);
        self.base.set_content_from_string(ctx, &w.wrapped_string());
        let resp = self.correct_offsets(ctx, w);
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
        if text.is_empty() {
            return Ok(());
        }
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
        rs.splice(
            *self.cursor_pos.borrow()..*self.cursor_pos.borrow(),
            cliprunes.iter().cloned(),
        );
        *self.text.borrow_mut() = rs;

        self.incr_cursor_pos(ctx, cliprunes.len() as isize);
        let w = self.get_wrapped(ctx);
        self.base.set_content_from_string(ctx, &w.wrapped_string()); // See NOTE-1

        let resp = self.correct_offsets(ctx, w);
        resps.push(resp);

        if let Some(hook) = &mut *self.text_changed_hook.borrow_mut() {
            resps.extend(hook(ctx.clone(), self.get_text()).0);
        }
        Ok(resps)
    }
}

impl Widget for TextBox {
    fn set_selectability_pre_hook(&self, _: &Context, s: Selectability) -> EventResponses {
        if self.base.get_selectability() == Selectability::Selected && s != Selectability::Selected
        {
            *self.visual_mode.borrow_mut() = false;
        }
        EventResponses::default()
    }
}

impl Element for TextBox {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }
                return match true {
                    //_ if ke[0].matches(&KB::KEY_SPACE) => {
                    //    if let Some(sb) = self.scrollbar.borrow().as_ref() {
                    //        if sb.get_selectability() != Selectability::Selected {
                    //            sb.set_selectability(ctx, Selectability::Selected);
                    //        }
                    //        sb.receive_event(ctx, Event::KeyCombo(ke))
                    //    } else {
                    //        (true, EventResponses::default())
                    //    }
                    //}
                    //_ if ke[0].matches(&KB::KEY_DOWN) || ke[0].matches(&KB::KEY_J) => {
                    //    self.cursor_down(ctx);
                    //    (true, EventResponses::default())
                    //}
                    //_ if ke[0].matches(&KB::KEY_UP) || ke[0].matches(&KB::KEY_K) => {
                    //    self.cursor_up(ctx);
                    //    (true, EventResponses::default())
                    //}
                    //_ if ke[0].matches(&KB::KEY_ENTER) => {
                    //    let Some(cursor) = *self.cursor.borrow() else {
                    //        return (true, EventResponses::default());
                    //    };
                    //    let entries_len = self.entries.borrow().len();
                    //    if cursor >= entries_len {
                    //        return (true, EventResponses::default());
                    //    }
                    //    return (true, self.toggle_entry_selected_at_i(ctx, cursor));
                    //}
                    _ => (false, EventResponses::default()),
                };
            }
            Event::Mouse(me) => {
                //let (mut clicked, mut dragging, mut scroll_up, mut scroll_down) =
                //    (false, false, false, false);
                //match me.kind {
                //    MouseEventKind::Up(MouseButton::Left) => clicked = true,
                //    MouseEventKind::Drag(MouseButton::Left) => dragging = true,
                //    MouseEventKind::ScrollUp => scroll_up = true,
                //    MouseEventKind::ScrollDown => scroll_down = true,
                //    _ => {}
                //}

                return match true {
                    _ => (false, EventResponses::default()),
                };
            }
            _ => {}
        }
        (false, EventResponses::default())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let w = self.get_wrapped(ctx);
        let wrapped = w.wrapped_string();
        self.base.set_content_from_string(ctx, &wrapped);

        // set styles from hooks
        if let Some(hook) = &mut *self.position_style_hook.borrow_mut() {
            for wr_ch in w.chs.iter() {
                let existing_sty = self.base.get_current_style();
                if wr_ch.abs_pos.is_none() {
                    continue;
                }
                let sty = hook(ctx.clone(), wr_ch.abs_pos.unwrap(), existing_sty);
                self.base
                    .sp
                    .content
                    .borrow_mut()
                    .change_style_at_xy(wr_ch.x_pos, wr_ch.y_pos, sty);
            }
        }

        // set cursor style
        if self.base.get_selectability() == Selectability::Selected && *self.ch_cursor.borrow() {
            let (cur_x, cur_y) = w.cursor_x_and_y(*self.cursor_pos.borrow());
            if let (Some(cur_x), Some(cur_y)) = (cur_x, cur_y) {
                self.base.sp.content.borrow_mut().change_style_at_xy(
                    cur_x,
                    cur_y,
                    *self.cursor_style.borrow(),
                );
            }
        }
        if *self.visual_mode.borrow() {
            let start_pos = *self.visual_mode_start_pos.borrow();
            let cur_pos = *self.cursor_pos.borrow();

            let start = if start_pos < cur_pos { start_pos } else { cur_pos };
            let end = if start_pos < cur_pos { cur_pos } else { start_pos };

            for i in start..=end {
                if let (Some(cur_x), Some(cur_y)) = w.cursor_x_and_y(i) {
                    self.base.sp.content.borrow_mut().change_style_at_xy(
                        cur_x,
                        cur_y,
                        *self.cursor_style.borrow(),
                    );
                }
            }
        }

        self.base.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.base.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.base.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.base.set_upward_propagator(up)
    }
}

/*

func (tb *TextBox) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
    resp = yh.NewEventResponse()

    if tb.Selectedness != Selected {
        return false, yh.NewEventResponse()
    }

    if !tb.chCursor {
        switch {
        case (yh.LeftEKC.Matches(evs) || yh.HLowerEKC.Matches(evs)):
            tb.ScrollLeft()
        case (yh.RightEKC.Matches(evs) || yh.LLowerEKC.Matches(evs)):
            tb.ScrollRight()
        case (yh.DownEKC.Matches(evs) || yh.JLowerEKC.Matches(evs)):
            tb.ScrollDown()
        case (yh.UpEKC.Matches(evs) || yh.KLowerEKC.Matches(evs)):
            tb.ScrollUp()
        }

        // call the offset hook
        if tb.YChangedHook != nil {
            tb.YChangedHook(tb.ContentYOffset, tb.ContentHeight())
        }

        return true, resp
    }

    visualModeEvent := false
    switch {
    case yh.ShiftLeftEKC.Matches(evs):
        visualModeEvent = true
        if !tb.visualMode {
            tb.visualMode = true
            tb.visualModeStartPos = tb.cursorPos
        }
        if tb.cursorPos > 0 {
            tb.IncrCursorPos(-1)
            w := tb.GetWrapped()
            tb.CorrectOffsets(w, &resp)
        }
    case yh.ShiftRightEKC.Matches(evs):
        visualModeEvent = true
        if !tb.visualMode {
            tb.visualMode = true
            tb.visualModeStartPos = tb.cursorPos
        }
        if tb.cursorPos < len(tb.text) {
            tb.IncrCursorPos(1)

            w := tb.GetWrapped()
            tb.CorrectOffsets(w, &resp)
        }

    case yh.ShiftUpEKC.Matches(evs):
        visualModeEvent = true
        if !tb.visualMode {
            tb.visualMode = true
            tb.visualModeStartPos = tb.cursorPos
        }
        w := tb.GetWrapped()
        tb.SetCursorPos(w.GetCursorAbovePosition(tb.cursorPos))
        tb.CorrectOffsets(w, &resp)

    case yh.ShiftDownEKC.Matches(evs):
        visualModeEvent = true
        if !tb.visualMode {
            tb.visualMode = true
            tb.visualModeStartPos = tb.cursorPos
        }
        w := tb.GetWrapped()
        tb.SetCursorPos(w.GetCursorBelowPosition(tb.cursorPos))

        tb.CorrectOffsets(w, &resp)

    case yh.LeftEKC.Matches(evs):

        if tb.cursorPos > 0 {
            // do not move left if at the beginning of a line
            if tb.text[tb.cursorPos-1] != '\n' {
                tb.IncrCursorPos(-1)

                w := tb.GetWrapped()
                tb.CorrectOffsets(w, &resp)
            }
        }
    case yh.RightEKC.Matches(evs) && tb.wordwrap:
        if tb.cursorPos < len(tb.text) {
            tb.IncrCursorPos(1)

            w := tb.GetWrapped()
            tb.CorrectOffsets(w, &resp)
        }

    // if wordwrap is disable do not move to the next line
    // when at the end of the line
    case yh.RightEKC.Matches(evs) && !tb.wordwrap:
        w := tb.GetWrapped()
        curX, curY := w.CursorXAndY(tb.cursorPos)
        l := w.GetLine(curY)
        if curX < len(l)-2 {
            tb.IncrCursorPos(1)
            tb.CorrectOffsets(w, &resp)
        }

    case yh.UpEKC.Matches(evs):
        w := tb.GetWrapped()
        tb.SetCursorPos(w.GetCursorAbovePosition(tb.cursorPos))
        tb.CorrectOffsets(w, &resp)
    case yh.DownEKC.Matches(evs):
        w := tb.GetWrapped()
        tb.SetCursorPos(w.GetCursorBelowPosition(tb.cursorPos))
        tb.CorrectOffsets(w, &resp)

    case tb.editable && yh.RunesEKC.Matches(evs):
        if len(evs) > 0 {
            rs := tb.text
            rs = append(rs[:tb.cursorPos], append([]rune{evs[0].Rune()},
                rs[tb.cursorPos:]...)...)
            tb.text = rs
            tb.IncrCursorPos(1)
            w := tb.GetWrapped()

            // NOTE-1: must call SetContentFromString to update the content
            // before updating the offset or else the offset amount may not
            // exist in the content and the widget base will reject the new
            // offset
            tb.SetContentFromString(w.WrappedStr())
            tb.CorrectOffsets(w, &resp)

            if tb.TextChangedHook != nil {
                resp = tb.TextChangedHook(string(tb.text))
            }
        }

    case yh.MetaCLowerEKC.Matches(evs):
        _ = tb.CopyToClipboard() // ignore err

    case tb.editable && yh.MetaVLowerEKC.Matches(evs): // TODO fix
        resp, _ = tb.PasteFromClipboard() // ignore err

    case tb.editable && (yh.BackspaceEKC.Matches(evs) || yh.Backspace2EKC.Matches(evs)):
        if tb.visualMode {
            resp = tb.DeleteVisualSelection()
        } else if tb.cursorPos > 0 {
            rs := tb.text
            rs = append(rs[:tb.cursorPos-1], rs[tb.cursorPos:]...)
            tb.text = rs
            tb.IncrCursorPos(-1)
            w := tb.GetWrapped()
            tb.SetContentFromString(w.WrappedStr()) // See NOTE-1
            tb.CorrectOffsets(w, &resp)
            if tb.TextChangedHook != nil {
                resp = tb.TextChangedHook(string(tb.text))
            }
        }
    case tb.editable && yh.EnterEKC.Matches(evs):
        rs := tb.text
        rs = append(rs[:tb.cursorPos], append([]rune{'\n'}, rs[tb.cursorPos:]...)...)
        tb.text = rs
        tb.IncrCursorPos(1)
        w := tb.GetWrapped()
        tb.SetContentFromString(w.WrappedStr()) // See NOTE-1
        tb.CorrectOffsets(w, &resp)
    }

    if tb.visualMode && !visualModeEvent {
        tb.visualMode = false
    }

    return true, resp
}

func (tb *TextBox) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
    resp = yh.NewEventResponse()

    // handle secondary click
    if tb.RightClickMenu != nil {
        // send the event to the right click menu to check for right click
        createRCM := tb.RightClickMenu.CreateMenuIfRightClick(ev)
        if createRCM.HasWindow() {
            return true, yh.NewEventResponse().WithWindow(createRCM)
        }
    }

    if ev.Buttons() == tcell.WheelDown && ev.Modifiers() == tcell.ModNone && tb.Selectedness == Selected {
        w := tb.GetWrapped()
        tb.SetCursorPos(w.GetCursorBelowPosition(tb.cursorPos))
        tb.CorrectOffsets(w, &resp)
        return true, yh.NewEventResponse()
    }

    if ev.Buttons() == tcell.WheelUp && ev.Modifiers() == tcell.ModNone && tb.Selectedness == Selected {
        w := tb.GetWrapped()
        tb.SetCursorPos(w.GetCursorAbovePosition(tb.cursorPos))
        tb.CorrectOffsets(w, &resp)
        return true, yh.NewEventResponse()
    }
    if ev.Buttons() == tcell.WheelDown && ev.Modifiers() == tcell.ModShift && tb.Selectedness == Selected {
        w := tb.GetWrapped()
        tb.SetCursorPos(w.GetCursorLeftPosition(tb.cursorPos))
        tb.CorrectOffsets(w, &resp)
        return true, yh.NewEventResponse()
    }

    if ev.Buttons() == tcell.WheelUp && ev.Modifiers() == tcell.ModShift && tb.Selectedness == Selected {
        w := tb.GetWrapped()
        tb.SetCursorPos(w.GetCursorRightPosition(tb.cursorPos))
        tb.CorrectOffsets(w, &resp)
        return true, yh.NewEventResponse()
    }

    if ev.Buttons() == tcell.ButtonNone && tb.Selectedness == Selected {
        tb.mouseDragging = false
    }

    // set the cursor to the mouse position on primary click
    if ev.Buttons() == tcell.Button1 && tb.Selectedness == Selected { //left click
        x, y := ev.Position()
        x += tb.ContentXOffset
        y += tb.ContentYOffset
        w := tb.GetWrapped()

        if tb.mouseDragging && !tb.visualMode {
            tb.visualMode = true
            tb.visualModeStartPos = tb.cursorPos
        }
        if !tb.mouseDragging && tb.visualMode {
            tb.visualMode = false
        }
        tb.mouseDragging = true

        tb.SetCursorPos(w.GetNearestValidCursorFromPosition(x, y))
        tb.CorrectOffsets(tb.GetWrapped(), &resp)
        return true, yh.NewEventResponse()
    }
    return tb.WidgetBase.ReceiveMouseEvent(ev)
}

*/

// wrapped character
#[derive(Clone, Default)]
pub struct WrCh {
    ch: char, // the character

    // absolute position in the text
    // If this character is a NOT a part of the text and only introduced
    // due to line wrapping, the absPos will be None (and ch='\n')
    abs_pos: Option<usize>,

    x_pos: usize, // x position in the line
    y_pos: usize, // y position of the line
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

// wrapped characters
#[derive(Clone, Default)]
pub struct WrChs {
    chs: Vec<WrCh>,
    max_x: usize, // the maximum x position within the wrapped characters
}

impl WrChs {
    pub fn wrapped_string(&self) -> String {
        self.chs.iter().map(|wr_ch| wr_ch.ch).collect()
    }

    // gets the cursor x and y position in the wrapped text
    // from the absolute cursor position provided.
    pub fn cursor_x_and_y(&self, cur_abs: usize) -> (Option<usize>, Option<usize>) {
        self.chs
            .iter()
            .find(|wr_ch| wr_ch.abs_pos == Some(cur_abs))
            .map(|wr_ch| (Some(wr_ch.x_pos), Some(wr_ch.y_pos)))
            .unwrap_or_default()
    }

    // gets the line at the given y position
    pub fn get_line(&self, y: usize) -> Vec<char> {
        self.chs
            .iter()
            .filter(|wr_ch| wr_ch.y_pos == y)
            .map(|wr_ch| wr_ch.ch)
            .collect()
    }

    // maximum y position in the wrapped text
    pub fn max_y(&self) -> usize {
        self.chs.last().cloned().unwrap_or_default().y_pos
    }

    pub fn max_x(&self) -> usize {
        self.max_x
    }

    // determine the cursor position above the current cursor position
    //                                                         cur_abs
    pub fn get_cursor_above_position(&self, cur_abs: usize) -> Option<usize> {
        let Some((cur_i, cur_x, cur_y)) = self
            .chs
            .iter()
            .enumerate()
            .find(|(_, wr_ch)| wr_ch.abs_pos == Some(cur_abs))
            .map(|(i, wr_ch)| (i, wr_ch.x_pos, wr_ch.y_pos))
        else {
            return None;
        };

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

    // determine the cursor position below the current cursor position.
    //                                                         cur_abs
    pub fn get_cursor_below_position(&self, cur_abs: usize) -> Option<usize> {
        let Some((cur_i, cur_x, cur_y)) = self
            .chs
            .iter()
            .enumerate()
            .find(|(_, wr_ch)| wr_ch.abs_pos == Some(cur_abs))
            .map(|(i, wr_ch)| (i, wr_ch.x_pos, wr_ch.y_pos))
        else {
            return None;
        };

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
        let Some((cur_i, cur_x, cur_y)) = self
            .chs
            .iter()
            .enumerate()
            .find(|(_, wr_ch)| wr_ch.abs_pos == Some(cur_abs))
            .map(|(i, wr_ch)| (i, wr_ch.x_pos, wr_ch.y_pos))
        else {
            return None;
        };
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
        let Some((cur_i, cur_x, cur_y)) = self
            .chs
            .iter()
            .enumerate()
            .find(|(_, wr_ch)| wr_ch.abs_pos == Some(cur_abs))
            .map(|(i, wr_ch)| (i, wr_ch.x_pos, wr_ch.y_pos))
        else {
            return None;
        };
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

            if y_diff < nearest_y_diff {
                nearest_abs_y_pos = Some(wr_ch.y_pos);
                nearest_abs_x_pos = Some(wr_ch.x_pos);
                nearest_abs = wr_ch.abs_pos;
            } else if y_diff == nearest_y_diff && x_diff < nearest_x_diff {
                nearest_abs_y_pos = Some(wr_ch.y_pos);
                nearest_abs_x_pos = Some(wr_ch.x_pos);
                nearest_abs = wr_ch.abs_pos;
            }
        }
        nearest_abs
    }
}
