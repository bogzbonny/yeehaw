use {
    super::{Megafont, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponses, Priority,
        ReceivableEventChanges, DynLocationSet, DynVal, SortingHat, UpwardPropagator,
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

    pub fn new(hat: &SortingHat, ctx: &Context, text: String, font: Megafont) -> Self {
        let mega_text = font.get_mega_text(&text);
        let size = mega_text.size();

        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            DynVal::new_fixed(size.width.into()),
            DynVal::new_fixed(size.height.into()),
            WBStyles::default(),
            vec![],
        );
        wb.set_content(mega_text);
        _ = wb.set_selectability(ctx, Selectability::Unselectable);
        Megatext { base: wb }
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn at(mut self, loc_x: DynVal, loc_y: DynVal) -> Self {
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
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
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
