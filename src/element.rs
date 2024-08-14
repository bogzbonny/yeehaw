use {
    crate::{prioritizer::Priority, DrawChPos, ElementID, Event, Location, LocationSet, Size},
    std::any::Any,
    std::{cell::RefCell, rc::Rc},
};

// Element is the base interface which all viewable elements are
// expected to fulfill
pub trait Element {
    fn kind(&self) -> &'static str; // a name for the kind of the element

    fn id(&self) -> &ElementID; // the element id as assigned by the SortingHat

    fn description(&self) -> &str; // element description, useful for debugging

    // Returns all events (key events and commands, etc.) that are registered to
    // an element. This includes all events that are registered to the element
    // itself, as well as its children, via its ElementOrganizer (if it has
    // one).
    // NOTE in this current design, elements are always expected to receive
    // mouse events.
    fn receivable(&self) -> Vec<(Event, Priority)>;

    // This is used to receive an event from a parent. The receiving element may
    // consume the event and/or pass it to a child. The element is expected to
    // return a response to the event, along with any changes to its
    // inputability that its parent will use as well pass up the tree. When the
    // event is captured, the element is expected to returns captured=true.
    //                                               (captured, response     )
    fn receive_event(&mut self, ctx: &Context, ev: Event) -> (bool, EventResponse);

    // ChangePriority will change the priority of an element relative to its
    // ancestors. All events owned directly by the element will have their local
    // priority changed to the given priority while everything registered in
    // this element's prioritizers will remain unchanged. The element will then
    // tell relay the appropriate changes in priority to its parent. This is
    // dependant on the directionality of the priority change.
    //
    // If the priority is changing from Focused to Unfocused, then the element
    // will tell its parent to change the priority of all the element's
    // registered evs (local and registered) to Unfocused in the
    // parent's prioritizers.
    //
    // If the priority is changed from Unfocused to Focused, then the element
    // will tell its parent to change the priority of the element's local evs
    // to the given priority, while telling to parent to change the priority of
    // all of the element's children's evs to whatever is recorded in the
    // element's prioritizers.
    fn change_priority(&mut self, ctx: &Context, p: Priority) -> ReceivableEventChanges;

    // get the element's full drawing for the provided width and height
    // this is provided as an ordered list of individual elements to draw
    // z = the element viewing depth, 0 is the topmost element
    // freeFloating = whether the element is constrained by the parent
    //                element border
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos>;

    // Assign a reference to the element's parent through the UpwardPropagator
    // interface. This is used to pass ReceivableEventChanges to the parent.
    fn set_upward_propagator(&mut self, up: Rc<RefCell<dyn UpwardPropagator>>);
}

impl PartialEq for dyn Element {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

// ----------------------------------------

pub trait UpwardPropagator {
    // The UpwardPropagator trait is a trait that a parent element should fulfill
    // which can then be provided to child elements as a means for those child elements to
    // propagate changes upward to their parent (and grand-parents etc.).
    //
    // In most cases, receivable event changes are passed to the parent in the return values of a
    // function invoked on the element by the parent (ex. ReceiveEvent). However, when changes are
    // initiated through hooks of non-parent elements, the parent must be notified of the changes
    // from the child directly. By providing this trait to a child element it enables it to propagate
    // receivable event changes when it hasn't been modified directly by its parent.
    //
    // For instance, a file-navigator may with to initiate a content change in an adjacent
    // content-pane in this case it could activate the content-pane and deactivate itself, this
    // newly activated content-pane would need a way to inform its parent pane of its receivable
    // event changes.
    //
    // child_el_id is the child element-id which is invoking the propagation from BELOW the element
    // fulfilling the UpwardPropagator trait. This is used by the parent element (this one) to
    // determine which events/cmds to update the prioritizers for.
    //
    // TRANSLATION NOTE PropagateUpwardChangesToInputability propagate_upward_changes_to_inputability
    fn propagate_receivable_event_changes_upward(
        &mut self, child_el_id: &ElementID, rec: ReceivableEventChanges,
    );

    // TODO add a function to get an elements name here (for debugging),
    // once an element has SetUpwardPropagator called on it, it will
    // be able to construct its own name, by tacking on it's own name to its
    // parents name.

    // returns a name of the element (for debugging purposes)
    //fn name(&self) -> String;
}

// Context is a struct which contains information about the current context of a
// given element.
// The context of an element is passed to the element during key function calls
// where the element may need to know its size and Visibility.
//
// Additionally, Metadata may be addended to the context to pass additional
// arbitrary information.
#[derive(Default, Clone)]
pub struct Context {
    pub s: Size,
    pub visible: bool,
    pub metadata: Option<String>, // should be in json
}

impl Context {
    pub fn new(s: Size, visible: bool) -> Context {
        Context {
            s,
            visible,
            metadata: None,
        }
    }

    // TODO return error
    pub fn new_context_for_screen() -> Context {
        let (xmax, ymax) = crossterm::terminal::size().unwrap();
        Context {
            s: Size::new(xmax, ymax),
            visible: true,
            metadata: None,
        }
    }

    pub fn with_metadata(self, md: String) -> Context {
        Context {
            s: self.s,
            visible: self.visible,
            metadata: Some(md),
        }
    }

    pub fn get_width(&self) -> u16 {
        self.s.width
    }

    pub fn get_height(&self) -> u16 {
        self.s.height
    }
}

// ----------------------------------------------------------------------------

// EventResponse is used to send information back to the parent that delivered
// the event to the element
#[derive(Default)]
pub struct EventResponse {
    // quit the application
    pub quit: bool,
    // deactivate (used with widgets
    pub deactivate: bool,
    // destroy the current element
    pub destruct: bool,
    // character updates to the element's drawing
    pub ch_updates: Option<Vec<DrawChPos>>,
    // metadata can be used to send back any arbitrary information
    // it is up to the parent to interpret the metadata
    pub metadata: Option<Box<dyn Any>>,
    // replace the current element with the provided element
    pub replacement: Option<Rc<RefCell<dyn Element>>>,
    // request that the provided window element be created at the location
    pub window: Option<CreateWindow>,
    // sends a request to the parent to change the size of the element
    pub resize: Option<Size>,
    // send a request to the parent to change the position of the element
    pub relocation: Option<RelocationRequest>,
    // sends a request to the parent to change the extra locations
    // of the element
    pub extra_locations: Option<ExtraLocationsRequest>,
    // contains priority updates that should be made to the receiver's prioritizer
    pub inputability_changes: Option<ReceivableEventChanges>,
    // for use with scrollbars
    pub scroll_x_static: Option<i32>,
    pub scroll_y_static: Option<i32>,
}

impl EventResponse {
    pub fn with_quit(mut self) -> EventResponse {
        self.quit = true;
        self
    }

    pub fn with_deactivate(mut self) -> EventResponse {
        self.deactivate = true;
        self
    }

    pub fn with_destruct(mut self) -> EventResponse {
        self.destruct = true;
        self
    }

    pub fn with_ch_updates(mut self, chs: Vec<DrawChPos>) -> EventResponse {
        self.ch_updates = Some(chs);
        self
    }

    pub fn with_metadata(mut self, md: Box<dyn Any>) -> EventResponse {
        self.metadata = Some(md);
        self
    }

    pub fn with_replacement(mut self, el: Rc<RefCell<dyn Element>>) -> EventResponse {
        self.replacement = Some(el);
        self
    }

    pub fn with_window(mut self, w: CreateWindow) -> EventResponse {
        self.window = Some(w);
        self
    }

    pub fn with_resize(mut self, s: Size) -> EventResponse {
        self.resize = Some(s);
        self
    }

    pub fn with_relocation(mut self, r: RelocationRequest) -> EventResponse {
        self.relocation = Some(r);
        self
    }

    pub fn with_extra_locations(mut self, elr: ExtraLocationsRequest) -> EventResponse {
        self.extra_locations = Some(elr);
        self
    }

    //pub fn with_inputability_changes(mut self, ic: ReceivableEventChanges) -> EventResponse {
    pub fn with_receivable_event_changes(mut self, ic: ReceivableEventChanges) -> EventResponse {
        self.inputability_changes = Some(ic);
        self
    }

    pub fn get_receivable_event_changes(&self) -> Option<ReceivableEventChanges> {
        self.inputability_changes.clone()
    }

    pub fn with_scroll_x_static(mut self, x: i32) -> EventResponse {
        self.scroll_x_static = Some(x);
        self
    }

    pub fn with_scroll_y_static(mut self, y: i32) -> EventResponse {
        self.scroll_y_static = Some(y);
        self
    }

    // --------------------------------------
    // ReceivableEventChanges

    // TRANSLATION NOTE, used to be called remove_evs
    pub fn remove(&mut self, evs: Vec<Event>) {
        if let Some(ic) = &mut self.inputability_changes {
            ic.remove_evs(evs);
        } else {
            self.inputability_changes =
                Some(ReceivableEventChanges::default().with_remove_evs(evs));
        }
    }

    pub fn add(&mut self, evs: Vec<(Event, Priority)>) {
        if let Some(ic) = &mut self.inputability_changes {
            ic.add_evs(evs);
        } else {
            self.inputability_changes = Some(ReceivableEventChanges::default().with_evs(evs));
        }
    }

    pub fn set_rm_receivable_evs(&mut self, evs: Vec<Event>) {
        if let Some(ic) = &mut self.inputability_changes {
            ic.remove = evs;
        } else {
            self.inputability_changes =
                Some(ReceivableEventChanges::default().with_remove_evs(evs));
        }
    }

    pub fn set_add_receivable_evs(&mut self, evs: Vec<(Event, Priority)>) {
        if let Some(ic) = &mut self.inputability_changes {
            ic.add = evs;
        } else {
            self.inputability_changes = Some(ReceivableEventChanges::default().with_evs(evs));
        }
    }

    // ----------------------------------------------------------------------------

    pub fn append_chs(&mut self, chs: Vec<DrawChPos>) {
        if let Some(existing_chs) = &mut self.ch_updates {
            existing_chs.extend(chs);
        } else {
            self.ch_updates = Some(chs);
        }
    }

    //pub fn concat_inputability_changes(&mut self, ic: ReceivableEventChanges) {
    pub fn concat_receivable_event_changes(&mut self, ic: ReceivableEventChanges) {
        if let Some(existing_ic) = &mut self.inputability_changes {
            existing_ic.concat(ic);
        } else {
            self.inputability_changes = Some(ic);
        }
    }
}

// ----------------------------------------------------------------------------

// response type for creating a window
#[derive(Clone)]
pub struct CreateWindow {
    pub el: Rc<RefCell<dyn Element>>,
    pub loc: LocationSet,
}

impl CreateWindow {
    pub fn new(el: Rc<RefCell<dyn Element>>, loc: LocationSet) -> CreateWindow {
        CreateWindow { el, loc }
    }
}

// ----------------------------------------------------------------------------

// ReceivableEventChanges is used to update the receivable events of an element
// registered in the prioritizers of all ancestors
// NOTE: While processing inputability changes, element organizers remove events
// BEFORE adding events.
//

#[derive(Clone, Default)]
// TRANSLATION NOTE used to be InputabilityChanges
pub struct ReceivableEventChanges {
    // Receivable events to deregistered from an element.
    // NOTE: one instance of an event being passed up the hierarchy through
    // RmRecEvs will remove ALL instances of that event from the prioritizer of
    // every element higher in the hierarchy that processes the
    // ReceivableEventChanges.
    pub remove: Vec<Event>,
    pub add: Vec<(Event, Priority)>, // receivable events being added to the element
}

impl ReceivableEventChanges {
    pub fn with_ev(mut self, p: Priority, ev: Event) -> ReceivableEventChanges {
        self.add.push((ev, p));
        self
    }

    pub fn with_evs(mut self, evs: Vec<(Event, Priority)>) -> ReceivableEventChanges {
        self.add.extend(evs);
        self
    }

    pub fn with_remove_ev(mut self, ev: Event) -> ReceivableEventChanges {
        self.remove.push(ev);
        self
    }

    pub fn with_remove_evs(mut self, evs: Vec<Event>) -> ReceivableEventChanges {
        self.remove.extend(evs);
        self
    }

    pub fn add_ev(&mut self, ev: Event, p: Priority) {
        self.add.push((ev, p));
    }

    pub fn add_evs(&mut self, evs: Vec<(Event, Priority)>) {
        self.add.extend(evs);
    }

    pub fn add_evs_single_priority(&mut self, evs: Vec<Event>, pr: Priority) {
        for ev in evs {
            self.add.push((ev, pr));
        }
    }

    pub fn remove_ev(&mut self, ev: Event) {
        self.remove.push(ev);
    }

    pub fn remove_evs(&mut self, evs: Vec<Event>) {
        self.remove.extend(evs);
    }

    pub fn update_priority_for_ev(&mut self, ev: Event, p: Priority) {
        self.remove.push(ev.clone());
        self.add.push((ev, p));
    }

    pub fn update_priority_for_evs(&mut self, evs: Vec<Event>, p: Priority) {
        for ev in evs {
            self.remove.push(ev.clone());
            self.add.push((ev, p));
        }
    }

    pub fn concat(&mut self, cti2: ReceivableEventChanges) {
        self.remove.extend(cti2.remove);
        self.add.extend(cti2.add);
    }
}

// ---------------------------------------------------------------------
// RESPONSE REQUESTS

// RelocationRequest contains info for moving an element within its context
#[derive(Clone, Default)]
pub struct RelocationRequest {
    pub up: i32,
    pub down: i32,
    pub left: i32,
    pub right: i32,
}

impl RelocationRequest {
    pub fn new_up(up: i32) -> RelocationRequest {
        RelocationRequest {
            up,
            ..Default::default()
        }
    }
    pub fn new_down(down: i32) -> RelocationRequest {
        RelocationRequest {
            down,
            ..Default::default()
        }
    }
    pub fn new_left(left: i32) -> RelocationRequest {
        RelocationRequest {
            left,
            ..Default::default()
        }
    }
    pub fn new_right(right: i32) -> RelocationRequest {
        RelocationRequest {
            right,
            ..Default::default()
        }
    }

    pub fn new_shift(move_right: i32, move_down: i32) -> RelocationRequest {
        RelocationRequest {
            up: move_down,
            down: move_down,
            left: move_right,
            right: move_right,
        }
    }

    pub fn new_shift_right(move_right: i32) -> RelocationRequest {
        RelocationRequest {
            right: move_right,
            left: move_right,
            ..Default::default()
        }
    }

    pub fn new_shift_down(move_down: i32) -> RelocationRequest {
        RelocationRequest {
            down: move_down,
            up: move_down,
            ..Default::default()
        }
    }
}

// ExtraLocationRequest contains info for adding or removing extra locations for
// the given element
pub struct ExtraLocationRequest {
    pub add: Vec<Location>,
    pub rm: Vec<Location>,
}

impl ExtraLocationRequest {
    pub fn new(add: Vec<Location>, rm: Vec<Location>) -> ExtraLocationRequest {
        ExtraLocationRequest { add, rm }
    }
}

// sends a request to the parent to change the extra locations
// of the element
pub struct ExtraLocationsRequest {
    pub requested: bool,
    pub extra_locs: Vec<Location>,
}

impl ExtraLocationsRequest {
    pub fn new(extra_locs: Vec<Location>) -> ExtraLocationsRequest {
        ExtraLocationsRequest {
            requested: true,
            extra_locs,
        }
    }
}
