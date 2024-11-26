use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::set_log_file("./debug_test.log".to_string());
    yeehaw::log::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let el = ParentPane::new(&ctx, "main").with_style(Style::default().with_bg(Color::GREY5));

    //let editor = TermEditorPane::new(&ctx, "EDITOR")
    let editor = TermEditorPane::new_with_custom_editor(&ctx, "EDITOR", None)
        .with_height(30.into())
        .with_width(30.into())
        .at(1.into(), 1.into());

    let label = Label::new(&ctx, "nothing yet set in $EDITOR textbox").at(50.into(), 1.into());

    let label_ = label.clone();
    let hook = Box::new(move |_ctx, text: String| {
        label_.set_text(format!("text set to:\n {text}"));
        EventResponses::default()
    });

    editor.set_text_changed_hook(hook);

    el.add_element(Box::new(editor));
    el.add_element(Box::new(label));
    tui.run(Box::new(el)).await
}
