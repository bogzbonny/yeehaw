use {crate::*, std::path::PathBuf};

// displays the size
#[derive(Clone)]
pub struct FileViewerPane {
    pub pane: ParentPane,
}

impl FileViewerPane {
    const KIND: &'static str = "file_viewer_pane";

    pub fn new(ctx: &Context, file_path: PathBuf) -> FileViewerPane {
        let content = std::fs::read_to_string(file_path);
        let mut no_file_text = None;
        let content = match content {
            Ok(c) => c,
            Err(e) => {
                no_file_text = Some(format!("Error reading file: {}", e));
                String::new()
            }
        };

        let pane = ParentPane::new(ctx, Self::KIND);
        let tb = TextBox::new(ctx, content)
            .with_width(DynVal::FULL)
            .with_height(DynVal::FULL)
            .with_right_scrollbar(ctx)
            .with_bottom_scrollbar(ctx)
            .editable(ctx)
            .with_no_wordwrap(ctx)
            .at(DynVal::new_fixed(0), DynVal::new_fixed(0));
        if let Some(no_file_text) = no_file_text {
            tb.set_text_when_empty(no_file_text);
        }
        pane.add_element(Box::new(tb));

        FileViewerPane { pane }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for FileViewerPane {}
