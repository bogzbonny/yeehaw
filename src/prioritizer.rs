use crate::{ElementID, Event, Keyboard};

// Priority is a rank to determine which element should be receiving user
// key strokes as they come in. When an element is in focus it should be given
// the priority of Focused which can only be exceeded if an element is given the
// Highest priority.
//
// NONE for unchanged
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Priority(pub u8);

pub const HIGHEST: Priority = Priority(0);
pub const ABOVE_FOCUSED: Priority = Priority(1);
pub const FOCUSED: Priority = Priority(2);
pub const UNFOCUSED: Priority = Priority(3);
//pub const UNCHANGED: Priority = Priority(None); // used for when changing priorities

// TRANSLATION NOTE PrioritizableEv was replaced by just Event
// TODO delete post translation
//
// PrioritizableEv is a type capable of being prioritized by the EvPrioritizer.
// It can be thought of as an Event or a category of Event capable of
// categorizing whether or not another event is a part of the category.
//pub trait PrioritizableEv: Clone + Sized {
//pub trait PrioritizableEv {
//    // The input_event  allows the PrioritizableEv to test whether it
//    // matches an input event of arbitrary kind.
//    fn matches(&self, input_event: &Event) -> bool;
//    fn key(&self) -> String; // unique key for the event
//}

// EventPrioritizer registers/provides elements and priorities which ought to
// execute specified events.
// NOTE: used to sort events by priority
#[derive(PartialEq, Eq, Clone, Default)]
pub struct EventPrioritizer(Vec<PriorityIdEvent>);

#[derive(PartialEq, Eq, Clone)]
pub struct PriorityIdEvent {
    pub priority: Priority,
    pub id: ElementID,
    pub event: Event,
}
impl Ord for PriorityIdEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority) // sort by priority
    }
}
impl PartialOrd for PriorityIdEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PriorityIdEvent {
    pub fn new(priority: Priority, id: ElementID, event: Event) -> PriorityIdEvent {
        PriorityIdEvent {
            priority,
            id,
            event,
        }
    }
}

impl EventPrioritizer {
    // are there any priority events already registered with the same priority and
    // event (independant of the event prioritizers element id).
    pub fn has_priority_ev(&self, priority_ev: &[(Event, Priority)]) -> bool {
        for pec in self.0.iter() {
            for p in priority_ev.iter() {
                if p.0 == pec.event && p.1 == pec.priority {
                    return true;
                }
            }
        }
        false
    }

    pub fn include(
        &mut self, id: ElementID, priority_ev: &Vec<(Event, Priority)>, panic_on_overload: bool,
    ) {
        // check for priority overloading
        if panic_on_overload {
            // TODO change to error
            if self.has_priority_ev(priority_ev) {
                panic!(
                    "EvPrioritizer found at least 2 events registered to different elements with the same priority: {:?}",
                    priority_ev
                );
            }
        }

        for pe in priority_ev {
            let peie = PriorityIdEvent::new(pe.1, id, pe.0.clone());
            self.0.push(peie);
            self.0.sort();
        }
    }

    // Remove removes all specified events for a given element id
    // from the EvPrioritizer.
    //
    // NOTE: Every event in the input slice will remove ALL instances of that event
    // from the prioritizer.
    pub fn remove(&mut self, id: ElementID, evs: &[Event]) {
        self.0.retain(|priority_id_event| {
            if id != priority_id_event.id {
                return true;
            }
            let has_event = evs.iter().any(|ev| ev == &priority_id_event.event);
            !has_event
        });
    }

    // removes all the registered events for the given id, returning them
    pub fn remove_entire_element(&mut self, id: ElementID) -> Vec<Event> {
        let mut out = vec![];
        self.0.retain(|priority_id_event| {
            if id != priority_id_event.id {
                return true;
            }
            out.push(priority_id_event.event.clone());
            false
        });
        out
    }

    // SPECIALTY FUNCTION
    // doesn't consider general PrioritizableEv's, only EvKeyCombos.
    //
    // GetDestinationEl gets the element able to accept the eventKey input at the
    // highest priority.
    // Since the EvPrioritizer is sorted by priority, the first element that matches
    // the eventKey will be the highest priority.
    pub fn get_destination_el_from_kb(
        &self, kb: &mut Keyboard,
    ) -> Option<(ElementID, Vec<crossterm::event::KeyEvent>)> {
        for priority_id_event in self.0.iter() {
            if priority_id_event.priority == UNFOCUSED {
                break;
            }
            let Event::KeyCombo(ref ekc) = priority_id_event.event else {
                continue;
            };
            if let Some(eks) = kb.matches(ekc, true) {
                return Some((priority_id_event.id, eks));
            }
        }
        None
    }

    // GetDestinationEl returns the id of the element that should
    // receive the given event.
    pub fn get_destination_el(&self, input_ev: &Event) -> Option<ElementID> {
        // loop through all events registered by elements (PriorityIdEvent's)
        // and check if the input_ev matches any of them
        for priority_id_event in self.0.iter() {
            if priority_id_event.priority == UNFOCUSED {
                // since the evprioritizer is sorted by priority, there is no point
                // in continuing to loop through the rest of the events as elements
                // with a priority of unfocused will never be sent events
                break;
            }

            // check if event registered w/ element matches the input_ev
            if priority_id_event.event == *input_ev {
                return Some(priority_id_event.id);
            }
        }
        None
    }
}
