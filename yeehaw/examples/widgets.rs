use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let el = ParentPaneOfSelectable::new(&ctx);

    let l1 = Label::new(&ctx, "some label");

    let l = l1.clone().at(DynVal::new_flex(0.5), DynVal::new_flex(0.5));
    let _ = el.add_element(Box::new(l));

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
    let _ = el.add_element(Box::new(button));
    let _ = el.add_element(Box::new(button_label));

    let button = Button::new(
        &ctx,
        "do not\nclick me",
        Box::new(|_, _| EventResponses::default()),
    )
    .with_description("a button!".to_string())
    .at(DynVal::new_flex(0.25), DynVal::new_flex(0.15));
    let _ = el.add_element(Box::new(button));

    let button2 = Button::new(&ctx, "button2", Box::new(|_, _| EventResponses::default()))
        .with_description("a button!".to_string())
        .with_sides(ButtonSides::default())
        .at(DynVal::new_flex(0.25), DynVal::new_flex(0.29));
    let _ = el.add_element(Box::new(button2));

    let button3 = Button::new(&ctx, "b", Box::new(|_, _| EventResponses::default()))
        .with_description("a button!".to_string())
        .with_micro_shadow(ButtonMicroShadow::default())
        .at(DynVal::new_flex(0.25), DynVal::new_flex(0.10));
    let _ = el.add_element(Box::new(button3));

    let cb = Checkbox::new(&ctx).at(DynVal::new_flex(0.1), DynVal::new_flex(0.1));
    let cb_label = Label::new_for_el(&ctx, cb.get_dyn_location_set().l.clone(), "check me");
    let _ = el.add_element(Box::new(cb));
    let _ = el.add_element(Box::new(cb_label));

    let cb2 = Checkbox::new(&ctx).at(DynVal::new_flex(0.1), DynVal::new_flex(0.1).plus_fixed(1));
    let cb2_label = Label::new_for_el(&ctx, cb2.get_dyn_location_set().l.clone(), "check me");
    let _ = el.add_element(Box::new(cb2));
    let _ = el.add_element(Box::new(cb2_label));

    let rbs = RadioButtons::new(
        &ctx,
        vec![
            "radio1".to_string(),
            "radio2".to_string(),
            "radio3".to_string(),
        ],
    )
    .at(DynVal::new_flex(0.1), DynVal::new_flex(0.1).plus_fixed(10));
    let _ = el.add_element(Box::new(rbs));

    let mut gr = Gradient::x_grad_rainbow(5);
    gr.angle_deg = 60.;

    let mtext = FigletText::new(
        &ctx,
        "HELLO, WERLD!",
        figlet_rs::FIGfont::from_content(std::include_str!("../../assets/figlet/ANSI_Shadow.flf"))
            .expect("missing asset"),
    )
    .with_style(Style::default().with_fg(Color::Gradient(gr)))
    .at(DynVal::new_flex(0.1), DynVal::new_flex(0.6));
    let _ = el.add_element(Box::new(mtext));

    let toggle = Toggle::new(&ctx, " ★ ".to_string(), " ⏾ ".to_string())
        .at(DynVal::new_flex(0.1), DynVal::new_flex(0.4));
    let _ = el.add_element(Box::new(toggle));

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
    let _ = el.add_element(Box::new(dropdown));

    let ld_entries = (1..=10)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();

    use yeehaw::elements::listbox::SelectionMode;
    let listbox = ListBox::new(&ctx, ld_entries, Box::new(|_, _| EventResponses::default()))
        .with_selection_mode(&ctx, SelectionMode::UpTo(3))
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
    let _ = el.add_element(Box::new(listbox));

    let tb = TextBox::new(
        &ctx,
        "012345678901234567890123456789\
        \n1\n2\n3\n4\n5\n6\n7\n8\n9\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9",
        //\n1",
    )
    //.with_width(DynVal::new_fixed(20))
    //.with_height(DynVal::new_fixed(10))
    .with_width(DynVal::new_flex(0.1))
    .with_height(DynVal::new_flex(0.2))
    //.with_top_scrollbar(&ctx)
    .with_bottom_scrollbar(&ctx)
    .with_left_scrollbar(&ctx)
    .with_line_numbers(&ctx)
    //.with_right_scrollbar(&ctx)
    .editable(&ctx)
    .with_no_wordwrap(&ctx)
    .at(DynVal::new_flex(0.35), DynVal::new_flex(0.1));
    let _ = el.add_element(Box::new(tb));

    let tb_with_grey = TextBox::new(&ctx, "")
        .with_width(DynVal::new_fixed(18))
        .with_height(DynVal::new_fixed(1))
        .editable(&ctx)
        .with_text_when_empty("enter text here")
        .with_no_wordwrap(&ctx)
        .at(DynVal::new_flex(0.25), DynVal::new_fixed(3));
    let _ = el.add_element(Box::new(tb_with_grey));

    let ntb = NumbersTextBox::new(&ctx, 0f64)
        .with_min(-10.0)
        .with_max(10.0)
        .at(DynVal::new_flex(0.75), DynVal::new_flex(0.5));
    let _ = el.add_element(Box::new(ntb.clone()));

    let ntb_ = ntb.clone();
    let slider = Slider::new_basic_block(&ctx)
        .with_gradient(Color::AQUA, Color::ORANGE)
        .with_width(DynVal::new_flex(0.1))
        .at(DynVal::new_flex(0.8), DynVal::new_flex(0.5))
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            ntb_.change_value(p);
            EventResponses::default()
        }));
    let _ = el.add_element(Box::new(slider));

    let reg_sty = Style::transparent().with_fg(Color::WHITE);

    #[rustfmt::skip]
    let drawing_base = "  H__A  \n".to_string()
                     + "G ╱  ╲ B\n"
                     + "F ╲__╱ C\n"
                     + "  E  D  ";
    let positions = "GHHHAAAB\n\
                     GGGHABBB\n\
                     FFFEDCCC\n\
                     FEEEDDDC";
    let sel_changes = vec![
        ('°', 4, 1),
        ('∘', 5, 1),
        ('°', 5, 2),
        ('∘', 4, 2),
        ('∘', 3, 2),
        ('°', 2, 2),
        ('∘', 2, 1),
        ('°', 3, 1),
    ];

    let dial = ArbSelector::new_with_uniform_style(
        &ctx,
        reg_sty,
        drawing_base.to_string(),
        positions.to_string(),
        sel_changes,
    )
    .at(DynVal::new_flex(0.8), DynVal::new_flex(0.7));
    let _ = el.add_element(Box::new(dial));

    //let dial1 = Dial::new_compact(
    //let dial1 = Dial::new_ultra_compact(
    //let dial1 = Dial::new_semi_compact(
    let dial1 = Dial::new_spacious(
        &ctx,
        vec![
            //(2, "OC"),
            //(0, "OptionA"),
            //(1, "OptionB"),
            //(2, "OptionC"),
            //(3, "OptionD"),
            //(4, "OptionE"),
            //(5, "OptionF"),
            (0, "OpA"),
            (1, "OpB"),
            (2, "OpC"),
            (3, "OpD"),
            (4, "OpE"),
            (5, "OpF"),
            (6, "OptionG"),
            (7, "OptionH"),
            (8, "OptionI"),
            (9, "OptionJ"),
            (10, "OptionK"),
            (11, "OptionL"),
        ],
    )
    .at(DynVal::new_flex(0.6), DynVal::new_flex(0.7));
    let _ = el.add_element(Box::new(dial1));

    tui.run(Box::new(el)).await
}
