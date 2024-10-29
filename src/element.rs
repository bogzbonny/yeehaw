use {
    crate::{prioritizer::Priority, Context, DrawChPos, DynLocationSet, ElementID, Event},
    std::ops::{Deref, DerefMut},
    std::{cell::RefCell, rc::Rc},
};

// Element is the base interface which all viewable elements are
// expected to fulfill
pub trait Element {
    // TODO consider removing kind
    fn kind(&self) -> &'static str; // a name for the kind of the element

    fn id(&self) -> ElementID; // the element id as assigned by the SortingHat

    // Returns all event kinds (key events and commands, etc.) which are receivable by the element.
    // This includes all events that are registered to the element itself, as well as its children,
    // via its ElementOrganizer (if it has one).
    //
    // NOTE in this current design, elements are always routed mouse events independently of
    // whether or not they are receivable according to this function.
    fn receivable(&self) -> Vec<(Event, Priority)>;

    // Receive an event from a parent. The receiving element may consume the event and/or pass it
    // to a child. The element is expected to return a response to the event, along with any
    // changes receivable events. When the event is captured, the element is expected to returns
    // captured=true.
    //                                                   (captured, response     )
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses);

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.call_hooks_of_kind(PRE_EVENT_HOOK_NAME);
        let (captured, resp) = self.receive_event_inner(ctx, ev);
        self.call_hooks_of_kind(POST_EVENT_HOOK_NAME);
        (captured, resp)
    }

    // change_priority is expected to change the priority of an element relative to its parents.
    // All receivable-events registered directly by the element should have their local priority
    // changed to 'p' while everything registered in this element's prioritizers will remain
    // unchanged (if this element is a parent element). The element will then respond with the
    // appropriate changes to its receivable events through the function return variable.
    //
    // If the priority is changing from Focused to Unfocused, then the element should respond with
    // ReceivableEventChanges where all events (local, and of its children) are set to Unfocused.
    //
    // If the priority is changed from Unfocused to Focused, then the element should respond with
    // ReceivableEventChanges with the element's local receivable events set to Focused, and all of
    // the elements children's receivable events set to whatever their local priority is.
    //
    // In all cases the reponse of this function is intended to be passed to the element's
    // parent's event prioritizer.
    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges;

    // Get the element's full drawing for the provided context.
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos>;

    // Element attributes can be used to store arbitrary values (as encoded bytes) within the
    // element. Typically if you are developing a new Element, you can simply store local values
    // within your struct, only in the situation where you a creating a new sub-class of Elements
    // (such as Widgets) does it make sense to be utilizing the get/set attribute functions.
    //
    // Current attributes used within this library:
    //  - "description": a string description of the element used everywhere
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>>;
    fn set_attribute(&self, key: &str, value: Vec<u8>);

    // sets the hook for the element, the hook is a function that is called when the element is
    // although a developer may implement any custom hook kind, the default hooks are:
    //  - "pre-visible-change": called before the element visibility changes
    //  - "post-visible-change": called before the element visibility changes
    //  - "pre-event": called before the element receives an event
    //  - "post-event": called after the element receives an event
    //  - "pre-location-change": called before the element location changes
    //  - "post-location-change": called after the element location changes
    // NOTE use caution when setting hooks, they can be used to create circular references between elements
    #[allow(clippy::type_complexity)]
    fn set_hook(
        &self,
        kind: &str,
        el_id: ElementID,
        //                  kind, hooked element
        hook: Box<dyn FnMut(&str, Box<dyn Element>)>,
    );

    fn remove_hook(&self, kind: &str, el_id: ElementID);

    // remove all hooks for the element with the given id
    fn clear_hooks_by_id(&self, el_id: ElementID);

    // calls all the hooks of the provided kind
    fn call_hooks_of_kind(&self, kind: &str);

    // Assign a reference to the element's parent through the Parent trait. This is used
    // to pass ReceivableEventChanges to the parent. (see UpwardPropogator for more context)
    fn set_parent(&self, up: Box<dyn Parent>);

    // get/set the scalable location of the widget
    // NOTE these functions should NOT be used to set values, use the set functions below to ensure
    // that hooks are called. TODO figure out some way of enforcing this
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>>;
    fn get_visible(&self) -> Rc<RefCell<bool>>;

    fn set_dyn_location_set(&self, l: DynLocationSet) {
        self.call_hooks_of_kind(PRE_LOCATION_CHANGE_HOOK_NAME);
        *self.get_dyn_location_set().borrow_mut() = l;
        self.call_hooks_of_kind(POST_LOCATION_CHANGE_HOOK_NAME);
    }

    fn set_visible(&self, v: bool) {
        self.call_hooks_of_kind(PRE_VISIBLE_CHANGE_HOOK_NAME);
        *self.get_visible().borrow_mut() = v;
        self.call_hooks_of_kind(POST_VISIBLE_CHANGE_HOOK_NAME);
    }

    // -------------------------------------------------------
    // Freebies

    fn with_description(self, desc: String) -> Self
    where
        Self: Sized,
    {
        self.set_description(desc);
        self
    }

    fn get_description(&self) -> Option<String> {
        let bz = self.get_attribute(ATTR_DESCRIPTION)?;
        match serde_json::from_slice(&bz) {
            Ok(v) => v,
            Err(_e) => None,
        }
    }

    fn set_description(&self, desc: String) {
        let bz = match serde_json::to_vec(&desc) {
            Ok(v) => v,
            Err(_e) => {
                return; // TODO log error
            }
        };
        self.set_attribute(ATTR_DESCRIPTION, bz)
    }
}

pub type HookFn = Box<dyn FnMut(&str, Rc<RefCell<dyn Element>>)>;

pub const ATTR_DESCRIPTION: &str = "standard_pane";

pub const PRE_VISIBLE_CHANGE_HOOK_NAME: &str = "pre-visible-change";
pub const POST_VISIBLE_CHANGE_HOOK_NAME: &str = "post-visible-change";
pub const PRE_EVENT_HOOK_NAME: &str = "pre-event";
pub const POST_EVENT_HOOK_NAME: &str = "post-event";
pub const PRE_LOCATION_CHANGE_HOOK_NAME: &str = "pre-location-change";
pub const POST_LOCATION_CHANGE_HOOK_NAME: &str = "post-location-change";

// ----------------------------------------

pub trait Parent: dyn_clone::DynClone {
    // The Parent is a trait that a parent element should fulfill which can then be
    // provided to child elements as a means for those child elements to propagate changes upward
    // to their parent (and grand-parents etc.).
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
    // fulfilling the Parent trait. This is used by the parent element (this one) to
    // determine which events/cmds to update the prioritizers for.
    //
    // TRANSLATION NOTE PropagateUpwardChangesToInputability propagate_upward_changes_to_inputability
    //fn propagate_receivable_event_changes_upward(
    //    &self, child_el_id: &ElementID, rec: ReceivableEventChanges,
    //);
    fn propagate_responses_upward(&self, child_el_id: &ElementID, resps: EventResponses);

    fn get_store_item(&self, key: &str) -> Option<Vec<u8>>;
    fn set_store_item(&self, key: &str, value: Vec<u8>);
}

// ----------------------------------------------------------------------------

#[derive(Default)]
pub struct EventResponses(pub Vec<EventResponse>);

impl From<EventResponse> for EventResponses {
    fn from(er: EventResponse) -> EventResponses {
        EventResponses(vec![er])
    }
}

impl Deref for EventResponses {
    type Target = Vec<EventResponse>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EventResponses {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl EventResponses {
    // retrieves only the ReceivableEventChanges from the EventResponses
    // and concats them together
    pub fn get_receivable_event_changes(&self) -> ReceivableEventChanges {
        let mut rec = ReceivableEventChanges::default();
        for er in &self.0 {
            if let EventResponse::ReceivableEventChanges(r) = er {
                rec.concat(r.clone());
            }
        }
        rec
    }
}

// EventResponse is used to send information back to the parent that delivered
// the event to the element
#[derive(Default)]
pub enum EventResponse {
    #[default]
    None,

    // quit the application
    Quit,

    // destroy the current element
    Destruct,

    // bring this element to the front (greatest z-index)
    BringToFront,

    // create an element, its location will be adjusted
    // by the elements current location.
    //
    // this response can be used to create a window
    // or when used in conjunction with destruct, it can be used to replace
    // the current element with another.
    NewElement(Rc<RefCell<dyn Element>>),

    // arbitrary custom metadatas which can be passed back to the parent
    //       key,   , value
    Metadata(String, Vec<u8>),

    // sends a request to the parent to change the extra locations
    // of the element. TODO refactor to remove this, it should just be taking
    // place on the element itself, ensure the text box right click menu will work.
    //ExtraLocations(Vec<DynLocation>),

    // contains priority updates that should be made to the receiver's prioritizer
    ReceivableEventChanges(ReceivableEventChanges),
}

impl EventResponse {
    pub fn has_metadata(&self, key: &str) -> bool {
        matches!(self, EventResponse::Metadata(k, _) if k == key)
    }

    // --------------------------------------
    // ReceivableEventChanges

    //// TRANSLATION NOTE, used to be called remove_evs
    //pub fn remove(&mut self, evs: Vec<Event>) {
    //    if let Some(ic) = &mut self.inputability_changes {
    //        ic.remove_evs(evs);
    //    } else {
    //        self.inputability_changes =
    //            Some(ReceivableEventChanges::default().with_remove_evs(evs));
    //    }
    //}

    //pub fn add(&mut self, evs: Vec<(Event, Priority)>) {
    //    if let Some(ic) = &mut self.inputability_changes {
    //        ic.set_add_evs(evs);
    //    } else {
    //        self.inputability_changes = Some(ReceivableEventChanges::default().with_add_evs(evs));
    //    }
    //}

    //pub fn set_rm_receivable_evs(&mut self, evs: Vec<Event>) {
    //    if let Some(ic) = &mut self.inputability_changes {
    //        ic.remove = evs;
    //    } else {
    //        self.inputability_changes =
    //            Some(ReceivableEventChanges::default().with_remove_evs(evs));
    //    }
    //}

    //pub fn set_add_receivable_evs(&mut self, evs: Vec<(Event, Priority)>) {
    //    if let Some(ic) = &mut self.inputability_changes {
    //        ic.add = evs;
    //    } else {
    //        self.inputability_changes = Some(ReceivableEventChanges::default().with_add_evs(evs));
    //    }
    //}

    //// ----------------------------------------------------------------------------

    ////pub fn concat_inputability_changes(&mut self, ic: ReceivableEventChanges) {
    //pub fn concat_receivable_event_changes(&mut self, ic: ReceivableEventChanges) {
    //    if let Some(existing_ic) = &mut self.inputability_changes {
    //        existing_ic.concat(ic);
    //    } else {
    //        self.inputability_changes = Some(ic);
    //    }
    //}
}

// ----------------------------------------------------------------------------

// ReceivableEventChanges is used to update the receivable events of an element
// registered in the prioritizers of all ancestors
// NOTE: While processing inputability changes, element organizers remove events
// BEFORE adding events.
//

#[derive(Clone, Default, Debug)]
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
    pub fn with_add_ev(mut self, p: Priority, ev: Event) -> ReceivableEventChanges {
        self.add.push((ev, p));
        self
    }

    pub fn with_add_evs(mut self, evs: Vec<(Event, Priority)>) -> ReceivableEventChanges {
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

    pub fn set_add_ev(&mut self, ev: Event, p: Priority) {
        self.add.push((ev, p));
    }

    pub fn set_add_evs(&mut self, evs: Vec<(Event, Priority)>) {
        self.add.extend(evs);
    }

    pub fn set_add_evs_single_priority(&mut self, evs: Vec<Event>, pr: Priority) {
        for ev in evs {
            self.add.push((ev, pr));
        }
    }

    pub fn set_remove_ev(&mut self, ev: Event) {
        self.remove.push(ev);
    }

    pub fn set_remove_evs(&mut self, evs: Vec<Event>) {
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

    pub fn concat(&mut self, rec: ReceivableEventChanges) {
        self.remove.extend(rec.remove);
        self.add.extend(rec.add);
    }
}
