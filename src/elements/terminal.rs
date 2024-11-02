// Original inspiration for this element taken from:
// https://github.com/a-kenji/tui-term/blob/development/examples/smux.rs
// (MIT LICENSE)

use {
    crate::{
        ChPlus, Color, Context, DrawCh, DrawChPos, DynLocationSet, DynVal, Element, ElementID,
        Event, EventResponse, EventResponses, KeyPossibility, Pane, Parent, Priority,
        ReceivableEventChanges, SelfReceivableEvents, Style, ZIndex,
    },
    compact_str::CompactString,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    // lazy
    portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize},
    std::{
        cell::RefCell,
        io::{BufWriter, Read, Write},
        rc::Rc,
        sync::{Arc, RwLock},
    },
    tokio::task::spawn_blocking,
};

// TODO use termwiz instead of vt100
//      https://docs.rs/termwiz/latest/termwiz/
//      https://github.com/wez/wezterm/blob/main/termwiz/examples/widgets_nested.rs

// TODO graceful shutdown of tokio tasks

#[derive(Clone)]
pub struct TerminalPane {
    pub pane: Pane,
    pub parser: Arc<RwLock<vt100::Parser>>,
    pub master_pty: Rc<RefCell<Box<dyn MasterPty>>>,
    pub writer: Rc<RefCell<BufWriter<Box<dyn Write + std::marker::Send>>>>,
    pub hide_cursor: Rc<RefCell<bool>>,
    pub cursor: Rc<RefCell<DrawCh>>,
    pub exit: Arc<RwLock<bool>>,

    // the exiters
    pub pty_killer: Rc<RefCell<Box<dyn ChildKiller>>>,
}

impl TerminalPane {
    pub fn new(ctx: &Context) -> Self {
        let cwd = std::env::current_dir().unwrap();
        let mut cmd = CommandBuilder::new_default_prog();
        cmd.cwd(cwd);
        Self::new_with_builder(ctx, cmd)
    }

    pub fn new_with_builder(ctx: &Context, cmd: CommandBuilder) -> Self {
        let size = ctx.s;
        let pane =
            Pane::new(ctx, "terminal_pane").with_self_receivable_events(Self::receivable_events());

        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: size.height,
                cols: size.width,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let parser = Arc::new(RwLock::new(vt100::Parser::new(size.height, size.width, 0)));

        let exit = Arc::new(RwLock::new(false));
        let exit_ = exit.clone();
        let mut child = pty_pair.slave.spawn_command(cmd).unwrap();
        let killer = child.clone_killer();
        //let mut killer_ = child.clone_killer();
        spawn_blocking(move || {
            // ignore exit status
            // NOTE this wait can be killed by the killer
            let _ = child.wait();
            drop(pty_pair.slave);
            *exit_.write().unwrap() = true;
        });

        let mut reader = pty_pair.master.try_clone_reader().unwrap();
        let parser_ = parser.clone();

        spawn_blocking(move || {
            //debug!("Terminal 2nd thread started");
            let mut processed_buf = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                let size = reader.read(&mut buf).unwrap();
                if size == 0 {
                    //killer_.kill().unwrap();
                    break;
                }
                processed_buf.extend_from_slice(&buf[..size]);
                parser_.write().unwrap().process(&processed_buf);

                // Clear the processed portion of the buffer
                processed_buf.clear();
            }
            //debug!("Terminal 2st thread complete")
        });

        // NOTE can only take the writer once
        let writer = BufWriter::new(pty_pair.master.take_writer().unwrap());

        let cur = DrawCh::new(
            ChPlus::Transparent,
            Style::default().with_fg(Color::BLACK).with_bg(Color::WHITE),
        );

        Self {
            pane,
            parser,
            master_pty: Rc::new(RefCell::new(pty_pair.master)),
            writer: Rc::new(RefCell::new(writer)),
            hide_cursor: Rc::new(RefCell::new(false)),
            cursor: Rc::new(RefCell::new(cur)),
            exit,
            pty_killer: Rc::new(RefCell::new(killer)),
        }
    }

    pub fn receivable_events() -> SelfReceivableEvents {
        vec![(KeyPossibility::Anything.into(), Priority::Focused)].into()
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
}

impl Element for TerminalPane {
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
        //debug!("TerminalPane({}) receive_event_inner: {:?}", self.id(), ev);
        if *self.exit.read().unwrap() {
            return (false, EventResponse::Destruct.into());
        }
        let mut captured = false;
        match ev {
            Event::KeyCombo(ref keys) => {
                captured = true;
                //debug!(
                //    "TerminalPane({}) receive_event_inner: {:?}",
                //    self.id(),
                //    keys
                //);
                handle_pane_key_event(self, &keys[0]); // handle empty case?
            }
            Event::Resize => {
                self.parser
                    .write()
                    .unwrap()
                    .set_size(ctx.s.height, ctx.s.width);
                self.master_pty
                    .borrow()
                    .resize(PtySize {
                        rows: ctx.s.height,
                        cols: ctx.s.width,
                        pixel_width: 0,
                        pixel_height: 0,
                    })
                    .unwrap();
            }
            Event::Exit => {
                self.pty_killer.borrow_mut().kill().unwrap();
            }
            _ => {}
        }
        (captured, EventResponses::default())
    }

    fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(p)
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        if *self.exit.read().unwrap() {
            self.pane
                .send_responses_upward(ctx, EventResponse::Destruct.into());
            return Vec::with_capacity(0);
        }
        //let mut cursor = Cursor::default();
        //cursor.hide();
        let mut out = vec![];

        let sc = self.parser.read().unwrap();
        let screen = sc.screen();

        //pub fn handle<S: Screen>(term: &PseudoTerminal<S>, area: Rect, buf: &mut Buffer) {
        let cols = ctx.s.width;
        let rows = ctx.s.height;

        // The screen is made out of rows of cells
        for row in 0..rows {
            for col in 0..cols {
                if row > ctx.s.height || col > ctx.s.width {
                    continue;
                }

                if let Some(screen_cell) = screen.cell(row, col) {
                    let fg = screen_cell.fgcolor();
                    let bg = screen_cell.bgcolor();
                    let ch = if screen_cell.has_contents() {
                        ChPlus::Str(CompactString::new(screen_cell.contents()))
                    } else {
                        ChPlus::Char(' ')
                    };
                    let fg: Color = fg.into();
                    let bg: Color = bg.into();
                    let mut sty = Style::default().with_fg(fg).with_bg(bg);
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
                    out.push(DrawChPos {
                        ch: DrawCh::new(ch, sty),
                        x: col,
                        y: row,
                    });
                }
            }
        }

        if !*self.hide_cursor.borrow() {
            let (y, x) = screen.cursor_position();
            out.push(DrawChPos {
                ch: self.cursor.borrow().clone(),
                x,
                y,
            });
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

pub fn handle_pane_key_event(pane: &TerminalPane, key: &KeyEvent) {
    let input_bytes = match key.code {
        KeyCode::Char(ch) => {
            let mut send = vec![ch as u8];
            let upper = ch.to_ascii_uppercase();
            if key.modifiers == KeyModifiers::CONTROL {
                match upper {
                    // https://github.com/fyne-io/terminal/blob/master/input.go
                    // https://gist.github.com/ConnerWill/d4b6c776b509add763e17f9f113fd25b
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
        KeyCode::Enter => vec![b'\n'],
        #[cfg(windows)]
        KeyCode::Enter => vec![b'\r', b'\n'],

        KeyCode::Backspace => vec![8],
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
        _ => return,
    };

    pane.writer.borrow_mut().write_all(&input_bytes).unwrap();
    pane.writer.borrow_mut().flush().unwrap();
}
