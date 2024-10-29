use {
    crate::{
        ReceivableEventChanges, Context, DrawCh, DrawChPos, DrawChs2D, DynLocation,
        DynLocationSet, DynVal, Element, ElementID, Event, EventResponse, EventResponses, Parent,
        Priority, SortingHat, Style, ZIndex,
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

    pub parent: Rc<RefCell<Option<Box<dyn Parent>>>>,

    #[allow(clippy::type_complexity)]
    pub hooks:
        Rc<RefCell<HashMap<String, Vec<(ElementID, Box<dyn FnMut(&str, Box<dyn Element>)>)>>>>,

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
    // NOTE use getters/setters to ensure hook calls
    loc: Rc<RefCell<DynLocationSet>>,
    visible: Rc<RefCell<bool>>,
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
            parent: Rc::new(RefCell::new(None)),
            hooks: Rc::new(RefCell::new(HashMap::new())),
            content: Rc::new(RefCell::new(DrawChs2D::default())),
            default_ch: Rc::new(RefCell::new(DrawCh::default())),
            default_line: Rc::new(RefCell::new(vec![])),
            content_view_offset_x: Rc::new(RefCell::new(0)),
            content_view_offset_y: Rc::new(RefCell::new(0)),
            loc: Rc::new(RefCell::new(DynLocationSet::new_full())),
            visible: Rc::new(RefCell::new(true)),
        }
    }

    pub fn with_kind(self, kind: &'static str) -> Pane {
        *self.kind.borrow_mut() = kind;
        self
    }

    pub fn at(self, x: DynVal, y: DynVal) -> Pane {
        self.set_at(x, y);
        self
    }

    pub fn set_at(&self, x: DynVal, y: DynVal) {
        self.loc.borrow_mut().l.set_at(x, y);
    }

    pub fn set_kind(&self, kind: &'static str) {
        *self.kind.borrow_mut() = kind;
    }

    pub fn with_z(self, z: ZIndex) -> Pane {
        self.loc.borrow_mut().set_z(z);
        self
    }

    pub fn with_start_x(self, x: DynVal) -> Pane {
        self.loc.borrow_mut().l.set_start_x(x);
        self
    }

    pub fn with_start_y(self, y: DynVal) -> Pane {
        self.loc.borrow_mut().l.set_start_y(y);
        self
    }

    pub fn set_start_x(&self, x: DynVal) {
        self.loc.borrow_mut().l.set_start_x(x);
    }

    pub fn set_start_y(&self, y: DynVal) {
        self.loc.borrow_mut().l.set_start_y(y);
    }

    pub fn set_end_x(&self, x: DynVal) {
        self.loc.borrow_mut().l.set_end_x(x);
    }

    pub fn set_end_y(&self, y: DynVal) {
        self.loc.borrow_mut().l.set_end_y(y);
    }

    pub fn get_start_x(&self, ctx: &Context) -> i32 {
        self.loc.borrow().l.get_start_x(ctx)
    }

    pub fn get_start_y(&self, ctx: &Context) -> i32 {
        self.loc.borrow().l.get_start_y(ctx)
    }

    pub fn get_end_x(&self, ctx: &Context) -> i32 {
        self.loc.borrow().l.get_end_x(ctx)
    }

    pub fn get_end_y(&self, ctx: &Context) -> i32 {
        self.loc.borrow().l.get_end_y(ctx)
    }

    pub fn get_dyn_end_x(&self) -> DynVal {
        self.loc.borrow().l.end_x.clone()
    }

    pub fn get_dyn_end_y(&self) -> DynVal {
        self.loc.borrow().l.end_y.clone()
    }

    pub fn get_height(&self, ctx: &Context) -> usize {
        self.loc.borrow().l.height(ctx)
    }

    pub fn get_width(&self, ctx: &Context) -> usize {
        self.loc.borrow().l.width(ctx)
    }

    pub fn with_dyn_height(self, h: DynVal) -> Pane {
        self.loc.borrow_mut().l.set_dyn_height(h);
        self
    }

    pub fn with_dyn_width(self, w: DynVal) -> Pane {
        self.loc.borrow_mut().l.set_dyn_width(w);
        self
    }

    pub fn set_dyn_height(&self, h: DynVal) {
        self.loc.borrow_mut().l.set_dyn_height(h);
    }

    pub fn set_dyn_width(&self, w: DynVal) {
        self.loc.borrow_mut().l.set_dyn_width(w);
    }

    pub fn set_z(&self, z: ZIndex) {
        self.loc.borrow_mut().set_z(z);
    }

    pub fn with_dyn_location(self, l: DynLocation) -> Pane {
        self.loc.borrow_mut().l = l;
        self
    }

    pub fn get_dyn_location(&self) -> DynLocation {
        self.loc.borrow().l.clone()
    }

    pub fn set_dyn_location(&self, l: DynLocation) {
        self.loc.borrow_mut().l = l;
    }

    pub fn with_content(self, content: DrawChs2D) -> Pane {
        *self.content.borrow_mut() = content;
        self
    }

    pub fn set_content(&self, content: DrawChs2D) {
        *self.content.borrow_mut() = content;
    }

    pub fn with_default_ch(self, ch: DrawCh) -> Pane {
        *self.default_ch.borrow_mut() = ch;
        self
    }

    pub fn with_style(self, style: Style) -> Pane {
        self.default_ch.borrow_mut().style = style;
        self
    }

    pub fn set_style(&self, style: Style) {
        self.default_ch.borrow_mut().style = style;
    }

    pub fn get_style(&self) -> Style {
        self.default_ch.borrow().style.clone()
    }

    pub fn set_default_ch(&self, ch: DrawCh) {
        *self.default_ch.borrow_mut() = ch;
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

    // focus all prioritized events
    pub fn focus(&self) {
        *self.element_priority.borrow_mut() = Priority::FOCUSED;
        self.self_evs
            .borrow_mut()
            .update_priority_for_all(Priority::FOCUSED);
        if let Some(parent) = self.parent.borrow().as_ref() {
            let rec = ReceivableEventChanges::default()
                .with_remove_evs(
                    self.self_evs
                        .borrow()
                        .0
                        .clone()
                        .iter()
                        .map(|(ev, _)| ev.clone())
                        .collect(),
                )
                .with_add_evs(self.self_evs.borrow().0.clone());
            let resps = EventResponse::ReceivableEventChanges(rec);
            parent.propagate_responses_upward(&self.id(), resps.into());
        }
    }

    // defocus all prioritized events
    pub fn unfocus(&self) {
        *self.element_priority.borrow_mut() = Priority::UNFOCUSED;
        self.self_evs
            .borrow_mut()
            .update_priority_for_all(Priority::UNFOCUSED);
        if let Some(parent) = self.parent.borrow().as_ref() {
            let rec = ReceivableEventChanges::default()
                .with_remove_evs(
                    self.self_evs
                        .borrow()
                        .0
                        .clone()
                        .iter()
                        .map(|(ev, _)| ev.clone())
                        .collect(),
                )
                .with_add_evs(self.self_evs.borrow().0.clone());
            let resps = EventResponse::ReceivableEventChanges(rec);
            parent.propagate_responses_upward(&self.id(), resps.into());
        }
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
    fn receive_event_inner(&self, _ctx: &Context, _ev: Event) -> (bool, EventResponses) {
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
        let rec = ReceivableEventChanges::default().with_add_evs(self.self_evs.borrow().0.clone());
        rec
    }

    // Drawing compiles all of the DrawChPos necessary to draw this element
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut chs = vec![];

        let (xmin, xmax, ymin, ymax) = if let Some(vis_region) = ctx.visible_region {
            (
                // take the intersection of the visibile region and the elements region
                (vis_region.start_x as usize).max(0),
                (vis_region.end_x as usize).min(ctx.s.width as usize),
                (vis_region.start_y as usize).max(0),
                (vis_region.end_y as usize).min(ctx.s.height as usize),
            )
        } else {
            (0, ctx.s.width as usize, 0, ctx.s.height as usize)
        };

        // convert the Content to DrawChPos
        for y in ymin..ymax {
            for x in xmin..xmax {
                // default ch being added next is the DefaultCh
                let mut ch_out = self.default_ch.borrow().clone();

                let offset_y = y + *self.content_view_offset_y.borrow();
                let offset_x = x + *self.content_view_offset_x.borrow();

                // if the offset isn't pushing all the content out of view,
                // assign the next ch to be the one at the offset in the Content
                // matrix
                if offset_y < self.content.borrow().0.len()
                    && offset_x < self.content.borrow().0[offset_y].len()
                {
                    ch_out = self.content.borrow().0[offset_y][offset_x].clone();
                }

                // if y is greater than the height of the visible content,
                // trigger a default line.
                // NOTE: height of the visible content is the height of the
                // content minus the offset
                if y > self.content.borrow().0.len() {
                    if x < self.default_line.borrow().len() {
                        ch_out = self.default_line.borrow()[x].clone();
                    } else {
                        ch_out = self.default_ch.borrow().clone();
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

    fn set_parent(&self, parent: Box<dyn Parent>) {
        *self.parent.borrow_mut() = Some(parent);
    }

    fn set_hook(
        &self,
        kind: &str,
        el_id: ElementID,
        //                  kind, hooked element
        hook: Box<dyn FnMut(&str, Box<dyn Element>)>,
    ) {
        self.hooks
            .borrow_mut()
            .entry(kind.to_string())
            .or_default()
            .push((el_id, hook));
    }

    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        if let Some(hook) = self.hooks.borrow_mut().get_mut(kind) {
            hook.retain(|(el_id_, _)| *el_id_ != el_id);
        };
    }

    // remove all hooks for the element with the given id
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        for (_, hook) in self.hooks.borrow_mut().iter_mut() {
            hook.retain(|(el_id_, _)| *el_id_ != el_id);
        }
    }

    // calls all the hooks of the provided kind
    fn call_hooks_of_kind(&self, kind: &str) {
        for (kind_, v) in self.hooks.borrow_mut().iter_mut() {
            if kind == kind_ {
                for (_, hook) in v.iter_mut() {
                    hook(kind, Box::new(self.clone()));
                }
            }
        }
    }

    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
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

    pub fn update_priority_for_all(&mut self, p: Priority) {
        for i in 0..self.0.len() {
            self.0[i].1 = p;
        }
    }
}
