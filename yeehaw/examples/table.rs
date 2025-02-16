use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let main = ParentPane::new(&ctx, "main");

    let table = Table::new(&ctx)
        //.with_fixed_row_height(1)
        //.with_fixed_column_width(20)
        .with_border(Some(BoxSideAttr::Thin))
        .with_vertical_lines(Some(BoxSideAttr::Thick))
        .with_header_line(Some(BoxSideAttr::Double));
    //.with_horizontal_lines(Some(BoxSideAttr::Thin));

    table.set_header(&ctx, vec!["ID", "Name", "Status", "Actions"]);
    table.set_data(
        &ctx,
        vec![
            vec!["1", "Alice Thompson", "✓ Active\ntest", "Edit"],
            vec!["2", "Bob Wilson", "⚠ Pending", "Edit"],
            vec!["3", "Charlie Brown", "✗ Inactive", "Edit"],
        ],
    );
    table.clear_data();
    table.set_data(
        &ctx,
        vec![
            vec!["1", "Alice Thompson", "✓ Active\ntest", "Edit"],
            vec!["2", "Bob Wilson", "⚠ Pending", "Edit"],
            vec!["3", "Charlie Brown", "✗ Inactive", "Edit"],
        ],
    );

    table.highlight_row(
        0,
        Style::default()
            .with_bg(Color::YELLOW)
            .with_fg(Color::BLACK),
    );
    table.clear_highlights();

    table.pane.set_at(0.into(), 3.into());
    let limiter = PaneLimiter::new(Box::new(table.clone()), 100, 15);
    main.add_element(Box::new(limiter));

    let ctx_ = ctx.clone();
    let table_ = table.clone();
    let counter = Rc::new(RefCell::new(4));
    let add_row_btn = Button::new(&ctx, "add_row")
        .with_fn(Box::new(move |_, _| {
            let count = *counter.borrow();
            let c = count.to_string();
            table_.push_row(&ctx_, vec![&c, "Alice Thompson", "✓ Active", "Edit"]);
            counter.replace(count + 1);
            EventResponses::default()
        }))
        .at(1, 0);
    main.add_element(Box::new(add_row_btn));

    let table_ = table.clone();
    let remove_row_btn = Button::new(&ctx, "remove_row")
        .with_fn(Box::new(move |_, _| {
            table_.remove_row(0);
            EventResponses::default()
        }))
        .at(12, 0);
    main.add_element(Box::new(remove_row_btn));

    tui.run(Box::new(main)).await
}
