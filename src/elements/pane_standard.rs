use {
    crate::{
        element::ReceivableEventChanges, Context, DrawCh, DrawChPos, Element, Event, EventResponse,
        Priority, UpwardPropagator,
    },
    std::ops::{Deref, DerefMut},
    std::{cell::RefCell, rc::Rc},
};

// StandardPane is a pane element which other objects can embed and build off
// of. It defines the basic draw functionality of a pane.
pub struct StandardPane {
    // The SelfEvs are NOT handled by the standard pane. The element inheriting the
    // standard pane is expected to handle all SelfReceivableEvents in the
    // ReceiveEvent function. The standard pane is only responsible for
    // listing the receivable events when Receivable() is called
    self_evs: SelfReceivableEvents,

    // This elements "overall" reference priority
    //
    // TODO this is only used currently by the ParentPane,
    // consider moving this field into the ParentPane, if nothing
    // else ever uses it.
    element_priority: Priority,

    up: Option<Rc<RefCell<dyn UpwardPropagator>>>,

    view_height: u16,
    view_width: u16,

    // The pane's Content need not be the dimensions provided within
    // the Location, however the Content will simply be cut off if it exceeds
    // any dimension of the Location. If the Content is less than the dimensions
    // of the Location all extra characters will be filled with the DefaultCh.
    // The location of where to begin drawing from within the Content can be
    // offset using content_view_offset_x and content_view_offset_y
    pub content: Vec<Vec<DrawCh>>,
    pub default_ch: DrawCh,
    pub default_line: Vec<DrawCh>,
    pub content_view_offset_x: u16,
    pub content_view_offset_y: u16,
}

impl Default for StandardPane {
    fn default() -> Self {
        StandardPane::new(vec![], DrawCh::default(), vec![], 0, 0)
    }
}

impl StandardPane {
    pub fn new(
        content: Vec<Vec<DrawCh>>, default_ch: DrawCh, default_line: Vec<DrawCh>,
        content_view_offset_x: u16, content_view_offset_y: u16,
    ) -> StandardPane {
        StandardPane {
            self_evs: SelfReceivableEvents::default(),
            element_priority: Priority::UNFOCUSED,
            up: None,
            view_height: 0,
            view_width: 0,
            content,
            default_ch,
            default_line,
            content_view_offset_x,
            content_view_offset_y,
        }
    }

    // TODO delete post translation... just made the fields public
    //pub fn get_content_view_offset_x(&self) -> usize {
    //    self.content_view_offset_x
    //}
    //pub fn get_content_view_offset_y(&self) -> usize {
    //    self.content_view_offset_y
    //}
    //pub fn set_content_view_offset_x(&mut self, x: usize) {
    //    self.content_view_offset_x = x
    //}
    //pub fn set_content_view_offset_y(&mut self, y: usize) {
    //    self.content_view_offset_y = y
    //}
}

impl UpwardPropagator for StandardPane {
    // NOTE this function should NOT be used if the standard pane is used as a base for a
    // more complex element. As the developer you should be fulfilling the
    // PropagateUpwardChangesToInputability function directly in your element.
    fn propagate_receivable_event_changes_upward(
        &mut self, child_el: Rc<RefCell<dyn Element>>, rec: ReceivableEventChanges,
        update_this_elements_prioritizers: bool,
    ) {
        if let Some(up) = self.up.as_ref() {
            up.borrow_mut().propagate_receivable_event_changes_upward(
                child_el,
                rec,
                update_this_elements_prioritizers,
            );
        }
    }
}

impl Element for StandardPane {
    // Receivable returns the event keys and commands that can
    // be received by this element along with their priorities
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.self_evs.0.clone()
    }

    //                                               (captured, resp         )
    fn receive_event(&mut self, ctx: &Context, _ev: Event) -> (bool, EventResponse) {
        self.view_height = ctx.get_height();
        self.view_width = ctx.get_width();
        (false, EventResponse::default())
    }

    // ChangePriority returns a priority change request to its parent organizer so
    // as to update the priority of all commands registered to this element.
    // The element iterates through its registered cmds/evCombos, and returns a
    // priority change request for each one.
    fn change_priority(&mut self, _: &Context, p: Priority) -> ReceivableEventChanges {
        // update the priority of all registered events
        for pef in self.self_evs.iter_mut() {
            pef.1 = p;
        }
        self.element_priority = p;
        ReceivableEventChanges::default().with_evs(self.self_evs.0.clone())
    }

    // Drawing compiles all of the DrawChPos necessary to draw this element
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut chs = vec![];

        // convert the Content to DrawChPos
        // NOTE: width/height values must subtract 1 to get final cell locations
        for y in 0..=ctx.s.height - 1 {
            for x in 0..=ctx.s.width - 1 {
                // default ch being added next is the DefaultCh
                let mut ch_out = self.default_ch.clone();

                // TODO XXX allow for negative offsets! currently crashes

                let offset_y = (y + self.content_view_offset_y) as usize;
                let offset_x = (x + self.content_view_offset_x) as usize;

                // if the offset isn't pushing all the content out of view,
                // assign the next ch to be the one at the offset in the Content
                // matrix
                if offset_y < self.content.len() && offset_x < self.content[offset_y].len() {
                    ch_out = self.content[offset_y][offset_x].clone();
                }

                // if y is greater than the height of the visible content,
                // trigger a default line.
                // NOTE: height of the visible content is the height of the
                // content minus the offset
                if (y as usize) > self.content.len() {
                    if (x as usize) < self.default_line.len() {
                        ch_out = self.default_line[x as usize].clone();
                    } else {
                        ch_out = self.default_ch.clone();
                    }
                }

                // convert the DrawCh to a DrawChPos
                chs.push(DrawChPos::new(ch_out, x, y))
            }
        }
        chs
    }

    fn set_upward_propagator(&mut self, up: Rc<RefCell<dyn UpwardPropagator>>) {
        self.up = Some(up);
    }
}

// ---------------------------------------------------------------------------
// The SelfReceivableEvents are used to manage events and associated functions
// registered directly to an element (AND NOT to that elements children!). They
// are similar to the EvPrioritizer, but they are used to manage the events and
// commands that are registered locally to this specific element.
// (The EvPrioritizer and CmdPrioritizer are used to manage the events and
// commands that are registered to an element's
// children in the ElementOrganizer).
// NOTE: these fulfill a similar function to the prioritizers
// in that they manage inclusion/removal more cleanly and can be sorted
#[derive(Clone, Default)]
pub struct SelfReceivableEvents(pub Vec<(Event, Priority)>);

impl Deref for SelfReceivableEvents {
    type Target = Vec<(Event, Priority)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SelfReceivableEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SelfReceivableEvents {
    // TRANSLATION NewSelfReceivableEventsFromPrioritizableEv new_self_receivable_events_from_prioritizable_ev
    pub fn new_from_priority_events(p: Priority, evs: Vec<Event>) -> SelfReceivableEvents {
        SelfReceivableEvents(evs.into_iter().map(|ev| (ev, p)).collect())
    }

    // TRANSLATION Include include
    pub fn push(&mut self, ev: Event, p: Priority) {
        self.0.push((ev, p))
    }

    // TRANSLATION IncludeMany include_many
    pub fn extend(&mut self, evs: Vec<(Event, Priority)>) {
        self.0.extend(evs)
    }

    pub fn remove(&mut self, ev: Event) {
        self.0.retain(|(e, _)| e != &ev)
    }

    pub fn remove_many(&mut self, evs: Vec<Event>) {
        self.0.retain(|(e, _)| !evs.contains(e))
    }

    // update_priority_for_ev updates the priority of the given event
    // registered directly to this element
    // TRANSLATION UpdatePriorityForEvCombo update_priority_for_ev_combo
    pub fn update_priority_for_ev(&mut self, ev: Event, p: Priority) {
        for i in 0..self.0.len() {
            if self.0[i].0 != ev {
                continue;
            }
            self.0[i].1 = p;
            break;
        }
    }

    pub fn update_priority_for_evs(&mut self, evs: Vec<Event>, p: Priority) {
        for ev in evs {
            self.update_priority_for_ev(ev, p)
        }
    }
}
