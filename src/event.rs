use {
    crate::{prioritizer::Priority, Element},
    std::ops::{Deref, DerefMut},
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Event {
    KeyCombo(Vec<KeyPossibility>),

    Mouse(crossterm::event::MouseEvent),

    // The ExternalMouseEvent is send to all elements
    // who are not considered to be the "Receiver" of the event
    //
    // This is used to receive from a parent, a mouse event that neither it, nor its
    // children, are meant to consume. This is used to tell an element that
    // another element, somewhere in the cui, has received/consumed a mouse event.
    //
    /// NOTE the column and row are the column and row of the mouse event relative
    /// to the element receiving the event, hence they may be negative.
    ExternalMouse(RelMouseEvent),

    // Used to tell an element that the screen has resized. The element should
    // then adjust all of its children based on the given context
    Resize,

    // A signal to an element that it should closedown.
    Exit,

    // Refresh resets an element's organizer's prioritizers as well as triggering a
    // Resize event in all children. This essentially refreshes the state of the
    // element organizer.
    // Currently only relevant for elements that have an element organizer.
    Refresh,

    Custom(String, Vec<u8>), // custom event type with a name and a payload
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct RelMouseEvent {
    /// The kind of mouse event that was caused.
    pub kind: crossterm::event::MouseEventKind,
    /// The relative column that the event occurred on.
    pub column: i32,
    /// The relative row that the event occurred on.
    pub row: i32,
    /// The key modifiers active when the event occurred.
    pub modifiers: crossterm::event::KeyModifiers,
}

impl From<crossterm::event::MouseEvent> for RelMouseEvent {
    fn from(me: crossterm::event::MouseEvent) -> Self {
        RelMouseEvent {
            kind: me.kind,
            column: me.column as i32,
            row: me.row as i32,
            modifiers: me.modifiers,
        }
    }
}

impl From<RelMouseEvent> for crossterm::event::MouseEvent {
    fn from(me: RelMouseEvent) -> Self {
        let column = if me.column < 0 { 0 } else { me.column as u16 };
        let row = if me.row < 0 { 0 } else { me.row as u16 };
        crossterm::event::MouseEvent {
            kind: me.kind,
            column,
            row,
            modifiers: me.modifiers,
        }
    }
}

impl From<KeyPossibility> for Event {
    fn from(key: KeyPossibility) -> Self {
        Event::KeyCombo(vec![key])
    }
}

impl From<crossterm::event::KeyEvent> for Event {
    fn from(key: crossterm::event::KeyEvent) -> Self {
        Event::KeyCombo(vec![KeyPossibility::Key(key)])
    }
}

impl From<Vec<crossterm::event::KeyEvent>> for Event {
    fn from(keys: Vec<crossterm::event::KeyEvent>) -> Self {
        Event::KeyCombo(keys.into_iter().map(KeyPossibility::Key).collect())
    }
}

impl Event {
    // translation note used to be called 'key', TRANSLATION
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
            Event::Refresh => "REFRESH".to_string(),
            Event::Exit => "EXIT".to_string(),
            Event::Custom(name, _) => "CUSTOM=".to_string() + name,
        }
    }

    pub fn matches(&self, other: &Event) -> bool {
        match (self, other) {
            (Event::Mouse(me1), Event::Mouse(me2)) => me1 == me2,
            (Event::KeyCombo(k1), Event::KeyCombo(k2)) => {
                if k1.len() != k2.len() {
                    return false;
                }
                for (i, k) in k1.iter().enumerate() {
                    if !k.matches(&k2[i]) {
                        return false;
                    }
                }
                true
            }
            (Event::ExternalMouse(eme1), Event::ExternalMouse(eme2)) => eme1 == eme2,
            (Event::Resize, Event::Resize) => true,
            (Event::Refresh, Event::Refresh) => true,
            (Event::Custom(kind1, bz1), Event::Custom(kind2, bz2)) => kind1 == kind2 && bz1 == bz2,
            _ => false,
        }
    }
}

// Event for triggering a command execution for an element
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CommandEvent {
    pub cmd: String,
    pub args: Vec<String>,
}

// KeyPossibility is used to match a key event
// with a specific key or a group of keys
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum KeyPossibility {
    Key(crossterm::event::KeyEvent),
    Chars,  // any char
    Digits, // any digit
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

    // defocus all other elements
    UnfocusOthers,

    Focus, // focus this element

    // create an element, its location will be adjusted
    // by the elements current location.
    //
    // this response can be used to create a window
    // or when used in conjunction with destruct, it can be used to replace
    // the current element with another.
    // Additionally EventResponses can be passed with the new element,
    // these EventResponses are then considered to have come from the new element
    NewElement(Box<(dyn Element)>, Option<EventResponses>),

    // arbitrary custom metadatas which can be passed back to the parent
    //       key,   , value
    Metadata(String, Vec<u8>),

    // contains priority updates that should be made to the receiver's prioritizer
    ReceivableEventChanges(ReceivableEventChanges),
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
            EventResponse::Metadata(k, v) => write!(f, "EventResponse::Metadata({}, {:?})", k, v),
            EventResponse::ReceivableEventChanges(rec) => {
                write!(f, "EventResponse::ReceivableEventChanges({:?})", rec)
            }
        }
    }
}

impl EventResponse {
    pub fn has_metadata(&self, key: &str) -> bool {
        matches!(self, EventResponse::Metadata(k, _) if k == key)
    }
}

// ----------------------------------------------------------------------------

// ReceivableEventChanges is used to update the receivable events of an element
// registered in the prioritizers of all ancestors
// NOTE: While processing inputability changes, element organizers remove events
// BEFORE adding events.

#[derive(Clone, Default, Debug)]
// TRANSLATION NOTE used to be InputabilityChanges
pub struct ReceivableEventChanges {
    pub add: Vec<(Event, Priority)>, // receivable events being added to the element

    // Receivable events to deregistered from an element.
    // NOTE: one instance of an event being passed up the hierarchy through
    // RmRecEvs will remove ALL instances of that event from the prioritizer of
    // every element higher in the hierarchy that processes the
    // ReceivableEventChanges.
    pub remove: Vec<Event>,
}

impl ReceivableEventChanges {
    pub fn new(add: Vec<(Event, Priority)>, remove: Vec<Event>) -> ReceivableEventChanges {
        ReceivableEventChanges { add, remove }
    }

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

    pub fn push_add_ev(&mut self, ev: Event, p: Priority) {
        self.add.push((ev, p));
    }

    pub fn push_add_evs(&mut self, evs: Vec<(Event, Priority)>) {
        self.add.extend(evs);
    }

    pub fn push_add_evs_single_priority(&mut self, evs: Vec<Event>, pr: Priority) {
        for ev in evs {
            self.add.push((ev, pr));
        }
    }

    pub fn push_remove_ev(&mut self, ev: Event) {
        self.remove.push(ev);
    }

    pub fn push_remove_evs(&mut self, evs: Vec<Event>) {
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

    pub fn extend(&mut self, rec: ReceivableEventChanges) {
        self.remove.extend(rec.remove);
        self.add.extend(rec.add);
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
    // retrieves only the ReceivableEventChanges from the EventResponses
    // and concats them together
    pub fn get_receivable_event_changes(&self) -> ReceivableEventChanges {
        let mut rec = ReceivableEventChanges::default();
        for er in &self.0 {
            if let EventResponse::ReceivableEventChanges(r) = er {
                rec.extend(r.clone());
            }
        }
        rec
    }
}
