use crate::Event;

// Priority is a rank to determine which element should be receiving user
// key strokes as they come in. When an element is in focus it should be given
// the priority of Focused which can only be exceeded if an element is given the
// Highest priority.
// NONE for unchanged
pub struct Priority(Option<usize>);

//const (
//    Highest      Priority = 0
//    AboveFocused Priority = 1
//    Focused      Priority = 2 // standard priority of a pane in focus
//    Unfocused    Priority = 3 // will never receive key events

//    Unchanged Priority = -1 // used for when changing priorities
//)

// PrioritizableEv is a type capable of being prioritized by the EvPrioritizer.
// It can be thought of as an Event or a category of Event capable of
// categorizing whether or not another event is a part of the category.
pub trait PrioritizableEv {
    // The input_event  allows the PrioritizableEv to test whether it
    // matches an input event of arbitrary kind.
    fn matches(&self, input_event: &Event) -> bool;
    fn key(&self) -> String;
}
