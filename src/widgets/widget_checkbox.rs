use {
    super::{Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Attributes, Color, Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Keyboard as KB, Priority, ReceivableEventChanges,  Style,
        Parent,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct Checkbox {
    pub base: WidgetBase,
    pub checked: Rc<RefCell<bool>>, // whether the checkbox is checked or not

    pub clicked_down: Rc<RefCell<bool>>, // activated when mouse is clicked down while over object

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
        selected_style: Style::new(Some(Color::BLACK), Some(Color::LIGHT_YELLOW2), None)
            .with_attr(Attributes::new().with_bold()),
        ready_style: Style::new(Some(Color::BLACK), Some(Color::WHITE), None)
            .with_attr(Attributes::new().with_bold()),
        unselectable_style: Style::new(Some(Color::BLACK), Some(Color::GREY13), None)
            .with_attr(Attributes::new().with_bold()),
    };

    pub fn default_receivable_events() -> Vec<Event> {
        vec![KB::KEY_ENTER.into()] // when "active" hitting enter will click the button
    }

    pub fn new(ctx: &Context) -> Self {
        let wb = WidgetBase::new(
            ctx,
            Self::KIND,
            DynVal::new_fixed(1),
            DynVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        Checkbox {
            base: wb,
            checked: Rc::new(RefCell::new(false)),
            clicked_down: Rc::new(RefCell::new(false)),
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

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
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

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
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
                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, EventResponses::default());
                    }
                    MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        *self.clicked_down.borrow_mut() = false;
                        return (true, self.click(ctx));
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }
            }
            _ => {}
        }
        (false, EventResponses::default())
    }

    fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority( p)
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
    fn set_parent(&self, up: Box<dyn Parent>) {
        self.base.set_parent(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.base.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.base.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.base.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.base.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.base.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.base.get_visible()
    }
}
