use {crate::*, std::path::PathBuf};

// displays the size
#[derive(Clone)]
pub struct FileViewerPane {
    pub pane: ParentPane,
}

impl FileViewerPane {
    const KIND: &'static str = "file_viewer_pane";

    pub fn new(ctx: &Context, file_path: PathBuf) -> FileViewerPane {
        let _content = std::fs::read_to_string(file_path).unwrap();

        let pane = ParentPane::new(ctx, Self::KIND);

        // XXX TODO uncomment/fix post widger recall
        //let tb = TextBox::new(ctx, content)
        //    .with_width(DynVal::full())
        //    .with_height(DynVal::full())
        //    .with_right_scrollbar()
        //    .with_lower_scrollbar()
        //    .editable()
        //    .with_no_wordwrap()
        //    .at(DynVal::new_fixed(0), DynVal::new_fixed(0))
        //    .to_widgets(ctx);
        //pane.add_widgets(tb);

        FileViewerPane { pane }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for FileViewerPane {}
