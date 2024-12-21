// XXX delete file

use crate::{ElementID, Event, Keyboard, ReceivableEvent, ReceivableEventChanges};

/// Priority is a rank to determine which element should be receiving user
/// key strokes as they come in. When an element is in focus it should be given
/// the priority of Focused which can only be exceeded if an element is given the
/// Highest priority.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Priority {
    Focused,
    Unfocused,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Priority::Focused => write!(f, "Focused"),
            Priority::Unfocused => write!(f, "Unfocused"),
        }
    }
}

/// EventPrioritizer registers/provides elements and priorities which ought to
/// execute specified events.
/// NOTE: used to sort events by priority
#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct EventPrioritizer(Vec<PriorityIdEvent>);

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PriorityIdEvent {
    pub priority: Priority,
    pub id: ElementID,
    pub event: ReceivableEvent,
}
impl Ord for PriorityIdEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority) // / sort by priority
    }
}
impl PartialOrd for PriorityIdEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PriorityIdEvent {
    pub fn new(priority: Priority, id: ElementID, event: ReceivableEvent) -> PriorityIdEvent {
        PriorityIdEvent {
            priority,
            id,
            event,
        }
    }
}

impl EventPrioritizer {
    /// write func to remove/add evCombos and commands from EvPrioritizer and
    /// CommandPrioritizer, using the ReceivableEventChanges struct
    pub fn process_receivable_event_changes(
        &mut self, el_id: &ElementID, rec: &ReceivableEventChanges,
    ) {
        self.remove(el_id, &rec.remove);
        self.include(el_id, &rec.add);
    }

    /// are there any priority events already registered with the same priority and
    /// event (independant of the event prioritizers element id).
    pub fn get_priority_ev_id(
        &self, priority_ev: &(ReceivableEvent, Priority),
    ) -> Option<ElementID> {
        for pec in self.0.iter() {
            if priority_ev.0 == pec.event && priority_ev.1 == pec.priority {
                return Some(pec.id.clone());
            }
        }
        None
    }

    pub fn include(&mut self, id: &ElementID, priority_ev: &Vec<(ReceivableEvent, Priority)>) {
        for pe in priority_ev {
            let peie = PriorityIdEvent::new(pe.1, id.clone(), pe.0.clone());
            self.0.push(peie);
            self.0.sort();
        }
    }

    // check for priority overloading.
    // Panic if two children have registered the same ev/cmd at the same priority
    // (excluding Unfocused). If false the event will be sent to the ev/cmd to which
    // happens to be first in the prioritizer
    pub fn ensure_no_duplicate_priorities(&self, _parent: &ElementID) {
        #[cfg(debug_assertions)]
        for pe in self.0.iter() {
            if pe.priority != Priority::Unfocused {
                if let Some(existing_id) = self.get_priority_ev_id(&(pe.event.clone(), pe.priority))
                {
                    if existing_id != *pe.id {
                        panic!(
                            "EvPrioritizer found at least 2 events registered to different \
                             elements with the same priority for parent {_parent}. \
                             \n\tid1: {existing_id} \
                             \n\tid2: {}\n\tpr: {}\n\tev: {:?}",
                            pe.id, pe.priority, pe.event
                        )
                    }
                }
            }
        }
    }

    /// Remove removes all specified events for a given element id
    /// from the EvPrioritizer.
    ///
    /// NOTE: Every event in the input slice will remove ALL instances of that event
    /// from the prioritizer.
    pub fn remove(&mut self, id: &ElementID, evs: &[ReceivableEvent]) {
        self.0.retain(|priority_id_event| {
            if id != &priority_id_event.id {
                return true;
            }
            let has_event = evs.iter().any(|ev| ev == &priority_id_event.event);
            !has_event
        });
    }

    /// removes all the registered events for the given id, returns the removed events
    pub fn remove_entire_element(&mut self, id: &ElementID) -> Vec<ReceivableEvent> {
        let mut removed = vec![];
        self.0.retain(|priority_id_event| {
            if id != &priority_id_event.id {
                return true;
            }
            removed.push(priority_id_event.event.clone());
            false
        });
        removed
    }

    /// SPECIALTY FUNCTION
    /// doesn't consider general PrioritizableEv's, only EvKeyCombos.
    ///
    /// GetDestinationEl gets the element able to accept the eventKey input at the
    /// highest priority.
    /// Since the EvPrioritizer is sorted by priority, the first element that matches
    /// the eventKey will be the highest priority.
    pub fn get_destination_el_from_kb(
        &self, kb: &mut Keyboard,
    ) -> Option<(ElementID, Vec<crossterm::event::KeyEvent>)> {
        for priority_id_event in self.0.iter() {
            if priority_id_event.priority == Priority::Unfocused {
                break;
            }
            let ReceivableEvent::KeyCombo(ref ekc) = priority_id_event.event else {
                continue;
            };
            if let Some(eks) = kb.matches(ekc, true) {
                return Some((priority_id_event.id.clone(), eks));
            }
        }
        None
    }

    /// GetDestinationEl returns the id of the element that should
    /// receive the given event.
    pub fn get_destination_el(&self, input_ev: &Event) -> Vec<ElementID> {
        let mut dests = vec![];
        // loop through all events registered by elements (PriorityIdEvent's)
        // and check if the input_ev matches any of them
        for priority_id_event in self.0.iter() {
            //if priority_id_event.priority == Priority::Unfocused {
            //    // since the ev prioritizer is sorted by priority, there is no point
            //    // in continuing to loop through the rest of the events as elements
            //    // with a priority of unfocused will never be sent events
            //    break;
            //}

            // check if event registered w/ element matches the input_ev
            if priority_id_event.event.matches(input_ev) && !dests.contains(&priority_id_event.id) {
                dests.push(priority_id_event.id.clone());
            }
        }
        dests
    }
}
