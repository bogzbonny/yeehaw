use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        widgets::{Button, Label},
        Context, Cui, Error, EventResponses, MenuBar, ParentPane, SortingHat, VerticalStack,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let (exit_tx, exit_recv) = tokio::sync::watch::channel(false);
    let ctx = Context::new_context_for_screen_no_dur(exit_recv.clone());

    let vstack = VerticalStack::new(&hat);
    let mb = MenuBar::top_menu_bar(&hat)
        .with_height(1.into())
        .with_width(1.0.into());
    let lower = ParentPane::new(&hat, "lower")
        .with_dyn_height(1.0.into())
        .with_dyn_width(1.0.into())
        .with_z(100);

    let label = Label::new(&hat, &ctx, "label").at(0.into(), 10.into());

    let label_ = label.clone();
    let btn_a = Button::new(
        &hat,
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
        &hat,
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
        &hat,
        &ctx,
        "C",
        Box::new(move |_, ctx_| {
            label_.set_text(&ctx_, "Button C clicked".to_string());
            EventResponses::default()
        }),
    )
    .at(9.into(), 0.into());

    lower.add_element(Rc::new(RefCell::new(label)));
    lower.add_element(Rc::new(RefCell::new(btn_a)));
    lower.add_element(Rc::new(RefCell::new(btn_b)));
    lower.add_element(Rc::new(RefCell::new(btn_c)));

    vstack.push(&ctx, Rc::new(RefCell::new(mb.clone())));
    vstack.push(&ctx, Rc::new(RefCell::new(lower)));

    mb.add_item(&hat, &ctx, "hello/asdg/2222/3".to_string(), None);
    mb.add_item(
        &hat,
        &ctx,
        "hello/asdg/444ll/3adsf3/sdlkjf".to_string(),
        None,
    );
    mb.add_item(&hat, &ctx, "hello/as33/222222/33".to_string(), None);
    mb.add_item(&hat, &ctx, "world/yo".to_string(), None);
    mb.add_item(&hat, &ctx, "world/yosdfjldsffff/asdkjl".to_string(), None);
    mb.add_item(&hat, &ctx, "diner/yoyo/hi/asgd".to_string(), None);

    Cui::new(Rc::new(RefCell::new(vstack)), exit_tx, exit_recv)?
        .run()
        .await
}
