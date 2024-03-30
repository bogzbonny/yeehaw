#[derive(Clone)]
pub enum Event {
    Mouse(crossterm::event::MouseEvent),

    KeyCombo(Vec<crossterm::event::KeyEvent>),

    // The ExternalMouseEvent is send to all elements
    // who are not considered to be the "Receiver" of the event
    //
    // This is used to receive from a parent, a mouse event that neither it, nor its
    // children, are meant to consume. This is used to tell an element that
    // another element, somewhere in the cui, has received/consumed a mouse event.
    ExternalMouse(crossterm::event::MouseEvent),

    // Used to tell an element that the screen has resized. The element should
    // then adjust all of its children based on the given context
    Resize,

    // Refresh resets an element's organizer's prioritizers as well as triggering a
    // Resize event in all children. This essentially refreshes the state of the
    // element organizer.
    // Currently only relevant for elements that have an element organizer.
    Refresh,

    Command(CommandEvent),
}

impl Event {
    pub fn key(&self) -> String {
        match self {
            Event::Mouse(_) => "MOUSE".to_string(),
            Event::KeyCombo(keys) => {
                let mut out = String::new();
                for key in keys {
                    out += &format!("{:?}, ", key);
                }
                if out.len() > 2 {
                    // remove the final ", "
                    out.pop();
                    out.pop();
                }
                format!("KEY_COMBO=[{}]", out)
            }
            Event::ExternalMouse(_) => "EXTERNAL_MOUSE".to_string(),
            Event::Resize => "RESIZE".to_string(),
            Event::Refresh => "REFRESH".to_string(),
            Event::Command(ev) => "COMMAND=".to_string() + &ev.cmd,
        }
    }
}

// ----------------------------------------

// Event for triggering a command execution for an element
#[derive(Clone)]
pub struct CommandEvent {
    pub cmd: String,
    pub args: Vec<String>,
}
