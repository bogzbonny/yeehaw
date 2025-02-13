use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

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
            //vec!["1", "Alice Thompson", "01234567890123456789", "Edit"],
            //vec!["2", "Bob Wilson", "Pending", "Edit"],
            //vec!["3", "Charlie Brown", "Inactive", "Edit"],
            vec!["1", "Alice Thompson", "✓ Active\ntest", "Edit"],
            vec!["2", "Bob Wilson", "⚠ Pending", "Edit"],
            vec!["3", "Charlie Brown", "✗ Inactive", "Edit"],
        ],
    );

    let limiter = PaneLimiter::new(Box::new(table), 100, 30);

    tui.run(Box::new(limiter)).await
}
