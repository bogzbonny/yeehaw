use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::debug::set_log_file("./debug_test.log".to_string());
    yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let el = ParentPaneOfSelectable::new(&ctx);

    let l1 = Label::new(&ctx, "some label");

    let l = l1.clone().at(DynVal::new_flex(0.5), DynVal::new_flex(0.5));
    el.add_element(Box::new(l));

    let button_click_fn = Box::new(move |_, _| {
        let t = l1.get_text();
        let t = t + "0";
        l1.set_text(t);
        EventResponses::default()
    });
    let button = Button::new(&ctx, "click me", button_click_fn)
        .with_description("a button!".to_string())
        .at(DynVal::new_flex(0.25), DynVal::new_flex(0.25));
    let button_label = Label::new_for_el(
        &ctx,
        button.get_dyn_location_set().l.clone(),
        "button-label",
    );
    el.add_element(Box::new(button));
    el.add_element(Box::new(button_label));

    let button = Button::new(
        &ctx,
        "do not\nclick me",
        Box::new(|_, _| EventResponses::default()),
    )
    .with_description("a button!".to_string())
    .at(DynVal::new_flex(0.25), DynVal::new_flex(0.15));
    el.add_element(Box::new(button));

    let button2 = Button::new(&ctx, "button2", Box::new(|_, _| EventResponses::default()))
        .with_description("a button!".to_string())
        .with_sides(ButtonSides::default())
        .at(DynVal::new_flex(0.25), DynVal::new_flex(0.29));

    el.add_element(Box::new(button2));

    let cb = Checkbox::new(&ctx).at(DynVal::new_flex(0.1), DynVal::new_flex(0.1));
    let cb_label = Label::new_for_el(&ctx, cb.get_dyn_location_set().l.clone(), "check me");
    el.add_element(Box::new(cb));
    el.add_element(Box::new(cb_label));

    let cb2 = Checkbox::new(&ctx).at(DynVal::new_flex(0.1), DynVal::new_flex(0.1).plus_fixed(1));
    let cb2_label = Label::new_for_el(&ctx, cb2.get_dyn_location_set().l.clone(), "check me");
    el.add_element(Box::new(cb2));
    el.add_element(Box::new(cb2_label));

    let rbs = RadioButtons::new(
        &ctx,
        vec![
            "radio1".to_string(),
            "radio2".to_string(),
            "radio3".to_string(),
        ],
    )
    .at(DynVal::new_flex(0.1), DynVal::new_flex(0.1).plus_fixed(10));
    el.add_element(Box::new(rbs));

    let mut gr = Gradient::x_grad_rainbow(5);
    gr.angle_deg = 60.;

    let mtext = FigletText::new(
        &ctx,
        "HELLO, WERLD!",
        figlet_rs::FIGfont::from_content(std::include_str!("../../assets/figlet/ANSI_Shadow.flf"))
            .unwrap(),
    )
    .with_style(Style::default().with_fg(Color::Gradient(gr)))
    .at(DynVal::new_flex(0.1), DynVal::new_flex(0.6));
    el.add_element(Box::new(mtext));

    let toggle = Toggle::new(
        &ctx,
        " ★ ".to_string(),
        " ⏾ ".to_string(),
        Box::new(|_, _| EventResponses::default()),
    )
    .at(DynVal::new_flex(0.1), DynVal::new_flex(0.4));
    el.add_element(Box::new(toggle));

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
        .at(DynVal::new_flex(0.1), DynVal::new_flex(0.8));
    el.add_element(Box::new(dropdown));

    let ld_entries = (1..=10)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();

    use yeehaw::elements::listbox::SelectionMode;
    let listbox = ListBox::new(&ctx, ld_entries, Box::new(|_, _| EventResponses::default()))
        .with_selection_mode(&ctx, SelectionMode::UpTo(3))
        //.with_width(&ctx, DynVal::new_fixed(12))
        //.with_height(&ctx, DynVal::new_fixed(5))
        .with_width(
            &ctx,
            DynVal::default()
                .plus_max_of(DynVal::new_flex(0.15))
                .plus_max_of(DynVal::new_fixed(12)),
        )
        .with_height(
            &ctx,
            DynVal::default()
                .plus_max_of(DynVal::new_flex(0.13))
                .plus_max_of(DynVal::new_fixed(5)),
        )
        .with_scrollbar(&ctx)
        .at(DynVal::new_flex(0.5), DynVal::new_flex(0.1));
    el.add_element(Box::new(listbox));

    // XXX uncomment
    //let tb = TextBox::new(&ctx, "hellllllllllllllllllllllllllo\nworld")
    //    .with_width(DynVal::new_fixed(20))
    //    .with_height(DynVal::new_fixed(10))
    //    .with_line_numbers()
    //    .with_right_scrollbar()
    //    .with_lower_scrollbar()
    //    .editable()
    //    .with_no_wordwrap()
    //    .at(DynVal::new_fixed(70), DynVal::new_fixed(6))
    //    .to_widgets(&ctx);

    //el.add_element(tb);

    //let tb_with_grey = TextBox::new(&ctx, "")
    //    .with_width(DynVal::new_fixed(18))
    //    .with_height(DynVal::new_fixed(1))
    //    .editable()
    //    .with_text_when_empty("enter text here")
    //    .with_no_wordwrap()
    //    .at(DynVal::new_flex(0.25), DynVal::new_fixed(6))
    //    .to_widgets(&ctx);

    //el.add_element(tb_with_grey);

    //let ntb = NumbersTextBox::new(&ctx, 0)
    //    .with_min(-10)
    //    .with_max(10)
    //    .at(DynVal::new_flex(0.75), DynVal::new_flex(0.5))
    //    .to_widgets(&ctx);
    //el.add_element(ntb);

    tui.run(Box::new(el)).await
}
