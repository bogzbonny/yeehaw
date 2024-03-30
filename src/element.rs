use {
    crate::{
        prioritizer::{PrioritizableEv, Priority},
        DrawChPos, Event, Location, Locations, Size,
    },
    std::any::Any,
};

// Element is the base interface which all viewable elements are
// expected to fulfill
pub trait Element {
    // Returns all events (key events and commands, etc.) that are registered to
    // an element. This includes all events that are registered to the element
    // itself, as well as its children, via its ElementOrganizer (if it has
    // one).
    // NOTE in this current design, elements are always expected to receive
    // mouse events.
    fn receivable(&self) -> Vec<(Priority, Box<dyn PrioritizableEv>)>;

    // This is used to receive an event from a parent. The receiving element may
    // consume the event and/or pass it to a child. The element is expected to
    // return a response to the event, along with any changes to its
    // inputability that its parent will use as well pass up the tree. When the
    // event is captured, the element is expected to returns captured=true.
    fn receive_event(&self, ctx: Context, ev: Event) -> (bool, EventResponse);

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
    fn change_priority(&self, ctx: Context, p: Priority) -> InputabilityChanges;

    // get the element's full drawing for the provided width and height
    // this is provided as an ordered list of individual elements to draw
    // z = the element viewing depth, 0 is the topmost element
    // freeFloating = whether the element is constrained by the parent
    //                element border
    fn drawing(&self, ctx: Context) -> Vec<DrawChPos>;

    // Passes ChangesInInputability to the parent of this element, while
    // optionally making changes to the calling element organizer's prioritizers.
    //
    // childEl is the element which is invoking the propagation from BELOW this
    // parent pane. This is used by the parent to determine which events/cmds to
    // update the prioritizers for.
    //
    // If updateThisElementsPrioritizers is true, then the prioritizers for this
    // element will be updated. This should always be the case except at the
    // initialization of the upward propagation process. In that case, the
    // changes to the element calling should have alread been handled by said
    // element.
    //
    // NOTE: In most cases, changes in inputability are passed to the parent in
    // the return values of a function invoked on the element by the parent (ex.
    // ReceiveEvent). However, when changes are initiated laterally (by a
    // sibling), the parent must be notified of the changes. This function
    // accomplishes that.
    fn propagate_upward_changes_to_inputability(
        &self,
        el: Box<dyn Element>,
        ic: InputabilityChanges,
        update_this_elements_prioritizers: bool,
    );

    // Assign a reference to the element's parent through the UpwardPropagator
    // interface. This is used to pass changes in inputability to the parent.
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>);
}

// Context is a struct which contains information about the current context of a
// given element.
// The context of an element is passed to the element during key function calls
// where the element may need to know its size and Visibility.
//
// Additionally, Metadata may be addended to the context to pass additional
// arbitrary information.
pub struct Context {
    pub s: Size,
    pub visible: bool,
    pub metadata: Option<Box<dyn Any>>,
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
    fn new_context_for_screen() -> Context {
        let (xmax, ymax) = crossterm::terminal::size().unwrap();
        Context {
            s: Size::new(xmax, ymax),
            visible: true,
            metadata: None,
        }
    }

    fn with_metadata(self, md: Box<dyn Any>) -> Context {
        Context {
            s: self.s,
            visible: self.visible,
            metadata: Some(md),
        }
    }

    fn get_width(&self) -> u16 {
        self.s.width
    }

    fn get_height(&self) -> u16 {
        self.s.height
    }
}

// ----------------------------------------------------------------------------

// EventResponse is used to send information back to the parent that delivered
// the event to the element
#[derive(Default)]
pub struct EventResponses(Vec<EventResponse>);

pub enum EventResponse {
    // character updates to the element's drawing
    ChUpdates(Vec<DrawChPos>),
    // metadata can be used to send back any arbitrary information
    // it is up to the parent to interpret the metadata
    Metadata(Box<dyn Any>),
    // replace the current element with the provided element
    Replacement(Box<dyn Element>),
    // request that the provided window element be created at the location
    Window(CreateWindow), // TODO
    // destroy the current element
    Destruct,
    // quit the application
    Quit,
    // deactivate (used with widgets
    Deactivate,
    // sends a request to the parent to change the size of the element
    Resize(Size),
    // send a request to the parent to change the position of the element
    Relocation(RelocationRequest),
    // sends a request to the parent to change the extra locations
    // of the element
    ExtraLocations(ExtraLocationsRequest),
    // contains priority updates that should be made to the receiver's prioritizer
    InputabilityChanges(InputabilityChanges),
    // for use with scrollbars
    ScrollXStatic(i32),
    ScrollYStatic(i32),
}

impl EventResponses {
    pub fn with_ch_updates(mut self, chs: Vec<DrawChPos>) -> EventResponses {
        self.0.push(EventResponse::ChUpdates(chs));
        self
    }

    pub fn with_metadata(mut self, md: Box<dyn Any>) -> EventResponses {
        self.0.push(EventResponse::Metadata(md));
        self
    }

    pub fn with_replacement(mut self, el: Box<dyn Element>) -> EventResponses {
        self.0.push(EventResponse::Replacement(el));
        self
    }

    pub fn with_window(mut self, w: CreateWindow) -> EventResponses {
        self.0.push(EventResponse::Window(w));
        self
    }

    pub fn with_destruct(mut self) -> EventResponses {
        self.0.push(EventResponse::Destruct);
        self
    }

    pub fn with_quit(mut self) -> EventResponses {
        self.0.push(EventResponse::Quit);
        self
    }

    pub fn with_deactivate(mut self) -> EventResponses {
        self.0.push(EventResponse::Deactivate);
        self
    }

    pub fn with_resize(mut self, s: Size) -> EventResponses {
        self.0.push(EventResponse::Resize(s));
        self
    }

    pub fn with_relocation(mut self, rr: RelocationRequest) -> EventResponses {
        self.0.push(EventResponse::Relocation(rr));
        self
    }

    pub fn with_extra_locations(mut self, elr: ExtraLocationsRequest) -> EventResponses {
        self.0.push(EventResponse::ExtraLocations(elr));
        self
    }

    pub fn with_inputability_changes(mut self, ic: InputabilityChanges) -> EventResponses {
        self.0.push(EventResponse::InputabilityChanges(ic));
        self
    }

    pub fn with_scroll_x_static(mut self, i: i32) -> EventResponses {
        self.0.push(EventResponse::ScrollXStatic(i));
        self
    }

    pub fn with_scroll_y_static(mut self, i: i32) -> EventResponses {
        self.0.push(EventResponse::ScrollYStatic(i));
        self
    }

    // --------------------

    fn get_ch_updates(&self) -> Option<&Vec<DrawChPos>> {
        for er in &self.0 {
            if let EventResponse::ChUpdates(chs) = er {
                return Some(chs.clone());
            }
        }
        None
    }

    fn remove_ch_updates(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::ChUpdates(_) = er {
                false
            } else {
                true
            }
        });
    }

    fn get_metadata(&self) -> Option<&Box<dyn Any>> {
        for er in &self.0 {
            if let EventResponse::Metadata(md) = er {
                return Some(md.clone());
            }
        }
        None
    }

    fn remove_metadata(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::Metadata(_) = er {
                false
            } else {
                true
            }
        });
    }
    fn get_replacement(&self) -> Option<&Box<dyn Element>> {
        for er in &self.0 {
            if let EventResponse::Replacement(el) = er {
                return Some(el.clone());
            }
        }
        None
    }
    fn remove_replacement(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::Replacement(_) = er {
                false
            } else {
                true
            }
        });
    }
    fn get_window(&self) -> Option<&CreateWindow> {
        for er in &self.0 {
            if let EventResponse::Window(w) = er {
                return Some(w.clone());
            }
        }
        None
    }
    fn remove_window(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::Window(_) = er {
                false
            } else {
                true
            }
        });
    }
    fn get_destruct(&self) -> bool {
        for er in &self.0 {
            if let EventResponse::Destruct = er {
                return true;
            }
        }
        false
    }
    fn remove_destruct(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::Destruct = er {
                false
            } else {
                true
            }
        });
    }
    fn get_quit(&self) -> bool {
        for er in &self.0 {
            if let EventResponse::Quit = er {
                return true;
            }
        }
        false
    }
    fn remove_quit(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::Quit = er {
                false
            } else {
                true
            }
        });
    }
    fn get_deactivate(&self) -> bool {
        for er in &self.0 {
            if let EventResponse::Deactivate = er {
                return true;
            }
        }
        false
    }
    fn remove_deactivate(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::Deactivate = er {
                false
            } else {
                true
            }
        });
    }
    fn get_resize(&self) -> Option<&Size> {
        for er in &self.0 {
            if let EventResponse::Resize(s) = er {
                return Some(s.clone());
            }
        }
        None
    }
    fn remove_resize(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::Resize(_) = er {
                false
            } else {
                true
            }
        });
    }
    fn get_relocation(&self) -> Option<RelocationRequest> {
        for er in &self.0 {
            if let EventResponse::Relocation(rr) = er {
                return Some(rr.clone());
            }
        }
        None
    }
    fn remove_relocation(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::Relocation(_) = er {
                false
            } else {
                true
            }
        });
    }
    fn get_extra_locations(&self) -> Option<&ExtraLocationsRequest> {
        for er in &self.0 {
            if let EventResponse::ExtraLocations(elr) = er {
                return Some(elr.clone());
            }
        }
        None
    }
    fn remove_extra_locations(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::ExtraLocations(_) = er {
                false
            } else {
                true
            }
        });
    }
    fn get_inputability_changes(&self) -> Option<&InputabilityChanges> {
        for er in &self.0 {
            if let EventResponse::InputabilityChanges(ic) = er {
                return Some(ic.clone());
            }
        }
        None
    }
    fn remove_inputability_changes(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::InputabilityChanges(_) = er {
                false
            } else {
                true
            }
        });
    }
    fn get_scroll_x_static(&self) -> Option<i32> {
        for er in &self.0 {
            if let EventResponse::ScrollXStatic(i) = er {
                return Some(*i);
            }
        }
        None
    }
    fn remove_scroll_x_static(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::ScrollXStatic(_) = er {
                false
            } else {
                true
            }
        });
    }
    fn get_scroll_y_static(&self) -> Option<i32> {
        for er in &self.0 {
            if let EventResponse::ScrollYStatic(i) = er {
                return Some(*i);
            }
        }
        None
    }
    fn remove_scroll_y_static(&mut self) {
        self.0.retain(|er| {
            if let EventResponse::ScrollYStatic(_) = er {
                false
            } else {
                true
            }
        });
    }

    // --------------------------------------
    // InputabilityChanges

    fn set_rm_rec_evs(&mut self, rm_rec_evs: Vec<Box<dyn PrioritizableEv>>) {
        if let Some(mut ic) = self.get_inputability_changes() {
            ic.rm_rec_evs = rm_rec_evs;
            self.remove_inputability_changes();
            self.0.push(EventResponse::InputabilityChanges(*ic));
        } else {
            self.0
                .push(EventResponse::InputabilityChanges(InputabilityChanges {
                    rm_rec_evs,
                    add_rec_evs: Vec::new(),
                }));
        }
    }

    fn set_add_rec_evs(&mut self, add_rec_evs: Vec<(Box<dyn PrioritizableEv>, Priority)>) {
        if let Some(mut ic) = self.get_inputability_changes() {
            ic.add_rec_evs = add_rec_evs;
            self.remove_inputability_changes();
            self.0.push(EventResponse::InputabilityChanges(*ic));
        } else {
            self.0
                .push(EventResponse::InputabilityChanges(InputabilityChanges {
                    rm_rec_evs: Vec::new(),
                    add_rec_evs,
                }));
        }
    }

    fn remove_evs(&mut self, ecs: Vec<Box<dyn PrioritizableEv>>) {
        if let Some(mut ic) = self.get_inputability_changes() {
            ic.remove_evs(ecs);
            self.remove_inputability_changes();
            self.0.push(EventResponse::InputabilityChanges(ic));
        } else {
            self.0
                .push(EventResponse::InputabilityChanges(InputabilityChanges {
                    rm_rec_evs: ecs,
                    add_rec_evs: Vec::new(),
                }));
        }
    }
}

// ----------------------------------------------------------------------------

// response type for creating a window

pub struct CreateWindow {
    pub el: Option<Box<dyn Element>>,
    pub loc: Locations,
}

impl CreateWindow {
    pub fn new(el: Box<dyn Element>, loc: Locations) -> CreateWindow {
        CreateWindow { el: Some(el), loc }
    }

    pub fn has_window(&self) -> bool {
        self.el.is_some()
    }
}

// ----------------------------------------------------------------------------

// InputabilityChanges is used to update the receivable events of an element
// registered in the prioritizers of all ancestors
// NOTE: While processing inputability changes, element organizers remove events
// BEFORE adding events.
#[derive(Default)]
pub struct InputabilityChanges {
    // Receivable events to deregistered from an element.
    // NOTE: one instance of an event being passed up the hierarchy through
    // RmRecEvs will remove ALL instances of that event from the prioritizer of
    // every element higher in the hierarchy that processes the
    // InputabilityChanges.
    rm_rec_evs: Vec<Box<dyn PrioritizableEv>>,

    add_rec_evs: Vec<(Box<dyn PrioritizableEv>, Priority)>, // receivable events being added to the element
}

impl InputabilityChanges {
    pub fn with_ev(mut self, ev: Box<dyn PrioritizableEv>, p: Priority) -> InputabilityChanges {
        self.add_rec_evs.push((ev, p));
        self
    }

    pub fn with_evs(
        mut self,
        evs: Vec<(Box<dyn PrioritizableEv>, Priority)>,
    ) -> InputabilityChanges {
        self.add_rec_evs.extend(evs);
        self
    }

    pub fn with_remove_ev(mut self, ev: Box<dyn PrioritizableEv>) -> InputabilityChanges {
        self.rm_rec_evs.push(ev);
        self
    }

    pub fn with_remove_evs(mut self, evs: Vec<Box<dyn PrioritizableEv>>) -> InputabilityChanges {
        self.rm_rec_evs.extend(evs);
        self
    }

    pub fn update_priority_for_ev(
        mut self,
        ev: Box<dyn PrioritizableEv>,
        p: Priority,
    ) -> InputabilityChanges {
        self.rm_rec_evs.push(ev);
        self.add_rec_evs.push((ev, p));
        self
    }

    pub fn update_priority_for_evs(
        mut self,
        evs: Vec<Box<dyn PrioritizableEv>>,
        p: Priority,
    ) -> InputabilityChanges {
        for ev in evs {
            self.rm_rec_evs.push(ev);
            self.add_rec_evs.push((ev, p));
        }
        self
    }

    pub fn concat(&mut self, cti2: InputabilityChanges) {
        self.rm_rec_evs.extend(cti2.rm_rec_evs);
        self.add_rec_evs.extend(cti2.add_rec_evs);
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
    add: Vec<Location>,
    rm: Vec<Location>,
}

impl ExtraLocationRequest {
    pub fn new(add: Vec<Location>, rm: Vec<Location>) -> ExtraLocationRequest {
        ExtraLocationRequest { add, rm }
    }
}

// sends a request to the parent to change the extra locations
// of the element
pub struct ExtraLocationsRequest {
    requested: bool,
    extra_locs: Vec<Location>,
}

impl ExtraLocationsRequest {
    pub fn new(extra_locs: Vec<Location>) -> ExtraLocationsRequest {
        ExtraLocationsRequest {
            requested: true,
            extra_locs,
        }
    }
}

// ---------------------------------------------------

pub trait UpwardPropagator {
    fn propagate_upward_changes_to_inputability(
        &self,
        el: Box<dyn Element>,
        ic: InputabilityChanges,
        update_this_elements_prioritizers: bool,
    );

    // XXX TODO add a function to get an elements name here (for debugging),
    // once an element has SetUpwardPropagator called on it, it will
    // be able to construct its own name, by tacking on it's own name to its
    // parents name.

    // returns a name of the element (for debugging purposes)
    //fn name(&self) -> String;
}
