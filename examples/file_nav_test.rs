use yeehaw::{
    //debug,
    Context,
    Cui,
    Error,
    FileNavPane,
    FileViewerPane,
    HorizontalStack,
    SortingHat,
    VerticalStack,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./widgt_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let ctx = Context::new_context_for_screen_no_dur();

    let hstack = HorizontalStack::new(&hat);
    let vstack2 = VerticalStack::new(&hat);

    let nav = FileNavPane::new(&hat, &ctx, std::env::current_dir().unwrap());
    let nav_ = nav.clone();
    let vstack2_ = vstack2.clone();
    let ctx_ = ctx.clone();
    let hat_ = hat.clone();
    let open_fn = Box::new(move |path| {
        // TODO replace with some form of box wrapper element
        if !vstack2_.is_empty() {
            vstack2_.remove(&ctx_, 0);
        }
        nav_.pane.unfocus();
        let viewer = Box::new(FileViewerPane::new(&hat_, &ctx_, path));
        vstack2_.push(&ctx_, viewer);
    });
    nav.set_open_fn(open_fn);

    hstack.push(&ctx, Box::new(nav.clone()));
    hstack.push(&ctx, Box::new(vstack2.clone()));
    Cui::new(Box::new(hstack))?.run().await
}
