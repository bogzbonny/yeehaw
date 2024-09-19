use {
    crate::{
        element::ReceivableEventChanges, Context, DrawChPos, DrawChPosVec, DynLocationSet, DynVal,
        Element, ElementID, Event, EventResponses, Pane, Priority, SortingHat, UpwardPropagator,
        ZIndex,
    },
    ratatui::widgets::StatefulWidget,
    ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage},
    std::{cell::RefCell, rc::Rc},
};

// displays the size
#[derive(Clone)]
pub struct ImageViewer {
    pub pane: Pane,
    //image: Rc<RefCell<DynamicImage>>,
    st_pro: Rc<RefCell<Box<dyn StatefulProtocol>>>,
}

impl ImageViewer {
    pub fn new(hat: &SortingHat, img_path: &str) -> Self {
        // Load an image with the image crate.
        let dyn_img = image::ImageReader::open(img_path)
            .unwrap()
            .decode()
            .unwrap();

        let Ok(mut picker) = Picker::from_termios() else {
            panic!("Could not create picker");
        };

        // Guess the protocol.
        picker.guess_protocol();

        // Create the Protocol which will be used by the widget.
        let st_pro = picker.new_resize_protocol(dyn_img);
        Self {
            pane: Pane::new(hat, "debug_size_pane"),
            //image: Rc::new(RefCell::new(dyn_img)),
            st_pro: Rc::new(RefCell::new(st_pro)),
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
        let area = ratatui::layout::Rect::new(0, 0, ctx.s.width, ctx.s.height);
        let mut buffer = ratatui::buffer::Buffer::empty(area);
        let st_image = StatefulImage::new(None);
        st_image.render(area, &mut buffer, &mut self.st_pro.borrow_mut());

        let out: DrawChPosVec = buffer.into();
        out.0
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
