use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::log::reset_log_file("./debug_test.log".to_string());
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let tabs = Tabs::new(&ctx);
    //let el1 = DebugSizePane::new(&ctx).with_text("tab 1".to_string());
    //let el2 = DebugSizePane::new(&ctx).with_text("tab 2".to_string());
    //let el3 = DebugSizePane::new(&ctx).with_text("tab 3".to_string());

    let el1 = DebugSizePane::new(&ctx)
        .with_bg(Color::RED)
        .with_text("tab 1".to_string());
    let el2 = DebugSizePane::new(&ctx)
        .with_bg(Color::BLUE)
        .with_text("tab 2".to_string());
    let el3 = DebugSizePane::new(&ctx)
        .with_bg(Color::GREEN)
        .with_text("tab 3".to_string());

    tabs.push(Box::new(el1), "tab 1");
    tabs.push(Box::new(el2), "tab 2");
    tabs.push(Box::new(el3), "tab 3");
    let _ = tabs.select(0);

    tui.run(Box::new(tabs)).await
}
