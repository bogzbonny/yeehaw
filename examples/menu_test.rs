use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{Context, Cui, Error, MenuBar, SortingHat},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./widget_test.log".to_string());
    //yeehaw::debug::clear();

    std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let mb = MenuBar::top_bar(&hat);
    let _ctx = Context::new_context_for_screen();
    mb.add_item(&hat, "hello".to_string(), None);
    mb.add_item(&hat, "world".to_string(), None);
    mb.add_item(&hat, "world/yo".to_string(), None);
    mb.add_item(&hat, "dine/yo".to_string(), None);

    Cui::new(Rc::new(RefCell::new(mb)))?.run().await
}
