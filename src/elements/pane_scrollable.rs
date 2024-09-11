use {
    crate::{
        elements::menu::{MenuItem, MenuPath, MenuStyle},
        widgets::{
            common, HorizontalSBPositions, HorizontalScrollbar, Label, Selectability,
            VerticalSBPositions, VerticalScrollbar, WBStyles, Widget, WidgetBase, Widgets,
        },
        Context, DrawCh, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Error, Event,
        EventResponse, EventResponses, KeyPossibility, Keyboard as KB, ParentPane, Priority,
        ReceivableEventChanges, RgbColour, RightClickMenu, SortingHat, Style, UpwardPropagator,
        VerticalStack,
    },
    crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

pub struct PaneScrollable {
    pane: VerticalStack,

    view_height: Rc<RefCell<DynVal>>,
    view_width: Rc<RefCell<DynVal>>,
    content_width: Rc<RefCell<usize>>,
    content_height: Rc<RefCell<usize>>,
    content_view_offset_x: Rc<RefCell<usize>>,
    content_view_offset_y: Rc<RefCell<usize>>,

    pub x_scrollbar: Rc<RefCell<Option<HorizontalScrollbar>>>,
    pub y_scrollbar: Rc<RefCell<Option<VerticalScrollbar>>>,
    // for when there are two scrollbars
    pub corner_decor: Rc<RefCell<DrawCh>>,
}

impl PaneScrollable {
    pub fn new(
        hat: &SortingHat, ctx: &Context, main_pane: Box<dyn Element>, x_scrollbar_op: HorizontalSBPositions,
        y_scrollbar_op: VerticalSBPositions,
    ) -> Self {
        let pane = VerticalStack::new(hat);
        let x_scrollbar = Rc::new(RefCell::new(None));
        let y_scrollbar = Rc::new(RefCell::new(None));
        let corner_decor = Rc::new(RefCell::new(DrawCh::new('⁙', false, Style::new())));

        let main_height = 

        match (y_scrollbar_op, x_scrollbar_op) {
            (VerticalSBPositions::None, HorizontalSBPositions::None) => {}
            (VerticalSBPositions::Right, HorizontalSBPositions::None) => {
                let y_scrollbar = VerticalScrollbar::new(hat, y_scrollbar_op);
                pane.push(ctx, y_scrollbar.clone());
                *y_scrollbar.borrow_mut() = Some(y_scrollbar);
            }
        }

        Self {
            pane,
            x_scrollbar_op,
            y_scrollbar_op,
            x_scrollbar,
            y_scrollbar,
            corner_decor,
        }
    }
}

impl Element for PaneScrollable {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.pane.receive_event(ctx, ev.clone());
        match ev {
            Event::Mouse(me) => {
                self.select();
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    if let Some(ref mut click_fn) = *self.click_fn.borrow_mut() {
                        return (true, click_fn(ctx.clone()));
                    }
                }
                (true, EventResponses::default())
            }
            Event::ExternalMouse(_) => {
                self.unselect();
                (true, EventResponses::default())
            }
            _ => (true, EventResponses::default()),
        }
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        pane.drawing()
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.pane.set_upward_propagator(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.pane.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.pane.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.pane.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.pane.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.pane.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.pane.get_visible()
    }
}
