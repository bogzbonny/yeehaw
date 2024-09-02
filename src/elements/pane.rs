use {
    crate::{
        element::ReceivableEventChanges, Context, DrawCh, DrawChPos, DrawChs2D, Element, ElementID,
        Event, EventResponses, Priority, SclLocation, SclLocationSet, SclVal, SortingHat,
        UpwardPropagator,
    },
    std::{
        collections::HashMap,
        ops::{Deref, DerefMut},
        {cell::RefCell, rc::Rc},
    },
};

// Pane is a pane element which other objects can embed and build off
// of. It defines the basic draw functionality of a pane.
#[derive(Clone)]
pub struct Pane {
    kind: Rc<RefCell<&'static str>>,

    id: Rc<RefCell<String>>, // element-id as assigned by the sorting-hat

    attributes: Rc<RefCell<HashMap<String, Vec<u8>>>>,

    // The SelfEvs are NOT handled by the standard pane. The element inheriting the
    // standard pane is expected to handle all SelfReceivableEvents in the
    // ReceiveEvent function. The standard pane is only responsible for
    // listing the receivable events when Receivable() is called
    pub self_evs: Rc<RefCell<SelfReceivableEvents>>,

    // This elements "overall" reference priority
    //
    // TODO this is only used currently by the ParentPane,
    // consider moving this field into the ParentPane, if nothing
    // else ever uses it.
    element_priority: Rc<RefCell<Priority>>,

    pub up: Rc<RefCell<Option<Box<dyn UpwardPropagator>>>>,

    // The pane's Content need not be the dimensions provided within
    // the Location, however the Content will simply be cut off if it exceeds
    // any dimension of the Location. If the Content is less than the dimensions
    // of the Location all extra characters will be filled with the DefaultCh.
    // The location of where to begin drawing from within the Content can be
    // offset using content_view_offset_x and content_view_offset_y
    pub content: Rc<RefCell<DrawChs2D>>,
    pub default_ch: Rc<RefCell<DrawCh>>,
    pub default_line: Rc<RefCell<Vec<DrawCh>>>,
    pub content_view_offset_x: Rc<RefCell<usize>>,
    pub content_view_offset_y: Rc<RefCell<usize>>,

    // scaleable values of x, y, width, and height in the parent context
    //pub pos_x: Rc<RefCell<SclVal>>,
    //pub pos_y: Rc<RefCell<SclVal>>,
    //pub width: Rc<RefCell<SclVal>>,
    //pub height: Rc<RefCell<SclVal>>,
    pub loc: Rc<RefCell<SclLocationSet>>,
    pub visible: Rc<RefCell<bool>>,
}

impl Pane {
    // NOTE kind is a name for the kind of pane, typically a different kind will be applied
    // to the standard pane, as the standard pane is only boilerplate.
    pub const KIND: &'static str = "standard_pane";

    pub fn new(hat: &SortingHat, kind: &'static str) -> Pane {
        Pane {
            kind: Rc::new(RefCell::new(kind)),
            id: Rc::new(RefCell::new(hat.create_element_id(kind))),
            attributes: Rc::new(RefCell::new(HashMap::new())),
            self_evs: Rc::new(RefCell::new(SelfReceivableEvents::default())),
            element_priority: Rc::new(RefCell::new(Priority::UNFOCUSED)),
            up: Rc::new(RefCell::new(None)),
            content: Rc::new(RefCell::new(DrawChs2D::default())),
            default_ch: Rc::new(RefCell::new(DrawCh::default())),
            default_line: Rc::new(RefCell::new(vec![])),
            content_view_offset_x: Rc::new(RefCell::new(0)),
            content_view_offset_y: Rc::new(RefCell::new(0)),
            loc: Rc::new(RefCell::new(SclLocationSet::default())),
            visible: Rc::new(RefCell::new(true)),
        }
    }

    pub fn with_start_x(self, x: SclVal) -> Pane {
        self.loc.borrow_mut().l.set_start_x(x);
        self
    }

    pub fn with_start_y(self, y: SclVal) -> Pane {
        self.loc.borrow_mut().l.set_start_y(y);
        self
    }

    pub fn with_width(self, w: SclVal) -> Pane {
        self.loc.borrow_mut().l.set_width(w);
        self
    }

    pub fn with_height(self, h: SclVal) -> Pane {
        self.loc.borrow_mut().l.set_height(h);
        self
    }

    pub fn with_scl_location(self, l: SclLocation) -> Pane {
        self.loc.borrow_mut().l = l;
        self
    }

    pub fn with_content(self, content: DrawChs2D) -> Pane {
        *self.content.borrow_mut() = content;
        self
    }

    pub fn with_default_ch(self, ch: DrawCh) -> Pane {
        *self.default_ch.borrow_mut() = ch;
        self
    }

    pub fn with_default_line(self, line: Vec<DrawCh>) -> Pane {
        *self.default_line.borrow_mut() = line;
        self
    }

    pub fn with_content_view_offset(self, x: usize, y: usize) -> Pane {
        *self.content_view_offset_x.borrow_mut() = x;
        *self.content_view_offset_y.borrow_mut() = y;
        self
    }

    pub fn with_self_receivable_events(self, evs: Vec<(Event, Priority)>) -> Pane {
        *self.self_evs.borrow_mut() = SelfReceivableEvents(evs);
        self
    }

    pub fn set_self_receivable_events(&self, evs: Vec<(Event, Priority)>) {
        *self.self_evs.borrow_mut() = SelfReceivableEvents(evs);
    }

    // -----------------------

    pub fn get_element_priority(&self) -> Priority {
        *self.element_priority.borrow()
    }
}
impl Element for Pane {
    fn kind(&self) -> &'static str {
        *self.kind.borrow()
    }

    fn id(&self) -> ElementID {
        self.id.borrow().clone()
    }

    // Receivable returns the event keys and commands that can
    // be received by this element along with their priorities
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.self_evs.borrow().0.clone()
    }

    //                                               (captured, resp         )
    fn receive_event(&self, _ctx: &Context, _ev: Event) -> (bool, EventResponses) {
        (false, EventResponses::default())
    }

    // ChangePriority returns a priority change request to its parent organizer so
    // as to update the priority of all commands registered to this element.
    // The element iterates through its registered cmds/evCombos, and returns a
    // priority change request for each one.
    fn change_priority(&self, _: &Context, p: Priority) -> ReceivableEventChanges {
        // update the priority of all registered events
        for pef in self.self_evs.borrow_mut().iter_mut() {
            pef.1 = p;
        }
        *self.element_priority.borrow_mut() = p;
        let rec = ReceivableEventChanges::default().with_evs(self.self_evs.borrow().0.clone());
        rec
    }

    // Drawing compiles all of the DrawChPos necessary to draw this element
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut chs = vec![];

        // convert the Content to DrawChPos
        for y in 0..ctx.s.height as usize {
            for x in 0..ctx.s.width as usize {
                // default ch being added next is the DefaultCh
                let mut ch_out = *self.default_ch.borrow();

                // TODO XXX allow for negative offsets! currently crashes

                let offset_y = y + *self.content_view_offset_y.borrow();
                let offset_x = x + *self.content_view_offset_x.borrow();

                // if the offset isn't pushing all the content out of view,
                // assign the next ch to be the one at the offset in the Content
                // matrix
                if offset_y < self.content.borrow().0.len()
                    && offset_x < self.content.borrow().0[offset_y].len()
                {
                    ch_out = self.content.borrow().0[offset_y][offset_x];
                }

                // if y is greater than the height of the visible content,
                // trigger a default line.
                // NOTE: height of the visible content is the height of the
                // content minus the offset
                if y > self.content.borrow().0.len() {
                    if x < self.default_line.borrow().len() {
                        ch_out = self.default_line.borrow()[x];
                    } else {
                        ch_out = *self.default_ch.borrow();
                    }
                }

                // convert the DrawCh to a DrawChPos
                chs.push(DrawChPos::new(ch_out, x as u16, y as u16))
            }
        }
        chs
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.attributes.borrow().get(key).cloned()
    }

    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.attributes.borrow_mut().insert(key.to_string(), value);
    }

    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        *self.up.borrow_mut() = Some(up);
    }

    fn get_scl_location_set(&self) -> Rc<RefCell<SclLocationSet>> {
        self.loc.clone()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.visible.clone()
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

    pub fn push_many_at_priority(&mut self, evs: Vec<Event>, p: Priority) {
        for ev in evs {
            self.push(ev, p)
        }
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
