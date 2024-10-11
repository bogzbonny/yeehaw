use {
    crate::{
        element::ReceivableEventChanges, keyboard::Keyboard, ChPlus, Context, DynLocation,
        DynLocationSet, Element, ElementID, ElementOrganizer, Error, Event, EventResponse,
        Priority, UpwardPropagator,
    },
    crossterm::{
        cursor,
        cursor::MoveTo,
        event as ct_event,
        event::EventStream,
        event::{DisableMouseCapture, EnableMouseCapture, Event as CTEvent},
        execute, queue, style,
        style::{ContentStyle, StyledContent},
        terminal,
    },
    futures::{future::FutureExt, StreamExt},
    std::collections::HashMap,
    std::io::{stdout, Write},
    std::{cell::RefCell, rc::Rc},
    tokio::time::{self, Duration},
};

// the amount of time the cui will wait in between calls to re-render
// NOTE: Currently the cui does not re-render when an event it called, hence if
// this value is set too large it will give the cui a laggy feel.
const ANIMATION_SPEED: Duration = Duration::from_micros(100);

// configuration of a cui zelt instance
pub struct Cui {
    eo: ElementOrganizer,
    main_el_id: ElementID,
    kb: Keyboard,

    pub kill_on_ctrl_c: bool,

    // last flushed internal screen, used to determine what needs to be flushed next
    //                            x  , y
    pub sc_last_flushed: HashMap<(u16, u16), StyledContent<ChPlus>>,
}

impl Cui {
    pub fn new(main_el: Rc<RefCell<dyn Element>>) -> Result<Cui, Error> {
        let eo = ElementOrganizer::default();
        let cui = Cui {
            eo: eo.clone(),
            main_el_id: main_el.borrow().id().clone(),
            kb: Keyboard::default(),
            kill_on_ctrl_c: true,
            sc_last_flushed: HashMap::new(),
        };

        // add the element here after the location has been created
        let ctx = Context::new_context_for_screen();
        let loc = DynLocation::new_fixed(0, ctx.s.width.into(), 0, ctx.s.height.into());
        let loc = DynLocationSet::new(loc, vec![], 0);
        main_el.borrow_mut().set_dyn_location_set(loc);
        main_el.borrow_mut().set_visible(true);
        main_el
            .borrow_mut()
            .change_priority(&ctx, Priority::FOCUSED);

        let cup = Box::new(CuiUpwardPropagator::new(eo));

        // when adding the main element, nil is passed in as the parent
        // this is because the top of the tree is the CUI's main EO and so no parent
        // is necessary
        cui.eo.add_element(main_el.clone(), Some(cup));
        cui.eo.refresh(&ctx);
        //debug!("mail_el rec: {:?}", main_el.borrow().receivable());

        set_panic_hook_with_closedown();
        Ok(cui)
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        sc_startup()?;
        self.launch().await;
        sc_closedown()?;
        Ok(())
    }

    async fn launch(&mut self) {
        let mut reader = EventStream::new();

        loop {
            let delay = time::sleep(ANIMATION_SPEED).fuse();
            let event = reader.next().fuse();

            tokio::select! {
                ev_res = event => {
                    match ev_res {
                        Some(Ok(ev)) => {
                            match ev {
                                CTEvent::Key(key_ev) => {
                                    //debug!("cui Key event: {:?}", key_ev);
                                    if self.process_event_key(key_ev) {
                                        break;
                                    }
                                }
                                CTEvent::Mouse(mouse_ev) => {
                                    if self.process_event_mouse(mouse_ev) {
                                        break;
                                    }
                                }

                                CTEvent::Resize(_, _) => {
                                    let ctx = Context::new_context_for_screen();
                                    let loc = DynLocation::new_fixed(0, ctx.s.width.into(), 0, ctx.s.height.into());
                                    // There should only be one element at index 0 in the upper level EO
                                    self.eo.update_el_primary_location(self.main_el_id.clone(), loc);
                                    self.eo.get_element(&self.main_el_id).unwrap().borrow_mut().receive_event(&ctx, Event::Resize{});
                                    self.clear_screen();
                                    self.render()
                                }
                                _ => {}
                            }
                        }
                        Some(Err(e)) => println!("Error: {e:?}\r"),
                        None => break,
                    }
                }

                _ = delay => {
                    self.render()
                },
            };
        }
    }

    pub fn close(&mut self) -> Result<(), Error> {
        Ok(())
    }

    // process_event_key handles key events
    //                                                                 exit-cui
    pub fn process_event_key(&mut self, key_ev: ct_event::KeyEvent) -> bool {
        self.kb.add_ev(key_ev);

        if key_ev == Keyboard::KEY_CTRL_C && self.kill_on_ctrl_c {
            return true;
        }

        //debug!("cui Key event: {:?}", key_ev);

        // we only care about the event response as all keys are sent to the main
        // element no matter what
        self.kb.add_ev(key_ev);

        // get the event combos and consume the keyboard events
        // NOTE the element ID is not used from GetDestinationElFromKB as it is
        // re-determined within the KeyEventsProcess (inefficient but convenient)

        let Some((_, evs)) = self
            .eo
            .prioritizer
            .borrow()
            .get_destination_el_from_kb(&mut self.kb)
        else {
            //debug!("no dest");
            return false;
        };
        let ctx = Context::new_context_for_screen();
        //debug!("cui destination: {:?}", dest);

        let Some((_, resps)) = self.eo.key_events_process(&ctx, evs) else {
            return false;
        };

        // only check for response for quit
        for resp in resps.iter() {
            if matches!(resp, EventResponse::Quit | EventResponse::Destruct) {
                return true;
            }
        }
        false
    }

    // process_event_mouse handles mouse events
    //                                                                       exit-cui
    pub fn process_event_mouse(&mut self, mouse_ev: ct_event::MouseEvent) -> bool {
        let ctx = Context::new_context_for_screen();
        let Some((_, resps)) = self.eo.mouse_event_process(&ctx, &mouse_ev) else {
            return false;
        };

        // only check for response for quit
        for resp in resps.iter() {
            if matches!(resp, EventResponse::Quit | EventResponse::Destruct) {
                return true;
            }
        }
        false
    }

    pub fn clear_screen(&mut self) {
        self.sc_last_flushed.clear();
        let mut sc = stdout();
        execute!(
            sc,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )
        .unwrap();
    }

    // Render all elements, draws the screen using the DrawChPos array passed to it
    // by the element organizers.
    //
    // NOTE: when the cui calls all_drawing on the top level ElementOrganizer,
    // all_drawing gets recursively called on every element organizer in the tree.
    // Each one returns a DrawChPos array which is then passed to the next level up
    // in the tree, with that level appending its own DrawChPos array to the end.
    // Render then iterates through the DrawChPos array and sets the content
    // provided by each element in order from the bottom of the tree to the top.
    // This results in elements higher up the tree being able to overwrite elements
    // lower down the tree.
    pub fn render(&mut self) {
        let mut sc = stdout();
        let ctx = Context::new_context_for_screen();
        let chs = self.eo.all_drawing(&ctx);

        let mut dedup_chs: HashMap<(u16, u16), StyledContent<ChPlus>> = HashMap::new();
        for c in chs {
            // remove out of bounds
            if c.x >= ctx.s.width || c.y >= ctx.s.height {
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

        let mut do_flush = false;
        for ((x, y), sty) in dedup_chs {
            if self.is_ch_style_at_position_dirty(x, y, &sty) {
                queue!(
                    &mut sc,
                    MoveTo(x, y),
                    style::PrintStyledContent(sty.clone())
                )
                .unwrap();
                self.sc_last_flushed.insert((x, y), sty);
                do_flush = true;
            }
        }

        if do_flush {
            sc.flush().unwrap();
        }
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

pub struct CuiUpwardPropagator {
    eo: ElementOrganizer,
}

impl CuiUpwardPropagator {
    pub fn new(eo: ElementOrganizer) -> CuiUpwardPropagator {
        CuiUpwardPropagator { eo }
    }
}

impl UpwardPropagator for CuiUpwardPropagator {
    // Receives the final upward propagation of changes to inputability
    // NOTE: It is necessary for the CUI to fulfill the UpwardPropagator interface
    // as it is the top of the element tree, despite it not itself being an element
    // (it does not fulfill the Element interface). For the MainEl to pass changes
    // in inputability to the cui's ElementOrganizer, it must hold a reference to
    // the CUI and  be able to call this function (as opposed to calling it on an
    // Element, as a child normally would in the rest of the tree).
    fn propagate_receivable_event_changes_upward(
        &self, child_el_id: &ElementID, rec: ReceivableEventChanges,
    ) {
        // process changes in element organizer
        self.eo.process_receivable_event_changes(child_el_id, &rec);
    }
}

pub fn sc_startup() -> Result<(), Error> {
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
        sc_closedown().unwrap();
        prev_hook(info);
    }));
}
