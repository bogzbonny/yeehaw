use yeehaw_tui::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::log::reset_log_file("./debug_test.log".to_string());
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let vstack = VerticalStack::new(&ctx);
    let mb = MenuBar::top_menu_bar(&ctx)
        .with_dyn_height(1)
        .with_dyn_width(DynVal::FULL);
    let lower = ParentPane::new(&ctx, "lower")
        .with_dyn_height(1.0)
        .with_dyn_width(1.0)
        .with_z(100);

    let label = Label::new(&ctx, "label").at(0, 10);

    let label_ = label.clone();
    let btn_a = Button::new(&ctx, "A")
        .with_fn(Box::new(move |_, _| {
            label_.set_text("Button A clicked".to_string());
            EventResponses::default()
        }))
        .at(1, 0);

    let label_ = label.clone();
    let btn_b = Button::new(&ctx, "B")
        .with_fn(Box::new(move |_, _| {
            label_.set_text("Button B clicked".to_string());
            EventResponses::default()
        }))
        .at(5, 0);

    let label_ = label.clone();
    let btn_c = Button::new(&ctx, "C")
        .with_fn(Box::new(move |_, _| {
            label_.set_text("Button C clicked".to_string());
            EventResponses::default()
        }))
        .at(9, 0);

    lower.add_element(Box::new(label));
    lower.add_element(Box::new(btn_a));
    lower.add_element(Box::new(btn_b));
    lower.add_element(Box::new(btn_c));

    vstack.push(Box::new(mb.clone()));
    vstack.push(Box::new(lower));

    mb.add_item(&ctx, "hello/asdg/2222/3".to_string(), None);
    mb.add_item(&ctx, "hello/asdg/444ll/3adsf3/sdlkjf".to_string(), None);
    mb.add_item(&ctx, "hello/as33/222222/33".to_string(), None);
    mb.add_item(&ctx, "world/yo".to_string(), None);
    mb.add_item(&ctx, "world/yosdfjldsffff/asdkjl".to_string(), None);
    mb.add_item(&ctx, "diner/yoyo/hi/asgd".to_string(), None);

    tui.run(Box::new(vstack)).await
}
