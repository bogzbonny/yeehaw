use {
    crate::{
        element::ReceivableEventChanges, keyboard::Keyboard, Context, Element, ElementOrganizer,
        Error, Event, Location, LocationSet, UpwardPropagator,
    },
    crossterm::{
        cursor,
        cursor::MoveTo,
        event as ct_event,
        event::EventStream,
        event::{DisableMouseCapture, EnableMouseCapture, Event as CTEvent},
        execute, queue, style,
        style::StyledContent,
        terminal,
    },
    futures::{future::FutureExt, StreamExt},
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
    eo: Rc<RefCell<ElementOrganizer>>,
    kb: Keyboard,
}

impl Cui {
    pub fn new(main_el: Rc<RefCell<dyn Element>>) -> Result<Cui, Error> {
        let eo = Rc::new(RefCell::new(ElementOrganizer::default()));
        let cui = Cui {
            eo: eo.clone(),
            kb: Keyboard::default(),
        };

        // add the element here after the location has been created
        let ctx = Context::new_context_for_screen();
        let loc = Location::new(0, (ctx.s.width - 1).into(), 0, (ctx.s.height - 1).into());
        let loc = LocationSet::default().with_location(loc);

        let cup = Rc::new(RefCell::new(CuiUpwardPropagator::new(eo)));

        // when adding the main element, nil is passed in as the parent
        // this is because the top of the tree is the CUI's main EO and so no parent
        // is necessary
        cui.eo
            .borrow_mut()
            .add_element(main_el, Some(cup), loc, true);

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
                                    let loc = Location::new(0, (ctx.s.width - 1).into(), 0, (ctx.s.height - 1).into());
                                    let loc = LocationSet::default().with_location(loc);
                                    // There should only be one element at index 0 in the upper level EO
                                    self.eo.borrow_mut().update_el_locations_by_id(0, loc);
                                    self.eo.borrow().get_element_by_id(0).unwrap().borrow().receive_event(&ctx, Event::Resize{});
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

        if key_ev == Keyboard::KEY_CTRL_C {
            return true;
        }

        // we only care about the event response as all keys are sent to the main
        // element no matter what
        self.kb.add_ev(key_ev);

        // get the event combos and consume the keyboard events
        // NOTE the element ID is not used from GetDestinationElFromKB as it is
        // re-determined within the KeyEventsProcess (inefficient but convenient)

        let Some((_, evs)) = self
            .eo
            .borrow()
            .prioritizer
            .get_destination_el_from_kb(&mut self.kb)
        else {
            return false;
        };
        let Some((_, resp)) = self.eo.borrow_mut().key_events_process(evs) else {
            return false;
        };

        // only check for response for quit
        resp.quit
    }

    // process_event_mouse handles mouse events
    //                                                                       exit-cui
    pub fn process_event_mouse(&mut self, mouse_ev: ct_event::MouseEvent) -> bool {
        let Some((_, resp)) = self.eo.borrow_mut().mouse_event_process(&mouse_ev) else {
            return false;
        };

        // only check for response for quit
        resp.quit
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
        let mut sc = stdout();
        let chs = self.eo.borrow().all_drawing();
        for c in chs {
            if c.ch.transparent {
                // TODO see ISSUE 2206-1000
            } else {
                let st = StyledContent::new((c.ch.style).into(), &c.ch.ch);
                queue!(&mut sc, MoveTo(c.x, c.y), style::PrintStyledContent(st)).unwrap();
            }
        }
        sc.flush().unwrap();
    }
}

pub struct CuiUpwardPropagator {
    eo: Rc<RefCell<ElementOrganizer>>,
}

impl CuiUpwardPropagator {
    pub fn new(eo: Rc<RefCell<ElementOrganizer>>) -> CuiUpwardPropagator {
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
        &mut self, child_el: Rc<RefCell<dyn Element>>, rec: ReceivableEventChanges,
        update_this_elements_prioritizers: bool,
    ) {
        if update_this_elements_prioritizers {
            // get the id of the element propagating changes to the CUI
            let el_id = self.eo.borrow().get_id_from_el(child_el);

            // process changes in element organizer
            self.eo
                .borrow_mut()
                .process_receivable_event_changes(el_id, &rec);
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
