use yeehaw::{
    widgets::{Button, Label},
    Cui, Error, EventResponses, MenuBar, ParentPane, VerticalStack,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut cui, ctx) = Cui::new()?;

    let vstack = VerticalStack::new(&ctx);
    let mb = MenuBar::top_menu_bar(&ctx)
        .with_height(1.into())
        .with_width(1.0.into());
    let lower = ParentPane::new(&ctx, "lower")
        .with_dyn_height(1.0.into())
        .with_dyn_width(1.0.into())
        .with_z(100);

    let label = Label::new(&ctx, "label").at(0.into(), 10.into());

    let label_ = label.clone();
    let btn_a = Button::new(
        &ctx,
        "A",
        Box::new(move |_, ctx_| {
            label_.set_text(&ctx_, "Button A clicked".to_string());
            EventResponses::default()
        }),
    )
    .at(1.into(), 0.into());

    let label_ = label.clone();
    let btn_b = Button::new(
        &ctx,
        "B",
        Box::new(move |_, ctx_| {
            label_.set_text(&ctx_, "Button B clicked".to_string());
            EventResponses::default()
        }),
    )
    .at(5.into(), 0.into());

    let label_ = label.clone();
    let btn_c = Button::new(
        &ctx,
        "C",
        Box::new(move |_, ctx_| {
            label_.set_text(&ctx_, "Button C clicked".to_string());
            EventResponses::default()
        }),
    )
    .at(9.into(), 0.into());

    lower.add_element(Box::new(label));
    lower.add_element(Box::new(btn_a));
    lower.add_element(Box::new(btn_b));
    lower.add_element(Box::new(btn_c));

    vstack.push(&ctx, Box::new(mb.clone()));
    vstack.push(&ctx, Box::new(lower));

    mb.add_item(&ctx, "hello/asdg/2222/3".to_string(), None);
    mb.add_item(&ctx, "hello/asdg/444ll/3adsf3/sdlkjf".to_string(), None);
    mb.add_item(&ctx, "hello/as33/222222/33".to_string(), None);
    mb.add_item(&ctx, "world/yo".to_string(), None);
    mb.add_item(&ctx, "world/yosdfjldsffff/asdkjl".to_string(), None);
    mb.add_item(&ctx, "diner/yoyo/hi/asgd".to_string(), None);

    cui.run(Box::new(vstack)).await
}
