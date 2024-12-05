use {
    crate::*,
    image::DynamicImage,
    ratatui::widgets::StatefulWidget,
    //ratatui::widgets::StatefulWidget;
    ratatui_image::{picker::Picker, protocol::StatefulProtocol, Resize, StatefulImage},
    std::{cell::RefCell, rc::Rc},
};

// TODO integrate in resize kind

// displays the size
#[derive(Clone)]
pub struct ImageViewer {
    pub pane: Pane,
    st_pro: Rc<RefCell<StatefulProtocol>>,
}

impl ImageViewer {
    pub fn new(ctx: &Context, img_path: &str) -> Result<Self, Error> {
        let dyn_img = image::ImageReader::open(img_path)?.decode()?;

        let mut picker = Picker::from_query_stdio()?;

        // Create the Protocol which will be used by the widget.
        let st_pro = picker.new_resize_protocol(dyn_img);
        Ok(Self {
            pane: Pane::new(ctx, "image_viewer_pane"),
            st_pro: Rc::new(RefCell::new(st_pro)),
        })
    }

    pub fn new_from_dyn_image(ctx: &Context, dyn_img: DynamicImage) -> Result<Self, Error> {
        let Ok(mut picker) = Picker::from_query_stdio() else {
            panic!("Could not create picker");
        };

        // Create the Protocol which will be used by the widget.
        let st_pro = picker.new_resize_protocol(dyn_img);
        Ok(Self {
            pane: Pane::new(ctx, "debug_size_pane"),
            //image: Rc::new(RefCell::new(dyn_img)),
            st_pro: Rc::new(RefCell::new(st_pro)),
        })
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

#[yeehaw_derive::impl_element_from(pane)]
impl Element for ImageViewer {
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let area = ratatui::layout::Rect::new(0, 0, ctx.size.width, ctx.size.height);
        let mut buffer = ratatui::buffer::Buffer::empty(area);
        let st_image = StatefulImage::new(None).resize(Resize::Crop(None));

        st_image.render(area, &mut buffer, &mut self.st_pro.borrow_mut());
        let out: DrawChPosVec = buffer.into();
        out.0
    }
}
