use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::log::set_log_file("./debug_test.log".to_string());
    //yeehaw::log::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    // get the first arg
    let args: Vec<String> = std::env::args().collect();
    let img_path = if args.len() > 1 {
        args[1].clone()
    } else {
        // exit
        return Err(Error::new("No image path provided"));
    };

    let vstack = VerticalStack::new(&ctx);
    let sel_pane = ParentPaneOfSelectable::new(&ctx).with_dyn_height(3.into());
    let hstack = HorizontalStack::new(&ctx).with_height(1.0.into());
    vstack.push(Box::new(sel_pane.clone()));
    vstack.push(Box::new(hstack.clone()));

    let hstack__ = hstack.clone();
    let remove_button_click_fn = Box::new(move |_, _| {
        if !hstack__.is_empty() {
            hstack__.remove(0);
        }
        EventResponses::default()
    });
    let remove_button =
        Button::new(&ctx, "remove_pane", remove_button_click_fn).at(13.into(), 1.into());
    sel_pane.add_element(Box::new(remove_button));

    let hstack_ = hstack.clone();
    let ctx_ = ctx.clone();
    let add_button_click_fn = Box::new(move |_, _| {
        if hstack_.len() == 3 {
            let el = ImageViewer::new(&ctx_, &img_path)
                .expect("could not create image viewer")
                .with_width(hstack_.avg_width(&ctx_));
            hstack_.push(Box::new(el));
            EventResponses::default()
        } else {
            let el = DebugSizePane::new(&ctx_).with_dyn_width(hstack_.avg_width(&ctx_));
            hstack_.push(Box::new(el));
            EventResponses::default()
        }
    });
    let add_button = Button::new(&ctx, "add_pane", add_button_click_fn).at(1.into(), 1.into());
    sel_pane.add_element(Box::new(add_button));

    tui.run(Box::new(vstack)).await
}
