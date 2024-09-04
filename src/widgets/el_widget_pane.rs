use {
    super::{Widget, WidgetOrganizer, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponses, Pane, Priority,
        ReceivableEventChanges, DynLocationSet, SortingHat, UpwardPropagator,
    },
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct WidgetPane {
    pub pane: Pane,
    pub org: Rc<RefCell<WidgetOrganizer>>,
}

impl WidgetPane {
    pub const KIND: &'static str = "widget_pane";

    pub fn new(hat: &SortingHat) -> Self {
        let wp = WidgetPane {
            pane: Pane::new(hat, Self::KIND),
            org: Rc::new(RefCell::new(WidgetOrganizer::default())),
        };
        wp.pane.self_evs.borrow_mut().push_many_at_priority(
            WidgetOrganizer::default_receivable_events(),
            Priority::FOCUSED,
        );
        wp
    }

    pub fn add_widget(&mut self, w: Box<dyn Widget>) {
        self.pane.self_evs.borrow_mut().extend(w.receivable());
        w.get_dyn_location_set().borrow_mut().set_z(w.get_z_index());
        self.org.borrow_mut().add_widget(w);
    }

    pub fn add_widgets(&mut self, ws: Widgets) {
        for w in ws.0 {
            self.add_widget(w);
        }
    }

    pub fn remove_widget(&mut self, w: Box<dyn Widget>) {
        self.org.borrow_mut().remove_widget(w);
    }

    pub fn clear_widgets(&mut self) {
        self.org.borrow_mut().clear_widgets();
    }
}

impl Element for WidgetPane {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn id(&self) -> ElementID {
        self.pane.id()
    }

    // Returns the widget organizer's receivable events along
    // with the standard pane's self events.
    fn receivable(&self) -> Vec<(Event, Priority)> {
        // all of the events returned by the widget organizer are set to
        // focused because WO.Receivable only returns the events associated with
        // widget that is currently active.
        let wpes = self.org.borrow_mut().receivable();
        // Add the widget pane's self events. These are default receivable events of the widget
        // organizer
        let mut rec = self.pane.receivable();
        rec.extend(wpes);
        rec
    }

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::Mouse(me) => {
                return self.org.borrow_mut().capture_mouse_event(ctx, me);
            }
            Event::KeyCombo(ke) => {
                //debug!("WidgetPane::receive_key_event: {:?}", ke);
                //debug!("widget el rec evs: {:?}", self.receivable());
                return self.org.borrow_mut().capture_key_event(ctx, ke);
            }
            Event::Resize => {
                self.org.borrow_mut().resize_event(ctx);
            }
            Event::Refresh => {}
            _ => {}
        }
        self.pane.receive_event(ctx, ev)
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        let mut rec = if p == Priority::UNFOCUSED {
            self.org
                .borrow_mut()
                .unselect_selected_widget_process_resp(ctx)
        } else {
            ReceivableEventChanges::default()
        };
        rec.concat(self.pane.change_priority(ctx, p));
        rec
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut chs = self.pane.drawing(ctx);
        chs.extend(self.org.borrow_mut().drawing(ctx));
        chs
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
