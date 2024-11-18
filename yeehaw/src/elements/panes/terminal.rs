/// Original inspiration for this element taken from:
/// https:///github.com/a-kenji/tui-term/blob/development/examples/smux.rs
/// (MIT LICENSE)
use {
    crate::{
        ChPlus, Color, Context, DrawCh, DrawChPos, DynLocationSet, DynVal, Element, ElementID,
        Event, EventResponse, EventResponses, KeyPossibility, Pane, Parent, Priority,
        ReceivableEvent, ReceivableEventChanges, SelfReceivableEvents, Style, ZIndex,
    },
    compact_str::CompactString,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize},
    std::{
        cell::RefCell,
        io::{BufWriter, Read, Write},
        rc::Rc,
        sync::{Arc, RwLock},
    },
    tokio::task::spawn_blocking,
};

/// TODO use termwiz instead of vt100
///      https:///docs.rs/termwiz/latest/termwiz/
///      https:///github.com/wez/wezterm/blob/main/termwiz/examples/widgets_nested.rs

/// TODO graceful shutdown of tokio tasks

#[derive(Clone)]
pub struct TerminalPane {
    pub pane: Pane,
    pub parser: Arc<RwLock<vt100::Parser>>,
    pub master_pty: Rc<RefCell<Box<dyn MasterPty>>>,
    pub writer: Rc<RefCell<BufWriter<Box<dyn Write + std::marker::Send>>>>,
    pub hide_cursor: Rc<RefCell<bool>>,
    pub cursor: Rc<RefCell<DrawCh>>,

    pub pty_killer: Rc<RefCell<Box<dyn ChildKiller>>>,
}

impl TerminalPane {
    pub const KIND: &'static str = "terminal_pane";

    pub fn new(ctx: &Context) -> Self {
        let cwd = std::env::current_dir().unwrap();
        let mut cmd = CommandBuilder::new_default_prog();
        cmd.cwd(cwd);
        Self::new_with_builder(ctx, cmd)
    }

    pub fn new_with_builder(ctx: &Context, cmd: CommandBuilder) -> Self {
        let size = ctx.s;
        let pane = Pane::new(ctx, Self::KIND);

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

        let mut child = pty_pair.slave.spawn_command(cmd).unwrap();
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

        let out = Self {
            pane,
            parser,
            master_pty: Rc::new(RefCell::new(pty_pair.master)),
            writer: Rc::new(RefCell::new(writer)),
            hide_cursor: Rc::new(RefCell::new(false)),
            cursor: Rc::new(RefCell::new(cur)),
            pty_killer: Rc::new(RefCell::new(killer)),
        };
        out.pane.set_self_receivable_events(out.receivable_events());
        out
    }

    pub fn receivable_events(&self) -> SelfReceivableEvents {
        vec![
            (KeyPossibility::Anything.into(), Priority::Focused),
            (
                ReceivableEvent::Custom(Self::custom_destruct_event_name(self.id())),
                Priority::Focused,
            ),
        ]
        .into()
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
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for TerminalPane {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::KeyCombo(ref keys) => {
                let captured = handle_pane_key_event(self, &keys[0]);
                // handle empty case?
                return (captured, EventResponses::default());
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
                // this will error is the pty_killer has already been killed
                // ignore the error
                let _ = self.pty_killer.borrow_mut().kill();
            }
            Event::Custom(name, _) => {
                if name == Self::custom_destruct_event_name(self.id()) {
                    return (true, EventResponse::Destruct.into());
                }
            }
            _ => {}
        }
        (false, EventResponses::default())
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mp_size = self.master_pty.borrow().get_size();
        let resize = if let Ok(mp_size) = mp_size {
            mp_size.rows != ctx.s.height || mp_size.cols != ctx.s.width
        } else {
            true
        };
        if resize {
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

        let mut out = vec![];

        let sc = self.parser.read().unwrap();
        let screen = sc.screen();

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
}

pub fn handle_pane_key_event(pane: &TerminalPane, key: &KeyEvent) -> bool {
    let input_bytes = match key.code {
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
        _ => return true, // ignore key but still capture
    };

    // if there is an error here, the pty has been closed, therefor do not capture the event. this
    // could happen in the split second between the pty being closed and the exit event being
    // received by this terminal pane.
    if pane.writer.borrow_mut().write_all(&input_bytes).is_err() {
        return false;
    }
    if pane.writer.borrow_mut().flush().is_err() {
        return false;
    }
    true
}
