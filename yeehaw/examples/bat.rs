use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // uncomment the following line to enable logging
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let main_el = ParentPane::new(&ctx, "main_element").with_bg(Color::MIDNIGHT_BLUE);

    let hstack = HorizontalStack::new(&ctx)
        .with_min_resize_width(2)
        .with_dyn_height(DynVal::FULL);

    let sty = Style::default()
        .with_fg(Color::WHITE)
        .with_bg(Color::MIDNIGHT_BLUE);

    let el = Box::new(
        Bordered::new_resizer(
            &ctx,
            Box::new(DebugSizePane::new(&ctx).with_style(sty)),
            Style::default(),
        )
        .with_dyn_width(hstack.avg_width()),
    );
    hstack.push(el);

    // place the button 1 character below the label
    let x = DynVal::new_flex(0.1); // 30% screen width
    let y = DynVal::new_flex(0.1).plus(1.into()); // 30% screen height + 1 ch

    let bz = std::include_str!("../src/tui.rs").as_bytes();

    let bat_viewer = BatViewer::new(&ctx, 20, 20)
        .with_input_from_bytes(bz)?
        .at(x, y);

    hstack.push(Box::new(bat_viewer));

    main_el.add_element(Box::new(hstack));
    tui.run(Box::new(main_el)).await
}
