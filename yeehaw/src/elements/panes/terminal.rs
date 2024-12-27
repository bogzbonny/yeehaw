/// Original inspiration for this element taken from:
/// https:///github.com/a-kenji/tui-term/blob/development/examples/smux.rs
/// (MIT LICENSE)
use {
    crate::*,
    compact_str::CompactString,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize},
    std::{
        io::{BufWriter, Read, Write},
        sync::{Arc, RwLock},
    },
    tokio::task::spawn_blocking,
};

// TODO use termwiz+wezterm_term instead of vt100? ... would maybe have to use the wezterm-term crate too, which is not published
// one issue is that to be to query for the terminal mouse state you'd need to use wezterm-term
//          https://docs.rs/termwiz/latest/termwiz/
//          https://github.com/wez/wezterm/blob/main/termwiz/examples/widgets_nested.rs
//  code -> https://github.com/doy/vt100-rust/issues/7
// issue publishing while depending on unpublished crate:
//    https://github.com/rust-lang/cargo/issues/6738
//    https://users.rust-lang.org/t/publishing-a-package-with-a-vendored-crate-but-is-not-listed-as-a-dependency-because-the-vendored-crate-author-did-not-publish-his-crate-onto-crates-io-lol/108971/3

// TODO graceful shutdown of tokio tasks

#[derive(Clone)]
pub struct TerminalPane {
    pub pane: Pane,
    pub parser: Arc<RwLock<vt100_yh::Parser>>,
    pub master_pty: Rc<RefCell<Box<dyn MasterPty>>>,
    pub writer: Rc<RefCell<BufWriter<Box<dyn Write + std::marker::Send>>>>,
    pub disable_cursor: Rc<RefCell<bool>>,
    pub cursor: Rc<RefCell<DrawCh>>,

    /// previous draw by the terminal
    pub prev_draw: Rc<RefCell<Vec<DrawChPos>>>,

    pub pty_killer: Rc<RefCell<Box<dyn ChildKiller>>>,
}

impl TerminalPane {
    pub const KIND: &'static str = "terminal_pane";

    pub fn new(ctx: &Context) -> Result<Self, Error> {
        let cwd = std::env::current_dir()?;
        let mut cmd = CommandBuilder::new_default_prog();
        cmd.cwd(cwd);
        Self::new_with_builder(ctx, cmd)
    }

    pub fn new_with_builder(ctx: &Context, cmd: CommandBuilder) -> Result<Self, Error> {
        let mut size = ctx.size;

        // need this as the pty will not open if the size is 0
        if size.width == 0 {
            size.width = 30;
        }
        if size.height == 0 {
            size.height = 30;
        }

        let pane = Pane::new(ctx, Self::KIND);

        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: size.height,
            cols: size.width,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let parser = Arc::new(RwLock::new(vt100_yh::Parser::new(
            size.height,
            size.width,
            0,
        )));

        let mut child = pty_pair.slave.spawn_command(cmd)?;
        let killer = child.clone_killer();
        let ev_tx_ = ctx.ev_tx.clone();
        let n = Self::custom_destruct_event_name(pane.id());
        spawn_blocking(move || {
            // ignore exit status
            // NOTE this wait can be killed by the killer
            let _ = child.wait();
            drop(pty_pair.slave);

            // here blocking send will generate an error if the
            // TUI is closed by the time this send is called, which
            // is not a problem, so ignore this error.
            let _ = ev_tx_.blocking_send(Event::Custom(n, Vec::with_capacity(0)));
        });

        let mut reader = pty_pair.master.try_clone_reader()?;
        let parser_ = parser.clone();

        spawn_blocking(move || {
            let mut processed_buf = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                let size = reader.read(&mut buf)?;
                if size == 0 {
                    //killer_.kill().unwrap();
                    break;
                }
                processed_buf.extend_from_slice(&buf[..size]);
                let Ok(mut parser) = parser_.write() else {
                    log_err!("error getting vt100_yh parser");
                    break;
                };
                parser.process(&processed_buf);

                // Clear the processed portion of the buffer
                processed_buf.clear();
            }
            Ok::<(), Error>(())
        });

        // NOTE can only take the writer once
        let writer = BufWriter::new(pty_pair.master.take_writer()?);

        let cur = DrawCh::new(
            ChPlus::Transparent,
            Style::default().with_fg(Color::BLACK).with_bg(Color::WHITE),
        );

        let out = Self {
            pane,
            parser,
            master_pty: Rc::new(RefCell::new(pty_pair.master)),
            writer: Rc::new(RefCell::new(writer)),
            disable_cursor: Rc::new(RefCell::new(false)),
            cursor: Rc::new(RefCell::new(cur)),
            prev_draw: Rc::new(RefCell::new(Vec::new())),
            pty_killer: Rc::new(RefCell::new(killer)),
        };
        out.pane
            .set_focused_receivable_events(out.focused_rec_evs());
        out.pane.set_always_receivable_events(out.always_rec_evs());
        Ok(out)
    }

    pub fn focused_rec_evs(&self) -> ReceivableEvents {
        vec![(KeyPossibility::Anything.into())].into()
    }

    pub fn always_rec_evs(&self) -> ReceivableEvents {
        vec![(ReceivableEvent::Custom(Self::custom_destruct_event_name(self.id())))].into()
    }

    pub fn custom_destruct_event_name(id: ElementID) -> String {
        format!("destruct_{id}")
    }

    pub fn with_height(self, h: DynVal) -> Self {
        self.pane.set_dyn_height(h);
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

    pub fn disable_cursor(&self) {
        *self.disable_cursor.borrow_mut() = true;
    }

    pub fn execute_command<S: Into<String>>(&self, cmd: S) {
        for ch in cmd.into().chars() {
            let key_ev = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::empty());
            let _ = self.handle_pane_key_event(&key_ev);
        }
        let key_ev = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let _ = self.handle_pane_key_event(&key_ev);
    }

    pub fn resize_pty(&self, ctx: &Context) {
        let Ok(mut parser) = self.parser.write() else {
            log_err!("TerminalPane: failed to write to parser");
            return;
        };
        parser
            .screen_mut()
            .set_size(ctx.size.height, ctx.size.width);
        if let Err(e) = self.master_pty.borrow().resize(PtySize {
            rows: ctx.size.height,
            cols: ctx.size.width,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            log_err!("TerminalPane: failed to resize pty: {}", e);
        }
    }

    pub fn handle_pane_mouse_event(&self, mouse: &MouseEvent) -> bool {
        // query for the mouse protocol being used by the terminal
        // either sgr_mouse, normal_mouse, or rxvt_mouse
        let parser = match self.parser.read() {
            Ok(parser) => parser,
            Err(e) => {
                log_err!("TerminalPane: failed to read parser: {}", e);
                return false;
            }
        };
        if (*parser).screen().mouse_protocol_encoding() == vt100_yh::MouseProtocolEncoding::Sgr {
            let input_bz = create_csi_sgr_mouse(*mouse);
            if self.writer.borrow_mut().write_all(&input_bz).is_err() {
                return false;
            }
            if self.writer.borrow_mut().flush().is_err() {
                return false;
            }
        }
        true
    }

    pub fn handle_pane_key_event(&self, key: &KeyEvent) -> bool {
        let input_bz = match key.code {
            KeyCode::Char(ch) => {
                let mut send = vec![ch as u8];
                let upper = ch.to_ascii_uppercase();
                if key.modifiers == KeyModifiers::CONTROL {
                    match upper {
                        // https:///github.com/fyne-io/terminal/blob/master/input.go
                        // https:///gist.github.com/ConnerWill/d4b6c776b509add763e17f9f113fd25b
                        '2' | '@' | ' ' => send = vec![0],
                        '3' | '[' => send = vec![27],
                        '4' | '\\' => send = vec![28],
                        '5' | ']' => send = vec![29],
                        '6' | '^' => send = vec![30],
                        '7' | '-' | '_' => send = vec![31],
                        char if ('A'..='_').contains(&char) => {
                            // Since A == 65,
                            // we can safely subtract 64 to get the corresponding control character
                            let ascii_val = char as u8;
                            let ascii_to_send = ascii_val - 64;
                            send = vec![ascii_to_send];
                        }
                        _ => {}
                    }
                }
                send
            }

            #[cfg(unix)]
            KeyCode::Enter => vec![b'\r'],
            #[cfg(windows)]
            KeyCode::Enter => vec![b'\r', b'\n'],

            //KeyCode::Backspace => vec![8],
            // 0x7F is the ASCII code for delete, but seems to work as backspace for crossterm
            // TODO possibly use 8 for windows?
            KeyCode::Backspace => vec![0x7F],

            KeyCode::Left => vec![27, 91, 68],
            KeyCode::Right => vec![27, 91, 67],
            KeyCode::Up => vec![27, 91, 65],
            KeyCode::Down => vec![27, 91, 66],
            KeyCode::Tab => vec![9],
            KeyCode::Home => vec![27, 91, 72],
            KeyCode::End => vec![27, 91, 70],
            KeyCode::PageUp => vec![27, 91, 53, 126],
            KeyCode::PageDown => vec![27, 91, 54, 126],
            KeyCode::BackTab => vec![27, 91, 90],
            KeyCode::Delete => vec![27, 91, 51, 126],
            KeyCode::Insert => vec![27, 91, 50, 126],
            KeyCode::Esc => vec![27],
            _ => return true, // ignore key but still capture
        };

        // if there is an error here, the pty has been closed, therefor do not capture the event. this
        // could happen in the split second between the pty being closed and the exit event being
        // received by this terminal pane.
        if self.writer.borrow_mut().write_all(&input_bz).is_err() {
            return false;
        }
        if self.writer.borrow_mut().flush().is_err() {
            return false;
        }
        true
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for TerminalPane {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, resps) = match ev {
            Event::KeyCombo(ref keys) => {
                let captured = self.handle_pane_key_event(&keys[0]);
                (captured, EventResponses::default())
            }
            Event::Mouse(ref mouse) => {
                let captured = self.handle_pane_mouse_event(mouse);
                (captured, EventResponses::default())
            }
            Event::Resize => {
                self.resize_pty(ctx);
                (false, EventResponses::default())
            }
            Event::Exit => {
                // this will error is the pty_killer has already been killed
                // ignore the error
                let _ = self.pty_killer.borrow_mut().kill();
                (false, EventResponses::default())
            }
            Event::Custom(name, _) => {
                if name == Self::custom_destruct_event_name(self.id()) {
                    (true, EventResponse::Destruct.into())
                } else {
                    (false, EventResponses::default())
                }
            }
            _ => (false, EventResponses::default()),
        };

        (captured, resps)
    }
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        let mp_size = self.master_pty.borrow().get_size();
        let resize = if let Ok(mp_size) = mp_size {
            mp_size.rows != ctx.size.height || mp_size.cols != ctx.size.width
        } else {
            true
        };
        if resize {
            self.resize_pty(ctx);
        }

        let mut out = Vec::with_capacity(ctx.size.width as usize * ctx.size.height as usize);

        let Ok(sc) = self.parser.read() else {
            log_err!("TerminalPane: failed to read parser");
            return Vec::new();
        };
        let screen = sc.screen();

        let mut dirty = force_update;
        let mut prev_draw_i = 0;
        let grid = screen.grid();
        for (y, row) in grid.visible_rows().enumerate() {
            if y > ctx.size.height as usize {
                break;
            }
            for (x, screen_cell) in row.cells().enumerate() {
                if x > ctx.size.width as usize {
                    break;
                }

                let fg = screen_cell.fgcolor();
                let bg = screen_cell.bgcolor();
                let ul = screen_cell.ulcolor();
                let ch = if screen_cell.has_contents() {
                    ChPlus::Str(CompactString::new(screen_cell.contents()))
                } else {
                    ChPlus::Char(' ')
                };
                let fg: Color = fg.into();
                let bg: Color = bg.into();
                let ul: Color = ul.into();
                let mut sty = Style::default()
                    .with_fg(fg)
                    .with_bg(bg)
                    .with_underline_color(ul);
                if screen_cell.bold() {
                    sty.attr.bold = true;
                }
                if screen_cell.italic() {
                    sty.attr.italic = true;
                }
                if screen_cell.underline() {
                    sty.attr.underlined = true;
                }
                if screen_cell.inverse() {
                    sty.attr.reverse = true;
                }
                let ch_out = DrawChPos {
                    ch: DrawCh::new(ch, sty),
                    x: x as u16,
                    y: y as u16,
                };
                if !dirty {
                    if let Some(prev_draw) = self.prev_draw.borrow().get(prev_draw_i) {
                        if prev_draw != &ch_out {
                            dirty = true;
                        }
                    } else {
                        dirty = true;
                    }
                    prev_draw_i += 1;
                }
                out.push(ch_out);
                //}
            }
        }

        let hide_cursor = screen.hide_cursor();
        if !*self.disable_cursor.borrow() && !hide_cursor {
            let (y, x) = screen.cursor_position();
            out.push(DrawChPos {
                ch: self.cursor.borrow().clone(),
                x,
                y,
            });
        }

        if dirty {
            DrawUpdate::update(out).into()
        } else {
            Vec::new()
        }
    }
}

// this function takes a MouseEvent and returns the bytes that represent the mouse input.
pub fn create_csi_sgr_mouse(ev: MouseEvent) -> Vec<u8> {
    let mut kind = ev.kind;
    let modifiers = ev.modifiers;

    // shenanigans because up buttons aren't encoded
    let is_up = matches!(kind, MouseEventKind::Up(_));
    if is_up {
        kind = match kind {
            MouseEventKind::Up(button) => MouseEventKind::Down(button),
            _ => kind,
        };
    }

    let cb = create_cb(kind, modifiers);
    let cx = ev.column + 1;
    let cy = ev.row + 1;

    let out = format!(
        "\x1B[<{};{};{}{}",
        cb,
        cx,
        cy,
        if is_up { "m" } else { "M" }
    );
    out.into_bytes()
}

// this function takes a MouseEventKind and KeyModifiers and returns the byte that represents the
// mouse input.
pub fn create_cb(kind: MouseEventKind, modifiers: KeyModifiers) -> u8 {
    let (button_number, dragging) = match kind {
        MouseEventKind::Down(MouseButton::Left) => (0, false),
        MouseEventKind::Down(MouseButton::Middle) => (1, false),
        MouseEventKind::Down(MouseButton::Right) => (2, false),
        MouseEventKind::Drag(MouseButton::Left) => (0, true),
        MouseEventKind::Drag(MouseButton::Middle) => (1, true),
        MouseEventKind::Drag(MouseButton::Right) => (2, true),
        MouseEventKind::Up(MouseButton::Left) => (3, false),
        MouseEventKind::Up(MouseButton::Middle) => (3, false), // don't know if this is correct, crossterm doesn't parse this
        MouseEventKind::Up(MouseButton::Right) => (3, false), // don't know if this is correct, crossterm doesn't parse this
        MouseEventKind::Moved => (3, true),
        MouseEventKind::ScrollUp => (4, false),
        MouseEventKind::ScrollDown => (5, false),
        MouseEventKind::ScrollLeft => (6, false),
        MouseEventKind::ScrollRight => (7, false),
    };

    let mut cb = (button_number & 0b0000_0011) | ((button_number << 4) & 0b1100_0000);
    if dragging {
        cb |= 0b0010_0000;
    }

    if modifiers.contains(KeyModifiers::SHIFT) {
        cb |= 0b0000_0100;
    }
    if modifiers.contains(KeyModifiers::ALT) {
        cb |= 0b0000_1000;
    }
    if modifiers.contains(KeyModifiers::CONTROL) {
        cb |= 0b0001_0000;
    }

    cb
}
