#[derive(Clone)]
pub enum Event {
    MouseEvent(crossterm::event::MouseEvent),

    KeysEvent(Vec<crossterm::event::KeyEvent>),

    // The ExternalMouseEvent is send to all elements
    // who are not considered to be the "Receiver" of the event
    //
    // This is used to receive from a parent, a mouse event that neither it, nor its
    // children, are meant to consume. This is used to tell an element that
    // another element, somewhere in the cui, has received/consumed a mouse event.
    ExternalMouseEvent(crossterm::event::MouseEvent),

    // Used to tell an element that the screen has resized. The element should
    // then adjust all of its children based on the given context
    ResizeEvent,

    // Refresh resets an element's organizer's prioritizers as well as triggering a
    // Resize event in all children. This essentially refreshes the state of the
    // element organizer.
    // Currently only relevant for elements that have an element organizer.
    RefreshEvent,

    CommandEvent(CommandEvent),
}

// ----------------------------------------

// Event for triggering a command execution for an element
#[derive(Clone)]
pub struct CommandEvent {
    pub cmd: CommandStr,
    pub args: Vec<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CommandStr(String);

impl CommandStr {
    pub fn new(s: String) -> CommandStr {
        CommandStr(s)
    }
    pub fn key(&self) -> String {
        "COMMAND=".to_string() + &self.0
    }
}
