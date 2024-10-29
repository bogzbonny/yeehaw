use {
    super::{Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, DrawChs2D, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Priority, ReceivableEventChanges, SortingHat, Style, Parent,
    },
    figlet_rs::FIGfont,
    std::{cell::RefCell, rc::Rc},
};

// TODO click function

#[derive(Clone)]
pub struct FigletText {
    pub base: WidgetBase,
}

impl FigletText {
    const KIND: &'static str = "widget_megatext";

    pub fn new(hat: &SortingHat, ctx: &Context, text: &str, font: FIGfont) -> Self {
        let Some(fig_text) = font.convert(text) else {
            return FigletText {
                base: WidgetBase::new(
                    hat,
                    Self::KIND,
                    DynVal::new_fixed(0),
                    DynVal::new_fixed(0),
                    WBStyles::default(),
                    vec![],
                ),
            };
        };

        let text = format!("{}", fig_text);
        let text = text.trim_end_matches('\n'); // remove the last newline
        let content = DrawChs2D::from_string(text.to_string(), Style::default());
        let size = content.size();

        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            DynVal::new_fixed(size.width as i32),
            DynVal::new_fixed(size.height as i32),
            WBStyles::default(),
            vec![],
        );
        wb.set_content(content);
        _ = wb.set_selectability(ctx, Selectability::Unselectable);
        FigletText { base: wb }
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base
            .set_content_style(styles.unselectable_style.clone());
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

impl Widget for FigletText {}

impl Element for FigletText {
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
