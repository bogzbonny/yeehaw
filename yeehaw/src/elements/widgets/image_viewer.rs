use {
    crate::*,
    image::DynamicImage,
    ratatui::widgets::StatefulWidget,
    //ratatui::widgets::StatefulWidget;
    ratatui_image::{
        picker::Picker, protocol::StatefulProtocol, Resize as RatResize, StatefulImage,
    },
};

// displays the size
#[derive(Clone)]
pub struct ImageViewer {
    pub pane: Pane,
    st_pro: Rc<RefCell<StatefulProtocol>>,
    last_size: Rc<RefCell<Size>>,
    resize: Rc<RefCell<Resize>>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl ImageViewer {
    pub fn new(ctx: &Context, dyn_img: DynamicImage, bg: Color) -> Result<Self, Error> {
        let picker = Picker::from_query_stdio();
        let mut picker = match picker {
            Ok(p) => p,
            Err(_) => {
                // NOTE this is used currently if the query stdio fails
                // which is does when using an inner terminal
                //
                // TODO put in actual font size. / query the terminal picker
                // from environment variables which are provided to this terminal.
                //
                // OR somehow actually fix this issue in the terminal pane (if possible)
                Picker::from_fontsize((10, 20))
            }
        };
        let rgba = bg.to_rgba();
        picker.set_background_color([rgba.r, rgba.g, rgba.b, rgba.a]);

        // Create the Protocol which will be used by the widget.
        let st_pro = picker.new_resize_protocol(dyn_img.clone());
        let out = Self {
            pane: Pane::new(ctx, "debug_size_pane"),
            st_pro: Rc::new(RefCell::new(st_pro)),
            last_size: Rc::new(RefCell::new(ctx.size)),
            resize: Rc::new(RefCell::new(Resize::Scale(None))),
        };
        out.update_content(ctx);
        Ok(out)
    }

    pub fn new_from_path(ctx: &Context, img_path: &str, bg: Color) -> Result<Self, Error> {
        let dyn_img = image::ImageReader::open(img_path)?.decode()?;
        Self::new(ctx, dyn_img, bg)
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    pub fn with_resize(self, resize: RatResize) -> Self {
        self.set_resize(resize);
        self
    }

    pub fn set_resize(&self, resize: RatResize) {
        *self.resize.borrow_mut() = resize.into();
    }

    pub fn update_content(&self, ctx: &Context) {
        let area = ratatui::layout::Rect::new(0, 0, ctx.size.width, ctx.size.height);
        //let cell = ratatui::buffer::Cell::default();
        //cell.set_bg(self.bg.into());

        let mut buffer = ratatui::buffer::Buffer::empty(area);
        //let mut buffer = ratatui::buffer::Buffer::filled(area, cell);
        let st_image = StatefulImage::default().resize(self.resize.borrow().clone().into());
        st_image.render(area, &mut buffer, &mut self.st_pro.borrow_mut());

        let mut content: DrawChs2D = buffer.into();
        content.change_all_bg(&Color::TRANSPARENT);

        self.pane.set_content_if_diff(content);
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for ImageViewer {
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        if ctx.size == *self.last_size.borrow() && !force_update {
            return Vec::with_capacity(0);
        }
        self.update_content(ctx);
        self.pane.drawing(ctx, force_update)
    }
}

#[derive(Debug, Clone)]
/// Resize method, mirrors the enum in ratatui_image but has clone
/// TODO remove once ratatui_image gets clone
pub enum Resize {
    Fit(Option<ratatui_image::FilterType>),
    Crop(Option<ratatui_image::CropOptions>),
    Scale(Option<ratatui_image::FilterType>),
}
impl From<Resize> for RatResize {
    fn from(resize: Resize) -> Self {
        match resize {
            Resize::Fit(filter) => RatResize::Fit(filter),
            Resize::Crop(crop) => RatResize::Crop(crop),
            Resize::Scale(filter) => RatResize::Scale(filter),
        }
    }
}
impl From<RatResize> for Resize {
    fn from(resize: RatResize) -> Self {
        match resize {
            RatResize::Fit(filter) => Resize::Fit(filter),
            RatResize::Crop(crop) => Resize::Crop(crop),
            RatResize::Scale(filter) => Resize::Scale(filter),
        }
    }
}
