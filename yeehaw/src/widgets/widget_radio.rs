use {
    super::{Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Color, Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Keyboard as KB, Parent, Priority, ReceivableEvent, ReceivableEventChanges,
        SelfReceivableEvents, Style,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

/// TODO multiline text support for each radio
/// TODO option to start with nothing selected

#[derive(Clone)]
pub struct RadioButtons {
    pub base: WidgetBase,
    pub on_ch: Rc<RefCell<char>>,
    /// ch used for the selected
    pub off_ch: Rc<RefCell<char>>,
    /// ch used for the unselected
    pub radios: Rc<RefCell<Vec<String>>>,
    /// the text for each radio button
    pub clicked_down: Rc<RefCell<bool>>,
    /// activated when mouse is clicked down while over object
    pub selected: Rc<RefCell<usize>>,
    /// which radio button is selected

    /// function which executes when the radio selection is changed
    ///                                           (index, selected)
    #[allow(clippy::type_complexity)]
    pub radio_selected_fn: Rc<RefCell<dyn FnMut(Context, usize, String) -> EventResponses>>,
}

/// inspiration:
/// ◯ ◉ ◯ ○
/// ◯ ◯   ●
/// ⍟ ◉ ◯ ○

impl RadioButtons {
    const KIND: &'static str = "widget_radio";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new_const(Color::LIGHT_YELLOW2, Color::TRANSPARENT),
        ready_style: Style::new_const(Color::WHITE, Color::TRANSPARENT),
        unselectable_style: Style::new_const(Color::GREY13, Color::TRANSPARENT),
    };

    pub fn default_receivable_events() -> Vec<ReceivableEvent> {
        vec![
            KB::KEY_UP.into(),
            KB::KEY_DOWN.into(),
            KB::KEY_J.into(),
            KB::KEY_K.into(),
        ]
    }

    pub fn new(ctx: &Context, radios: Vec<String>) -> Self {
        let max_width = radios.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32 + 1; // +1 for the radio button
        let wb = WidgetBase::new(
            ctx,
            Self::KIND,
            DynVal::new_fixed(max_width),
            DynVal::new_fixed(radios.len() as i32),
            // TODO change for multiline support
            Self::STYLE,
            Self::default_receivable_events(),
        );
        RadioButtons {
            base: wb,
            on_ch: Rc::new(RefCell::new('⍟')),
            off_ch: Rc::new(RefCell::new('◯')),
            clicked_down: Rc::new(RefCell::new(false)),
            radios: Rc::new(RefCell::new(radios)),
            selected: Rc::new(RefCell::new(0)),
            radio_selected_fn: Rc::new(RefCell::new(|_, _, _| EventResponses::default())),
        }
    }

    // ----------------------------------------------
    // decorators

    pub fn with_radio_selected_fn(
        mut self, clicked_fn: Box<dyn FnMut(Context, usize, String) -> EventResponses>,
    ) -> Self {
        self.radio_selected_fn = Rc::new(RefCell::new(clicked_fn));
        self
    }

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }
}

impl Widget for RadioButtons {}

#[yeehaw_derive::impl_element_from(base)]
impl Element for RadioButtons {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }

                match true {
                    _ if ke[0] == KB::KEY_DOWN || ke[0] == KB::KEY_J => {
                        if *self.selected.borrow() < self.radios.borrow().len() - 1 {
                            *self.selected.borrow_mut() += 1;
                            let sel_i = *self.selected.borrow();
                            let sel_str = self.radios.borrow()[sel_i].clone();
                            let resp =
                                self.radio_selected_fn.borrow_mut()(ctx.clone(), sel_i, sel_str);
                            return (true, resp);
                        }
                    }
                    _ if ke[0] == KB::KEY_UP || ke[0] == KB::KEY_K => {
                        if *self.selected.borrow() > 0 {
                            *self.selected.borrow_mut() -= 1;
                            let sel_i = *self.selected.borrow();
                            let sel_str = self.radios.borrow()[sel_i].clone();
                            let resp =
                                self.radio_selected_fn.borrow_mut()(ctx.clone(), sel_i, sel_str);
                            return (true, resp);
                        }
                    }
                    _ => {}
                }
                return (false, EventResponses::default());
            }
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {}

                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, EventResponses::default());
                    }
                    MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        *self.clicked_down.borrow_mut() = false;
                        let y = me.row as usize;
                        if y < self.radios.borrow().len() {
                            *self.selected.borrow_mut() = y;
                            let resp = self.radio_selected_fn.borrow_mut()(
                                ctx.clone(),
                                y,
                                self.radios.borrow()[y].clone(),
                            );
                            return (true, resp);
                        }
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }

                return (false, EventResponses::default());
            }
            _ => {}
        }
        (false, EventResponses::default())
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
        self.base.set_content_from_string(ctx, &s);
        self.base.drawing(ctx)
    }
}
