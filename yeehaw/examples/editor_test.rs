use yeehaw::{Color, Cui, Error, EventResponses, ParentPane, Style, TermEditorPane};

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::debug::set_log_file("./debug_test.log".to_string());
    yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut cui, ctx) = Cui::new()?;

    let el = ParentPane::new(&ctx, "main").with_style(Style::default().with_bg(Color::GREY5));

    //let editor = TermEditorPane::new(&ctx, "EDITOR")
    let editor = TermEditorPane::new_with_custom_editor(&ctx, "EDITOR", None)
        .with_height(30.into())
        .with_width(30.into())
        .at(1.into(), 1.into());

    let label = yeehaw::widgets::Label::new(&ctx, "nothing yet set in $EDITOR textbox")
        .at(50.into(), 1.into());

    let label_ = label.clone();
    let hook = Box::new(move |ctx_, text: String| {
        label_.set_text(&ctx_, format!("text set to:\n {text}"));
        EventResponses::default()
    });

    editor.set_text_changed_hook(hook);

    el.add_element(Box::new(editor));
    el.add_element(Box::new(label));
    cui.run(Box::new(el)).await
}