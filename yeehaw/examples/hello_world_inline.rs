use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::log::reset_log_file("./debug_test.log".to_string());
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let main_el = ParentPane::new(&ctx, "main_element").with_bg(Color::GREY10);

    // place the label at 30% of the screen width and height
    let label = Label::new(&ctx, "Hello, World!").at(0.3, 0.3);

    // place the button 1 character below the label
    let x = DynVal::new_flex(0.3); // 30% screen width
    let y = DynVal::new_flex(0.3).plus(1.into()); // 30% screen height + 1 ch

    let label_ = label.clone(); // clone for closure move
    let button = Button::new(&ctx, "Click Here!")
        .with_fn(Box::new(move |_, _| {
            label_.set_text("Button clicked!".to_string());
            EventResponses::default()
        }))
        .at(x, y);

    main_el.add_element(Box::new(label));
    main_el.add_element(Box::new(button));
    tui.run_in_line(Box::new(main_el), 10).await
}
