use yeehaw::{
    widgets::Button, Cui, DebugSizePane, Error, EventResponses, HorizontalStack, VerticalStack,
    WidgetPane,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut cui, ctx) = Cui::new()?;

    let vstack = VerticalStack::new(&ctx);
    //let mut widget_pane = WidgetPane::new(&ctx).with_height(DynVal::new_flex_with_max_fixed(0., 3));
    let widget_pane = WidgetPane::new(&ctx).with_height(3.into());
    let hstack = HorizontalStack::new(&ctx).with_height(1.0.into());
    vstack.push(&ctx, Box::new(widget_pane.clone()));
    vstack.push(&ctx, Box::new(hstack.clone()));

    let hstack__ = hstack.clone();
    let remove_button_click_fn = Box::new(move |_, ctx_| {
        if !hstack__.is_empty() {
            hstack__.remove(&ctx_, 0);
        }
        EventResponses::default()
    });
    let remove_button = Button::new(&ctx, "remove_pane", remove_button_click_fn)
        .at(13.into(), 1.into())
        .to_widgets();
    widget_pane.add_widgets(remove_button);

    let hstack_ = hstack.clone();
    let ctx_ = ctx.clone();
    let add_button_click_fn = Box::new(move |_, _| {
        let el = DebugSizePane::new(&ctx_).with_width(hstack_.avg_width(&ctx_));
        hstack_.push(&ctx_, Box::new(el));
        EventResponses::default()
    });
    let add_button = Button::new(&ctx, "add_pane", add_button_click_fn)
        .at(1.into(), 1.into())
        .to_widgets();
    widget_pane.add_widgets(add_button);

    cui.run(Box::new(vstack)).await
}
