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
    /// Uncropped source image (what the camera captured or what was cropped).
    src_img: Rc<RefCell<DynamicImage>>,
    /// Currently displayed image (src_img with mirror applied, or preview crop).
    dyn_img: Rc<RefCell<DynamicImage>>,
    stopped: Rc<RefCell<bool>>,
    /// Camera stored for frame capture in drawing().
    camera: Rc<RefCell<Option<Camera>>>,
    /// Last capture timestamp for frame rate control.
    last_capture: Rc<RefCell<Option<std::time::Instant>>>,
    /// True until the first camera frame is captured.
    first_frame: Rc<RefCell<bool>>,
    // Crop control state — Option because textboxes only exist when crop controls enabled.
    crop_x0: Option<Rc<RefCell<i32>>>,
    crop_y0: Option<Rc<RefCell<i32>>>,
    crop_x1: Option<Rc<RefCell<i32>>>,
    crop_y1: Option<Rc<RefCell<i32>>>,
    /// Textbox references for crop value reset display update.
    crop_x0_tb: Option<Rc<NumbersTextBox<i32>>>,
    crop_y0_tb: Option<Rc<NumbersTextBox<i32>>>,
    crop_x1_tb: Option<Rc<NumbersTextBox<i32>>>,
    crop_y1_tb: Option<Rc<NumbersTextBox<i32>>>,
    /// True while a crop preview is being displayed.
    preview_active: Rc<RefCell<bool>>,
    /// Holds the preview image (cropped region from src_img, mirror not applied).
    preview_img: Rc<RefCell<Option<DynamicImage>>>,
    /// Stored context for button/label creation.
    ctx: Context,
}

impl Drop for Photobooth {
    fn drop(&mut self) {
        // Take camera out of Option; Camera::drop closes the stream automatically
        self.camera.borrow_mut().take();
        // Clean up temp crop preview file if it exists
        let preview_path = make_preview_path(&self.save_path.borrow());
        let _ = std::fs::remove_file(&preview_path);
    }
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
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::DefaultValue),
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

        let mut out = Self {
            pane: ParentPaneOfSelectable::new(ctx),
            save_path: Rc::new(RefCell::new(save_path)),
            frame_rate_ms,
            mirror: Rc::new(RefCell::new(mirror)),
            show_crop_controls,
            image_viewer: Rc::new(image_viewer),
            src_img: Rc::new(RefCell::new(blank.clone())),
            dyn_img: Rc::new(RefCell::new(blank)),
            stopped: Rc::new(RefCell::new(false)),
            camera: Rc::new(RefCell::new(Some(camera))),
            last_capture: Rc::new(RefCell::new(None)),
            first_frame: Rc::new(RefCell::new(true)),
            crop_x0: None,
            crop_y0: None,
            crop_x1: None,
            crop_y1: None,
            crop_x0_tb: None,
            crop_y0_tb: None,
            crop_x1_tb: None,
            crop_y1_tb: None,
            preview_active: Rc::new(RefCell::new(false)),
            preview_img: Rc::new(RefCell::new(None)),
            ctx: ctx.clone(),
        };

        // Build UI
        out.build_ui();

        Ok(out)
    }

    fn build_ui(&mut self) {
        let bottom_rows = if self.show_crop_controls { 2 } else { 1 };

        // Add image viewer — fill full width, height minus bottom rows
        let iv = self.image_viewer.as_ref().clone()
            .with_dyn_width(DynVal::FULL)
            .with_dyn_height(DynVal::FULL.minus_fixed(bottom_rows))
            .at(DynVal::new_fixed(0), DynVal::new_fixed(0));
        self.pane.add_element(Box::new(iv));

        // Add buttons row at bottom
        self.add_buttons_row();

        // Add crop controls if enabled
        if self.show_crop_controls {
            self.add_crop_row();
        }
    }

    fn add_buttons_row(&self) {
        // Buttons always go on the bottommost row
        let row_y = DynVal::FULL.minus_fixed(1);

        // Snap button
        let dyn_img = self.dyn_img.clone();
        let save_path = self.save_path.clone();
        let stopped = self.stopped.clone();
        let snap_btn = Button::new(&self.ctx, "snap");
        snap_btn.set_fn(Box::new(move |_, _| {
            let img = dyn_img.borrow().clone();
            if let Err(e) = img.save(save_path.borrow().as_str()) {
                log_err!("Failed to save image: {}", e);
            }
            *stopped.borrow_mut() = true;
            EventResponses::default()
        }));
        self.pane.add_element(Box::new(snap_btn.at(DynVal::new_fixed(0), row_y.clone())));

        // Discard button
        let stopped = self.stopped.clone();
        let preview_active = self.preview_active.clone();
        let preview_img = self.preview_img.clone();
        let discard_btn = Button::new(&self.ctx, "discard");
        discard_btn.set_fn(Box::new(move |_, _| {
            *stopped.borrow_mut() = false;
            // Clear any active crop preview when returning to capture mode
            *preview_active.borrow_mut() = false;
            *preview_img.borrow_mut() = None;
            EventResponses::default()
        }));
        self.pane.add_element(Box::new(discard_btn.at(DynVal::FULL.minus_fixed(15), row_y.clone())));

        // Mirror checkbox
        let mirror = self.mirror.clone();
        let mirror_clone = mirror.clone();
        let src_img = self.src_img.clone();
        let dyn_img = self.dyn_img.clone();
        let image_viewer = self.image_viewer.clone();
        let preview_active = self.preview_active.clone();
        let preview_img = self.preview_img.clone();
        let mirror_cb = Checkbox::new(&self.ctx).with_fn(Box::new(move |_, checked| {
            *mirror.borrow_mut() = checked;
            // Apply mirror to the correct base image depending on mode
            let base_img = if *preview_active.borrow() {
                // In preview mode, apply mirror to the preview crop
                preview_img.borrow().as_ref().cloned().unwrap_or_else(|| {
                    // Fallback to src_img if preview_img is empty
                    src_img.borrow().clone()
                })
            } else {
                src_img.borrow().clone()
            };
            let img = base_img.into_rgb8();
            let final_img = if checked {
                DynamicImage::ImageRgb8(image::imageops::flip_horizontal(&img))
            } else {
                DynamicImage::ImageRgb8(img)
            };
            *dyn_img.borrow_mut() = final_img.clone();
            image_viewer.update_image(final_img);
            EventResponses::default()
        }));
        // Set initial checked state
        *mirror_cb.checked.borrow_mut() = *mirror_clone.borrow();
        mirror_cb.pane.set_content_from_string(mirror_cb.text());
        self.pane.add_element(Box::new(mirror_cb.at(DynVal::FULL.minus_fixed(30), row_y)));
    }

    fn add_crop_row(&mut self) {
        // Crop row sits one row above the button row
        let row_y = DynVal::FULL.minus_fixed(2);

        // Create textboxes with initial values; min=0, max updated on first frame
        let x0_tb = Rc::new(NumbersTextBox::new(&self.ctx, 0_i32).with_min(0));
        let y0_tb = Rc::new(NumbersTextBox::new(&self.ctx, 0_i32).with_min(0));
        let x1_tb = Rc::new(NumbersTextBox::new(&self.ctx, 0_i32).with_min(0));
        let y1_tb = Rc::new(NumbersTextBox::new(&self.ctx, 0_i32).with_min(0));

        // Share textbox value Rc's with struct fields
        self.crop_x0 = Some(x0_tb.value.clone());
        self.crop_y0 = Some(y0_tb.value.clone());
        self.crop_x1 = Some(x1_tb.value.clone());
        self.crop_y1 = Some(y1_tb.value.clone());

        // Store textbox references for crop button display update
        self.crop_x0_tb = Some(x0_tb.clone());
        self.crop_y0_tb = Some(y0_tb.clone());
        self.crop_x1_tb = Some(x1_tb.clone());
        self.crop_y1_tb = Some(y1_tb.clone());

        // Add labels and textboxes
        let x0_label = Label::new(&self.ctx, "x0:");
        self.pane.add_element(Box::new(x0_label.at(DynVal::new_fixed(0), row_y.clone())));
        self.pane.add_element(Box::new((*x0_tb.clone()).clone().at(DynVal::new_fixed(3), row_y.clone())));

        let y0_label = Label::new(&self.ctx, "y0:");
        self.pane.add_element(Box::new(y0_label.at(DynVal::new_fixed(15), row_y.clone())));
        self.pane.add_element(Box::new((*y0_tb.clone()).clone().at(DynVal::new_fixed(18), row_y.clone())));

        let x1_label = Label::new(&self.ctx, "x1:");
        self.pane.add_element(Box::new(x1_label.at(DynVal::new_fixed(30), row_y.clone())));
        self.pane.add_element(Box::new((*x1_tb.clone()).clone().at(DynVal::new_fixed(33), row_y.clone())));

        let y1_label = Label::new(&self.ctx, "y1:");
        self.pane.add_element(Box::new(y1_label.at(DynVal::new_fixed(45), row_y.clone())));
        self.pane.add_element(Box::new((*y1_tb.clone()).clone().at(DynVal::new_fixed(48), row_y.clone())));

        // Helper: sync all crop textboxes from their text content into the shared Rc values
        let sync_crop_values = |x0_tb: &Option<Rc<NumbersTextBox<i32>>>,
                                y0_tb: &Option<Rc<NumbersTextBox<i32>>>,
                                x1_tb: &Option<Rc<NumbersTextBox<i32>>>,
                                y1_tb: &Option<Rc<NumbersTextBox<i32>>>| {
            if let Some(tb) = x0_tb { tb.update_value_from_tb(); }
            if let Some(tb) = y0_tb { tb.update_value_from_tb(); }
            if let Some(tb) = x1_tb { tb.update_value_from_tb(); }
            if let Some(tb) = y1_tb { tb.update_value_from_tb(); }
        };

        // Preview button — crop from src_img, display only (do NOT modify src_img or dyn_img)
        let src_img = self.src_img.clone();
        let save_path = self.save_path.clone();
        let image_viewer = self.image_viewer.clone();
        let dyn_img = self.dyn_img.clone();
        let crop_x0 = self.crop_x0.clone();
        let crop_y0 = self.crop_y0.clone();
        let crop_x1 = self.crop_x1.clone();
        let crop_y1 = self.crop_y1.clone();
        let crop_x0_tb = self.crop_x0_tb.clone();
        let crop_y0_tb = self.crop_y0_tb.clone();
        let crop_x1_tb = self.crop_x1_tb.clone();
        let crop_y1_tb = self.crop_y1_tb.clone();
        let mirror = self.mirror.clone();
        let preview_active = self.preview_active.clone();
        let preview_img = self.preview_img.clone();
        let preview_btn = Button::new(&self.ctx, "preview");
        preview_btn.set_fn(Box::new(move |_, _| {
            // Sync textbox values before reading
            sync_crop_values(&crop_x0_tb, &crop_y0_tb, &crop_x1_tb, &crop_y1_tb);

            let img = src_img.borrow().clone();
            let (x0, y0) = match (&crop_x0, &crop_y0) {
                (Some(a), Some(b)) => (*a.borrow(), *b.borrow()),
                _ => return EventResponses::default(),
            };
            let (x1, y1) = match (&crop_x1, &crop_y1) {
                (Some(a), Some(b)) => (*a.borrow(), *b.borrow()),
                _ => return EventResponses::default(),
            };
            if let Some(cropped) = crop_image(&img, x0, y0, x1, y1) {
                let preview_path = make_preview_path(&save_path.borrow());
                if let Err(e) = cropped.save(&preview_path) {
                    log_err!("Failed to save crop preview: {}", e);
                }
                // Store the uncropped preview for mirror reapplication
                *preview_img.borrow_mut() = Some(cropped.clone());
                // Apply current mirror setting to the preview
                let display_img = if *mirror.borrow() {
                    let rgb = cropped.to_rgb8();
                    DynamicImage::ImageRgb8(image::imageops::flip_horizontal(&rgb))
                } else {
                    cropped
                };
                *dyn_img.borrow_mut() = display_img.clone();
                image_viewer.update_image(display_img);
                *preview_active.borrow_mut() = true;
            }
            EventResponses::default()
        }));
        self.pane.add_element(Box::new(preview_btn.at(DynVal::new_fixed(60), row_y.clone())));

        // Crop button — crop from src_img, then make it the new src_img
        let src_img = self.src_img.clone();
        let dyn_img = self.dyn_img.clone();
        let save_path = self.save_path.clone();
        let image_viewer = self.image_viewer.clone();
        let crop_x0 = self.crop_x0.clone();
        let crop_y0 = self.crop_y0.clone();
        let crop_x1 = self.crop_x1.clone();
        let crop_y1 = self.crop_y1.clone();
        let crop_x0_tb = self.crop_x0_tb.clone();
        let crop_y0_tb = self.crop_y0_tb.clone();
        let crop_x1_tb = self.crop_x1_tb.clone();
        let crop_y1_tb = self.crop_y1_tb.clone();
        let mirror = self.mirror.clone();
        let preview_active = self.preview_active.clone();
        let preview_img = self.preview_img.clone();
        let crop_btn = Button::new(&self.ctx, "crop");
        crop_btn.set_fn(Box::new(move |_, _| {
            // Sync textbox values before reading
            sync_crop_values(&crop_x0_tb, &crop_y0_tb, &crop_x1_tb, &crop_y1_tb);

            let img = src_img.borrow().clone();
            let (x0, y0) = match (&crop_x0, &crop_y0) {
                (Some(a), Some(b)) => (*a.borrow(), *b.borrow()),
                _ => return EventResponses::default(),
            };
            let (x1, y1) = match (&crop_x1, &crop_y1) {
                (Some(a), Some(b)) => (*a.borrow(), *b.borrow()),
                _ => return EventResponses::default(),
            };
            if let Some(cropped) = crop_image(&img, x0, y0, x1, y1) {
                // Save cropped image to original path
                if let Err(e) = cropped.save(save_path.borrow().as_str()) {
                    log_err!("Failed to save cropped image: {}", e);
                }
                // Delete preview file
                let preview_path = make_preview_path(&save_path.borrow());
                let _ = std::fs::remove_file(&preview_path);
                // Update src_img to the cropped image
                let (cw, ch) = cropped.dimensions();
                *src_img.borrow_mut() = cropped.clone();
                // Re-derive display image from new src_img with current mirror setting
                let rgb = cropped.into_rgb8();
                let final_img = if *mirror.borrow() {
                    DynamicImage::ImageRgb8(image::imageops::flip_horizontal(&rgb))
                } else {
                    DynamicImage::ImageRgb8(rgb)
                };
                *dyn_img.borrow_mut() = final_img.clone();
                image_viewer.update_image(final_img);
                // Clear preview state
                *preview_active.borrow_mut() = false;
                *preview_img.borrow_mut() = None;
                // Reset crop values and max bounds to current image dimensions
                if (cw, ch) != (0, 0) {
                    if let Some(tb) = &crop_x0_tb { tb.max_value.borrow_mut().replace(cw as i32); tb.set_value(0); }
                    if let Some(tb) = &crop_y0_tb { tb.max_value.borrow_mut().replace(ch as i32); tb.set_value(0); }
                    if let Some(tb) = &crop_x1_tb { tb.max_value.borrow_mut().replace(cw as i32); tb.set_value(cw as i32); }
                    if let Some(tb) = &crop_y1_tb { tb.max_value.borrow_mut().replace(ch as i32); tb.set_value(ch as i32); }
                }
            }
            EventResponses::default()
        }));
        self.pane.add_element(Box::new(crop_btn.at(DynVal::new_fixed(70), row_y.clone())));

        // Reset button — reset crop coords to full-image bounds and clear preview
        let src_img = self.src_img.clone();
        let dyn_img = self.dyn_img.clone();
        let image_viewer = self.image_viewer.clone();
        let mirror = self.mirror.clone();
        let preview_active = self.preview_active.clone();
        let preview_img = self.preview_img.clone();
        let crop_x0_tb = self.crop_x0_tb.clone();
        let crop_y0_tb = self.crop_y0_tb.clone();
        let crop_x1_tb = self.crop_x1_tb.clone();
        let crop_y1_tb = self.crop_y1_tb.clone();
        let reset_btn = Button::new(&self.ctx, "reset");
        reset_btn.set_fn(Box::new(move |_, _| {
            let img = src_img.borrow();
            let (w, h) = img.dimensions();
            drop(img);
            // Reset crop values to full image bounds
            if let Some(tb) = &crop_x0_tb { tb.max_value.borrow_mut().replace(w as i32); tb.set_value(0); }
            if let Some(tb) = &crop_y0_tb { tb.max_value.borrow_mut().replace(h as i32); tb.set_value(0); }
            if let Some(tb) = &crop_x1_tb { tb.max_value.borrow_mut().replace(w as i32); tb.set_value(w as i32); }
            if let Some(tb) = &crop_y1_tb { tb.max_value.borrow_mut().replace(h as i32); tb.set_value(h as i32); }
            // Clear preview state and restore src_img as display
            *preview_active.borrow_mut() = false;
            *preview_img.borrow_mut() = None;
            let src = src_img.borrow().clone();
            let rgb = src.into_rgb8();
            let final_img = if *mirror.borrow() {
                DynamicImage::ImageRgb8(image::imageops::flip_horizontal(&rgb))
            } else {
                DynamicImage::ImageRgb8(rgb)
            };
            *dyn_img.borrow_mut() = final_img.clone();
            image_viewer.update_image(final_img);
            EventResponses::default()
        }));
        self.pane.add_element(Box::new(reset_btn.at(DynVal::new_fixed(76), row_y)));
    }
}

/// Build the preview file path by stripping the extension and appending _crop_preview.jpg.
fn make_preview_path(save_path: &str) -> String {
    let stem = std::path::Path::new(save_path)
        .file_stem()
        .unwrap_or_else(|| std::ffi::OsStr::new("photobooth"))
        .to_string_lossy()
        .to_string();
    let parent = std::path::Path::new(save_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    if parent.is_empty() {
        format!("{}_crop_preview.jpg", stem)
    } else {
        format!("{}/{}_crop_preview.jpg", parent, stem)
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
                let mut captured = false;
                if let Some(cam) = self.camera.borrow_mut().as_mut() {
                    if let Ok(buffer) = cam.frame() {
                        if let Ok(img) = buffer_to_dynamic_image(&buffer) {
                            let (w, h) = img.dimensions();

                            // Update crop bounds on first frame
                            if *self.first_frame.borrow() {
                                *self.first_frame.borrow_mut() = false;
                                if let Some(rc) = &self.crop_x1 { *rc.borrow_mut() = w as i32; }
                                if let Some(rc) = &self.crop_y1 { *rc.borrow_mut() = h as i32; }
                                // Set max bounds and initial values on all crop textboxes
                                if let Some(tb) = &self.crop_x0_tb { tb.max_value.borrow_mut().replace(w as i32); tb.set_value(w as i32); }
                                if let Some(tb) = &self.crop_y0_tb { tb.max_value.borrow_mut().replace(h as i32); tb.set_value(h as i32); }
                                if let Some(tb) = &self.crop_x1_tb { tb.max_value.borrow_mut().replace(w as i32); tb.set_value(w as i32); }
                                if let Some(tb) = &self.crop_y1_tb { tb.max_value.borrow_mut().replace(h as i32); tb.set_value(h as i32); }
                            }

                            // Clear any active crop preview when a new frame arrives
                            *self.preview_active.borrow_mut() = false;
                            *self.preview_img.borrow_mut() = None;

                            // Store uncropped source image
                            *self.src_img.borrow_mut() = img.clone();

                            // Derive display image from src_img with current mirror setting
                            let final_img = if *self.mirror.borrow() {
                                let rgb = img.to_rgb8();
                                DynamicImage::ImageRgb8(image::imageops::flip_horizontal(&rgb))
                            } else {
                                img
                            };

                            *self.dyn_img.borrow_mut() = final_img.clone();
                            self.image_viewer.update_image(final_img);
                            captured = true;
                        }
                    }
                }
                // Only advance timestamp on success; retry next cycle on failure
                if captured {
                    *self.last_capture.borrow_mut() = Some(now);
                }
            }
        }
        self.pane.drawing(ctx, dr, force_update)
    }
}
