use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{Context, Cui, DebugSizePane, Error, MenuBar, SortingHat, VerticalStack},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./widget_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let ctx = Context::new_context_for_screen();
    let hat = SortingHat::default();

    let vstack = VerticalStack::new(&hat);
    let lower = DebugSizePane::new(&hat)
        .with_height(1.0.into())
        .with_width(1.0.into())
        .with_z(100);
    let mb = MenuBar::top_menu_bar(&hat)
        .with_height(1.into())
        .with_width(1.0.into());

    vstack.push(&ctx, Rc::new(RefCell::new(mb.clone())));
    vstack.push(&ctx, Rc::new(RefCell::new(lower)));

    mb.add_item(&hat, &ctx, "hello/asdg/2222/3".to_string(), None);
    mb.add_item(&hat, &ctx, "hello/asdg/444ll/3adsf3".to_string(), None);
    mb.add_item(&hat, &ctx, "hello/as33/222222/33".to_string(), None);
    mb.add_item(&hat, &ctx, "world/yo".to_string(), None);
    mb.add_item(&hat, &ctx, "world/yosdfjldsf/asdkjl".to_string(), None);
    mb.add_item(&hat, &ctx, "diner/yoyo/hi".to_string(), None);

    Cui::new(Rc::new(RefCell::new(vstack)))?.run().await
}
