use {
    crate::KeyPossibility,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    std::time::{Duration, Instant},
};

// Keyboard is a FILO stack of key events.
#[derive(Clone)]
pub struct Keyboard {
    // when an event is consumed it is set to None
    // the Instant is the time when the event was registered
    evs: Vec<(Option<KeyEvent>, Instant)>,
}

const MAX_RUNES: usize = 20;

impl Default for Keyboard {
    fn default() -> Self {
        // create a KeyEvent of with a space and no modifiers
        // and push it to the stack
        let mut k = Keyboard {
            evs: Vec::with_capacity(MAX_RUNES),
        };
        for _ in 0..MAX_RUNES - 1 {
            k.evs.push((None, Instant::now()));
        }
        k
    }
}

impl Keyboard {
    pub fn new_with_ev(ev: KeyEvent) -> Keyboard {
        let mut k = Keyboard::default();
        k.add_ev(ev);
        k
    }

    // TRANSLATION NOTE used to be called add_ev_to_keyboard
    // must be called before all functions
    pub fn add_ev(&mut self, ev: KeyEvent) {
        self.evs.push((Some(ev), Instant::now()));
        if self.evs.len() > MAX_RUNES {
            self.evs.remove(0);
        }
    }

    // returns true if the last_key_time is withing 500 ms of the current time
    pub fn just_hit(&self) -> bool {
        // get the final two events
        if self.evs.len() < 2 {
            return false;
        }
        let last_ev = self.evs[self.evs.len() - 1];
        let prev_ev = self.evs[self.evs.len() - 2];

        let diff = last_ev.1.duration_since(prev_ev.1);
        diff.as_millis() < 250
    }

    pub fn get_current_key(&self) -> Option<KeyEvent> {
        self.evs.last().map(|x| x.0).unwrap_or(None)
    }

    // consume the current key and return whatever was consumed
    pub fn consume_current_key(&mut self) -> Option<KeyEvent> {
        self.evs.last_mut().map(|x| x.0.take()).unwrap_or(None)
    }

    // returns the last event character in the stack, previous and then the current
    //                                  (previous        , current         )
    pub fn get_prev_curr_keys(&self) -> (Option<KeyEvent>, Option<KeyEvent>) {
        (
            self.evs[self.evs.len() - 2].0,
            self.evs[self.evs.len() - 1].0,
        )
    }

    // consume the current key and return whatever was consumed
    pub fn consume_prev_curr_keys(&mut self) -> (Option<KeyEvent>, Option<KeyEvent>) {
        let curr = self.evs.last_mut().map(|x| x.0.take()).unwrap_or(None);

        let prev = {
            let prev_i = self.evs.len().checked_sub(2);
            match prev_i {
                Some(i) => self.evs.get_mut(i).map(|x| x.0.take()).unwrap_or(None),
                None => None,
            }
        };
        (prev, curr)
    }

    // is the current key one within the list provided
    // NOTE crossterm normalizes modifiers and casing
    pub fn is_key_one_of(k: KeyEvent, keys: Vec<KeyEvent>) -> bool {
        for key in keys {
            if key == k {
                return true;
            }
        }
        false
    }

    pub fn get_char(k: KeyEvent) -> Option<char> {
        match k.code {
            KeyCode::Char(c) => Some(c),
            _ => None,
        }
    }

    //---------------------------------------

    // events older than the eventLifetime are not considered
    pub const EVENT_LIFETIME: Duration = Duration::from_secs(10);

    // returns Some when the event combo matches the keyboard state
    //
    // NOTE we return the key events that were matched as the event combo
    // which is fed in may have wildcard KeyPossibilities such as Digits
    // which are matched by the actual key event.
    //
    // TODO see ISSUE 2206-1001
    pub fn matches(
        &mut self, ec: &[KeyPossibility], consume_events: bool,
    ) -> Option<Vec<KeyEvent>> {
        if ec.len() > self.evs.len() {
            return None;
        }

        let now = Instant::now();

        let mut out_ev = vec![];
        let mut j = self.evs.len() - 1; // index of last element in queue
        let j_end = j;
        for i in (0..=ec.len() - 1).rev() {
            let Some(ev) = self.evs[j].0 else {
                // if already consumed, no match
                return None;
            };

            // if EventKey has timed out, no match
            if now.duration_since(self.evs[j].1) > Self::EVENT_LIFETIME {
                return None;
            }

            if !ec[i].matches(&ev) {
                return None;
            }

            out_ev.insert(0, ev);
            j -= 1;
        }
        let j_start = j + 1; // compensate for the final --

        // mark all the matched events as consumed
        if consume_events {
            // ConsumeEvents marks all events in the most recent EvKeyCombo
            // match (as determined by kb.Matches) as consumed
            // NOTE: if Matches is called with consumeEvents set to false, this must be
            // called externally to mark the events as consumed.
            //   - This currently happens in EvPrioritizer.GetDestinationEl() to facilitate
            //     the Organizer's RelaxPriority field.
            for l in j_start..=j_end {
                self.evs[l].0 = None;
            }
        }

        Some(out_ev)
    }

    //---------------------------------------

    // The last number ignoring the previous [ignoring_previous] number of prevRunes
    //pub fn last_number(&self, ignoring_previous: usize) -> Option<i64> {
    // NOTE translation ignoreing_previous must be one greater than whatever
    // it was set too in the original code
    pub fn last_number_ignoring(&self, ignoring_previous: usize) -> Option<u64> {
        let mut s = "".to_string();
        for i in (0..=(self.evs.len() - 1 - ignoring_previous)).rev() {
            // get the char at i
            let ev = self.evs[i];
            let c = match ev.0 {
                Some(ev) => ev.code,
                _ => break,
            };
            if let KeyCode::Char(c) = c {
                if !c.is_ascii_digit() {
                    break;
                }
                s = c.to_string() + s.as_str();
            }
        }
        if !s.is_empty() {
            // convert s to a number
            Some(s.parse::<u64>().unwrap()) // ignore errors this can't fail
        } else {
            None
        }
    }

    //--------------------------------------------

    pub const KEY_ESC: KeyEvent = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    pub const KEY_ENTER: KeyEvent = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    pub const KEY_SHIFT_ENTER: KeyEvent = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
    pub const KEY_ALT_ENTER: KeyEvent = KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT);
    pub const KEY_BACKSPACE: KeyEvent = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
    pub const KEY_DELETE: KeyEvent = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    pub const KEY_COLON: KeyEvent = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);
    pub const KEY_SLASH: KeyEvent = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
    pub const KEY_BACKSLASH: KeyEvent = KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::NONE);
    pub const KEY_TAB: KeyEvent = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    pub const KEY_BACKTAB: KeyEvent = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
    pub const KEY_SPACE: KeyEvent = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);

    pub const KEY_PLUS: KeyEvent = KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE);
    pub const KEY_UNDERSCORE: KeyEvent = KeyEvent::new(KeyCode::Char('_'), KeyModifiers::NONE);
    pub const KEY_MINUS: KeyEvent = KeyEvent::new(KeyCode::Char('-'), KeyModifiers::NONE);
    pub const KEY_EQUALS: KeyEvent = KeyEvent::new(KeyCode::Char('='), KeyModifiers::NONE);
    pub const KEY_ALT_MINUS: KeyEvent = KeyEvent::new(KeyCode::Char('-'), KeyModifiers::ALT);
    pub const KEY_ALT_EQUALS: KeyEvent = KeyEvent::new(KeyCode::Char('='), KeyModifiers::ALT);
    pub const KEY_OPT_MINUS: KeyEvent = KeyEvent::new(KeyCode::Char('–'), KeyModifiers::NONE);
    pub const KEY_OPT_EQUALS: KeyEvent = KeyEvent::new(KeyCode::Char('≠'), KeyModifiers::NONE);

    pub const KEY_LEFT: KeyEvent = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
    pub const KEY_RIGHT: KeyEvent = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
    pub const KEY_UP: KeyEvent = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    pub const KEY_DOWN: KeyEvent = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);

    pub const KEY_CTRL_LEFT: KeyEvent = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);
    pub const KEY_CTRL_RIGHT: KeyEvent = KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL);
    pub const KEY_CTRL_UP: KeyEvent = KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL);
    pub const KEY_CTRL_DOWN: KeyEvent = KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL);

    pub const KEY_SHIFT_LEFT: KeyEvent = KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT);
    pub const KEY_SHIFT_RIGHT: KeyEvent = KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT);
    pub const KEY_SHIFT_UP: KeyEvent = KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT);
    pub const KEY_SHIFT_DOWN: KeyEvent = KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT);

    pub const KEY_ALT_LEFT: KeyEvent = KeyEvent::new(KeyCode::Left, KeyModifiers::ALT);
    pub const KEY_ALT_RIGHT: KeyEvent = KeyEvent::new(KeyCode::Right, KeyModifiers::ALT);
    pub const KEY_ALT_UP: KeyEvent = KeyEvent::new(KeyCode::Up, KeyModifiers::ALT);
    pub const KEY_ALT_DOWN: KeyEvent = KeyEvent::new(KeyCode::Down, KeyModifiers::ALT);

    pub const KEY_0: KeyEvent = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::NONE);
    pub const KEY_1: KeyEvent = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
    pub const KEY_2: KeyEvent = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
    pub const KEY_3: KeyEvent = KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE);
    pub const KEY_4: KeyEvent = KeyEvent::new(KeyCode::Char('4'), KeyModifiers::NONE);
    pub const KEY_5: KeyEvent = KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE);
    pub const KEY_6: KeyEvent = KeyEvent::new(KeyCode::Char('6'), KeyModifiers::NONE);
    pub const KEY_7: KeyEvent = KeyEvent::new(KeyCode::Char('7'), KeyModifiers::NONE);
    pub const KEY_8: KeyEvent = KeyEvent::new(KeyCode::Char('8'), KeyModifiers::NONE);
    pub const KEY_9: KeyEvent = KeyEvent::new(KeyCode::Char('9'), KeyModifiers::NONE);

    pub const KEY_SHIFT_0: KeyEvent = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::SHIFT);
    pub const KEY_SHIFT_1: KeyEvent = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::SHIFT);
    pub const KEY_SHIFT_2: KeyEvent = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::SHIFT);
    pub const KEY_SHIFT_3: KeyEvent = KeyEvent::new(KeyCode::Char('3'), KeyModifiers::SHIFT);
    pub const KEY_SHIFT_4: KeyEvent = KeyEvent::new(KeyCode::Char('4'), KeyModifiers::SHIFT);
    pub const KEY_SHIFT_5: KeyEvent = KeyEvent::new(KeyCode::Char('5'), KeyModifiers::SHIFT);
    pub const KEY_SHIFT_6: KeyEvent = KeyEvent::new(KeyCode::Char('6'), KeyModifiers::SHIFT);
    pub const KEY_SHIFT_7: KeyEvent = KeyEvent::new(KeyCode::Char('7'), KeyModifiers::SHIFT);
    pub const KEY_SHIFT_8: KeyEvent = KeyEvent::new(KeyCode::Char('8'), KeyModifiers::SHIFT);
    pub const KEY_SHIFT_9: KeyEvent = KeyEvent::new(KeyCode::Char('9'), KeyModifiers::SHIFT);

    pub const KEY_EXCLAMATION: KeyEvent = KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE);
    pub const KEY_AT_SIGN: KeyEvent = KeyEvent::new(KeyCode::Char('@'), KeyModifiers::NONE);
    pub const KEY_HASH: KeyEvent = KeyEvent::new(KeyCode::Char('#'), KeyModifiers::NONE);
    pub const KEY_DOLLAR: KeyEvent = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::NONE);
    pub const KEY_PERCENT: KeyEvent = KeyEvent::new(KeyCode::Char('%'), KeyModifiers::NONE);
    pub const KEY_CARET: KeyEvent = KeyEvent::new(KeyCode::Char('^'), KeyModifiers::NONE);
    pub const KEY_AND_SIGN: KeyEvent = KeyEvent::new(KeyCode::Char('&'), KeyModifiers::NONE);
    pub const KEY_ASTERIX: KeyEvent = KeyEvent::new(KeyCode::Char('*'), KeyModifiers::NONE);
    pub const KEY_OPEN_BRACKET: KeyEvent = KeyEvent::new(KeyCode::Char('('), KeyModifiers::NONE);
    pub const KEY_CLOSE_BRACKET: KeyEvent = KeyEvent::new(KeyCode::Char(')'), KeyModifiers::NONE);
    pub const KEY_OPEN_CURLY_BRACKET: KeyEvent =
        KeyEvent::new(KeyCode::Char('{'), KeyModifiers::NONE);
    pub const KEY_CLOSE_CURLY_BRACKET: KeyEvent =
        KeyEvent::new(KeyCode::Char('}'), KeyModifiers::NONE);
    pub const KEY_CLIP: KeyEvent = KeyEvent::new(KeyCode::Char('📎'), KeyModifiers::NONE);

    pub const KEY_SHIFT_A: KeyEvent = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);
    pub const KEY_SHIFT_B: KeyEvent = KeyEvent::new(KeyCode::Char('B'), KeyModifiers::NONE);
    pub const KEY_SHIFT_C: KeyEvent = KeyEvent::new(KeyCode::Char('C'), KeyModifiers::NONE);
    pub const KEY_SHIFT_D: KeyEvent = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE);
    pub const KEY_SHIFT_E: KeyEvent = KeyEvent::new(KeyCode::Char('E'), KeyModifiers::NONE);
    pub const KEY_SHIFT_F: KeyEvent = KeyEvent::new(KeyCode::Char('F'), KeyModifiers::NONE);
    pub const KEY_SHIFT_G: KeyEvent = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
    pub const KEY_SHIFT_H: KeyEvent = KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE);
    pub const KEY_SHIFT_I: KeyEvent = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);
    pub const KEY_SHIFT_J: KeyEvent = KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE);
    pub const KEY_SHIFT_K: KeyEvent = KeyEvent::new(KeyCode::Char('K'), KeyModifiers::NONE);
    pub const KEY_SHIFT_L: KeyEvent = KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE);
    pub const KEY_SHIFT_M: KeyEvent = KeyEvent::new(KeyCode::Char('M'), KeyModifiers::NONE);
    pub const KEY_SHIFT_N: KeyEvent = KeyEvent::new(KeyCode::Char('N'), KeyModifiers::NONE);
    pub const KEY_SHIFT_O: KeyEvent = KeyEvent::new(KeyCode::Char('O'), KeyModifiers::NONE);
    pub const KEY_SHIFT_P: KeyEvent = KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE);
    pub const KEY_SHIFT_Q: KeyEvent = KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::NONE);
    pub const KEY_SHIFT_R: KeyEvent = KeyEvent::new(KeyCode::Char('R'), KeyModifiers::NONE);
    pub const KEY_SHIFT_S: KeyEvent = KeyEvent::new(KeyCode::Char('S'), KeyModifiers::NONE);
    pub const KEY_SHIFT_T: KeyEvent = KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE);
    pub const KEY_SHIFT_U: KeyEvent = KeyEvent::new(KeyCode::Char('U'), KeyModifiers::NONE);
    pub const KEY_SHIFT_V: KeyEvent = KeyEvent::new(KeyCode::Char('V'), KeyModifiers::NONE);
    pub const KEY_SHIFT_W: KeyEvent = KeyEvent::new(KeyCode::Char('W'), KeyModifiers::NONE);
    pub const KEY_SHIFT_X: KeyEvent = KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE);
    pub const KEY_SHIFT_Y: KeyEvent = KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::NONE);
    pub const KEY_SHIFT_Z: KeyEvent = KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::NONE);

    pub const KEY_A: KeyEvent = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    pub const KEY_B: KeyEvent = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
    pub const KEY_C: KeyEvent = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
    pub const KEY_D: KeyEvent = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    pub const KEY_E: KeyEvent = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    pub const KEY_F: KeyEvent = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
    pub const KEY_G: KeyEvent = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    pub const KEY_H: KeyEvent = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
    pub const KEY_I: KeyEvent = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    pub const KEY_J: KeyEvent = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    pub const KEY_K: KeyEvent = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
    pub const KEY_L: KeyEvent = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
    pub const KEY_M: KeyEvent = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE);
    pub const KEY_N: KeyEvent = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    pub const KEY_O: KeyEvent = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    pub const KEY_P: KeyEvent = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
    pub const KEY_Q: KeyEvent = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    pub const KEY_R: KeyEvent = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    pub const KEY_S: KeyEvent = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    pub const KEY_T: KeyEvent = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    pub const KEY_U: KeyEvent = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    pub const KEY_V: KeyEvent = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE);
    pub const KEY_W: KeyEvent = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
    pub const KEY_X: KeyEvent = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    pub const KEY_Y: KeyEvent = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    pub const KEY_Z: KeyEvent = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE);

    pub const KEY_CTRL_A: KeyEvent = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_B: KeyEvent = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_C: KeyEvent = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_D: KeyEvent = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_E: KeyEvent = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_F: KeyEvent = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_G: KeyEvent = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_H: KeyEvent = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_I: KeyEvent = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_J: KeyEvent = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_K: KeyEvent = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_L: KeyEvent = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_M: KeyEvent = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_N: KeyEvent = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_O: KeyEvent = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_P: KeyEvent = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_Q: KeyEvent = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_R: KeyEvent = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_S: KeyEvent = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_T: KeyEvent = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_U: KeyEvent = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_V: KeyEvent = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_W: KeyEvent = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_X: KeyEvent = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_Y: KeyEvent = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
    pub const KEY_CTRL_Z: KeyEvent = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);

    pub const KEY_ALT_A: KeyEvent = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
    pub const KEY_ALT_B: KeyEvent = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT);
    pub const KEY_ALT_C: KeyEvent = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::ALT);
    pub const KEY_ALT_D: KeyEvent = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::ALT);
    pub const KEY_ALT_E: KeyEvent = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT);
    pub const KEY_ALT_F: KeyEvent = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::ALT);
    pub const KEY_ALT_G: KeyEvent = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT);
    pub const KEY_ALT_H: KeyEvent = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::ALT);
    pub const KEY_ALT_I: KeyEvent = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::ALT);
    pub const KEY_ALT_J: KeyEvent = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::ALT);
    pub const KEY_ALT_K: KeyEvent = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::ALT);
    pub const KEY_ALT_L: KeyEvent = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::ALT);
    pub const KEY_ALT_M: KeyEvent = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::ALT);
    pub const KEY_ALT_N: KeyEvent = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::ALT);
    pub const KEY_ALT_O: KeyEvent = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::ALT);
    pub const KEY_ALT_P: KeyEvent = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT);
    pub const KEY_ALT_Q: KeyEvent = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT);
    pub const KEY_ALT_R: KeyEvent = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::ALT);
    pub const KEY_ALT_S: KeyEvent = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT);
    pub const KEY_ALT_T: KeyEvent = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::ALT);
    pub const KEY_ALT_U: KeyEvent = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::ALT);
    pub const KEY_ALT_V: KeyEvent = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::ALT);
    pub const KEY_ALT_W: KeyEvent = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::ALT);
    pub const KEY_ALT_X: KeyEvent = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT);
    pub const KEY_ALT_Y: KeyEvent = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::ALT);
    pub const KEY_ALT_Z: KeyEvent = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::ALT);

    pub const KEY_META_A: KeyEvent = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::META);
    pub const KEY_META_B: KeyEvent = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::META);
    pub const KEY_META_C: KeyEvent = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::META);
    pub const KEY_META_D: KeyEvent = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::META);
    pub const KEY_META_E: KeyEvent = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::META);
    pub const KEY_META_F: KeyEvent = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::META);
    pub const KEY_META_G: KeyEvent = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::META);
    pub const KEY_META_H: KeyEvent = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::META);
    pub const KEY_META_I: KeyEvent = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::META);
    pub const KEY_META_J: KeyEvent = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::META);
    pub const KEY_META_K: KeyEvent = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::META);
    pub const KEY_META_L: KeyEvent = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::META);
    pub const KEY_META_M: KeyEvent = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::META);
    pub const KEY_META_N: KeyEvent = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::META);
    pub const KEY_META_O: KeyEvent = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::META);
    pub const KEY_META_P: KeyEvent = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::META);
    pub const KEY_META_Q: KeyEvent = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::META);
    pub const KEY_META_R: KeyEvent = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::META);
    pub const KEY_META_S: KeyEvent = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::META);
    pub const KEY_META_T: KeyEvent = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::META);
    pub const KEY_META_U: KeyEvent = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::META);
    pub const KEY_META_V: KeyEvent = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::META);
    pub const KEY_META_W: KeyEvent = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::META);
    pub const KEY_META_X: KeyEvent = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::META);
    pub const KEY_META_Y: KeyEvent = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::META);
    pub const KEY_META_Z: KeyEvent = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::META);

    // option keys, useful on mac
    pub const KEY_OPT_A: KeyEvent = KeyEvent::new(KeyCode::Char('å'), KeyModifiers::NONE);
    pub const KEY_OPT_B: KeyEvent = KeyEvent::new(KeyCode::Char('∫'), KeyModifiers::NONE);
    pub const KEY_OPT_C: KeyEvent = KeyEvent::new(KeyCode::Char('ç'), KeyModifiers::NONE);
    pub const KEY_OPT_D: KeyEvent = KeyEvent::new(KeyCode::Char('∂'), KeyModifiers::NONE);
    pub const KEY_OPT_E: KeyEvent = KeyEvent::new(KeyCode::Char('´'), KeyModifiers::NONE);
    pub const KEY_OPT_F: KeyEvent = KeyEvent::new(KeyCode::Char('ƒ'), KeyModifiers::NONE);
    pub const KEY_OPT_G: KeyEvent = KeyEvent::new(KeyCode::Char('©'), KeyModifiers::NONE);
    pub const KEY_OPT_H: KeyEvent = KeyEvent::new(KeyCode::Char('˙'), KeyModifiers::NONE);
    pub const KEY_OPT_I: KeyEvent = KeyEvent::new(KeyCode::Char('ˆ'), KeyModifiers::NONE);
    pub const KEY_OPT_J: KeyEvent = KeyEvent::new(KeyCode::Char('∆'), KeyModifiers::NONE);
    pub const KEY_OPT_K: KeyEvent = KeyEvent::new(KeyCode::Char('˚'), KeyModifiers::NONE);
    pub const KEY_OPT_L: KeyEvent = KeyEvent::new(KeyCode::Char('¬'), KeyModifiers::NONE);
    pub const KEY_OPT_M: KeyEvent = KeyEvent::new(KeyCode::Char('µ'), KeyModifiers::NONE);
    pub const KEY_OPT_N: KeyEvent = KeyEvent::new(KeyCode::Char('˜'), KeyModifiers::NONE);
    pub const KEY_OPT_O: KeyEvent = KeyEvent::new(KeyCode::Char('ø'), KeyModifiers::NONE);
    pub const KEY_OPT_P: KeyEvent = KeyEvent::new(KeyCode::Char('π'), KeyModifiers::NONE);
    pub const KEY_OPT_Q: KeyEvent = KeyEvent::new(KeyCode::Char('œ'), KeyModifiers::NONE);
    pub const KEY_OPT_R: KeyEvent = KeyEvent::new(KeyCode::Char('®'), KeyModifiers::NONE);
    pub const KEY_OPT_S: KeyEvent = KeyEvent::new(KeyCode::Char('ß'), KeyModifiers::NONE);
    pub const KEY_OPT_T: KeyEvent = KeyEvent::new(KeyCode::Char('†'), KeyModifiers::NONE);
    pub const KEY_OPT_U: KeyEvent = KeyEvent::new(KeyCode::Char('¨'), KeyModifiers::NONE);
    pub const KEY_OPT_V: KeyEvent = KeyEvent::new(KeyCode::Char('√'), KeyModifiers::NONE);
    pub const KEY_OPT_W: KeyEvent = KeyEvent::new(KeyCode::Char('∑'), KeyModifiers::NONE);
    pub const KEY_OPT_X: KeyEvent = KeyEvent::new(KeyCode::Char('≈'), KeyModifiers::NONE);
    pub const KEY_OPT_Y: KeyEvent = KeyEvent::new(KeyCode::Char('¥'), KeyModifiers::NONE);
    pub const KEY_OPT_Z: KeyEvent = KeyEvent::new(KeyCode::Char('Ω'), KeyModifiers::NONE);
}
