use yeehaw::{
    //debug,
    Cui,
    DebugSizePane,
    Error,

    Tabs,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut cui, ctx) = Cui::new()?;

    let tabs = Tabs::new(&ctx);
    let el1 = DebugSizePane::new(&ctx).with_text("tab 1".to_string());
    let el2 = DebugSizePane::new(&ctx).with_text("tab 2".to_string());
    let el3 = DebugSizePane::new(&ctx).with_text("tab 3".to_string());

    tabs.push(&ctx, Box::new(el1), "tab 1");
    tabs.push(&ctx, Box::new(el2), "tab 2");
    tabs.push(&ctx, Box::new(el3), "tab 3");
    tabs.select(&ctx, 0);

    cui.run(Box::new(tabs)).await
}
