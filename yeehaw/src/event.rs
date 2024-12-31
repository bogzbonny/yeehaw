use {
    crate::{DrawRegion, Element, ElementID},
    std::ops::{Deref, DerefMut},
};

// TODO build in mouse events here

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ReceivableEvent {
    KeyCombo(Vec<KeyPossibility>),
    /// custom event name
    Custom(String),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Event {
    /// The Initialize event resets an element's organizer's prioritizers. This essentially
    /// refreshes the state of the element organizer for elements which have an organizer.
    Initialize,

    /// A signal to an element that it should closedown.
    Exit,

    /// Used to tell an element that the screen has resized. The element should
    /// then adjust all of its children based on the given context
    Resize,

    KeyCombo(Vec<crossterm::event::KeyEvent>),

    Mouse(MouseEvent),

    /// The ExternalMouseEvent is send to all elements
    /// who are not considered to be the "Receiver" of the event
    ///
    /// This is used to receive from a parent, a mouse event that neither it, nor its
    /// children, are meant to consume. This is used to tell an element that
    /// another element, somewhere in the tui, has received/consumed a mouse event.
    ///
    /// NOTE the column and row are the column and row of the mouse event relative
    /// to the element receiving the event, hence they may be negative.
    ExternalMouse(MouseEvent),

    /// custom event type with a name and a payload
    Custom(String, Vec<u8>),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct MouseEvent {
    /// The draw region of the element that received the event. This is necessary for the element
    /// to know where the event occurred relative to itself. and also which sub-element to possibly
    /// send the event to. This is coupled with the MouseEvent and not the Event because the
    /// MouseEvent is the only event which should require knowing the DrawRegion when receiving an
    /// event.
    pub dr: DrawRegion,

    /// The kind of mouse event that was caused.
    pub kind: crossterm::event::MouseEventKind,
    /// The column that the event occurred on relative to the start of the element.
    pub column: i32,
    /// The row that the event occurred on relative to the start of the element.
    pub row: i32,
    /// The key modifiers active when the event occurred.
    pub modifiers: crossterm::event::KeyModifiers,
}

impl MouseEvent {
    pub fn new(dr: DrawRegion, ev: crossterm::event::MouseEvent) -> Self {
        MouseEvent {
            dr,
            kind: ev.kind,
            column: ev.column as i32,
            row: ev.row as i32,
            modifiers: ev.modifiers,
        }
    }
}

impl From<crossterm::event::KeyEvent> for ReceivableEvent {
    fn from(key: crossterm::event::KeyEvent) -> Self {
        ReceivableEvent::KeyCombo(vec![key.into()])
    }
}

impl From<KeyPossibility> for ReceivableEvent {
    fn from(key: KeyPossibility) -> Self {
        ReceivableEvent::KeyCombo(vec![key])
    }
}

impl From<crossterm::event::KeyEvent> for Event {
    fn from(key: crossterm::event::KeyEvent) -> Self {
        Event::KeyCombo(vec![key])
    }
}

impl From<Vec<crossterm::event::KeyEvent>> for Event {
    fn from(keys: Vec<crossterm::event::KeyEvent>) -> Self {
        Event::KeyCombo(keys)
    }
}

impl Event {
    pub fn identifier(&self) -> String {
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
            Event::Initialize => "REFRESH".to_string(),
            Event::Exit => "EXIT".to_string(),
            Event::Custom(name, _) => "CUSTOM=".to_string() + name,
        }
    }
}

impl ReceivableEvent {
    pub fn matches(&self, other: &Event) -> bool {
        match (self, other) {
            (ReceivableEvent::KeyCombo(k1), Event::KeyCombo(k2)) => {
                if k1.len() != k2.len() {
                    return false;
                }
                for (i, k) in k1.iter().enumerate() {
                    if !k.matches_key(&k2[i]) {
                        return false;
                    }
                }
                true
            }
            (ReceivableEvent::Custom(kind1), Event::Custom(kind2, _)) => kind1 == kind2,
            _ => false,
        }
    }
}

/// Event for triggering a command execution for an element
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CommandEvent {
    pub cmd: String,
    pub args: Vec<String>,
}

/// KeyPossibility is used to match a key event
/// with a specific key or a group of keys
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum KeyPossibility {
    Key(crossterm::event::KeyEvent),
    Chars,
    /// any char
    Digits,
    /// any digit
    Anything,
}

impl From<crossterm::event::KeyEvent> for KeyPossibility {
    fn from(k: crossterm::event::KeyEvent) -> Self {
        KeyPossibility::Key(k)
    }
}

impl KeyPossibility {
    pub fn get_key(&self) -> Option<crossterm::event::KeyEvent> {
        match self {
            KeyPossibility::Key(k) => Some(*k),
            _ => None,
        }
    }

    pub fn matches(&self, key_p: &KeyPossibility) -> bool {
        match self {
            KeyPossibility::Key(k) => key_p.matches_key(k),
            KeyPossibility::Chars => {
                if matches!(key_p, KeyPossibility::Chars) {
                    return true;
                }
                if let KeyPossibility::Key(k) = key_p {
                    matches!(k.code, crossterm::event::KeyCode::Char(_))
                } else {
                    false
                }
            }
            KeyPossibility::Digits => matches!(key_p, KeyPossibility::Digits),
            KeyPossibility::Anything => true,
        }
    }

    pub fn matches_key(&self, ct_key: &crossterm::event::KeyEvent) -> bool {
        match self {
            KeyPossibility::Key(k) => k == ct_key,
            KeyPossibility::Chars => {
                matches!(ct_key.code, crossterm::event::KeyCode::Char(_))
            }
            KeyPossibility::Digits => {
                let crossterm::event::KeyCode::Char(c) = ct_key.code else {
                    return false;
                };
                c.is_ascii_digit()
            }
            KeyPossibility::Anything => true,
        }
    }

    pub fn get_char(&self) -> Option<char> {
        match self {
            KeyPossibility::Key(k) => {
                if let crossterm::event::KeyCode::Char(c) = k.code {
                    Some(c)
                } else {
                    None
                }
            }
            KeyPossibility::Chars => None,
            KeyPossibility::Digits => None,
            KeyPossibility::Anything => None,
        }
    }
}

// -------------------------------------------------------------------------------------

impl From<Box<dyn Element>> for EventResponse {
    fn from(el: Box<dyn Element>) -> EventResponse {
        EventResponse::NewElement(el, None)
    }
}

impl From<()> for EventResponse {
    fn from(_: ()) -> EventResponse {
        EventResponse::None
    }
}

/// EventResponse is used to send information back to the parent that delivered
/// the event to the element
#[derive(Default)]
pub enum EventResponse {
    #[default]
    None,

    /// quit the application
    Quit,

    /// destroy the current element
    Destruct,

    /// bring this element to the front (greatest z-index)
    BringToFront,

    /// defocus all other elements
    UnfocusOthers,

    /// focus this element
    Focus,

    /// create an element, its location will be adjusted
    /// by the elements current location.
    ///
    /// this response can be used to create a window
    /// or when used in conjunction with destruct, it can be used to replace
    /// the current element with another.
    /// Additionally EventResponses can be passed with the new element,
    /// these EventResponses are then considered to have come from the new element
    NewElement(Box<(dyn Element)>, Option<EventResponses>),

    Move(MoveResponse),

    Resize(ResizeResponse),

    /// arbitrary custom value which can be passed back to the parent
    ///       key,   , value
    Custom(String, Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct MoveResponse {
    /// the element sending the move response
    pub el_id: ElementID,

    pub dx: i32,
    pub dy: i32,
}

#[derive(Clone, Debug)]
pub struct ResizeResponse {
    /// the element sending the resize response
    pub el_id: ElementID,

    pub left_dx: i32,
    pub right_dx: i32,
    pub top_dy: i32,
    pub bottom_dy: i32,
}

impl std::fmt::Debug for EventResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventResponse::None => write!(f, "EventResponse::None"),
            EventResponse::Quit => write!(f, "EventResponse::Quit"),
            EventResponse::Destruct => write!(f, "EventResponse::Destruct"),
            EventResponse::BringToFront => write!(f, "EventResponse::BringToFront"),
            EventResponse::UnfocusOthers => write!(f, "EventResponse::UnfocusOthers"),
            EventResponse::Focus => write!(f, "EventResponse::Focus"),
            EventResponse::NewElement(el, resp) => write!(
                f,
                "EventResponse::NewElement id: {:?}, resp: {:?}",
                el.id(),
                resp
            ),
            EventResponse::Move(m) => write!(f, "EventResponse::Move({:?})", m),
            EventResponse::Resize(r) => write!(f, "EventResponse::Resize({:?})", r),
            EventResponse::Custom(k, v) => write!(f, "EventResponse::Custom({}, {:?})", k, v),
        }
    }
}

impl EventResponse {
    pub fn has_metadata(&self, key: &str) -> bool {
        matches!(self, EventResponse::Custom(k, _) if k == key)
    }
}

#[derive(Default, Debug)]
pub struct EventResponses(pub Vec<EventResponse>);

impl From<EventResponse> for EventResponses {
    fn from(er: EventResponse) -> EventResponses {
        EventResponses(vec![er])
    }
}

impl From<Vec<EventResponse>> for EventResponses {
    fn from(v: Vec<EventResponse>) -> EventResponses {
        EventResponses(v)
    }
}

impl From<()> for EventResponses {
    fn from(_: ()) -> EventResponses {
        EventResponses::default()
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
    /// retrieves only the ReceivableEventChanges from the EventResponses
    /// and concats them together
    pub fn extend(&mut self, other: EventResponses) {
        self.0.extend(other.0)
    }
}

// -------------------------------------------------------------------------------------

// ---------------------------------------------------------------------------
/// The ReceivableEvents are used to manage events and associated functions
/// registered directly to an element (AND NOT to that elements children!). They
/// are similar to the EvPrioritizer, but they are used to manage the events and
/// commands that are registered locally to this specific element.
/// (The EvPrioritizer and CmdPrioritizer are used to manage the events and
/// commands that are registered to an element's
/// children in the ElementOrganizer).
/// NOTE: these fulfill a similar function to the prioritizers
/// in that they manage inclusion/removal more cleanly and can be sorted
#[derive(Clone, Default, Debug)]
pub struct ReceivableEvents(pub Vec<ReceivableEvent>);

impl Deref for ReceivableEvents {
    type Target = Vec<ReceivableEvent>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReceivableEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<ReceivableEvent>> for ReceivableEvents {
    fn from(v: Vec<ReceivableEvent>) -> ReceivableEvents {
        ReceivableEvents(v)
    }
}

impl ReceivableEvents {
    pub fn push(&mut self, ev: ReceivableEvent) {
        self.0.push(ev)
    }

    pub fn extend(&mut self, evs: Vec<ReceivableEvent>) {
        self.0.extend(evs)
    }

    pub fn remove(&mut self, ev: ReceivableEvent) {
        self.0.retain(|e| e != &ev)
    }

    pub fn remove_many(&mut self, evs: Vec<ReceivableEvent>) {
        self.0.retain(|e| !evs.contains(e))
    }

    pub fn contains_match(&self, ev: &Event) -> bool {
        for ev_ in &self.0 {
            if ev_.matches(ev) {
                return true;
            }
        }
        false
    }
}
