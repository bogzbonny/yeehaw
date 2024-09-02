use {
    //std::env,
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        debug,
        widgets::{
            megafonts, Button, Checkbox, DropdownList, Label, ListBox, Megatext, NumbersTextBox,
            RadioButtons, TextBox, Toggle,
        },
        Context, Cui, Element, Error, EventResponses, SclVal, SortingHat, WidgetPane,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::debug::set_log_file("./widget_test.log".to_string());
    yeehaw::debug::clear();
    std::env::set_var("RUST_BACKTRACE", "1");

    debug!("1");
    let hat = SortingHat::default();

    let mut el = WidgetPane::new(&hat);
    let ctx = Context::new_context_for_screen();

    let l1 = Label::new(&hat, &ctx, "some label");

    let l = l1
        .clone()
        .at(SclVal::new_frac(0.5), SclVal::new_frac(0.5))
        .to_widgets();

    el.add_widgets(l);

    let button_click_fn = Box::new(move |ctx_| {
        let t = l1.get_text();
        let t = t + "0";
        l1.set_text(&ctx_, t);
        EventResponses::default()
    });
    let button = Button::new(&hat, &ctx, "click me".to_string(), button_click_fn)
        .with_description("a button!".to_string())
        .at(SclVal::new_frac(0.25), SclVal::new_frac(0.25))
        .to_widgets()
        .with_label(&hat, &ctx, "button-label");
    el.add_widgets(button);

    let cb = Checkbox::new(&hat)
        .at(SclVal::new_frac(0.1), SclVal::new_frac(0.1))
        .to_widgets()
        .with_label(&hat, &ctx, "check me");
    el.add_widgets(cb);

    let cb2 = Checkbox::new(&hat)
        .at(SclVal::new_frac(0.1), SclVal::new_frac(0.1).plus_fixed(1))
        .to_widgets()
        .with_label(&hat, &ctx, "check me2");
    el.add_widgets(cb2);

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
    el.add_widgets(rbs);
    debug!("2");

    let mtext = Megatext::new(
        &hat,
        &ctx,
        "HELLO, WERLD!".to_string(),
        megafonts::ansi_regular_ex(),
    )
    .at(SclVal::new_frac(0.1), SclVal::new_frac(0.6))
    .to_widgets();
    el.add_widgets(mtext);
    debug!("3");

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
    el.add_widgets(toggle);
    debug!("4");

    // fill dd entries with 20 items
    let dd_entries = (1..=20)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();

    let dropdown = DropdownList::new(
        &hat,
        &ctx,
        dd_entries,
        Box::new(|_, _| EventResponses::default()),
    )
    .with_max_expanded_height(10)
    .with_width(
        SclVal::default()
            .plus_max_of(SclVal::new_frac(0.2))
            .plus_max_of(SclVal::new_fixed(12)),
    )
    .at(SclVal::new_frac(0.1), SclVal::new_frac(0.8))
    .to_widgets();
    el.add_widgets(dropdown);
    debug!("5");

    let ld_entries = (1..=10)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();

    use yeehaw::widgets::widget_listbox::SelectionMode;
    let listbox = ListBox::new(
        &hat,
        &ctx,
        ld_entries,
        Box::new(|_, _| EventResponses::default()),
    )
    .with_selection_mode(&ctx, SelectionMode::UpTo(3))
    .with_width(&ctx, SclVal::new_fixed(10))
    .with_height(&ctx, SclVal::new_fixed(5))
    .with_scrollbar()
    .at(SclVal::new_frac(0.5), SclVal::new_frac(0.1))
    .to_widgets(&hat);
    el.add_widgets(listbox);
    debug!("6");

    let tb = TextBox::new(
        &hat,
        &ctx,
        "hellllllllllllllllllllllllllo\nworld".to_string(),
    )
    .with_width(SclVal::new_fixed(20))
    .with_height(SclVal::new_fixed(10))
    .with_line_numbers()
    .with_right_scrollbar()
    .with_lower_scrollbar()
    .editable()
    .with_no_wordwrap()
    .at(SclVal::new_fixed(70), SclVal::new_fixed(6))
    .to_widgets(&hat, &ctx);
    debug!("7");

    el.add_widgets(tb);

    let ntb = NumbersTextBox::new(&hat, &ctx, 0)
        .with_min(-10)
        .with_max(10)
        .at(SclVal::new_frac(0.75), SclVal::new_frac(0.5))
        .to_widgets(&hat, &ctx);
    el.add_widgets(ntb);
    debug!("8");

    Cui::new(Rc::new(RefCell::new(el)))?.run().await
}
