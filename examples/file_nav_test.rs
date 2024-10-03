use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        //debug,
        Context,
        Cui,
        Error,
        FileNavPane,
        SortingHat,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./widget_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let ctx = Context::new_context_for_screen();

    let nav = FileNavPane::new(
        &hat,
        &ctx,
        std::env::current_dir().unwrap(),
        Box::new(|_path| {}),
    );

    Cui::new(Rc::new(RefCell::new(nav)))?.run().await
}
