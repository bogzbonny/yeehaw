use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        widgets::{Button, Label, SclVal},
        Context, Cui, Error, EventResponse, SortingHat, WidgetPane,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let hat = SortingHat::default();

    let mut el = WidgetPane::new(&hat);
    let ctx = Context::new_context_for_screen();

    let l1 = Label::new(&hat, &ctx, "some label".to_string());

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
        .to_widgets();
    el.add_widgets(&ctx, button);

    //let text = DrawChs2D::from_string("Hello, Werld!".to_string(), Style::default());
    //let el = StandardPane::new(&hat, StandardPane::KIND).with_content(text);
    Cui::new(Rc::new(RefCell::new(el)))?.run().await
}
