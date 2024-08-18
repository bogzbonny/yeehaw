use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        //debug,
        widgets::{megafonts, Button, Checkbox, Label, Megatext, RadioButtons, SclVal, Toggle},
        Context,
        Cui,
        Error,
        EventResponses,
        SortingHat,
        WidgetPane,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //debug::set_log_file("./widget_test.log".to_string());
    //debug::clear();
    let hat = SortingHat::default();

    let mut el = WidgetPane::new(&hat);
    let ctx = Context::new_context_for_screen();

    let l1 = Label::new(&hat, &ctx, "some label");

    let l = l1
        .clone()
        .at(SclVal::new_frac(0.5), SclVal::new_frac(0.5))
        .to_widgets();

    el.add_widgets(&ctx, l);

    let button_click_fn = Box::new(move |ctx_| {
        let t = l1.get_text();
        let t = t + "0";
        l1.set_text(&ctx_, t);
        EventResponses::default()
    });
    let button = Button::new(&hat, &ctx, "click me".to_string(), button_click_fn)
        .at(SclVal::new_frac(0.25), SclVal::new_frac(0.25))
        .to_widgets()
        .with_label(&hat, &ctx, "button-label");
    el.add_widgets(&ctx, button);

    let cb = Checkbox::new(&hat)
        .at(SclVal::new_frac(0.1), SclVal::new_frac(0.1))
        .to_widgets()
        .with_label(&hat, &ctx, "check me");
    el.add_widgets(&ctx, cb);

    let cb2 = Checkbox::new(&hat)
        .at(SclVal::new_frac(0.1), SclVal::new_frac(0.1).plus_fixed(1))
        .to_widgets()
        .with_label(&hat, &ctx, "check me2");
    el.add_widgets(&ctx, cb2);

    let rbs = RadioButtons::new(
        &hat,
        vec![
            "radio1".to_string(),
            "radio2".to_string(),
            "radio3".to_string(),
        ],
    )
    .at(SclVal::new_frac(0.1), SclVal::new_frac(0.1).plus_fixed(10))
    .to_widgets();
    el.add_widgets(&ctx, rbs);

    let mtext = Megatext::new(
        &hat,
        "HELLO, WERLD!".to_string(),
        megafonts::ansi_regular_ex(),
    )
    .at(SclVal::new_frac(0.1), SclVal::new_frac(0.6))
    .to_widgets();
    el.add_widgets(&ctx, mtext);

    // moon runes: ⏾
    // sun runes: ★

    let toggle = Toggle::new(
        &hat,
        &ctx,
        " ★ ".to_string(),
        " ⏾ ".to_string(),
        Box::new(|_, _| EventResponses::default()),
    )
    .at(SclVal::new_frac(0.1), SclVal::new_frac(0.4))
    .to_widgets();
    el.add_widgets(&ctx, toggle);

    Cui::new(Rc::new(RefCell::new(el)))?.run().await
}
