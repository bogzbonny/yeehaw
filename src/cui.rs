use {
    crate::{
        element::ReceivableEventChanges, keyboard::Keyboard, Context, Element, ElementOrganizer,
        Error, Event, EventResponse, Location, LocationSet,
    },
    crossterm::{
        cursor, event as ct_event,
        event::EventStream,
        event::{DisableMouseCapture, EnableMouseCapture, Event as CTEvent},
        execute, style, terminal,
    },
    futures::{future::FutureExt, StreamExt},
    parking_lot::Mutex,
    std::io::stdout,
    std::sync::Arc,
    tokio::time::{self, Duration},
};

// the amount of time the cui will wait in between calls to re-render
// NOTE: Currently the cui does not re-render when an event it called, hence if
// this value is set too large it will give the cui a laggy feel.
const ANIMATION_SPEED: Duration = Duration::from_micros(100);

// configuration of a cui zelt instance
pub struct Cui {
    eo: ElementOrganizer,
    kb: Keyboard,
}

impl Cui {
    pub fn new(main_el: Arc<Mutex<dyn Element>>) -> Result<Cui, Error> {
        let mut cui = Cui {
            eo: ElementOrganizer::default(),
            kb: Keyboard::default(),
        };

        // add the element here after the location has been created
        let ctx = Context::new_context_for_screen();
        let loc = Location::new(0, (ctx.s.width - 1).into(), 0, (ctx.s.height - 1).into());
        let loc = LocationSet::default().with_location(loc);

        // when adding the main element, nil is passed in as the parent
        // this is because the top of the tree is the CUI's main EO and so no parent
        // is necessary
        // XXX this used to use the cui as the upward propagator, need to fix
        cui.eo.add_element(main_el, cui, loc, true);

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
                                    self.process_event_key(key_ev);
                                }
                                CTEvent::Mouse(mouse_ev) => {
                                    self.process_event_mouse(mouse_ev);
                                }

                                CTEvent::Resize(_, _) => {
                                    let ctx = Context::new_context_for_screen();
                                    let loc = Location::new(0, (ctx.s.width - 1).into(), 0, (ctx.s.height - 1).into());
                                    let loc = LocationSet::default().with_location(loc);
                                    self.eo.update_el_locations_by_id(0, loc);
                                    self.eo.get_element_by_id(0).unwrap().lock().receive_event(&ctx, Event::Resize{});
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
    pub fn process_event_key(&mut self, key_ev: ct_event::KeyEvent) {
        self.kb.add_ev(key_ev);

        if key_ev == Keyboard::KEY_CTRL_C {
            // XXX exit program
            return;
        }

        // we only care about the event response as all keys are sent to the main
        // element no matter what
        self.kb.add_ev(key_ev); // XXX

        // get the event combos and consume the keyboard events
        // NOTE the element ID is not used from GetDestinationElFromKB as it is
        // re-determined within the KeyEventsProcess (inefficient but convenient)

        let Some((_, evs)) = self.eo.prioritizer.get_destination_el_from_kb(&mut self.kb) else {
            return;
        };
        let Some((_, resp)) = self.eo.key_events_process(evs) else {
            return;
        };
        //Debug("\nCUI.ProcessEventKey: keyEvents resp.IC: %v\n", resp.IC)
        self.process_resp(resp);
    }

    // process_event_mouse handles mouse events
    pub fn process_event_mouse(&mut self, mouse_ev: ct_event::MouseEvent) {
        let Some((_, resp)) = self.eo.mouse_event_process(&mouse_ev) else {
            return;
        };
        self.process_resp(resp);
    }

    // process_resp closes the application if appropriate
    pub fn process_resp(&mut self, resp: EventResponse) {
        if resp.quit {
            close();
        }
    }

    // Render all elements, draws the screen using the DrawChPos array passed to it
    // by the element organizers.
    //
    // NOTE: when the cui calls AllDrawing on the top level ElementOrganizer,
    // AllDrawing gets recursively called on every element organizer in the tree.
    // Each one returns a DrawChPos array which is then passed to the next level up
    // in the tree, with that level appending its own DrawChPos array to the end.
    // Render then iterates through the DrawChPos array and sets the content
    // provided by each element in order from the bottom of the tree to the top.
    // This results in elements higher up the tree being able to overwrite elements
    // lower down the tree.
    //
    // TODO see ISSUE 2302-2203. Could optimize for elements not requiring updates by
    // sending in a timestamp. Would then need a force-render option (for startup)

    pub fn render(&mut self) {
        let chs = self.eo.all_drawing();
        for c in chs {
            if c.ch.transparent {
                // TODO see ISSUE 2206-1000
            } else {
                self.sc.set_content(c.x, c.y, c.ch.ch, nil, c.ch.style) // XXX
            }
        }
        self.sc.Show() // XXX
    }

    // Receives the final upward propagation of changes to inputability
    // NOTE: It is necessary for the CUI to fulfill the UpwardPropagator interface
    // as it is the top of the element tree, despite it not itself being an element
    // (it does not fulfill the Element interface). For the MainEl to pass changes
    // in inputability to the cui's ElementOrganizer, it must hold a reference to
    // the CUI and  be able to call this function (as opposed to calling it on an
    // Element, as a child normally would in the rest of the tree).
    pub fn propagate_receivable_event_changes_upward(
        &mut self, child_el: Arc<Mutex<dyn Element>>, rec: ReceivableEventChanges,
        update_this_elements_prioritizers: bool,
    ) {
        // get the id of the element propagating changes to the CUI
        let el_id = self.eo.get_id_from_el(child_el);

        if update_this_elements_prioritizers {
            // process changes in element organizer
            self.eo.process_receivable_event_changes(el_id, &rec);
        }
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
    //execute!(stdout, EnableMouseCapture)?;
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
