use {
    crate::{
        element::ReceivableEventChanges, Context, DrawChPos, DynLocationSet, DynVal, Element,
        ElementID, Event, EventResponses, KeyPossibility, Pane, Parent, Priority, SortingHat,
        ZIndex,
    },
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize},
    std::{cell::RefCell, rc::Rc},
    std::{
        io::{self, BufWriter, Read, Write},
        sync::{Arc, RwLock},
    },
    tokio::task::spawn_blocking,
};

#[derive(Clone)]
pub struct TerminalPane {
    pub pane: Pane,
    parser: Arc<RwLock<vt100::Parser>>,
    master_pty: Rc<RefCell<Box<dyn MasterPty>>>,
    writer: Rc<RefCell<BufWriter<Box<dyn Write + std::marker::Send>>>>,
}

impl TerminalPane {
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

    pub fn new(hat: &SortingHat, ctx: &Context, cmd: CommandBuilder) -> io::Result<Self> {
        //let cwd = std::env::current_dir().unwrap();
        //let mut cmd = CommandBuilder::new_default_prog();
        //cmd.cwd(cwd);
        //let mut cmd = CommandBuilder::new("nvim");
        //if let Ok(cwd) = std::env::current_dir() {
        //    cmd.cwd(cwd);
        //}

        let size = ctx.s;
        let pane = Pane::new(hat, "terminal_pane");

        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: size.height - 2,
                cols: size.width - 2,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let parser = Arc::new(RwLock::new(vt100::Parser::new(
            size.height - 2,
            size.width - 2,
            0,
        )));

        // XXX need a runtime from context
        spawn_blocking(move || {
            let mut child = pty_pair.slave.spawn_command(cmd).unwrap();
            let _ = child.wait(); // ignore exit status
            drop(pty_pair.slave);
            println!("Child process exited");

            // XXX need to destruct this pane now... introduce a arc
            // rw exit variable
        });

        let mut reader = pty_pair.master.try_clone_reader().unwrap();
        let parser_ = parser.clone();
        // XXX need a runtime from context
        tokio::spawn(async move {
            let mut processed_buf = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                let size = reader.read(&mut buf).unwrap();
                if size == 0 {
                    break;
                }
                processed_buf.extend_from_slice(&buf[..size]);
                parser_.write().unwrap().process(&processed_buf);

                // Clear the processed portion of the buffer
                processed_buf.clear();
            }
        });

        // NOTE can only take the writer once
        let writer = BufWriter::new(pty_pair.master.take_writer().unwrap());

        Ok(Self {
            pane,
            parser,
            master_pty: Rc::new(RefCell::new(pty_pair.master)),
            writer: Rc::new(RefCell::new(writer)),
        })
    }
}

impl Element for TerminalPane {
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
        match ev {
            Event::KeyCombo(ref keys) => {
                if let KeyPossibility::Key(key) = &keys[0] {
                    handle_pane_key_event(self, key);
                }
            }
            Event::Resize => {
                self.parser
                    .write()
                    .unwrap()
                    .set_size(ctx.s.height - 2, ctx.s.width - 2);
                self.master_pty
                    .borrow()
                    .resize(PtySize {
                        rows: ctx.s.height - 2,
                        cols: ctx.s.width - 2,
                        pixel_width: 0,
                        pixel_height: 0,
                    })
                    .unwrap();
            }
            _ => {}
        }
        self.pane.receive_event(ctx, ev.clone())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        //let mut cursor = Cursor::default();
        //cursor.hide();

        let screen = self.parser.read().unwrap().screen();
        //let pseudo_term = PseudoTerminal::new(screen).cursor(cursor);

        // XXX need to draw the screen

        self.pane.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn Parent>) {
        self.pane.set_upward_propagator(up)
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
