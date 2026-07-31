use std::path::PathBuf;
use yeehaw::*;

#[tokio::main]
async fn main() {
    let (mut tui, ctx) = Tui::new().expect("Failed to create TUI");

    let textbox = TextBox::new(&ctx, "")
        .with_text_when_empty("media file path (absolute)")
        .with_dyn_width(50.into())
        .with_dyn_height(2.into())
        .editable(&ctx)
        .at(1, 1);

    let multimedia_preview = MultiMediaPreview::new(&ctx)
        .at(DynVal::new_fixed(1), DynVal::new_fixed(4))
        .with_dyn_width(DynVal::new_fixed(50))
        .with_dyn_height(DynVal::new_fixed(20));

    let textbox_ = textbox.clone();
    let multimedia_preview_ = multimedia_preview.clone();

    let load_btn = Button::new(&ctx, "Load")
        .at(DynVal::new_fixed(52), DynVal::new_fixed(1))
        .with_fn(Box::new(move |_, ctx| {
            let path_str = textbox_.get_text();
            if !path_str.is_empty() {
                let path = PathBuf::from(path_str);
                let _ = multimedia_preview_.set_file(&ctx, path);
            }
            EventResponses::default()
        }));

    let root = ParentPane::new(&ctx, "root");
    root.add_element(Box::new(textbox));
    root.add_element(Box::new(multimedia_preview));
    root.add_element(Box::new(load_btn));

    let _ = tui.run(Box::new(root)).await;
}
