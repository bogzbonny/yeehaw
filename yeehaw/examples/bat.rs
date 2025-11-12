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
        .with_dyn_width(hstack.avg_width().div(2.0)),
    );
    hstack.push(el);

    let bz = std::include_str!("../src/tui.rs").as_bytes();

    let bat_viewer = BatViewer::new(&ctx, 300, 100)
        //.with_grid()
        //.with_paging_mode(bat::PagingMode::Never)
        //.with_highlight(10)
        //.with_rule()
        //.with_show_nonprintable()
        //.with_rule()
        .with_line_numbers()
        .with_language("rust")
        .with_input_from_bytes(bz)?;

    hstack.push(Box::new(bat_viewer));

    main_el.add_element(Box::new(hstack));
    tui.run(Box::new(main_el)).await
}
