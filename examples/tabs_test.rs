use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        //debug,
        Context,
        Cui,
        DebugSizePane,
        Error,
        SortingHat,
        Tabs,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let ctx = Context::new_context_for_screen_no_dur();

    let tabs = Tabs::new(&hat, &ctx);
    let el1 = DebugSizePane::new(&hat).with_text("tab 1".to_string());
    let el2 = DebugSizePane::new(&hat).with_text("tab 2".to_string());
    let el3 = DebugSizePane::new(&hat).with_text("tab 3".to_string());

    tabs.push(&ctx, Rc::new(RefCell::new(el1)), "tab 1");
    tabs.push(&ctx, Rc::new(RefCell::new(el2)), "tab 2");
    tabs.push(&ctx, Rc::new(RefCell::new(el3)), "tab 3");
    tabs.select(&ctx, 0);

    Cui::new(Rc::new(RefCell::new(tabs)))?.run().await
}
