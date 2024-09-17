use {
    crate::{
        element::ReceivableEventChanges, Context, DrawChPos, DynLocationSet, DynVal, Element,
        ElementID, Event, EventResponses, Pane, Priority, Rgba, SortingHat, Style,
        UpwardPropagator, ZIndex,
    },
    ratatui::{backend::TestBackend, terminal::Frame, Terminal},
    ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage},
    std::{cell::RefCell, rc::Rc},
};

// displays the size
#[derive(Clone)]
pub struct ImageViewer {
    pub pane: Pane,
    image: Box<dyn StatefulProtocol>,
}

impl ImageViewer {
    pub fn new(hat: &SortingHat) -> Self {
        Self {
            pane: Pane::new(hat, "debug_size_pane"),
            image: Box::new(StatefulImage::new(None)),
        }
    }

    pub fn with_height(self, h: DynVal) -> Self {
        self.pane.set_dyn_height(h);
        self
    }

    pub fn with_width(self, w: DynVal) -> Self {
        self.pane.set_dyn_width(w);
        self
    }

    pub fn with_z(self, z: ZIndex) -> Self {
        self.pane.set_z(z);
        self
    }
}

impl Element for ImageViewer {
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
        self.pane.receive_event(ctx, ev.clone())
    }
    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let size = ctx.s;
        let backend = TestBackend::new(80, 30);
        let Ok(mut terminal) = Terminal::new(backend) else {
            return vec![];
        };

        // Should use Picker::from_termios(), to get the font size,
        // but we can't put that here because that would break doctests!
        //let mut picker = Picker::new((8, 12));
        let Ok(mut picker) = Picker::from_termios() else {
            return vec![];
        };

        // Guess the protocol.
        picker.guess_protocol();

        // Load an image with the image crate.
        let dyn_img = image::io::Reader::open("./assets/Ada.png")?.decode()?;

        // Create the Protocol which will be used by the widget.
        let image = picker.new_resize_protocol(dyn_img);

        let image_ = StatefulImage::new(None);

        let area = image_.get_area(size.width, size.height);
        let buffer = String::new();
        image_.render(&mut terminal, area, buffer, &image);
        //f.render_stateful_widget(image, f.size(), &mut app.image);
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
