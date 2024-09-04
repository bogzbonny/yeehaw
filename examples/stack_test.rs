use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        widgets::Button, Context, Cui, DebugSizePane, DynVal, Error, EventResponses,
        HorizontalStack, SortingHat, VerticalStack, WidgetPane,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./widget_test.log".to_string());
    //yeehaw::debug::clear();

    //std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let ctx = Context::new_context_for_screen();

    let vstack = VerticalStack::new(&hat);
    let mut widget_pane = WidgetPane::new(&hat).with_height(DynVal::new_fixed(3));
    let hstack = HorizontalStack::new(&hat);

    let hstack__ = hstack.clone();
    let remove_button_click_fn = Box::new(move |ctx_| {
        hstack__.remove(&ctx_, 0);
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
        hstack_.push(&ctx_, Rc::new(RefCell::new(DebugSizePane::new(&hat_))));
        EventResponses::default()
    });
    let add_button = Button::new(&hat, &ctx, "add_pane".to_string(), add_button_click_fn)
        .at(1.into(), 1.into())
        .to_widgets();
    widget_pane.add_widgets(add_button);

    vstack.push(&ctx, Rc::new(RefCell::new(widget_pane)));
    vstack.push(&ctx, Rc::new(RefCell::new(hstack)));

    Cui::new(Rc::new(RefCell::new(vstack)))?.run().await
}
