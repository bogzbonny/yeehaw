use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let hstack = HorizontalStack::new(&ctx);

    // NOTE here the ParentPane effectively acts as a box.
    // if we don't use indirection then open_fn deadlocks
    let panebox = ParentPane::new(&ctx, "box");

    let nav = FileNavPane::new(&ctx, std::env::current_dir().expect("no current dir"));
    nav.pane.set_focused(&ctx);

    let nav_ = nav.clone();
    let panebox_ = panebox.clone();
    let outer_ctx = ctx.clone();
    let open_fn = Box::new(move |ctx, path| {
        if !panebox_.has_elements() {
            let _ = panebox_.clear_elements();
        }

        nav_.pane.set_unfocused(&ctx); // the only time which the inner ctx is relavent here
        let viewer = Box::new(FileViewerPane::new(&outer_ctx, path));
        let resp = panebox_.add_element(viewer).into();
        panebox_.pane.send_responses_upward(&outer_ctx, resp);

        EventResponses::default()
    });
    nav.set_open_fn(open_fn);

    let _ = hstack.push(Box::new(nav.clone()));
    let _ = hstack.push(Box::new(panebox.clone()));
    tui.run(Box::new(hstack)).await
}
