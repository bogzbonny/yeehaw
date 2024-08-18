use {
    super::{Megafont, SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponses, Priority,
        ReceivableEventChanges, SortingHat, UpwardPropagator,
    },
    std::{cell::RefCell, rc::Rc},
};

// TODO click function

#[derive(Clone)]
pub struct Megatext {
    pub base: WidgetBase,
}

impl Megatext {
    const KIND: &'static str = "widget_megatext";

    pub fn new(hat: &SortingHat, text: String, font: Megafont) -> Self {
        let mega_text = font.get_mega_text(&text);
        let size = mega_text.size();

        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            SclVal::new_fixed(size.width.into()),
            SclVal::new_fixed(size.height.into()),
            WBStyles::default(),
            vec![],
        );
        wb.set_content(mega_text);
        _ = wb.set_selectability(Selectability::Unselectable);
        Megatext { base: wb }
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
}

impl Widget for Megatext {}

impl Element for Megatext {
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
        self.base.receive_event(ctx, ev)
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
    fn set_upward_propagator(&self, up: Rc<RefCell<dyn UpwardPropagator>>) {
        self.base.set_upward_propagator(up)
    }
}
