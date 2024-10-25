use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        //debug,
        widgets::Button,
        Context,
        Cui,
        DebugSizePane,
        Error,
        EventResponses,
        HorizontalStack,
        ImageViewer,
        SortingHat,
        VerticalStack,
        WidgetPane,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    // get the first arg
    let args: Vec<String> = std::env::args().collect();
    let img_path = if args.len() > 1 {
        args[1].clone()
    } else {
        // exit
        return Err(Error::new("No image path provided"));
    };

    let hat = SortingHat::default();
    let ctx = Context::new_context_for_screen_no_dur();

    let vstack = VerticalStack::new(&hat);
    //let mut widget_pane = WidgetPane::new(&hat).with_height(DynVal::new_flex_with_max_fixed(0., 3));
    let widget_pane = WidgetPane::new(&hat).with_height(3.into());
    let hstack = HorizontalStack::new(&hat).with_height(1.0.into());
    vstack.push(&ctx, Rc::new(RefCell::new(widget_pane.clone())));
    vstack.push(&ctx, Rc::new(RefCell::new(hstack.clone())));

    let hstack__ = hstack.clone();
    let remove_button_click_fn = Box::new(move |ctx_| {
        if !hstack__.is_empty() {
            hstack__.remove(&ctx_, 0);
        }
        EventResponses::default()
    });
    let remove_button = Button::new(
        &hat,
        &ctx,
        "remove_pane".to_string(),
        remove_button_click_fn,
    )
    .at(13.into(), 1.into())
    .to_widgets();
    widget_pane.add_widgets(remove_button);

    let hstack_ = hstack.clone();
    let hat_ = hat.clone();
    let add_button_click_fn = Box::new(move |ctx_| {
        if hstack_.len() == 3 {
            let el = ImageViewer::new(&hat_, &img_path).with_width(hstack_.avg_width(&ctx_));
            hstack_.push(&ctx_, Rc::new(RefCell::new(el)));
            EventResponses::default()
        } else {
            let el = DebugSizePane::new(&hat_).with_width(hstack_.avg_width(&ctx_));
            hstack_.push(&ctx_, Rc::new(RefCell::new(el)));
            EventResponses::default()
        }
    });
    let add_button = Button::new(&hat, &ctx, "add_pane".to_string(), add_button_click_fn)
        .at(1.into(), 1.into())
        .to_widgets();
    widget_pane.add_widgets(add_button);

    Cui::new(Rc::new(RefCell::new(vstack)))?.run().await
}
