/// TerminalPane2 is a terminal pane which draws passively.
///
/// Unlike TerminalPane, which actively queries the full state of the
/// terminal on every draw and recomputes the changes, TerminalPane2
/// receives the specific changes from vt100_yh as they happen (via the
/// reader thread) and applies only those changes to the inner Pane's
/// content. The Pane then draws passively when its content is marked
/// dirty, the same way the Button element does.
use {
    super::terminal::{cell_to_drawch, create_csi_sgr_mouse},
    crate::*,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize},
    std::{
        io::{BufWriter, Read, Write},
        sync::{mpsc, Arc, RwLock},
    },
    tokio::task::spawn_blocking,
};

/// A specific change to the terminal screen, as reported by vt100_yh.
#[derive(Clone, Debug)]
pub struct TerminalUpdate {
    /// the changed visible rows, or `None` if the entire visible screen
    /// has changed
    pub rows: Option<Vec<u16>>,
    /// the cursor position (row, col), or `None` if the cursor is hidden
    pub cursor: Option<(u16, u16)>,
}

#[derive(Clone)]
pub struct TerminalPane2 {
    pub pane: Pane,
    pub parser: Arc<RwLock<vt100_yh::Parser>>,
    pub master_pty: Rc<RefCell<Box<dyn MasterPty>>>,
    pub writer: Rc<RefCell<BufWriter<Box<dyn Write + std::marker::Send>>>>,
    pub disable_cursor: Rc<RefCell<bool>>,
    pub cursor: Rc<RefCell<DrawCh>>,

    /// updates from the reader thread
    pub updates_rx: Rc<mpsc::Receiver<TerminalUpdate>>,
    /// the position (row, col) of the cursor overlay within the content
    /// matrix, or `None` if no cursor overlay is present
    pub cursor_pos: Rc<RefCell<Option<(u16, u16)>>>,

    pub pty_killer: Rc<RefCell<Box<dyn ChildKiller>>>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl TerminalPane2 {
    pub const KIND: &'static str = "terminal_pane2";

    pub fn new(ctx: &Context) -> Result<Self, Error> {
        let cwd = std::env::current_dir()?;
        let mut cmd = CommandBuilder::new_default_prog();
        cmd.cwd(cwd);
        Self::new_with_builder(ctx, cmd)
    }

    pub fn new_with_builder(ctx: &Context, cmd: CommandBuilder) -> Result<Self, Error> {
        // need this as the pty will not open if the size is 0
        let size = Size::new(10, 10);

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
            1000,
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
        let (updates_tx, updates_rx) = mpsc::channel();

        spawn_blocking(move || {
            let mut processed_buf = Vec::new();
            let mut buf = [0u8; 8192];
            let mut last_cursor = None;
            loop {
                let size = reader.read(&mut buf)?;
                if size == 0 {
                    break;
                }
                processed_buf.extend_from_slice(&buf[..size]);
                let Ok(mut parser) = parser_.write() else {
                    log_err!("TerminalPane2: error getting vt100_yh parser");
                    break;
                };
                parser.process(&processed_buf);

                // Clear the processed portion of the buffer
                processed_buf.clear();

                // collect the specific changes made by vt100_yh and send
                // them to the drawing thread
                if let Some(upd) = collect_terminal_update(&mut parser, &mut last_cursor) {
                    // an error here means the pane has been dropped
                    if tx_send(&updates_tx, upd) {
                        break;
                    }
                }
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
            updates_rx: Rc::new(updates_rx),
            cursor_pos: Rc::new(RefCell::new(None)),
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

    pub fn set_env<S: Into<String>, S2: Into<String>>(&self, key: S, val: S2) {
        let key = key.into();
        let val = val.into();
        let cmd = format!("export {}={}", key, val);
        self.execute_command(cmd);
    }

    pub fn resize_pty(&self, dr: &DrawRegion) {
        let Ok(mut parser) = self.parser.write() else {
            log_err!("TerminalPane2: failed to write to parser");
            return;
        };
        parser.screen_mut().set_size(dr.size.height, dr.size.width);
        if let Err(e) = self.master_pty.borrow().resize(PtySize {
            rows: dr.size.height,
            cols: dr.size.width,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            log_err!("TerminalPane2: failed to resize pty: {}", e);
        }
    }

    pub fn handle_pane_mouse_event(&self, mouse: &MouseEvent) -> bool {
        // query for the mouse protocol being used by the terminal
        // either sgr_mouse, normal_mouse, or rxvt_mouse
        let parser = match self.parser.read() {
            Ok(parser) => parser,
            Err(e) => {
                log_err!("TerminalPane2: failed to read parser: {}", e);
                return false;
            }
        };
        if (*parser).screen().mouse_protocol_encoding() == vt100_yh::MouseProtocolEncoding::Sgr {
            let input_bz = create_csi_sgr_mouse(mouse);
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
impl Element for TerminalPane2 {
    fn receive_event(&self, _: &Context, ev: Event) -> (bool, EventResponses) {
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
                // resize will be handled in the drawing function
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

    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        let mp_size = self.master_pty.borrow().get_size();
        let resize = if let Ok(mp_size) = mp_size {
            mp_size.rows != dr.size.height || mp_size.cols != dr.size.width
        } else {
            true
        };
        if resize {
            self.resize_pty(dr);
        }

        let w = dr.size.width as usize;
        let h = dr.size.height as usize;
        let needs_full = {
            let content = self.pane.get_content();
            content.width() != w || content.height() != h
        };
        if needs_full {
            // (re)build the entire content matrix from the current screen
            // state. this is only done on the first draw and on resizes.
            if let Ok(parser) = self.parser.read() {
                let screen = parser.screen();
                let mut new_content = DrawChs2D::new_empty_of_size(w, h, Style::default());
                let grid = screen.grid();
                for (y, row) in grid.visible_rows().enumerate() {
                    if y >= h {
                        break;
                    }
                    for (x, cell) in row.cells().enumerate() {
                        if x >= w {
                            break;
                        }
                        new_content.0[y][x] = cell_to_drawch(cell);
                    }
                }
                self.pane.set_content(new_content);
                // the cursor overlay will be re-tracked below
                *self.cursor_pos.borrow_mut() = None;
            }
        }

        // apply the specific changes provided by vt100_yh
        if let Ok(first) = self.updates_rx.try_recv() {
            let Ok(parser) = self.parser.read() else {
                log_err!("TerminalPane2: failed to read parser");
                return self.pane.drawing(ctx, dr, force_update);
            };
            let screen = parser.screen();
            let cursor_ch = self.cursor.borrow().clone();
            let mut cursor_pos = self.cursor_pos.borrow_mut();
            let mut content = self.pane.get_content_mut();
            let updates = std::iter::once(first)
                .chain(std::iter::from_fn(|| self.updates_rx.try_recv().ok()));
            for upd in updates {
                apply_terminal_update(
                    screen,
                    &upd,
                    &mut content,
                    dr.size.width,
                    dr.size.height,
                    &cursor_ch,
                    &mut cursor_pos,
                );
            }
        }

        // the Pane draws passively from its content when the content is
        // dirty, the same way the Button element does
        self.pane.drawing(ctx, dr, force_update)
    }
}

/// Collects the specific changes made by vt100_yh since the last call,
/// if any.
///
/// `last_cursor` tracks the cursor position which was last sent, so that
/// cursor-only changes are also reported.
fn collect_terminal_update(
    parser: &mut vt100_yh::Parser,
    last_cursor: &mut Option<(u16, u16)>,
) -> Option<TerminalUpdate> {
    let screen = parser.screen_mut();
    if !screen.is_dirty() {
        return None;
    }
    let rows = screen.take_changed_rows();
    let cursor = if screen.hide_cursor() {
        None
    } else {
        Some(screen.cursor_position())
    };
    let cursor_changed = *last_cursor != cursor;
    *last_cursor = cursor;
    let rows_changed = match &rows {
        None => true,
        Some(r) => !r.is_empty(),
    };
    screen.set_dirty(false);
    if rows_changed || cursor_changed {
        Some(TerminalUpdate { rows, cursor })
    } else {
        None
    }
}

/// Applies a terminal update to the content matrix.
///
/// `cursor_pos` tracks the position (row, col) of the cursor overlay
/// currently present in the content matrix.
fn apply_terminal_update(
    screen: &vt100_yh::Screen,
    upd: &TerminalUpdate,
    content: &mut DrawChs2D,
    width: u16,
    height: u16,
    cursor_ch: &DrawCh,
    cursor_pos: &mut Option<(u16, u16)>,
) {
    match &upd.rows {
        None => {
            // the entire visible screen has changed, rebuild the content
            let grid = screen.grid();
            for (y, row) in grid.visible_rows().enumerate() {
                if y >= height as usize {
                    break;
                }
                for (x, cell) in row.cells().enumerate() {
                    if x >= width as usize {
                        break;
                    }
                    content.0[y][x] = cell_to_drawch(cell);
                }
            }
            // the cursor overlay will be redrawn below
            *cursor_pos = None;
        }
        Some(rows) => {
            for &y in rows {
                if y >= height {
                    continue;
                }
                for x in 0..width {
                    if let Some(cell) = screen.cell(y, x) {
                        let dch = cell_to_drawch(cell);
                        if content.0[y as usize][x as usize] != dch {
                            content.0[y as usize][x as usize] = dch;
                        }
                    }
                }
            }
        }
    }

    // restore the cell underneath the old cursor overlay if the cursor
    // moved or was hidden
    if let Some((oy, ox)) = *cursor_pos {
        if Some((oy, ox)) != upd.cursor && oy < height && ox < width {
            if let Some(cell) = screen.cell(oy, ox) {
                content.0[oy as usize][ox as usize] = cell_to_drawch(cell);
            }
            *cursor_pos = None;
        }
    }

    // draw the new cursor overlay
    // simplification: a cursor position beyond the content bounds is not
    // drawn; this can happen when the cursor sits at the end of a row
    // (col == width). the upgrade path would be to draw it in the first
    // cell of the next row, as a real terminal would.
    if let Some((cy, cx)) = upd.cursor {
        if cy < height && cx < width {
            content.0[cy as usize][cx as usize] = cursor_ch.clone();
            *cursor_pos = Some((cy, cx));
        }
    }
}

/// Returns `true` if the send failed (the receiver has been dropped).
fn tx_send(tx: &mpsc::Sender<TerminalUpdate>, upd: TerminalUpdate) -> bool {
    tx.send(upd).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::panes::terminal::active_full_draw;

    fn cursor_drawch() -> DrawCh {
        DrawCh::new(
            ChPlus::Transparent,
            Style::default().with_fg(Color::BLACK).with_bg(Color::WHITE),
        )
    }

    /// A workload resembling a shell running `sl`: mostly small row-level
    /// changes and cursor moves, with occasional full-screen operations and
    /// idle frames (no output) mixed in.
    fn sl_like_workload() -> Vec<Vec<u8>> {
        let mut frames = vec![
            b"user@host:~$ ".to_vec(),
            b"sl\r\n".to_vec(),
            vec![], // idle
            b"\x1b[2J\x1b[H".to_vec(), // full clear
        ];
        for i in 0..40u16 {
            if i % 10 == 7 {
                // idle frame
                frames.push(vec![]);
            } else if i % 10 == 9 {
                // full clear + redraw (like the start of an sl animation frame)
                frames.push(
                    format!(
                        "\x1b[2J\x1b[Htrain frame {}\r\n  (o)___(o)_____(_)\r\n",
                        i
                    )
                    .into_bytes(),
                );
            } else if i % 7 == 3 {
                // cursor-only movement
                frames.push(format!("\x1b[{};{}H", i % 20 + 1, i % 60 + 1).into_bytes());
            } else {
                // a few rows of art redrawn in place
                frames.push(
                    format!(
                        "\x1b[{};1Hsl frame {}\r\n\x1b[{};1H(o)___(o)_____(_)\r\n",
                        i % 10,
                        i,
                        (i % 10) + 1
                    )
                    .into_bytes(),
                );
            }
        }
        frames.push(b"\x1b[?25l".to_vec()); // hide cursor
        frames.push(b"\x1b[?25h".to_vec()); // show cursor
        frames.push(b"\x1b[5;5H".to_vec()); // cursor-only movement
        // wide chars and colors on single rows
        frames.push("\x1b[3;1H\x1b[31mred \x1b[32mgreen \x1b[0m\u{65e5}\u{672c}\u{8a9e}\r\n".as_bytes().to_vec());
        // output past the bottom of the screen (scrolls)
        frames.push(
            (0..30)
                .map(|i| format!("scroll line {}\r\n", i))
                .collect::<String>()
                .into_bytes(),
        );
        frames
    }

    /// Mirrors the drawing logic of TerminalPane: actively queries and
    /// re-diffs the full screen on every frame.
    struct ActiveConsumer {
        parser: vt100_yh::Parser,
        prev_draw: Vec<DrawChPos>,
        cells_processed: u64,
        last_draw: Option<Vec<DrawChPos>>,
    }

    impl ActiveConsumer {
        fn new(rows: u16, cols: u16) -> Self {
            Self {
                parser: vt100_yh::Parser::new(rows, cols, 1000),
                prev_draw: vec![],
                cells_processed: 0,
                last_draw: None,
            }
        }

        fn feed_and_draw(&mut self, bytes: &[u8], size: &Size, cursor: &DrawCh) {
            if !bytes.is_empty() {
                self.parser.process(bytes);
            }
            let screen = self.parser.screen_mut();
            if !screen.is_dirty() {
                return;
            }
            let out = {
                let (out, _changed) = active_full_draw(screen, size, &mut self.prev_draw);
                out
            };
            self.cells_processed += out.len() as u64;
            let mut out = out;
            if !screen.hide_cursor() {
                let (y, x) = screen.cursor_position();
                out.push(DrawChPos {
                    ch: cursor.clone(),
                    x,
                    y,
                });
            }
            screen.set_dirty(false);
            self.last_draw = Some(out);
        }

        /// the currently visible state, clipped to the pane region
        fn visible(&self, w: u16, h: u16) -> Vec<DrawCh> {
            let mut cells = vec![DrawCh::default(); (w as usize) * (h as usize)];
            if let Some(out) = &self.last_draw {
                for dcp in out {
                    if dcp.x < w && dcp.y < h {
                        cells[(dcp.y as usize) * (w as usize) + dcp.x as usize] = dcp.ch.clone();
                    }
                }
            }
            cells
        }
    }

    /// Mirrors the update logic of TerminalPane2: passively applies only
    /// the specific changes provided by vt100_yh.
    struct PassiveConsumer {
        parser: vt100_yh::Parser,
        content: DrawChs2D,
        cursor_pos: Option<(u16, u16)>,
        last_cursor: Option<(u16, u16)>,
        cells_processed: u64,
    }

    impl PassiveConsumer {
        fn new(rows: u16, cols: u16) -> Self {
            Self {
                parser: vt100_yh::Parser::new(rows, cols, 1000),
                content: DrawChs2D::default(),
                cursor_pos: None,
                last_cursor: None,
                cells_processed: 0,
            }
        }

        fn feed_and_draw(&mut self, bytes: &[u8], w: u16, h: u16, cursor: &DrawCh) {
            if !bytes.is_empty() {
                self.parser.process(bytes);
            }
            // first draw / resize: rebuild the entire content matrix
            if self.content.width() != w as usize || self.content.height() != h as usize {
                let screen = self.parser.screen();
                let mut new_content =
                    DrawChs2D::new_empty_of_size(w as usize, h as usize, Style::default());
                let grid = screen.grid();
                for (y, row) in grid.visible_rows().enumerate() {
                    if y >= h as usize {
                        break;
                    }
                    for (x, cell) in row.cells().enumerate() {
                        if x >= w as usize {
                            break;
                        }
                        new_content.0[y][x] = cell_to_drawch(cell);
                    }
                }
                self.cells_processed += ((w as usize) * (h as usize)) as u64;
                self.content = new_content;
                self.cursor_pos = None;
            }
            // apply the specific changes provided by vt100_yh
            while let Some(upd) = collect_terminal_update(&mut self.parser, &mut self.last_cursor) {
                match &upd.rows {
                    None => self.cells_processed += ((w as usize) * (h as usize)) as u64,
                    Some(rows) => {
                        self.cells_processed +=
                            (rows.iter().filter(|&&y| y < h).count() * (w as usize)) as u64
                    }
                }
                let screen = self.parser.screen();
                apply_terminal_update(
                    screen,
                    &upd,
                    &mut self.content,
                    w,
                    h,
                    cursor,
                    &mut self.cursor_pos,
                );
            }
        }

        /// the currently visible state
        fn visible(&self, w: u16, h: u16) -> Vec<DrawCh> {
            (0..h as usize)
                .flat_map(|y| (0..w as usize).map(move |x| self.content.0[y][x].clone()))
                .collect()
        }
    }

    /// Proves that TerminalPane2 produces the same visible output as
    /// TerminalPane over the same stream of terminal bytes.
    #[test]
    fn terminal_pane2_output_matches_terminal_pane() {
        let (w, h) = (80u16, 24u16);
        let size = Size::new(w, h);
        let cursor = cursor_drawch();
        let frames = sl_like_workload();

        let mut active = ActiveConsumer::new(h, w);
        let mut passive = PassiveConsumer::new(h, w);

        for (i, frame) in frames.iter().enumerate() {
            active.feed_and_draw(frame, &size, &cursor);
            passive.feed_and_draw(frame, w, h, &cursor);
            assert_eq!(
                active.visible(w, h),
                passive.visible(w, h),
                "visible output mismatch at frame {}",
                i
            );
        }
    }

    /// Proves that the passive TerminalPane2 approach processes
    /// significantly fewer cells than the active TerminalPane approach.
    #[test]
    fn terminal_pane2_processes_fewer_cells_than_terminal_pane() {
        let (w, h) = (80u16, 24u16);
        let size = Size::new(w, h);
        let cursor = cursor_drawch();
        let frames = sl_like_workload();

        let mut active = ActiveConsumer::new(h, w);
        let mut passive = PassiveConsumer::new(h, w);

        for frame in &frames {
            active.feed_and_draw(frame, &size, &cursor);
            passive.feed_and_draw(frame, w, h, &cursor);
        }

        eprintln!(
            "cells processed: active (TerminalPane) = {}, passive (TerminalPane2) = {}",
            active.cells_processed, passive.cells_processed
        );
        assert!(
            passive.cells_processed > 0,
            "passive consumer should process at least some cells"
        );
        assert!(
            passive.cells_processed * 4 < active.cells_processed,
            "passive drawing should process significantly fewer cells: passive = {}, active = {}",
            passive.cells_processed,
            active.cells_processed
        );
    }
}
