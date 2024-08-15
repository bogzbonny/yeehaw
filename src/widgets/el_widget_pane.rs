use {
    super::{Widget, WidgetOrganizer},
    crate::{
        Context, DrawChPos, Element, ElementID, Event, EventResponse, LocationSet, Priority,
        ReceivableEventChanges, SortingHat, StandardPane, UpwardPropagator, ZIndex,
    },
    std::{cell::RefCell, rc::Rc},
};

pub struct WidgetPane {
    pub sp: StandardPane,
    pub org: WidgetOrganizer,
}

const WIDGET_Z_INDEX: ZIndex = 10;

impl WidgetPane {
    pub const KIND: &'static str = "widget_pane";

    pub fn new(hat: &SortingHat) -> Self {
        let mut wp = WidgetPane {
            sp: StandardPane::new(hat, Self::KIND),
            org: WidgetOrganizer::default(),
        };
        wp.sp.self_evs.push_many_at_priority(
            WidgetOrganizer::default_receivable_events(),
            Priority::FOCUSED,
        );
        wp
    }

    pub fn add_widget(&mut self, ctx: &Context, w: Box<dyn Widget>) {
        self.sp.self_evs.extend(w.receivable());
        let l = w.get_scl_location().get_location_for_context(ctx);
        let l = LocationSet::default()
            .with_location(l)
            .with_z(WIDGET_Z_INDEX);
        self.org.add_widget(w, l);
    }

    pub fn add_widgets(&mut self, ctx: &Context, ws: Vec<Box<dyn Widget>>) {
        for w in ws {
            self.add_widget(ctx, w);
        }
    }

    pub fn remove_widget(&mut self, w: Box<dyn Widget>) {
        self.org.remove_widget(w);
    }

    pub fn clear_widgets(&mut self) {
        self.org.clear_widgets();
    }
}

//fn kind(&self) -> &'static str;
//fn id(&self) -> &ElementID;
//fn receivable(&self) -> Vec<(Event, Priority)>;
//fn receive_event(&mut self, ctx: &Context, ev: Event) -> (bool, EventResponse);
//fn change_priority(&mut self, ctx: &Context, p: Priority) -> ReceivableEventChanges;
//fn drawing(&self, ctx: &Context) -> Vec<DrawChPos>;
//fn get_attribute(&self, key: &str) -> Option<&[u8]>;
//fn set_attribute(&mut self, key: &str, value: Vec<u8>);
//fn set_upward_propagator(&mut self, up: Rc<RefCell<dyn UpwardPropagator>>);
impl Element for WidgetPane {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn id(&self) -> &ElementID {
        self.sp.id()
    }

    // Returns the widget organizer's receivable events along
    // with the standard pane's self events.
    fn receivable(&self) -> Vec<(Event, Priority)> {
        // all of the events returned by the widget organizer are set to
        // focused because WO.Receivable only returns the events associated with
        // widget that is currently active.
        let wpes = self.org.receivable();
        // Add the widget pane's self events. These are default receivable events of the widget
        // organizer
        let mut rec = self.sp.receivable();
        rec.extend(wpes);
        rec
    }

    fn receive_event(&mut self, ctx: &Context, ev: Event) -> (bool, EventResponse) {
        match ev {
            Event::Mouse(me) => {
                return self.org.capture_mouse_event(ctx, me);
            }
            Event::KeyCombo(ke) => {
                return self.org.capture_key_event(ctx, ke);
            }
            Event::Resize => {
                self.org.resize_event(ctx);
            }
            Event::Refresh => {
                self.org.refresh();
            }
            _ => {}
        }
        self.sp.receive_event(ctx, ev)
    }

    fn change_priority(&mut self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        let mut rec = if p == Priority::UNFOCUSED {
            self.org.unselect_selected_widget()
        } else {
            ReceivableEventChanges::default()
        };
        rec.concat(self.sp.change_priority(ctx, p));
        rec
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut chs = self.sp.drawing(ctx);
        chs.extend(self.org.drawing(ctx));
        chs
    }

    fn get_attribute(&self, key: &str) -> Option<&[u8]> {
        self.sp.get_attribute(key)
    }

    fn set_attribute(&mut self, key: &str, value: Vec<u8>) {
        self.sp.set_attribute(key, value)
    }

    fn set_upward_propagator(&mut self, up: Rc<RefCell<dyn UpwardPropagator>>) {
        self.sp.set_upward_propagator(up)
    }
}
