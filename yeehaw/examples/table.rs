use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let table = Table::new(&ctx);
    table.set_header(&ctx, vec!["ID", "Name", "Status", "Actions"]);
    table.set_data(
        &ctx,
        vec![
            vec!["1", "Alice Thompson", "✓ Active", "Edit"],
            vec!["2", "Bob Wilson", "⚠ Pending", "Edit"],
            vec!["3", "Charlie Brown", "✗ Inactive", "Edit"],
        ],
    );

    tui.run(Box::new(table)).await
}
