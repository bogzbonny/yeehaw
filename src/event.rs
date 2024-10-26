#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Event {
    Mouse(crossterm::event::MouseEvent),

    KeyCombo(Vec<KeyPossibility>),

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

    Custom(String, Vec<u8>), // custom event type with a name and a payload
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
            Event::Command(ev) => "COMMAND=".to_string() + &ev.cmd,
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
            (Event::Command(ev1), Event::Command(ev2)) => ev1 == ev2,
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
        }
    }
}
