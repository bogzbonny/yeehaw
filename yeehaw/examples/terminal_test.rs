use yeehaw::{Error, TerminalPane, Tui};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let pane = TerminalPane::new(&ctx);
    tui.run(Box::new(pane)).await
}
