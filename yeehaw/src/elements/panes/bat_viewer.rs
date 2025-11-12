use {
    crate::*,
    bat::{
        self, PagingMode, PrettyPrinter, StripAnsiMode, SyntaxMapping, WrappingMode,
        line_range::LineRanges,
    },
    std::cell::RefCell,
    std::path::Path,
    std::rc::Rc,
};

#[derive(Clone)]
pub struct BatViewer {
    pane: PaneScrollable,
    // Store options mirroring PrettyPrinter; built on demand
    opts: Rc<RefCell<BatOptions>>,
}

#[derive(Default)]
struct BatOptions {
    language: Option<&'static str>,
    tab_width: Option<usize>,
    colored_output: bool,
    true_color: bool,
    header: bool,
    line_numbers: bool,
    grid: bool,
    rule: bool,
    vcs_modification_markers: bool,
    show_nonprintable: bool,
    snip: bool,
    strip_ansi: StripAnsiMode,
    wrapping_mode: WrappingMode,
    use_italics: bool,
    pager: Option<&'static str>,
    paging_mode: Option<PagingMode>,
    line_ranges: Option<LineRanges>,
    highlighted: Vec<(usize, usize)>,
    squeeze_empty_lines: Option<usize>,
    theme: Option<String>,
    syntax_mapping: Option<SyntaxMapping<'static>>,
}

impl BatViewer {
    pub fn new(ctx: &Context, width: usize, height: usize) -> Self {
        Self {
            pane: PaneScrollable::new_expanding(ctx, width, height),
            opts: Rc::new(RefCell::new(BatOptions {
                colored_output: true,
                true_color: true,
                ..Default::default()
            })),
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

    /// Sets the input for the pane from a file. NOTE this call should be made after setting all
    /// settings
    pub fn set_input_from_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let mut pp = self.build_pretty_printer();
        //pp.term_width(self.pane.content_width()); // NOTE uncommenting will screw up line numbers
        pp.input_file(path);
        let mut buf = String::new(); // Create a buffer of 10 bytes
        let _ = pp.print_with_writer(Some(&mut buf)).map_err(Box::new)?;
        let d = ansi::get_chs_2d(buf.as_bytes(), Style::standard());
        self.pane.set_content_width(d.width());
        self.pane.set_content_height(d.height());
        self.pane.set_content(d);
        Ok(())
    }

    /// Sets the input for the pane from bytes. NOTE this call should be made after setting all
    /// settings
    pub fn set_input_from_bytes(&self, bz: &[u8]) -> Result<(), Error> {
        let mut pp = self.build_pretty_printer();
        //pp.term_width(self.pane.content_width()); // NOTE uncommenting will screw up line numbers
        pp.input_from_bytes(bz);
        let mut buf = String::new(); // Create a buffer of 10 bytes
        let _ = pp.print_with_writer(Some(&mut buf)).map_err(Box::new)?;
        let d = ansi::get_chs_2d(buf.as_bytes(), Style::standard());
        self.pane.set_content_width(d.width());
        self.pane.set_content_height(d.height());
        self.pane.set_content(d);
        Ok(())
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    // ------------------------------------
    // PaneScrollable configuration helpers
    pub fn set_expand_to_fill_width(&self, val: bool) -> &Self {
        *self.pane.expand_to_fill_width.borrow_mut() = val;
        self
    }
    pub fn with_expand_to_fill_width(self, val: bool) -> Self {
        self.set_expand_to_fill_width(val);
        self
    }

    pub fn set_expand_to_fill_height(&self, val: bool) -> &Self {
        *self.pane.expand_to_fill_height.borrow_mut() = val;
        self
    }
    pub fn with_expand_to_fill_height(self, val: bool) -> Self {
        self.set_expand_to_fill_height(val);
        self
    }

    pub fn set_scroll_rate(&self, rate: Option<i16>) -> &Self {
        *self.pane.scroll_rate.borrow_mut() = rate;
        self
    }
    pub fn with_scroll_rate(self, rate: Option<i16>) -> Self {
        self.set_scroll_rate(rate);
        self
    }

    // Build a PrettyPrinter from the stored options.
    fn build_pretty_printer(&self) -> PrettyPrinter<'_> {
        let mut pp = PrettyPrinter::new();
        let opts = self.opts.borrow();
        if let Some(lang) = opts.language {
            pp.language(lang);
        }
        //pp.tab_width(opts.tab_width);
        //pp.colored_output(opts.colored_output);
        //pp.true_color(opts.true_color);
        //pp.header(opts.header);
        pp.line_numbers(opts.line_numbers);
        //pp.grid(opts.grid);
        //pp.rule(opts.rule);
        //pp.vcs_modification_markers(opts.vcs_modification_markers);
        //pp.show_nonprintable(opts.show_nonprintable);
        //pp.snip(opts.snip);
        //pp.strip_ansi(opts.strip_ansi);
        //pp.wrapping_mode(opts.wrapping_mode);
        //pp.use_italics(opts.use_italics);
        //if let Some(pager) = opts.pager {
        //    pp.pager(pager);
        //}
        //if let Some(mode) = opts.paging_mode {
        //    pp.paging_mode(mode);
        //}
        //if let Some(ref ranges) = opts.line_ranges {
        //    pp.line_ranges(ranges.clone());
        //}
        //for &(from, to) in &opts.highlighted {
        //    pp.highlight_range(from, to);
        //}
        //pp.squeeze_empty_lines(opts.squeeze_empty_lines);
        //if let Some(ref theme) = opts.theme {
        //    pp.theme(theme);
        //}
        //if let Some(ref mapping) = opts.syntax_mapping {
        //    pp.syntax_mapping(mapping.clone());
        //}
        pp
    }

    // ---------------------------------------------
    // PrettyPrinter configuration helpers

    pub fn set_language(&self, language: &'static str) -> &Self {
        self.opts.borrow_mut().language = Some(language);
        self
    }
    pub fn with_language(self, language: &'static str) -> Self {
        self.set_language(language);
        self
    }

    pub fn set_tab_width(&self, tab_width: Option<usize>) -> &Self {
        self.opts.borrow_mut().tab_width = tab_width;
        self
    }
    pub fn with_tab_width(self, tab_width: Option<usize>) -> Self {
        self.set_tab_width(tab_width);
        self
    }

    pub fn set_colored_output(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().colored_output = yes;
        self
    }
    pub fn with_colored_output(self) -> Self {
        self.set_colored_output(true);
        self
    }

    pub fn set_true_color(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().true_color = yes;
        self
    }
    pub fn with_true_color(self) -> Self {
        self.set_true_color(true);
        self
    }

    // NOTE header option seems to stall the TUI in bat
    pub fn set_header(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().header = yes;
        self
    }
    // NOTE header option seems to stall TUI in bat
    pub fn with_header(self) -> Self {
        self.set_header(true);
        self
    }

    pub fn set_line_numbers(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().line_numbers = yes;
        self
    }
    pub fn with_line_numbers(self) -> Self {
        self.set_line_numbers(true);
        self
    }

    pub fn set_grid(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().grid = yes;
        self
    }
    pub fn with_grid(self) -> Self {
        self.set_grid(true);
        self
    }

    pub fn set_rule(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().rule = yes;
        self
    }
    pub fn with_rule(self) -> Self {
        self.set_rule(true);
        self
    }

    pub fn set_vcs_modification_markers(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().vcs_modification_markers = yes;
        self
    }
    pub fn with_vcs_modification_markers(self) -> Self {
        self.set_vcs_modification_markers(true);
        self
    }

    pub fn set_show_nonprintable(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().show_nonprintable = yes;
        self
    }
    pub fn with_show_nonprintable(self) -> Self {
        self.set_show_nonprintable(true);
        self
    }

    pub fn set_snip(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().snip = yes;
        self
    }
    pub fn with_snip(self) -> Self {
        self.set_snip(true);
        self
    }

    pub fn set_strip_ansi(&self, mode: bat::StripAnsiMode) -> &Self {
        self.opts.borrow_mut().strip_ansi = mode;
        self
    }
    pub fn with_strip_ansi(self, mode: bat::StripAnsiMode) -> Self {
        self.set_strip_ansi(mode);
        self
    }

    pub fn set_wrapping_mode(&self, mode: bat::WrappingMode) -> &Self {
        self.opts.borrow_mut().wrapping_mode = mode;
        self
    }
    pub fn with_wrapping_mode(self, mode: bat::WrappingMode) -> Self {
        self.set_wrapping_mode(mode);
        self
    }

    pub fn set_use_italics(&self, yes: bool) -> &Self {
        self.opts.borrow_mut().use_italics = yes;
        self
    }
    pub fn with_use_italics(self) -> Self {
        self.set_use_italics(true);
        self
    }

    pub fn set_pager(&self, pager: &'static str) -> &Self {
        self.opts.borrow_mut().pager = Some(pager);
        self
    }
    pub fn with_pager(self, pager: &'static str) -> Self {
        self.set_pager(pager);
        self
    }

    pub fn set_paging_mode(&self, mode: bat::PagingMode) -> &Self {
        self.opts.borrow_mut().paging_mode = Some(mode);
        self
    }
    pub fn with_paging_mode(self, mode: bat::PagingMode) -> Self {
        self.set_paging_mode(mode);
        self
    }

    pub fn set_line_ranges(&self, ranges: LineRanges) -> &Self {
        self.opts.borrow_mut().line_ranges = Some(ranges);
        self
    }
    pub fn with_line_ranges(self, ranges: LineRanges) -> Self {
        self.set_line_ranges(ranges);
        self
    }

    pub fn highlight(&self, line: usize) -> &Self {
        self.opts.borrow_mut().highlighted.push((line, line));
        self
    }
    pub fn with_highlight(self, line: usize) -> Self {
        self.highlight(line);
        self
    }

    pub fn highlight_range(&self, from: usize, to: usize) -> &Self {
        self.opts.borrow_mut().highlighted.push((from, to));
        self
    }
    pub fn with_highlight_range(self, from: usize, to: usize) -> Self {
        self.highlight_range(from, to);
        self
    }

    pub fn squeeze_empty_lines(&self, maximum: Option<usize>) -> &Self {
        self.opts.borrow_mut().squeeze_empty_lines = maximum;
        self
    }
    pub fn with_squeeze_empty_lines(self, maximum: Option<usize>) -> Self {
        self.squeeze_empty_lines(maximum);
        self
    }

    pub fn set_theme(&self, theme: impl AsRef<str>) -> &Self {
        self.opts.borrow_mut().theme = Some(theme.as_ref().to_string());
        self
    }
    pub fn with_theme(self, theme: impl AsRef<str>) -> Self {
        self.set_theme(theme);
        self
    }

    pub fn set_syntax_mapping(&self, mapping: bat::SyntaxMapping<'static>) -> &Self {
        self.opts.borrow_mut().syntax_mapping = Some(mapping);
        self
    }
    pub fn with_syntax_mapping(self, mapping: bat::SyntaxMapping<'static>) -> Self {
        self.set_syntax_mapping(mapping);
        self
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for BatViewer {}
