use yeehaw::*;

pub fn editor_demo(ctx: &Context) -> Box<dyn Element> {
    let el = ParentPane::new(ctx, "main").with_style(Style::default().with_bg(Color::GREY5));

    //// for testing for when no editor is found
    //let editor = TermEditorPane::new_with_custom_editor(
    //    &ctx,
    //    "custom",
    //    Some("bloop".into()),
    //    "editor not found",
    //)

    let editor = TermEditorPane::new(ctx, "custom")
        .with_height(0.7.into())
        .with_width(30.into())
        .at(1, 1);

    let label = Label::new(ctx, "nothing yet set in $EDITOR textbox").at(50, 1);

    let label_ = label.clone();
    let hook = Box::new(move |_, text: String| {
        label_.set_text(format!("text set to:\n{text}"));
        EventResponses::default()
    });

    editor.set_text_changed_hook(hook);

    el.add_element(Box::new(editor));
    el.add_element(Box::new(label));

    Box::new(el)
}
