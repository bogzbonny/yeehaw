use yeehaw::{Cui, Error, TerminalPane};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut cui, ctx) = Cui::new()?;
    let pane = TerminalPane::new(&ctx);
    cui.run(Box::new(pane)).await
}
