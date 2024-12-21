use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let el = ParentPane::new(&ctx, "main").with_style(Style::default().with_bg(Color::GREY5));

    //let editor = TermEditorPane::new_with_custom_editor(
    //    &ctx,
    //    "custom",
    //    Some("bloop".into()),
    //    "editor not found",
    //) // for testing no editor
    let editor = TermEditorPane::new(&ctx, "custom")
        .with_height(0.7.into())
        .with_width(30.into())
        .at(1, 1);

    let label = Label::new(&ctx, "nothing yet set in $EDITOR textbox").at(50, 1);

    let label_ = label.clone();
    let hook = Box::new(move |_ctx, text: String| {
        label_.set_text(format!("text set to:\n{text}"));
        EventResponses::default()
    });

    editor.set_text_changed_hook(hook);

    let _ = el.add_element(Box::new(editor));
    let _ = el.add_element(Box::new(label));
    tui.run(Box::new(el)).await
}
