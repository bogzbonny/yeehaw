use {
    crate::*,
    image::{DynamicImage, GenericImageView},
    nokhwa::{
        pixel_format::RgbFormat,
        utils::{CameraIndex, RequestedFormat, RequestedFormatType},
        Buffer, Camera,
    },
};

/// A photobooth element that captures images from a camera with snap/discard controls.
///
/// Supports optional image mirroring and crop controls.
#[derive(Clone)]
pub struct Photobooth {
    pane: ParentPaneOfSelectable,
    save_path: Rc<RefCell<String>>,
    frame_rate_ms: u64,
    mirror: Rc<RefCell<bool>>,
    show_crop_controls: bool,
    image_viewer: Rc<ImageViewer>,
    dyn_img: Rc<RefCell<DynamicImage>>,
    stopped: Rc<RefCell<bool>>,
    /// Camera stored for frame capture in drawing().
    camera: Rc<RefCell<Option<Camera>>>,
    /// Last capture timestamp for frame rate control.
    last_capture: Rc<RefCell<Option<std::time::Instant>>>,
    // Crop control state - RefCell<Rc<RefCell<i32>>> allows swapping Rc to share textbox values
    crop_x0: RefCell<Rc<RefCell<i32>>>,
    crop_y0: RefCell<Rc<RefCell<i32>>>,
    crop_x1: RefCell<Rc<RefCell<i32>>>,
    crop_y1: RefCell<Rc<RefCell<i32>>>,
    image_dims: Rc<RefCell<(u32, u32)>>,
    /// Stored context for button/label creation.
    ctx: Rc<Context>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl Photobooth {
    /// Set the position of the photobooth.
    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    /// Create a new Photobooth element.
    ///
    /// # Arguments
    /// * `ctx` - The TUI context
    /// * `save_path` - Optional path to save captured images. If None, uses a temp file.
    /// * `frame_rate_ms` - Capture interval in milliseconds (default: 500)
    /// * `mirror` - Whether to mirror the image horizontally (default: false)
    /// * `show_crop_controls` - Whether to show crop control buttons (default: false)
    pub fn new(
        ctx: &Context,
        save_path: Option<String>,
        frame_rate_ms: Option<u64>,
        mirror: Option<bool>,
        show_crop_controls: Option<bool>,
    ) -> Result<Self, Error> {
        let save_path = save_path.unwrap_or_else(|| {
            std::env::temp_dir()
                .join(format!("photobooth_{}.jpg", std::process::id()))
                .to_string_lossy()
                .to_string()
        });
        let frame_rate_ms = frame_rate_ms.unwrap_or(500);
        let mirror = mirror.unwrap_or(false);
        let show_crop_controls = show_crop_controls.unwrap_or(false);

        // Initialize nokhwa (required on macOS)
        nokhwa::nokhwa_initialize(|_| {});

        // Create camera
        let mut camera = Camera::new(
            CameraIndex::Index(0),
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
        )
        .map_err(|e| {
            let msg = format!("Failed to create camera: {}", e);
            Error::new(msg.as_str())
        })?;

        // Open camera stream
        camera
            .open_stream()
            .map_err(|e| {
                let msg = format!("Failed to open camera stream: {}", e);
                Error::new(msg.as_str())
            })?;

        // Create initial blank image (will be replaced by first frame)
        let blank = DynamicImage::new_rgb8(1, 1);

        // Create image viewer
        let image_viewer = ImageViewer::new(ctx, blank.clone(), Color::TRANSPARENT)?;

        let out = Self {
            pane: ParentPaneOfSelectable::new(ctx),
            save_path: Rc::new(RefCell::new(save_path)),
            frame_rate_ms,
            mirror: Rc::new(RefCell::new(mirror)),
            show_crop_controls,
            image_viewer: Rc::new(image_viewer),
            dyn_img: Rc::new(RefCell::new(blank)),
            stopped: Rc::new(RefCell::new(false)),
            camera: Rc::new(RefCell::new(Some(camera))),
            last_capture: Rc::new(RefCell::new(None)),
            crop_x0: RefCell::new(Rc::new(RefCell::new(0))),
            crop_y0: RefCell::new(Rc::new(RefCell::new(0))),
            crop_x1: RefCell::new(Rc::new(RefCell::new(0))),
            crop_y1: RefCell::new(Rc::new(RefCell::new(0))),
            image_dims: Rc::new(RefCell::new((0, 0))),
            ctx: Rc::new(ctx.clone()),
        };

        // Build UI
        out.build_ui();

        Ok(out)
    }

    fn build_ui(&self) {
        let bottom_rows = if self.show_crop_controls { 2 } else { 1 };

        // Add image viewer
        let img_viewer = self.image_viewer.clone();
        self.pane
            .add_element(Box::new((*img_viewer.clone()).clone().at(DynVal::FULL.minus_fixed(bottom_rows), DynVal::new_fixed(0))));

        // Add buttons row
        self.add_buttons_row();

        // Add crop controls if enabled
        if self.show_crop_controls {
            self.add_crop_row();
        }
    }

    fn add_buttons_row(&self) {
        let row_y = DynVal::FULL.minus_fixed(1);

        // Snap button
        let dyn_img = self.dyn_img.clone();
        let save_path = self.save_path.clone();
        let mirror = self.mirror.clone();
        let image_viewer = self.image_viewer.clone();
        let stopped = self.stopped.clone();
        let snap_btn = Button::new(&self.ctx, "snap");
        snap_btn.set_fn(Box::new(move |_, _| {
            let img = dyn_img.borrow().clone().into_rgb8();
            let final_img = if *mirror.borrow() {
                DynamicImage::ImageRgb8(image::imageops::flip_horizontal(&img))
            } else {
                DynamicImage::ImageRgb8(img)
            };
            // Save the image
            if let Err(e) = final_img.save(save_path.borrow().as_str()) {
                log_err!("Failed to save image: {}", e);
            }
            // Update the shared image
            *dyn_img.borrow_mut() = final_img;
            image_viewer.force_update();
            *stopped.borrow_mut() = true;
            EventResponses::default()
        }));
        self.pane.add_element(Box::new(snap_btn.at(DynVal::new_fixed(0), row_y.clone())));

        // Discard button
        let image_viewer = self.image_viewer.clone();
        let stopped = self.stopped.clone();
        let discard_btn = Button::new(&self.ctx, "discard");
        discard_btn.set_fn(Box::new(move |_, _| {
            *stopped.borrow_mut() = false;
            image_viewer.force_update();
            EventResponses::default()
        }));
        self.pane.add_element(Box::new(discard_btn.at(DynVal::FULL.minus_fixed(15), row_y.clone())));

        // Mirror checkbox
        let mirror = self.mirror.clone();
        let dyn_img = self.dyn_img.clone();
        let image_viewer = self.image_viewer.clone();
        let mirror_cb = Checkbox::new(&self.ctx).with_fn(Box::new(move |_, checked| {
            *mirror.borrow_mut() = checked;
            // Re-apply mirror to current image
            let img = dyn_img.borrow().clone().into_rgb8();
            if checked {
                *dyn_img.borrow_mut() =
                    DynamicImage::ImageRgb8(image::imageops::flip_horizontal(&img));
            } else {
                // Un-mirror: flip back
                *dyn_img.borrow_mut() = DynamicImage::ImageRgb8(img);
            }
            image_viewer.force_update();
            EventResponses::default()
        }));
        // Set initial checked state
        mirror_cb.pane.set_content_from_string(' ');
        self.pane.add_element(Box::new(mirror_cb.at(DynVal::FULL.minus_fixed(30), row_y)));
    }

    fn add_crop_row(&self) {
        let row_y = DynVal::FULL.minus_fixed(2);

        // Create textboxes with initial values
        let x0_tb = NumbersTextBox::new(&self.ctx, 0_i32);
        let y0_tb = NumbersTextBox::new(&self.ctx, 0_i32);
        let x1_tb = NumbersTextBox::new(&self.ctx, 0_i32);
        let y1_tb = NumbersTextBox::new(&self.ctx, 0_i32);

        // Share textbox value Rc's with struct fields
        *self.crop_x0.borrow_mut() = x0_tb.value.clone();
        *self.crop_y0.borrow_mut() = y0_tb.value.clone();
        *self.crop_x1.borrow_mut() = x1_tb.value.clone();
        *self.crop_y1.borrow_mut() = y1_tb.value.clone();

        // Add labels and textboxes
        let x0_label = Label::new(&self.ctx, "x0:");
        self.pane.add_element(Box::new(x0_label.at(DynVal::new_fixed(0), row_y.clone())));
        self.pane.add_element(Box::new(x0_tb.at(DynVal::new_fixed(3), row_y.clone())));

        let y0_label = Label::new(&self.ctx, "y0:");
        self.pane.add_element(Box::new(y0_label.at(DynVal::new_fixed(15), row_y.clone())));
        self.pane.add_element(Box::new(y0_tb.at(DynVal::new_fixed(18), row_y.clone())));

        let x1_label = Label::new(&self.ctx, "x1:");
        self.pane.add_element(Box::new(x1_label.at(DynVal::new_fixed(30), row_y.clone())));
        self.pane.add_element(Box::new(x1_tb.at(DynVal::new_fixed(33), row_y.clone())));

        let y1_label = Label::new(&self.ctx, "y1:");
        self.pane.add_element(Box::new(y1_label.at(DynVal::new_fixed(45), row_y.clone())));
        self.pane.add_element(Box::new(y1_tb.at(DynVal::new_fixed(48), row_y.clone())));

        // Preview button
        let dyn_img = self.dyn_img.clone();
        let save_path = self.save_path.clone();
        let image_viewer = self.image_viewer.clone();
        let crop_x0 = self.crop_x0.clone();
        let crop_y0 = self.crop_y0.clone();
        let crop_x1 = self.crop_x1.clone();
        let crop_y1 = self.crop_y1.clone();
        let preview_btn = Button::new(&self.ctx, "preview");
        preview_btn.set_fn(Box::new(move |_, _| {
            let img = dyn_img.borrow().clone();
            let (x0, y0) = (*crop_x0.borrow().borrow(), *crop_y0.borrow().borrow());
            let (x1, y1) = (*crop_x1.borrow().borrow(), *crop_y1.borrow().borrow());
            if let Some(cropped) = crop_image(&img, x0, y0, x1, y1) {
                let preview_path = format!("{}_crop_preview.jpg", save_path.borrow());
                if let Err(e) = cropped.save(&preview_path) {
                    log_err!("Failed to save crop preview: {}", e);
                }
                *dyn_img.borrow_mut() = cropped;
                image_viewer.force_update();
            }
            EventResponses::default()
        }));
        self.pane.add_element(Box::new(preview_btn.at(DynVal::new_fixed(60), row_y.clone())));

        // Crop button
        let dyn_img = self.dyn_img.clone();
        let save_path = self.save_path.clone();
        let image_viewer = self.image_viewer.clone();
        let crop_x0 = self.crop_x0.clone();
        let crop_y0 = self.crop_y0.clone();
        let crop_x1 = self.crop_x1.clone();
        let crop_y1 = self.crop_y1.clone();
        let image_dims = self.image_dims.clone();
        let crop_btn = Button::new(&self.ctx, "crop");
        crop_btn.set_fn(Box::new(move |_, _| {
            let img = dyn_img.borrow().clone();
            let (x0, y0) = (*crop_x0.borrow().borrow(), *crop_y0.borrow().borrow());
            let (x1, y1) = (*crop_x1.borrow().borrow(), *crop_y1.borrow().borrow());
            if let Some(cropped) = crop_image(&img, x0, y0, x1, y1) {
                // Save cropped image to original path
                if let Err(e) = cropped.save(save_path.borrow().as_str()) {
                    log_err!("Failed to save cropped image: {}", e);
                }
                // Delete preview file
                let preview_path = format!("{}_crop_preview.jpg", save_path.borrow());
                let _ = std::fs::remove_file(&preview_path);
                // Update shared image
                *dyn_img.borrow_mut() = cropped;
                image_viewer.force_update();
                // Reset crop values to full image
                let (w, h) = *image_dims.borrow();
                *crop_x0.borrow().borrow_mut() = 0;
                *crop_y0.borrow().borrow_mut() = 0;
                *crop_x1.borrow().borrow_mut() = w as i32;
                *crop_y1.borrow().borrow_mut() = h as i32;
            }
            EventResponses::default()
        }));
        self.pane.add_element(Box::new(crop_btn.at(DynVal::new_fixed(70), row_y)));
    }
}

/// Crop an image to the specified region. Returns None if the region is invalid.
fn crop_image(img: &DynamicImage, x0: i32, y0: i32, x1: i32, y1: i32) -> Option<DynamicImage> {
    let (width, height) = img.dimensions();
    let x0 = x0.max(0).min(width as i32) as u32;
    let y0 = y0.max(0).min(height as i32) as u32;
    let x1 = x1.max(0).min(width as i32) as u32;
    let y1 = y1.max(0).min(height as i32) as u32;
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(img.crop_imm(x0, y0, x1 - x0, y1 - y0).into())
}

/// Convert a nokhwa Buffer to a DynamicImage using RgbFormat decoder.
fn buffer_to_dynamic_image(buffer: &Buffer) -> Result<DynamicImage, String> {
    buffer
        .decode_image::<RgbFormat>()
        .map(|img| img.into())
        .map_err(|e| format!("Failed to decode frame: {}", e))
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Photobooth {
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        // Capture frame if not stopped and frame rate interval has passed
        if !*self.stopped.borrow() {
            let now = std::time::Instant::now();
            let should_capture = {
                let last = self.last_capture.borrow();
                match *last {
                    Some(instant) => now.duration_since(instant) >= std::time::Duration::from_millis(self.frame_rate_ms),
                    None => true,
                }
            };

            if should_capture {
                if let Some(cam) = self.camera.borrow_mut().as_mut() {
                    if let Ok(buffer) = cam.frame() {
                        if let Ok(img) = buffer_to_dynamic_image(&buffer) {
                            let (w, h) = img.dimensions();

                            // Update image dimensions on first frame
                            if *self.image_dims.borrow() == (0, 0) {
                                *self.image_dims.borrow_mut() = (w, h);
                                *self.crop_x1.borrow().borrow_mut() = w as i32;
                                *self.crop_y1.borrow().borrow_mut() = h as i32;
                            }

                            let final_img = if *self.mirror.borrow() {
                                let rgb = img.to_rgb8();
                                DynamicImage::ImageRgb8(image::imageops::flip_horizontal(&rgb))
                            } else {
                                img
                            };

                            *self.dyn_img.borrow_mut() = final_img;
                            self.image_viewer.force_update();
                        }
                    }
                }
                *self.last_capture.borrow_mut() = Some(now);
            }
        }
        self.pane.drawing(ctx, dr, force_update)
    }
}
