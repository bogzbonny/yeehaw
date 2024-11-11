use {
    crate::{prioritizer::Priority, Element},
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

    Mouse(crossterm::event::MouseEvent),

    /// The ExternalMouseEvent is send to all elements
    /// who are not considered to be the "Receiver" of the event
    ///
    /// This is used to receive from a parent, a mouse event that neither it, nor its
    /// children, are meant to consume. This is used to tell an element that
    /// another element, somewhere in the tui, has received/consumed a mouse event.
    ///
    /// NOTE the column and row are the column and row of the mouse event relative
    /// to the element receiving the event, hence they may be negative.
    ExternalMouse(RelMouseEvent),

    /// custom event type with a name and a payload
    Custom(String, Vec<u8>),
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

    Focus,
    /// focus this element

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

    /// XXX TODO rename to Custom
    /// arbitrary custom metadatas which can be passed back to the parent
    ///       key,   , value
    Metadata(String, Vec<u8>),

    /// contains priority updates that should be made to the receiver's prioritizer
    ReceivableEventChanges(ReceivableEventChanges),
}

#[derive(Clone, Copy, Debug)]
pub struct MoveResponse {
    pub dx: i32,
    pub dy: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct ResizeResponse {
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
    pub fn get_receivable_event_changes(&self) -> ReceivableEventChanges {
        let mut rec = ReceivableEventChanges::default();
        for er in &self.0 {
            if let EventResponse::ReceivableEventChanges(r) = er {
                rec.extend(r.clone());
            }
        }
        rec
    }

    pub fn extend(&mut self, other: EventResponses) {
        self.0.extend(other.0)
    }
}

// ----------------------------------------------------------------------------

/// ReceivableEventChanges is used to update the receivable events of an element
/// registered in the prioritizers of all ancestors
/// NOTE: While processing inputability changes, element organizers remove events
/// BEFORE adding events.

#[derive(Clone, Default, Debug)]
pub struct ReceivableEventChanges {
    pub add: Vec<(ReceivableEvent, Priority)>,
    /// receivable events being added to the element

    /// Receivable events to deregistered from an element.
    /// NOTE: one instance of an event being passed up the hierarchy through
    /// RmRecEvs will remove ALL instances of that event from the prioritizer of
    /// every element higher in the hierarchy that processes the
    /// ReceivableEventChanges.
    pub remove: Vec<ReceivableEvent>,
}

impl ReceivableEventChanges {
    pub fn new(
        add: Vec<(ReceivableEvent, Priority)>, remove: Vec<ReceivableEvent>,
    ) -> ReceivableEventChanges {
        ReceivableEventChanges { add, remove }
    }

    pub fn with_add_ev(mut self, p: Priority, ev: ReceivableEvent) -> ReceivableEventChanges {
        self.add.push((ev, p));
        self
    }

    pub fn with_add_evs(mut self, evs: Vec<(ReceivableEvent, Priority)>) -> ReceivableEventChanges {
        self.add.extend(evs);
        self
    }

    pub fn with_remove_ev(mut self, ev: ReceivableEvent) -> ReceivableEventChanges {
        self.remove.push(ev);
        self
    }

    pub fn with_remove_evs(mut self, evs: Vec<ReceivableEvent>) -> ReceivableEventChanges {
        self.remove.extend(evs);
        self
    }

    pub fn push_add_ev(&mut self, ev: ReceivableEvent, p: Priority) {
        self.add.push((ev, p));
    }

    pub fn push_add_evs(&mut self, evs: Vec<(ReceivableEvent, Priority)>) {
        self.add.extend(evs);
    }

    pub fn push_add_evs_single_priority(&mut self, evs: Vec<ReceivableEvent>, pr: Priority) {
        for ev in evs {
            self.add.push((ev, pr));
        }
    }

    pub fn push_remove_ev(&mut self, ev: ReceivableEvent) {
        self.remove.push(ev);
    }

    pub fn push_remove_evs(&mut self, evs: Vec<ReceivableEvent>) {
        self.remove.extend(evs);
    }

    pub fn update_priority_for_ev(&mut self, ev: ReceivableEvent, p: Priority) {
        self.remove.push(ev.clone());
        self.add.push((ev, p));
    }

    pub fn update_priority_for_evs(&mut self, evs: Vec<ReceivableEvent>, p: Priority) {
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

// -------------------------------------------------------------------------------------

// ---------------------------------------------------------------------------
/// The SelfReceivableEvents are used to manage events and associated functions
/// registered directly to an element (AND NOT to that elements children!). They
/// are similar to the EvPrioritizer, but they are used to manage the events and
/// commands that are registered locally to this specific element.
/// (The EvPrioritizer and CmdPrioritizer are used to manage the events and
/// commands that are registered to an element's
/// children in the ElementOrganizer).
/// NOTE: these fulfill a similar function to the prioritizers
/// in that they manage inclusion/removal more cleanly and can be sorted
#[derive(Clone, Default)]
pub struct SelfReceivableEvents(pub Vec<(ReceivableEvent, Priority)>);

impl Deref for SelfReceivableEvents {
    type Target = Vec<(ReceivableEvent, Priority)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SelfReceivableEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<(ReceivableEvent, Priority)>> for SelfReceivableEvents {
    fn from(v: Vec<(ReceivableEvent, Priority)>) -> SelfReceivableEvents {
        SelfReceivableEvents(v)
    }
}

impl SelfReceivableEvents {
    pub fn new_from_receivable_events(
        p: Priority, evs: Vec<ReceivableEvent>,
    ) -> SelfReceivableEvents {
        SelfReceivableEvents(evs.into_iter().map(|ev| (ev, p)).collect())
    }

    pub fn push(&mut self, ev: ReceivableEvent, p: Priority) {
        self.0.push((ev, p))
    }

    pub fn push_many_at_priority(&mut self, evs: Vec<ReceivableEvent>, p: Priority) {
        for ev in evs {
            self.push(ev, p)
        }
    }

    pub fn extend(&mut self, evs: Vec<(ReceivableEvent, Priority)>) {
        self.0.extend(evs)
    }

    pub fn remove(&mut self, ev: ReceivableEvent) {
        self.0.retain(|(e, _)| e != &ev)
    }

    pub fn remove_many(&mut self, evs: Vec<ReceivableEvent>) {
        self.0.retain(|(e, _)| !evs.contains(e))
    }

    /// update_priority_for_ev updates the priority of the given event
    /// registered directly to this element
    pub fn update_priority_for_ev(&mut self, ev: ReceivableEvent, p: Priority) {
        for i in 0..self.0.len() {
            if self.0[i].0 != ev {
                continue;
            }
            self.0[i].1 = p;
            break;
        }
    }

    pub fn update_priority_for_evs(&mut self, evs: Vec<ReceivableEvent>, p: Priority) {
        for ev in evs {
            self.update_priority_for_ev(ev, p)
        }
    }

    pub fn update_priority_for_all(&mut self, p: Priority) {
        for i in self.0.iter_mut() {
            i.1 = p;
        }
    }

    pub fn to_receivable_event_changes(&self) -> ReceivableEventChanges {
        let remove_evs = self.0.iter().map(|(ev, _)| ev.clone()).collect();
        ReceivableEventChanges::default()
            .with_add_evs(self.0.clone())
            .with_remove_evs(remove_evs)
    }
}
