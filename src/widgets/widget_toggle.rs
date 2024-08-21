use {
    super::{SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponses, Keyboard as KB, Priority,
        ReceivableEventChanges, RgbColour, SortingHat, Style, UpwardPropagator,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct Toggle {
    pub base: WidgetBase,
    pub left: Rc<RefCell<String>>,
    pub right: Rc<RefCell<String>>,
    pub left_selected: Rc<RefCell<bool>>, // otherwise right is selected
    pub selected_sty: Rc<RefCell<Style>>,
    //                                   selected
    pub toggled_fn: Rc<RefCell<dyn FnMut(Context, String) -> EventResponses>>,
}

impl Toggle {
    const KIND: &'static str = "widget_button";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new()
            .with_bg(RgbColour::LIGHT_YELLOW2)
            .with_fg(RgbColour::BLACK),
        ready_style: Style::new()
            .with_bg(RgbColour::WHITE)
            .with_fg(RgbColour::BLACK),
        unselectable_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::BLACK),
    };

    // for the selected toggle
    const DEFAULT_SELECTED_STY: Style = Style::new()
        .with_bg(RgbColour::LIGHT_BLUE)
        .with_fg(RgbColour::BLACK);

    pub fn default_receivable_events() -> Vec<Event> {
        vec![
            KB::KEY_ENTER.into(),
            KB::KEY_LEFT.into(),
            KB::KEY_RIGHT.into(),
            KB::KEY_H.into(),
            KB::KEY_L.into(),
        ]
    }

    pub fn new(
        hat: &SortingHat, ctx: &Context, left: String, right: String,
        toggeld_fn: Box<dyn FnMut(Context, String) -> EventResponses>,
    ) -> Self {
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            SclVal::new_fixed(left.chars().count() + right.chars().count()),
            SclVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        wb.set_content_from_string(ctx, &(left.clone() + &right));
        Toggle {
            base: wb,
            left: Rc::new(RefCell::new(left)),
            right: Rc::new(RefCell::new(right)),
            left_selected: Rc::new(RefCell::new(true)),
            selected_sty: Rc::new(RefCell::new(Self::DEFAULT_SELECTED_STY)),
            toggled_fn: Rc::new(RefCell::new(toggeld_fn)),
        }
    }

    // ----------------------------------------------
    // decorators

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

    // ----------------------------------------------

    pub fn selected(&self) -> String {
        if *self.left_selected.borrow() {
            return self.left.borrow().clone();
        }
        self.right.borrow().clone()
    }

    pub fn perform_toggle(&self, ctx: &Context) -> EventResponses {
        let l_sel = *self.left_selected.borrow();
        *self.left_selected.borrow_mut() = !l_sel;
        self.toggled_fn.borrow_mut()(ctx.clone(), self.selected())
    }
}

impl Widget for Toggle {}

impl Element for Toggle {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }
                match true {
                    _ if ke[0].matches_key(&KB::KEY_ENTER) => {
                        return (true, self.perform_toggle(ctx));
                    }
                    _ if ke[0].matches_key(&KB::KEY_LEFT) || ke[0].matches_key(&KB::KEY_H) => {
                        if !*self.left_selected.borrow() {
                            return (true, self.perform_toggle(ctx));
                        }
                        return (true, EventResponses::default());
                    }
                    _ if ke[0].matches_key(&KB::KEY_RIGHT) || ke[0].matches_key(&KB::KEY_L) => {
                        if *self.left_selected.borrow() {
                            return (true, self.perform_toggle(ctx));
                        }
                        return (true, EventResponses::default());
                    }
                    _ => {}
                }
                return (false, EventResponses::default());
            }
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    let x = me.column as usize;
                    let left_sel = *self.left_selected.borrow();
                    if (!left_sel && x < self.left.borrow().chars().count())
                        || (left_sel && x >= self.left.borrow().chars().count())
                    {
                        return (true, self.perform_toggle(ctx));
                    }
                }
                return (false, EventResponses::default());
            }
            _ => {}
        }
        (false, EventResponses::default())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        // need to re set the content in order to reflect active style
        let left = self.left.borrow();
        let right = self.right.borrow();
        let left_len = left.chars().count();
        let right_len = right.chars().count();
        self.base
            .set_content_from_string(ctx, &(left.clone() + &right));
        if *self.left_selected.borrow() {
            for i in 0..left_len {
                self.base.sp.content.borrow_mut()[0][i].style = *self.selected_sty.borrow();
            }
        } else {
            for i in left_len..left_len + right_len {
                self.base.sp.content.borrow_mut()[0][i].style = *self.selected_sty.borrow();
            }
        }
        self.base.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.base.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.base.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.base.set_upward_propagator(up)
    }
}
