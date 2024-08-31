use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{Context, Cui, Error, MenuBar, SortingHat},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./widget_test.log".to_string());
    //yeehaw::debug::clear();

    //std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let mb = MenuBar::top_menu_bar(&hat);
    let ctx = Context::new_context_for_screen();
    mb.add_item(&hat, &ctx, "hello/asdg/2222/3".to_string(), None);
    mb.add_item(&hat, &ctx, "hello/asdg/444ll/3adsf3".to_string(), None);
    mb.add_item(&hat, &ctx, "hello/as33/222222/33".to_string(), None);
    mb.add_item(&hat, &ctx, "world/yo".to_string(), None);
    mb.add_item(&hat, &ctx, "world/yosdfjldsf/asdkjl".to_string(), None);
    mb.add_item(&hat, &ctx, "diner/yoyo/hi".to_string(), None);

    Cui::new(Rc::new(RefCell::new(mb)))?.run().await
}
