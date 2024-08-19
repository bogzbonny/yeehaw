use {
    super::{Widget, WidgetOrganizer, Widgets},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponses, LocationSet, Priority,
        ReceivableEventChanges, SortingHat, StandardPane, UpwardPropagator,
    },
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct WidgetPane {
    pub sp: StandardPane,
    pub org: Rc<RefCell<WidgetOrganizer>>,
}

impl WidgetPane {
    pub const KIND: &'static str = "widget_pane";

    pub fn new(hat: &SortingHat) -> Self {
        let wp = WidgetPane {
            sp: StandardPane::new(hat, Self::KIND),
            org: Rc::new(RefCell::new(WidgetOrganizer::default())),
        };
        wp.sp.self_evs.borrow_mut().push_many_at_priority(
            WidgetOrganizer::default_receivable_events(),
            Priority::FOCUSED,
        );
        wp
    }

    pub fn add_widget(&mut self, ctx: &Context, w: Box<dyn Widget>) {
        self.sp.self_evs.borrow_mut().extend(w.receivable());
        let l = w.get_scl_location().get_location_for_context(ctx);
        let l = LocationSet::default()
            .with_location(l)
            .with_z(w.get_z_index());
        self.org.borrow_mut().add_widget(w, l);
    }

    pub fn add_widgets(&mut self, ctx: &Context, ws: Widgets) {
        for w in ws.0 {
            self.add_widget(ctx, w);
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
        self.sp.id()
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
        let mut rec = self.sp.receivable();
        rec.extend(wpes);
        rec
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::Mouse(me) => {
                return self.org.borrow_mut().capture_mouse_event(ctx, me);
            }
            Event::KeyCombo(ke) => {
                return self.org.borrow_mut().capture_key_event(ctx, ke);
            }
            Event::Resize => {
                self.org.borrow_mut().resize_event(ctx);
            }
            Event::Refresh => {
                self.org.borrow_mut().refresh();
            }
            _ => {}
        }
        self.sp.receive_event(ctx, ev)
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        let mut rec = if p == Priority::UNFOCUSED {
            self.org.borrow_mut().unselect_selected_widget(ctx)
        } else {
            ReceivableEventChanges::default()
        };
        rec.concat(self.sp.change_priority(ctx, p));
        rec
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut chs = self.sp.drawing(ctx);
        chs.extend(self.org.borrow_mut().drawing(ctx));
        chs
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.sp.get_attribute(key)
    }

    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.sp.set_attribute(key, value)
    }

    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.sp.set_upward_propagator(up)
    }
}
