use {
    super::{Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponses, Keyboard as KB, Priority,
        ReceivableEventChanges, RgbColour, SclLocationSet, SclVal, SortingHat, Style,
        UpwardPropagator, YHAttributes,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct Checkbox {
    pub base: WidgetBase,
    pub checked: Rc<RefCell<bool>>, // whether the checkbox is checked or not

    // rune to use for the checkmark
    // recommended:  √ X x ✖
    pub checkmark: Rc<RefCell<char>>,

    // function which executes when checkbox is checked or unchecked,
    // bool is the new state of the checkbox (true = checked)
    pub clicked_fn: Rc<RefCell<dyn FnMut(Context, bool) -> EventResponses>>,
}

impl Checkbox {
    const KIND: &'static str = "widget_checkbox";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new()
            .with_bg(RgbColour::LIGHT_YELLOW2)
            .with_fg(RgbColour::BLACK)
            .with_attr(YHAttributes::new().with_bold()),
        ready_style: Style::new()
            .with_bg(RgbColour::WHITE)
            .with_fg(RgbColour::BLACK)
            .with_attr(YHAttributes::new().with_bold()),
        unselectable_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::BLACK)
            .with_attr(YHAttributes::new().with_bold()),
    };

    pub fn default_receivable_events() -> Vec<Event> {
        vec![KB::KEY_ENTER.into()] // when "active" hitting enter will click the button
    }

    pub fn new(hat: &SortingHat) -> Self {
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            SclVal::new_fixed(1),
            SclVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        Checkbox {
            base: wb,
            checked: Rc::new(RefCell::new(false)),
            checkmark: Rc::new(RefCell::new('√')),
            clicked_fn: Rc::new(RefCell::new(|_, _| EventResponses::default())),
        }
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn with_clicked_fn(
        mut self, clicked_fn: Box<dyn FnMut(Context, bool) -> EventResponses>,
    ) -> Self {
        self.clicked_fn = Rc::new(RefCell::new(clicked_fn));
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

    pub fn text(&self) -> String {
        if *self.checked.borrow() {
            return self.checkmark.borrow().to_string();
        }
        " ".to_string()
    }

    pub fn click(&self, ctx: &Context) -> EventResponses {
        let checked = !*self.checked.borrow();
        self.checked.replace(checked);
        self.base.set_content_from_string(ctx, &self.text());
        (self.clicked_fn.borrow_mut())(ctx.clone(), checked)
    }
}

impl Widget for Checkbox {}

impl Element for Checkbox {
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
                if ke[0].matches_key(&KB::KEY_ENTER) {
                    return (true, self.click(ctx));
                }
            }
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    return (true, self.click(ctx));
                }
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
        self.base.set_content_from_string(ctx, &self.text());
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

    fn get_scl_location_set(&self) -> Rc<RefCell<SclLocationSet>> {
        self.base.get_scl_location_set()
    }
    fn set_scl_location_set(&self, loc: SclLocationSet) {
        self.base.set_scl_location_set(loc)
    }
    fn visible(&self) -> Rc<RefCell<bool>> {
        self.base.visible()
    }
    fn set_visible(&self, v: bool) {
        self.base.set_visible(v)
    }
}
