use {
    super::{widget::RESP_DEACTIVATE, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponse, EventResponses,
        Keyboard as KB, Priority, ReceivableEventChanges, RgbColour, SclLocationSet, SclVal,
        SortingHat, Style, UpwardPropagator,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// TODO simulate button press with down click, time based event

// button sides inspiration:
//	]button[  ⡇button⢸
//  ⢸button⡇   ⎤button⎣
//  ❳button❲  ⎣button⎤

#[derive(Clone)]
pub struct Button {
    pub base: WidgetBase,
    pub text: Rc<RefCell<String>>,
    pub sides: Rc<RefCell<(String, String)>>, // left right
    // function which executes when button moves from pressed -> unpressed
    #[allow(clippy::type_complexity)]
    pub clicked_fn: Rc<RefCell<dyn FnMut(Context) -> EventResponses>>,
}

impl Button {
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

    pub fn default_receivable_events() -> Vec<Event> {
        vec![KB::KEY_ENTER.into()] // when "active" hitting enter will click the button
    }

    pub fn button_text(&self) -> String {
        let (left, right) = &*self.sides.borrow();
        format!("{}{}{}", left, *self.text.borrow(), right)
    }

    pub fn new(
        hat: &SortingHat, ctx: &Context, text: String,
        clicked_fn: Box<dyn FnMut(Context) -> EventResponses>,
    ) -> Self {
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            SclVal::new_fixed(text.chars().count() as i32 + 2), // + 2 for sides
            SclVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        _ = wb.set_selectability(ctx, Selectability::Unselectable);
        let b = Button {
            base: wb,
            text: Rc::new(RefCell::new(text)),
            sides: Rc::new(RefCell::new((']'.to_string(), '['.to_string()))),
            clicked_fn: Rc::new(RefCell::new(clicked_fn)),
        };
        b.base.set_content_from_string(ctx, &b.button_text());
        b
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn with_sides(self, ctx: &Context, sides: (String, String)) -> Self {
        *self.sides.borrow_mut() = sides;
        let text = self.button_text();
        self.base
            .set_scl_width(SclVal::new_fixed(text.chars().count() as i32));
        self.base.set_content_from_string(ctx, &text);
        self
    }

    pub fn without_sides(self, ctx: &Context) -> Self {
        self.with_sides(ctx, ("".to_string(), "".to_string()))
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }

    // ----------------------------------------------
    pub fn click(&self, ctx: &Context) -> EventResponses {
        let mut resps = (self.clicked_fn.borrow_mut())(ctx.clone());
        //resp.with_deactivate()
        resps.push(EventResponse::Metadata((
            RESP_DEACTIVATE.to_string(),
            vec![],
        )));
        resps
    }
}

impl Widget for Button {}

impl Element for Button {
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
    fn get_scl_location_set(&self) -> Rc<RefCell<SclLocationSet>> {
        self.base.get_scl_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.base.get_visible()
    }
}
