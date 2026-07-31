use {
    crate::elements::panes::BatViewer,
    crate::elements::widgets::{AudioPlayer, ImageViewer},
    crate::*,
    std::cell::RefCell,
    std::path::{Path, PathBuf},
    std::rc::Rc,
};

/// Media type detected from file extension.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MediaType {
    Audio,
    Image,
    Text,
}

/// Detect media type from file extension.
fn detect_media_type(path: &Path) -> MediaType {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "mp3" | "ogg" | "flac" | "wav" | "m4a" => MediaType::Audio,
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => MediaType::Image,
        _ => MediaType::Text,
    }
}

/// MultiMediaPreview element that displays audio, image, or code files
/// depending on the loaded file's extension. Switches inner element live
/// when a new file is set.
///
/// File type detection by extension:
/// - Audio (.mp3, .ogg, .flac, .wav, .m4a) → AudioPlayer
/// - Image (.jpg, .jpeg, .png, .gif, .bmp, .webp) → ImageViewer
/// - Everything else → BatViewer
#[derive(Clone)]
pub struct MultiMediaPreview {
    pane: ParentPane,
    current_path: Rc<RefCell<Option<PathBuf>>>,
    autoplay: Rc<RefCell<bool>>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl MultiMediaPreview {
    /// Create a new empty MultiMediaPreview.
    pub fn new(ctx: &Context) -> Self {
        let pane = ParentPane::new(ctx, "multimedia_preview");
        pane.set_dyn_width(DynVal::FULL);
        pane.set_dyn_height(DynVal::FULL);
        Self {
            pane,
            current_path: Rc::new(RefCell::new(None)),
            autoplay: Rc::new(RefCell::new(false)),
        }
    }

    /// Builder: set the file to preview.
    pub fn with_file<P: AsRef<Path>>(self, ctx: &Context, path: P) -> Result<Self, Error> {
        self.set_file(ctx, path)?;
        Ok(self)
    }

    /// Set the file to preview. Clears the current inner element and creates
    /// a new one based on the file extension. Can be called at runtime to
    /// switch previews live.
    pub fn set_file<P: AsRef<Path>>(&self, ctx: &Context, path: P) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf();

        // Clear existing child element
        self.pane.clear_elements();

        // Check if file exists; show label if not
        if !path.exists() {
            let label = Label::new(ctx, "File not found");
            label.pane.set_at(DynVal::HALF.minus_fixed(8), DynVal::HALF);
            self.pane.add_element(Box::new(label));
            *self.current_path.borrow_mut() = Some(path);
            return Ok(());
        }

        let media_type = detect_media_type(&path);
        let child: Box<dyn Element> = match media_type {
            MediaType::Audio => {
                let player = AudioPlayer::new(ctx, vec![path.clone()]);
                if *self.autoplay.borrow() {
                    player.set_autoplay(true);
                }
                Box::new(player)
            }
            MediaType::Image => {
                let path_str = path.to_str().ok_or_else(|| Error::YeehawError {
                    message: "File path contains invalid UTF-8".to_string(),
                })?;
                Box::new(ImageViewer::new_from_path(ctx, path_str, Color::BLACK)?)
            }
            MediaType::Text => Box::new(
                BatViewer::new(ctx, 300, 100)
                    .with_line_numbers()
                    .with_input_from_file(&path)?,
            ),
        };

        // Center audio player vertically; fill parent for other types
        let loc_rc = child.get_ref_cell_dyn_location_set();
        let loc = if media_type == MediaType::Audio {
            DynLocation {
                start_x: DynVal::new_fixed(0),
                end_x: DynVal::FULL,
                start_y: DynVal::HALF.minus_fixed(1),
                end_y: DynVal::HALF.plus_fixed(1),
            }
        } else {
            DynLocation::full()
        };
        loc_rc.borrow_mut().l = loc;

        self.pane.add_element(child);
        *self.current_path.borrow_mut() = Some(path);
        Ok(())
    }

    /// Get the currently loaded file path, if any.
    pub fn current_path(&self) -> Option<PathBuf> {
        self.current_path.borrow().clone()
    }

    /// Get the autoplay setting.
    pub fn autoplay(&self) -> bool {
        *self.autoplay.borrow()
    }

    /// Set whether audio should automatically play when an audio file is loaded.
    pub fn set_autoplay(&self, autoplay: bool) {
        *self.autoplay.borrow_mut() = autoplay;
    }

    /// Builder: set whether audio should automatically play when an audio file is loaded.
    pub fn with_autoplay(self, autoplay: bool) -> Self {
        *self.autoplay.borrow_mut() = autoplay;
        self
    }

    /// Set the position of the element.
    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for MultiMediaPreview {}
