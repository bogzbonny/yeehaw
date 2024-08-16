use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        debug,
        widgets::{Button, Checkbox, Label, SclVal},
        Context, Cui, Error, EventResponse, SortingHat, WidgetPane,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    debug::set_log_file("./widget_test.log".to_string());
    debug::clear();
    let hat = SortingHat::default();

    let mut el = WidgetPane::new(&hat);
    let ctx = Context::new_context_for_screen();

    let l1 = Label::new(&hat, &ctx, "some label");

    let l = l1
        .clone()
        .at(SclVal::new_frac(0.5), SclVal::new_frac(0.5))
        .to_widgets();

    el.add_widgets(&ctx, l);

    let button_click_fn = Box::new(move || {
        let t = l1.get_text();
        let t = t + "0";
        l1.set_text(t);
        EventResponse::default()
    });
    let button = Button::new(&hat, &ctx, "click me".to_string(), button_click_fn)
        .at(SclVal::new_frac(0.25), SclVal::new_frac(0.25))
        .to_widgets()
        .with_label(&hat, &ctx, "button-label");
    el.add_widgets(&ctx, button);

    let cb = Checkbox::new(&hat, &ctx)
        .at(SclVal::new_frac(0.1), SclVal::new_frac(0.1))
        .to_widgets()
        .with_label(&hat, &ctx, "check me");
    el.add_widgets(&ctx, cb);

    let cb2 = Checkbox::new(&hat, &ctx)
        .at(SclVal::new_frac(0.1), SclVal::new_frac(0.1).plus_fixed(1))
        .to_widgets()
        .with_label(&hat, &ctx, "check me2");
    el.add_widgets(&ctx, cb2);

    //let text = DrawChs2D::from_string("Hello, Werld!".to_string(), Style::default());
    //let el = StandardPane::new(&hat, StandardPane::KIND).with_content(text);
    Cui::new(Rc::new(RefCell::new(el)))?.run().await
}
