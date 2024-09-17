use {
    super::{widget::RESP_DEACTIVATE, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event, EventResponse,
        EventResponses, Keyboard as KB, Priority, ReceivableEventChanges, Rgba, SortingHat,
        Style, UpwardPropagator,
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
    pub clicked_down: Rc<RefCell<bool>>, // activated when mouse is clicked down while over button
    // function which executes when button moves from pressed -> unpressed
    #[allow(clippy::type_complexity)]
    pub clicked_fn: Rc<RefCell<dyn FnMut(Context) -> EventResponses>>,
}

impl Button {
    const KIND: &'static str = "widget_button";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new()
            .with_bg(Rgba::LIGHT_YELLOW2)
            .with_fg(Rgba::BLACK),
        ready_style: Style::new()
            .with_bg(Rgba::WHITE)
            .with_fg(Rgba::BLACK),
        unselectable_style: Style::new()
            .with_bg(Rgba::GREY13)
            .with_fg(Rgba::BLACK),
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
            DynVal::new_fixed(text.chars().count() as i32 + 2), // + 2 for sides
            DynVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        _ = wb.set_selectability(ctx, Selectability::Unselectable);
        let b = Button {
            base: wb,
            text: Rc::new(RefCell::new(text)),
            sides: Rc::new(RefCell::new((']'.to_string(), '['.to_string()))),
            clicked_down: Rc::new(RefCell::new(false)),
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
            .set_dyn_width(DynVal::new_fixed(text.chars().count() as i32));
        self.base.set_content_from_string(ctx, &text);
        self
    }

    pub fn without_sides(self, ctx: &Context) -> Self {
        self.with_sides(ctx, ("".to_string(), "".to_string()))
    }

    pub fn at(mut self, loc_x: DynVal, loc_y: DynVal) -> Self {
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
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.base.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.base.get_visible()
    }
}
