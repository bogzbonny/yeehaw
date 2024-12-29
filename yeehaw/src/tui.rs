use {
    crate::{
        keyboard::Keyboard, ChPlus, ColorStore, Context, DrawingCache, DynLocation, DynLocationSet,
        Element, ElementID, ElementOrganizer, Error, Event, EventResponse, EventResponses, Parent,
        SortingHat,
    },
    crossterm::{
        cursor::{self, MoveTo},
        event::{
            DisableMouseCapture, EnableMouseCapture, Event as CTEvent, EventStream,
            KeyEvent as CTKeyEvent, MouseEvent as CTMouseEvent,
        },
        execute, queue, style,
        style::{ContentStyle, StyledContent},
        terminal,
    },
    futures::{future::FutureExt, StreamExt},
    std::collections::HashMap,
    std::io::{stdout, Write},
    std::{cell::RefCell, rc::Rc},
    tokio::sync::mpsc::{Receiver as MpscReceiver, Sender as MpscSender},
    tokio::sync::watch::{Receiver as WatchReceiver, Sender as WatchSender},
    tokio::time::{self, Duration},
};

/// the amount of time the tui will wait in between calls to re-render.
/// If this value is set too large it will give the tui a laggy feel, but if it's set too low then
/// drawing may consume excessive CPU time and starve events from being processed.
/// recommended setting: 10ms (100 frames per second)
const DEFAULT_ANIMATION_SPEED: Duration = Duration::from_millis(10);

/// configuration of a tui instance
pub struct Tui {
    cup: TuiParent,
    main_el_id: ElementID,
    kb: Keyboard,
    launch_instant: std::time::Instant,

    pub drawing_cache: DrawingCache,

    pub animation_speed: Duration,
    pub rendering: bool, // true if currently rendering
    /// the last time the screen was rendered
    pub last_render: std::time::Instant,

    pub kill_on_ctrl_c: bool,

    /// if Some then the TUI is rendered inline in the terminal
    pub inline: Option<Rc<RefCell<InlineTui>>>,

    /// last flushed internal screen, used to determine what needs to be flushed next
    //                            x  , y
    pub sc_last_flushed: HashMap<(u16, u16), StyledContent<ChPlus>>,
    /// true if exit
    pub exit_recv: WatchReceiver<bool>,
    /// event receiver for internally generated events
    pub ev_recv: MpscReceiver<Event>,
}

#[derive(Clone, Copy)]
pub struct InlineTui {
    /// the cursor start row of the tui
    pub cursor_start_row: u16,
    /// the height of the tui to take up inline
    pub tui_height: u16,

    /// last height of the screen
    pub scr_height: u16,
}

impl Tui {
    /// creates a new full screen TUI, also provides a context for sub-element initialization
    pub fn new() -> Result<(Tui, Context), Error> {
        let (exit_tx, exit_recv) = tokio::sync::watch::channel(false);
        let (ev_tx, ev_recv) = tokio::sync::mpsc::channel::<Event>(10); // no idea if this buffer size is right or wrong

        let cup = TuiParent::new(exit_tx, ev_tx);
        let tui = Tui {
            cup,
            main_el_id: "".to_string(),
            kb: Keyboard::default(),
            launch_instant: std::time::Instant::now(),
            drawing_cache: DrawingCache::default(),
            rendering: false,
            last_render: std::time::Instant::now(),
            animation_speed: DEFAULT_ANIMATION_SPEED,
            kill_on_ctrl_c: true,
            inline: None,
            sc_last_flushed: HashMap::new(),
            exit_recv,
            ev_recv,
        };

        let ctx = Context::new_context_for_screen_no_dur(
            &tui.cup.hat,
            tui.cup.ev_tx.clone(),
            &tui.cup.color_store,
        );
        let child_ctx = ctx.child_init_context();
        Ok((tui, child_ctx))
    }

    pub fn context(&self) -> Context {
        let mut ctx = Context::new_context_for_screen(
            self.launch_instant,
            &self.cup.hat,
            self.cup.ev_tx.clone(),
            &self.cup.color_store,
        );
        if let Some(inline) = &self.inline {
            ctx.size.height = inline.borrow().tui_height;
        }
        ctx
    }

    pub async fn run(&mut self, main_el: Box<dyn Element>) -> Result<(), Error> {
        self.main_el_id = main_el.id();
        // add the element here after the location has been created
        let ctx = Context::new_context_for_screen_no_dur(
            &self.cup.hat,
            self.cup.ev_tx.clone(),
            &self.cup.color_store,
        );
        let loc = DynLocation::new_fixed(0, ctx.size.width as i32, 0, ctx.size.height as i32);
        let loc = DynLocationSet::new(loc, vec![], 0);
        main_el.set_dyn_location_set(loc);
        main_el.set_visible(true);
        main_el.set_focused(true);

        // when adding the main element, nil is passed in as the parent
        // this is because the top of the tree is the TUI's main EO and so no parent
        // is necessary
        self.cup
            .eo
            .add_element(main_el.clone(), Some(Box::new(self.cup.clone())));
        self.cup.eo.initialize(&ctx, Box::new(self.cup.clone()));
        self.cup.main_el_id = main_el.id();

        sc_startup()?;
        self.launch().await?;
        sc_closedown()?;
        Ok(())
    }

    /// run the TUI in terminal line
    pub async fn run_in_line(
        &mut self, main_el: Box<dyn Element>, height: u16,
    ) -> Result<(), Error> {
        self.main_el_id = main_el.id();
        // add the element here after the location has been created
        let mut ctx = Context::new_context_for_screen_no_dur(
            &self.cup.hat,
            self.cup.ev_tx.clone(),
            &self.cup.color_store,
        );
        ctx.size.height = height;
        let loc = DynLocation::new_fixed(0, ctx.size.width as i32, 0, ctx.size.height as i32);
        let loc = DynLocationSet::new(loc, vec![], 0);
        main_el.set_dyn_location_set(loc);
        main_el.set_visible(true);
        main_el.set_focused(true);

        // when adding the main element, nil is passed in as the parent
        // this is because the top of the tree is the TUI's main EO and so no parent
        // is necessary
        self.cup
            .eo
            .add_element(main_el.clone(), Some(Box::new(self.cup.clone())));
        self.cup.eo.initialize(&ctx, Box::new(self.cup.clone()));
        self.cup.main_el_id = main_el.id();

        // get the cursor position
        let (_, mut cur_row) = cursor::position()?;
        let scr_height = terminal::size()?.1;
        let offset = if cur_row + height > scr_height { cur_row + height - scr_height } else { 0 };

        cur_row -= offset;
        let inline = Rc::new(RefCell::new(InlineTui {
            cursor_start_row: cur_row,
            tui_height: height,
            scr_height,
        }));
        self.inline = Some(inline.clone());

        // scroll if there isn't enough room in the terminal
        execute!(stdout(), terminal::ScrollUp(offset))?;

        sc_line_startup(inline.clone())?;
        self.launch().await?;
        sc_line_closedown(*inline.borrow())?;

        Ok(())
    }

    async fn launch(&mut self) -> Result<(), Error> {
        let mut reader = EventStream::new();
        self.launch_instant = std::time::Instant::now();

        loop {
            let delay = time::sleep(self.animation_speed).fuse();
            let event = reader.next().fuse();

            tokio::select! {
                ev_res = event => {
                    match ev_res {
                        Some(Ok(ev)) => {
                            match ev {
                                CTEvent::Key(key_ev) => {
                                    //debug!("tui Key event: {:?}", key_ev);
                                    if self.process_event_key(key_ev)? {
                                        break Ok(());
                                    }
                                }
                                CTEvent::Mouse(mouse_ev) => {
                                    if self.process_event_mouse(mouse_ev)? {
                                        break Ok(());
                                    }
                                }

                                CTEvent::Resize(_, _) => {
                                    if let Some(inline) = &mut self.inline {
                                        let scr_height = terminal::size()?.1;
                                        let mut inline = inline.borrow_mut();
                                        let last_scr_height = inline.scr_height;

                                        match scr_height.cmp(&last_scr_height) {
                                            std::cmp::Ordering::Equal => {}
                                            std::cmp::Ordering::Less => {
                                                let diff = last_scr_height - scr_height;
                                                execute!(stdout(), terminal::ScrollDown(diff))?;
                                                inline.scr_height = scr_height;
                                                inline.cursor_start_row = inline.cursor_start_row.saturating_sub(diff);
                                            }
                                            std::cmp::Ordering::Greater => {
                                                let diff = scr_height - last_scr_height;
                                                execute!(stdout(), terminal::ScrollUp(diff))?;

                                                inline.scr_height = scr_height;
                                                inline.cursor_start_row += diff;
                                                let cur_row = inline.cursor_start_row;
                                                let tui_height = inline.tui_height;

                                                // if increasing the screen height, and the tui is not
                                                // fully visible, then move the cursor start row up
                                                let offset = if cur_row + tui_height > scr_height {
                                                    cur_row + tui_height - scr_height } else { 0 };
                                                inline.cursor_start_row = cur_row.saturating_sub(offset);
                                            }
                                        }
                                    }

                                    let ctx = self.context();
                                    let loc = DynLocation::new_fixed(0, ctx.size.width as i32, 0, ctx.size.height as i32);
                                    // There should only be one element at index 0 in the upper level EO
                                    self.cup.eo.update_el_primary_location(self.main_el_id.clone(), loc);
                                    let _ = self.cup.eo.get_element(&self.main_el_id).expect("main element missing").receive_event(&ctx, Event::Resize{});
                                    self.clear_screen()?;
                                }
                                _ => {}
                            }
                            // important to render here to not starve rendering when there
                            // are ample events coming in. Within render it will skip renders if
                            // the animation speed is not met
                            self.render(true)?;
                        }
                        Some(Err(e)) => println!("Error: {e:?}\r"),
                        None => break Ok(()),
                    }
                }

                Some(ev_res) = self.ev_recv.recv() => {
                    let (_, resps) = self.cup.eo.event_process(&self.context(), ev_res, Box::new(self.cup.clone()));
                    if process_event_resps(resps, None, &self.cup.eo, self.main_el_id.clone())? {
                        break Ok(());
                    }
                }

                // exit
                _ = self.exit_recv.changed().fuse() => {
                    if *self.exit_recv.borrow() {
                        break Ok(());
                    }
                }

                _ = delay => {
                    self.render(false)?;
                },
            };
        }
    }

    pub fn close(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// process_event_key handles key events
    ///                                                                 exit-tui
    pub fn process_event_key(&mut self, key_ev: CTKeyEvent) -> Result<bool, Error> {
        self.kb.add_ev(key_ev);

        if key_ev == Keyboard::KEY_CTRL_C && self.kill_on_ctrl_c {
            self.cup
                .eo
                .event_process(&self.context(), Event::Exit, Box::new(self.cup.clone()));
            return Ok(true);
        }

        //debug!("tui Key event: {:?}", key_ev);

        // we only care about the event response as all keys are sent to the main
        // element no matter what
        self.kb.add_ev(key_ev);

        // get the event combos and consume the keyboard events
        // NOTE the element ID is not used from GetDestinationElFromKB as it is
        // re-determined within the KeyEventsProcess (inefficient but convenient)

        let Some((_, evs)) = self.cup.eo.get_destination_el_from_kb(&mut self.kb) else {
            return Ok(false);
        };
        let ctx = self.context();
        //debug!("tui destination: {:?}", dest);

        let (_, resps) =
            self.cup
                .eo
                .routed_event_process(&ctx, evs.into(), Box::new(self.cup.clone()));

        process_event_resps(resps, None, &self.cup.eo, self.main_el_id.clone())
    }

    /// process_event_mouse handles mouse events
    ///                                                                       exit-tui
    pub fn process_event_mouse(&mut self, mut mouse_ev: CTMouseEvent) -> Result<bool, Error> {
        let ctx = self.context();
        if let Some(inline) = &self.inline {
            let inline = inline.borrow();
            if mouse_ev.row < inline.cursor_start_row {
                return Ok(false);
            }
            mouse_ev.row -= inline.cursor_start_row;
            if mouse_ev.row >= inline.tui_height {
                return Ok(false);
            }
        }

        let (_, resps) =
            self.cup
                .eo
                .mouse_event_process(&ctx, &mouse_ev, Box::new(self.cup.clone()));

        process_event_resps(resps, None, &self.cup.eo, self.main_el_id.clone())
    }

    pub fn clear_screen(&mut self) -> Result<(), Error> {
        self.sc_last_flushed.clear();
        let mut sc = stdout();
        execute!(
            sc,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        Ok(())
    }

    /// Render all elements, draws the screen using the DrawChPos array passed to it
    /// by the element organizers.
    ///
    /// NOTE: when the tui calls all_drawing_updates on the top level ElementOrganizer,
    /// all_drawing_updates gets recursively called on every element organizer in the tree.
    /// Each one returns a DrawChPos array which is then passed to the next level up
    /// in the tree, with that level appending its own DrawChPos array to the end.
    /// Render then iterates through the DrawChPos array and sets the content
    /// provided by each element in order from the bottom of the tree to the top.
    /// This results in elements higher up the tree being able to overwrite elements
    /// lower down the tree.
    pub fn render(&mut self, from_event: bool) -> Result<(), Error> {
        // reduce the animation speed requirements if the tui is rendering from an event
        // this is to prevent the tui from feeling laggy when a lot of complex-to-render
        // events are being received
        let delay = if from_event { self.animation_speed * 2 } else { self.animation_speed };

        if self.last_render.elapsed() < delay || self.rendering {
            return Ok(());
        }
        self.rendering = true;

        let mut sc = stdout();
        let ctx = self.context();
        let updates = self.cup.eo.all_drawing_updates(&ctx, false);
        let chs = self.drawing_cache.update_and_get(updates);

        // TODO could be optimized with rayon if we could draw everything that doesn't
        // depend on anything else in seperate passes.
        let mut dedup_chs: HashMap<(u16, u16), StyledContent<ChPlus>> = HashMap::new();
        for c in chs {
            // remove out of bounds
            if c.x >= ctx.size.width || c.y >= ctx.size.height {
                continue;
            }

            // determine the character style, provide the underlying content
            // for alpha considerations
            let prev_sty = if let Some(prev_sty) = dedup_chs.get(&(c.x, c.y)) {
                prev_sty
            } else {
                &StyledContent::new(ContentStyle::default(), ChPlus::Char(' '))
            };
            let content = c.get_content_style(&ctx, prev_sty);
            dedup_chs.insert((c.x, c.y), content);
        }

        let y_offset =
            if let Some(inline) = &self.inline { inline.borrow().cursor_start_row } else { 0 };

        let mut do_flush = false;
        for ((x, y), sty) in dedup_chs {
            let y = y + y_offset;
            if self.is_ch_style_at_position_dirty(x, y, &sty) {
                queue!(
                    &mut sc,
                    MoveTo(x, y),
                    style::PrintStyledContent(sty.clone())
                )?;
                self.sc_last_flushed.insert((x, y), sty);
                do_flush = true;
            }
        }

        if do_flush {
            sc.flush()?;
        }
        self.last_render = std::time::Instant::now(); // important only set this at the end
        self.rendering = false;
        Ok(())
    }

    pub fn is_ch_style_at_position_dirty(
        &self, x: u16, y: u16, sty: &StyledContent<ChPlus>,
    ) -> bool {
        let Some(existing_sty) = self.sc_last_flushed.get(&(x, y)) else {
            return true;
        };
        !(existing_sty == sty)
    }
}

pub fn process_event_resps(
    resps: EventResponses, exit_tx: Option<WatchSender<bool>>, el_org: &ElementOrganizer,
    main_el_id: ElementID,
) -> Result<bool, Error> {
    // only check for response for quit
    for resp in resps.iter() {
        if matches!(resp, EventResponse::Quit | EventResponse::Destruct) {
            if let Some(exit_tx) = exit_tx {
                exit_tx.send(true)?;
            }
            return Ok(true); // quit
        }
    }
    // check to make sure the element organizer still has the main element
    if el_org.get_element(&main_el_id).is_none() {
        return Ok(true); // quit
    }

    Ok(false)
}

#[derive(Clone)]
pub struct TuiParent {
    pub hat: SortingHat,
    pub color_store: ColorStore,
    pub eo: ElementOrganizer,
    pub el_store: Rc<RefCell<HashMap<String, Vec<u8>>>>,
    pub exit_tx: WatchSender<bool>,
    /// event senter for internally generated events
    pub ev_tx: MpscSender<Event>,
    pub main_el_id: ElementID,
}

impl TuiParent {
    pub fn new(exit_tx: WatchSender<bool>, ev_tx: MpscSender<Event>) -> TuiParent {
        TuiParent {
            hat: SortingHat::default(),
            color_store: ColorStore::default(),
            eo: ElementOrganizer::default(),
            el_store: Rc::new(RefCell::new(HashMap::new())),
            exit_tx,
            ev_tx,
            main_el_id: "".to_string(),
        }
    }
}

/// NOTE: It is necessary for the TUI to fulfill the Parent interface
/// as it is the top of the element tree, despite it not itself being an element
/// (it does not fulfill the Element interface). For the MainEl to pass changes
/// in inputability to the tui's ElementOrganizer, it must hold a reference to
/// the TUI and  be able to call this function (as opposed to calling it on an
/// Element, as a child normally would in the rest of the tree).
impl Parent for TuiParent {
    /// Receives the final upward propagation of changes to inputability
    fn propagate_responses_upward(
        &self, parent_ctx: &Context, child_el_id: &ElementID, mut resps: EventResponses,
    ) {
        // process changes in element organizer
        let b: Box<dyn Parent> = Box::new(self.clone());
        self.eo
            .partially_process_ev_resps(parent_ctx, child_el_id, &mut resps, &b);
        if let Err(e) = process_event_resps(
            resps,
            Some(self.exit_tx.clone()),
            &self.eo,
            self.main_el_id.clone(),
        ) {
            log_err!(
                "Error in propagate_responses_upward, process_event_resps: {:?}",
                e
            );
        }
    }

    fn get_store_item(&self, key: &str) -> Option<Vec<u8>> {
        self.el_store.borrow().get(key).cloned()
    }

    fn set_store_item(&self, key: &str, value: Vec<u8>) {
        self.el_store.borrow_mut().insert(key.to_string(), value);
    }

    fn get_parent_focused(&self) -> bool {
        true
    }

    fn get_id(&self) -> ElementID {
        "TUI".to_string()
    }
}

pub fn sc_startup() -> Result<(), Error> {
    set_panic_hook_with_closedown();
    let mut sc = stdout();
    execute!(
        sc,
        cursor::Hide,
        terminal::EnterAlternateScreen,
        EnableMouseCapture
    )?;
    terminal::enable_raw_mode()?;
    Ok(())
}

pub fn sc_closedown() -> Result<(), Error> {
    let mut sc = stdout();
    execute!(
        sc,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal::disable_raw_mode()?;
    Ok(())
}
pub fn set_panic_hook_with_closedown() {
    use std::panic;

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        sc_closedown().expect("failed to close screen");
        debug!("Panic: {:?}", info);
        prev_hook(info);
    }));
}

// -------------

pub fn sc_line_startup(inline: Rc<RefCell<InlineTui>>) -> Result<(), Error> {
    set_line_panic_hook_with_closedown(inline);
    let mut sc = stdout();
    execute!(sc, cursor::Hide, EnableMouseCapture)?;
    terminal::enable_raw_mode()?;
    Ok(())
}

pub fn sc_line_closedown(inline: InlineTui) -> Result<(), Error> {
    let mut sc = stdout();
    let cur_row = inline.cursor_start_row;
    let tui_height = inline.tui_height;
    let scr_height = inline.scr_height;

    // set the cursor back to the bottom of the screen
    if cur_row + tui_height == scr_height {
        execute!(sc, terminal::ScrollUp(1))?;
        execute!(sc, cursor::MoveTo(0, cur_row + tui_height),)?;
    } else {
        execute!(sc, cursor::MoveTo(0, cur_row + tui_height),)?;
    }

    execute!(sc, style::ResetColor, cursor::Show, DisableMouseCapture)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn set_line_panic_hook_with_closedown(inline: Rc<RefCell<InlineTui>>) {
    use std::panic;

    let prev_hook = panic::take_hook();
    let inline_ = *inline.borrow();
    panic::set_hook(Box::new(move |info| {
        sc_line_closedown(inline_).expect("failed to close screen");
        debug!("Panic: {:?}", info);
        prev_hook(info);
    }));
}
