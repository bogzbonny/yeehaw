use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{Context, Cui, Error, SortingHat, TerminalPane},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::debug::set_log_file("./debug_test.log".to_string());
    yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let (exit_tx, exit_recv) = tokio::sync::watch::channel(false);
    let ctx = Context::new_context_for_screen_no_dur(exit_recv.clone());

    let pane = TerminalPane::new(&hat, &ctx);

    Cui::new(Rc::new(RefCell::new(pane)), exit_tx, exit_recv)?
        .run()
        .await
}
