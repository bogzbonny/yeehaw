use {
    super::{SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponse, Keyboard as KB, Priority,
        ReceivableEventChanges, RgbColour, SortingHat, Style, UpwardPropagator,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// TODO multiline text support for each radio
// TODO option to start with nothing selected

#[derive(Clone)]
pub struct RadioButtons {
    pub base: WidgetBase,
    pub on_ch: Rc<RefCell<char>>,  // ch used for the selected
    pub off_ch: Rc<RefCell<char>>, // ch used for the unselected

    pub radios: Rc<RefCell<Vec<String>>>, // the text for each radio button

    pub selected: Rc<RefCell<usize>>, // which radio button is selected

    // function which executes when the radio selection is changed
    //                                           (index, selected)
    pub radio_selected_fn: Rc<RefCell<dyn FnMut(usize, String) -> EventResponse>>,
}

// inspiration:
// ◯ ◉ ◯ ○
// ◯ ◯   ●
// ⍟ ◉ ◯ ○

impl RadioButtons {
    const KIND: &'static str = "widget_radio";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new().with_fg(RgbColour::LIGHT_YELLOW2),
        ready_style: Style::new().with_fg(RgbColour::WHITE),
        unselectable_style: Style::new().with_fg(RgbColour::GREY13),
    };

    pub fn default_receivable_events() -> Vec<Event> {
        vec![
            KB::KEY_UP.into(),
            KB::KEY_DOWN.into(),
            KB::KEY_J.into(),
            KB::KEY_K.into(),
        ]
    }

    pub fn new(hat: &SortingHat, ctx: &Context, radios: Vec<String>) -> Self {
        let max_width = radios.iter().map(|r| r.len()).max().unwrap_or(0) + 1; // +1 for the radio button
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            ctx.clone(),
            SclVal::new_fixed(max_width),
            SclVal::new_fixed(radios.len()), // TODO change for multiline support
            Self::STYLE,
            Self::default_receivable_events(),
        );
        RadioButtons {
            base: wb,
            on_ch: Rc::new(RefCell::new('⍟')),
            off_ch: Rc::new(RefCell::new('◯')),
            radios: Rc::new(RefCell::new(radios)),
            selected: Rc::new(RefCell::new(0)),
            radio_selected_fn: Rc::new(RefCell::new(|_, _| EventResponse::default())),
        }
    }

    // ----------------------------------------------
    // decorators

    pub fn with_radio_selected_fn(
        mut self, clicked_fn: Box<dyn FnMut(usize, String) -> EventResponse>,
    ) -> Self {
        self.radio_selected_fn = Rc::new(RefCell::new(clicked_fn));
        self
    }

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }
}

impl Widget for RadioButtons {}

impl Element for RadioButtons {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponse) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponse::default());
                }

                match true {
                    _ if ke[0].matches(&KB::KEY_DOWN) || ke[0].matches(&KB::KEY_J) => {
                        if *self.selected.borrow() < self.radios.borrow().len() - 1 {
                            *self.selected.borrow_mut() += 1;
                            let sel_i = *self.selected.borrow();
                            let sel_str = self.radios.borrow()[sel_i].clone();
                            let resp = self.radio_selected_fn.borrow_mut()(sel_i, sel_str);
                            return (true, resp);
                        }
                    }
                    _ if ke[0].matches(&KB::KEY_UP) || ke[0].matches(&KB::KEY_K) => {
                        if *self.selected.borrow() > 0 {
                            *self.selected.borrow_mut() -= 1;
                            let sel_i = *self.selected.borrow();
                            let sel_str = self.radios.borrow()[sel_i].clone();
                            let resp = self.radio_selected_fn.borrow_mut()(sel_i, sel_str);
                            return (true, resp);
                        }
                    }
                    _ => {}
                }
                return (false, EventResponse::default());
            }
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    let y = me.row as usize;
                    if y < self.radios.borrow().len() {
                        *self.selected.borrow_mut() = y;
                        let resp =
                            self.radio_selected_fn.borrow_mut()(y, self.radios.borrow()[y].clone());
                        return (true, resp);
                    }
                }
                return (false, EventResponse::default());
            }
            _ => {}
        }
        (false, EventResponse::default())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        // need to re set the content in order to reflect active style

        let selected_i = *self.selected.borrow();
        let s =
            self.radios
                .borrow()
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, radio)| {
                    if i == selected_i {
                        acc.push(*self.on_ch.borrow());
                    } else {
                        acc.push(*self.off_ch.borrow());
                    }
                    acc.push_str(radio);
                    if i != self.radios.borrow().len() - 1 {
                        acc.push('\n');
                    }
                    acc
                });
        self.base.set_content_from_string(&s);
        self.base.drawing(ctx)
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.base.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.base.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Rc<RefCell<dyn UpwardPropagator>>) {
        self.base.set_upward_propagator(up)
    }
}
