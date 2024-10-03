use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        //debug,
        Context,
        Cui,
        DebugSizePane,
        Error,
        FileNavPane,
        FileViewerPane,
        HorizontalStack,
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

    let hstack = HorizontalStack::new(&hat);
    let hstack2 = HorizontalStack::new(&hat);

    let nav = FileNavPane::new(&hat, &ctx, std::env::current_dir().unwrap());
    let nav_ = nav.clone();
    let hstack2_ = hstack2.clone();
    let ctx_ = ctx.clone();
    let hat_ = hat.clone();
    let open_fn = Box::new(move |path| {
        // TODO replace with some form of box wrapper element
        if !hstack2_.is_empty() {
            hstack2_.remove(&ctx_, 0);
        }
        let viewer = Rc::new(RefCell::new(FileViewerPane::new(&hat_, &ctx_, path)));
        hstack2_.push(&ctx_, viewer);
    });
    nav.set_open_fn(open_fn);

    hstack.push(&ctx, Rc::new(RefCell::new(nav.clone())));
    hstack.push(&ctx, Rc::new(RefCell::new(hstack2.clone())));

    Cui::new(Rc::new(RefCell::new(hstack)))?.run().await
}
