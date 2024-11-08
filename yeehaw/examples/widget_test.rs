use yeehaw::{
    //debug,
    widgets::{
        widget_button::ButtonSides, Button, Checkbox, DropdownList, FigletText, Label, ListBox,
        NumbersTextBox, RadioButtons, TextBox, Toggle, WBStyles,
    },
    Color,
    Tui,
    DynVal,
    Element,
    Error,
    EventResponses,
    Gradient,

    WidgetPane,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::debug::set_log_file("./debug_test.log".to_string());
    yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let el = WidgetPane::new(&ctx);

    let l1 = Label::new(&ctx, "some label");

    let l = l1
        .clone()
        .at(DynVal::new_flex(0.5), DynVal::new_flex(0.5))
        .to_widgets();

    el.add_widgets(l);

    let button_click_fn = Box::new(move |_, ctx_| {
        let t = l1.get_text();
        let t = t + "0";
        l1.set_text(&ctx_, t);
        EventResponses::default()
    });
    let button = Button::new(&ctx, "click me", button_click_fn)
        .with_description("a button!".to_string())
        .at(DynVal::new_flex(0.25), DynVal::new_flex(0.25))
        .to_widgets()
        .with_label(&ctx, "button-label");
    el.add_widgets(button);

    let button2 = Button::new(&ctx, "button2", Box::new(|_, _| EventResponses::default()))
        .with_description("a button!".to_string())
        .with_sides(ButtonSides::default())
        .at(DynVal::new_flex(0.25), DynVal::new_flex(0.29))
        .to_widgets();

    el.add_widgets(button2);

    let cb = Checkbox::new(&ctx)
        .at(DynVal::new_flex(0.1), DynVal::new_flex(0.1))
        .to_widgets()
        .with_label(&ctx, "check me");
    el.add_widgets(cb);

    let cb2 = Checkbox::new(&ctx)
        .at(DynVal::new_flex(0.1), DynVal::new_flex(0.1).plus_fixed(1))
        .to_widgets()
        .with_label(&ctx, "check me2");
    el.add_widgets(cb2);

    let rbs = RadioButtons::new(
        &ctx,
        vec![
            "radio1".to_string(),
            "radio2".to_string(),
            "radio3".to_string(),
        ],
    )
    .at(DynVal::new_flex(0.1), DynVal::new_flex(0.1).plus_fixed(10))
    .to_widgets();
    el.add_widgets(rbs);

    let mut mtext_sty = WBStyles::default();
    let mut gr = Gradient::x_grad_rainbow(5);
    gr.angle_deg = 60.;
    mtext_sty.unselectable_style.set_fg(Color::Gradient(gr));

    let mtext = FigletText::new(
        &ctx,
        "HELLO, WERLD!",
        figlet_rs::FIGfont::from_content(std::include_str!("../../assets/figlet/ANSI_Shadow.flf"))
            .unwrap(),
    )
    .with_styles(mtext_sty)
    .at(DynVal::new_flex(0.1), DynVal::new_flex(0.6))
    .to_widgets();
    el.add_widgets(mtext);

    // moon runes: ⏾
    // sun runes: ★

    let toggle = Toggle::new(
        &ctx,
        " ★ ".to_string(),
        " ⏾ ".to_string(),
        Box::new(|_, _| EventResponses::default()),
    )
    .at(DynVal::new_flex(0.1), DynVal::new_flex(0.4))
    .to_widgets();
    el.add_widgets(toggle);

    // fill dd entries with 20 items
    let dd_entries = (1..=20)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();

    let dropdown = DropdownList::new(&ctx, dd_entries, Box::new(|_, _| EventResponses::default()))
        .with_max_expanded_height(10)
        .with_width(
            DynVal::default()
                .plus_max_of(DynVal::new_flex(0.2))
                .plus_max_of(DynVal::new_fixed(12)),
        )
        .at(DynVal::new_flex(0.1), DynVal::new_flex(0.8))
        .to_widgets();
    el.add_widgets(dropdown);

    let ld_entries = (1..=10)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();

    use yeehaw::widgets::widget_listbox::SelectionMode;
    let listbox = ListBox::new(&ctx, ld_entries, Box::new(|_, _| EventResponses::default()))
        .with_selection_mode(&ctx, SelectionMode::UpTo(3))
        .with_width(&ctx, DynVal::new_fixed(10))
        .with_height(&ctx, DynVal::new_fixed(5))
        .with_scrollbar()
        .at(DynVal::new_flex(0.5), DynVal::new_flex(0.1))
        .to_widgets(&ctx);
    el.add_widgets(listbox);

    let tb = TextBox::new(&ctx, "hellllllllllllllllllllllllllo\nworld")
        .with_width(DynVal::new_fixed(20))
        .with_height(DynVal::new_fixed(10))
        .with_line_numbers()
        .with_right_scrollbar()
        .with_lower_scrollbar()
        .editable()
        .with_no_wordwrap()
        .at(DynVal::new_fixed(70), DynVal::new_fixed(6))
        .to_widgets(&ctx);

    el.add_widgets(tb);

    let tb_with_grey = TextBox::new(&ctx, "")
        .with_width(DynVal::new_fixed(18))
        .with_height(DynVal::new_fixed(1))
        .editable()
        .with_text_when_empty("enter text here")
        .with_no_wordwrap()
        .at(DynVal::new_flex(0.25), DynVal::new_fixed(6))
        .to_widgets(&ctx);

    el.add_widgets(tb_with_grey);

    let ntb = NumbersTextBox::new(&ctx, 0)
        .with_min(-10)
        .with_max(10)
        .at(DynVal::new_flex(0.75), DynVal::new_flex(0.5))
        .to_widgets(&ctx);
    el.add_widgets(ntb);

    tui.run(Box::new(el)).await
}
