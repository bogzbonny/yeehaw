use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let hstack = HorizontalStackFocuser::new(&ctx);

    let nav = FileNavPane::new(&ctx, std::env::current_dir().expect("no current dir"));
    nav.pane.set_focused(&ctx);

    let nav_ = nav.clone();
    let hstack_ = hstack.clone();
    let outer_ctx = ctx.clone();
    let open_fn = Box::new(move |_nav_ctx, path| {
        nav_.pane.set_dyn_width(0.2);
        let viewer = FileViewerPane::new(&outer_ctx, path).with_dyn_width(0.8);

        let mut resps =
            if hstack_.len() > 1 { hstack_.pop().into() } else { EventResponses::default() };
        let resp = hstack_.push(Box::new(viewer));
        resps.push(resp);

        // ignore the resps because this is the main element
        // otherwise they would be propogated up the tree
        let _ = resps;

        // unfocus the non-nav pane
        EventResponse::UnfocusOthers.into()
    });
    nav.set_open_fn(open_fn);

    let _ = hstack.push(Box::new(nav));
    tui.run(Box::new(hstack)).await
}
