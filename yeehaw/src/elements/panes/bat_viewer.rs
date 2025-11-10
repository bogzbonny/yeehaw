use {
    crate::*,
    bat::PrettyPrinter,
    //std::fmt::Write, std::io::Cursor,
    std::path::Path,
};

#[derive(Clone)]
pub struct BatViewer {
    pane: PaneScrollable,
}

impl BatViewer {
    pub fn new(ctx: &Context, width: usize, height: usize) -> Self {
        Self {
            pane: PaneScrollable::new(ctx, width, height),
        }
    }

    pub fn with_file_preview(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        self.set_file_preview(path)?;
        Ok(self)
    }

    pub fn set_file_preview(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let mut pp = PrettyPrinter::new();
        pp.input_file(path);
        pp.term_width(self.pane.content_width());
        let mut buf = String::new(); // Create a buffer of 10 bytes
        let _ = pp.print_with_writer(Some(&mut buf)).map_err(Box::new)?;
        let _ = buf;
        Ok(())
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for BatViewer {}
