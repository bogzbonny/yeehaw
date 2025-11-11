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
            pane: PaneScrollable::new_expanding(ctx, width, height),
        }
    }

    pub fn with_input_from_file(self, path: impl AsRef<Path>) -> Result<Self, Error> {
        self.set_input_from_file(path)?;
        Ok(self)
    }

    pub fn with_input_from_bytes(self, bz: &[u8]) -> Result<Self, Error> {
        self.set_input_from_bytes(bz)?;
        Ok(self)
    }

    pub fn set_input_from_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let mut pp = PrettyPrinter::new();
        pp.input_file(path);
        pp.term_width(self.pane.content_width());
        let mut buf = String::new(); // Create a buffer of 10 bytes
        let _ = pp.print_with_writer(Some(&mut buf)).map_err(Box::new)?;
        let d = ansi::get_chs_2d(buf.as_bytes(), Style::standard());
        self.pane.set_content_width(d.width());
        self.pane.set_content_height(d.height());
        self.pane.set_content(d);
        Ok(())
    }

    pub fn set_input_from_bytes(&self, bz: &[u8]) -> Result<(), Error> {
        let mut pp = PrettyPrinter::new();
        pp.input_from_bytes(bz);
        pp.language("rs"); // XXX
        pp.term_width(self.pane.content_width());
        let mut buf = String::new(); // Create a buffer of 10 bytes
        let _ = pp.print_with_writer(Some(&mut buf)).map_err(Box::new)?;
        let d = ansi::get_chs_2d(buf.as_bytes(), Style::standard());
        debug!("d.width: {}", d.width());
        debug!("d.height: {}", d.height());
        self.pane.set_content_width(d.width());
        self.pane.set_content_height(d.height());
        self.pane.set_content(d);
        Ok(())
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for BatViewer {}
