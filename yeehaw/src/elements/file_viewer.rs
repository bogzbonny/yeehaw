use {
    crate::{
        widgets::TextBox, Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Parent, Priority, ReceivableEventChanges, SelfReceivableEvents, WidgetPane,
    },
    std::path::PathBuf,
    std::{cell::RefCell, rc::Rc},
};

// displays the size
#[derive(Clone)]
pub struct FileViewerPane {
    pub pane: WidgetPane,
}

impl FileViewerPane {
    pub fn new(ctx: &Context, file_path: PathBuf) -> FileViewerPane {
        let content = std::fs::read_to_string(file_path).unwrap();

        let pane = WidgetPane::new(ctx);
        let tb = TextBox::new(ctx, content)
            .with_width(DynVal::new_flex(1.))
            .with_height(DynVal::new_flex(1.))
            .with_right_scrollbar()
            .with_lower_scrollbar()
            .editable()
            .with_no_wordwrap()
            .at(DynVal::new_fixed(0), DynVal::new_fixed(0))
            .to_widgets(ctx);
        pane.add_widgets(tb);

        FileViewerPane { pane }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for FileViewerPane {}
