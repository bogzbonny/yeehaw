use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // uncomment the following line to enable logging
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let main_el = ParentPane::new(&ctx, "main_element").with_bg(Color::MIDNIGHT_BLUE);

    // place the button 1 character below the label
    let x = DynVal::new_flex(0.3); // 30% screen width
    let y = DynVal::new_flex(0.3).plus(1.into()); // 30% screen height + 1 ch

    let entries = vec![
        "entry 1".to_string(),
        "entry 2".to_string(),
        "entry 3".to_string(),
    ];

    let lc = ListControl::new(&ctx, entries)
        .with_dyn_width(DynVal::new_fixed(30))
        .with_dyn_height(DynVal::new_fixed(5))
        .with_new_entry_tb(&ctx, "new entry...")
        .with_right_click_menu(&ctx)
        .with_scrollbar(&ctx)
        .with_deleting_allowed()
        .with_shifting_allowed()
        .with_duplicating_allowed()
        .with_renaming_allowed()
        .at(x, y);

    main_el.add_element(Box::new(lc));
    tui.run(Box::new(main_el)).await
}
