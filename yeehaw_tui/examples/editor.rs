#[allow(dead_code)]
mod shared;

use {shared::editor::*, yeehaw_tui::*};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::log::reset_log_file("./debug_test.log".to_string());
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let el = editor_demo(&ctx);
    tui.run(el).await
}
