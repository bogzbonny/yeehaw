use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    let (mut tui, ctx) = Tui::new()?;
    let main_el = ParentPane::new(&ctx, "main_element");

    // place the label at 30% of the screen width and height
    let label = Label::new(&ctx, "Hello, World!").at(0.3, 0.3);

    let label_ = label.clone(); // clone required so we can move it into the button closure
    let button = Button::new(
        &ctx,
        "Click Here!",
        Box::new(move |_, _| {
            label_.set_text("Button clicked!".to_string());
            EventResponses::default()
        }),
    )
    // place the button at 30% of the screen width and 30% of the screen height + 1 character
    .at(0.3, DynVal::new_flex(0.3).plus(1.into()));

    main_el.add_element(Box::new(label));
    main_el.add_element(Box::new(button));
    tui.run(Box::new(main_el)).await
}
