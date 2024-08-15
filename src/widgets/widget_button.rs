use {
    super::{SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponse, Keyboard as KB, Priority,
        ReceivableEventChanges, RgbColour, SortingHat, Style, UpwardPropagator,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// TODO simulate button press with down click

// button sides inspiration:
//	]button[  ⡇button⢸
//  ⢸button⡇   ⎤button⎣
//  ❳button❲  ⎣button⎤

pub struct Button {
    pub base: WidgetBase,
    pub text: String,
    pub sides: (char, char),               // left right
    pub clicked_fn: fn() -> EventResponse, // function which executes when button moves from pressed -> unpressed
}

impl Button {
    const KIND: &'static str = "widget_button";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new()
            .with_bg(RgbColour::YELLOW)
            .with_fg(RgbColour::BLACK),
        ready_style: Style::new()
            .with_bg(RgbColour::WHITE)
            .with_fg(RgbColour::BLACK),
        unselectable_style: Style::new()
            .with_bg(RgbColour::GREY5)
            .with_fg(RgbColour::BLACK),
    };

    pub fn default_receivable_events() -> Vec<Event> {
        vec![KB::KEY_ENTER.into()] // when "active" hitting enter will click the button
    }

    pub fn button_text(&self) -> String {
        format!("{}{}{}", self.text, self.sides.0, self.sides.1)
    }

    pub fn new(
        hat: &SortingHat, ctx: &Context, text: String, clicked_fn: fn() -> EventResponse,
    ) -> Self {
        let mut wb = WidgetBase::new(
            hat,
            Self::KIND,
            ctx.clone(),
            SclVal::new_fixed(text.len() + 2),
            SclVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        _ = wb.set_selectability(Selectability::Unselectable);
        let mut b = Button {
            base: wb,
            text,
            sides: (']', '['),
            clicked_fn,
        };
        b.base.set_content_from_string(&b.button_text());
        b
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(mut self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn with_sides(mut self, sides: (char, char)) -> Self {
        self.sides = sides;
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
    pub fn click(&self) -> EventResponse {
        let resp = (self.clicked_fn)();
        resp.with_deactivate()
    }
}

impl Widget for Button {}

impl Element for Button {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> &ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }

    fn receive_event(&mut self, ctx: &Context, ev: Event) -> (bool, EventResponse) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponse::default());
                }
                if ke[0].matches(&KB::KEY_ENTER) {
                    return (true, self.click());
                }
            }
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    return (true, self.click());
                }
            }
            _ => {}
        }
        (false, EventResponse::default())
    }

    fn change_priority(&mut self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.base.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<&[u8]> {
        self.base.get_attribute(key)
    }
    fn set_attribute(&mut self, key: &str, value: Vec<u8>) {
        self.base.set_attribute(key, value)
    }
    fn set_upward_propagator(&mut self, up: Rc<RefCell<dyn UpwardPropagator>>) {
        self.base.set_upward_propagator(up)
    }
}
