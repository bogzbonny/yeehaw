use yeehaw_tui::*;

pub fn editor_demo(ctx: &Context) -> Box<dyn Element> {
    let el = ParentPane::new(ctx, "main").with_style(Style::default().with_bg(Color::GREY5));

    let ed_height = DynVal::FULL.minus(4.into()).with_min(10).with_max(30);
    let ed_width = DynVal::HALF.minus(2.into()).with_min(20).with_max(70);

    // for testing for when no editor is found
    /*
    let editor = TermEditorPane::new_with_custom_editor(
        &ctx,
        "custom",
        Some("bloop".into()),
        "editor not found",
    )
    */
    let editor = TermEditorPane::new(ctx, "custom")
        .with_dyn_height(ed_height)
        .with_dyn_width(ed_width)
        .at(1, 4);

    let x = DynVal::x_after(&editor, 2);
    let output = Label::new(ctx, "nothing yet set in $EDITOR textbox").at(x, 1);

    let output_ = output.clone();
    let hook = Box::new(move |_, text: String| {
        output_.set_text(format!("text set to:\n{text}"));
        EventResponses::default()
    });

    editor.set_text_changed_hook(hook);

    el.add_element(Box::new(editor.label(
        ctx,
        "You can use any editor set in the\n\
        $EDITOR environment variable as a\n\
        yeehaw element",
    )));
    el.add_element(Box::new(editor));
    el.add_element(Box::new(output));

    Box::new(el)
}
