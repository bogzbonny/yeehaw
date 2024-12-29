use yeehaw::*;

pub fn widgets_demo(ctx: &Context) -> Box<dyn Element> {
    let el = ParentPaneOfSelectable::new(ctx).with_bg(Color::MIDNIGHT_BLUE);

    let col1_width = DynVal::QUARTER.minus(1.into()).with_min(17);
    let col2_width = DynVal::QUARTER.minus(1.into()).with_min(19);
    let col3_width = DynVal::HALF.minus(1.into()).with_min(32);

    // fill dd entries with 20 items
    let dd_entries = (1..=20)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();

    let x_min = 1;
    let y = DynVal::new_fixed(2);
    let y_padding = DynVal::new_flex(0.1).with_min(2);

    let dropdown = DropdownList::new(ctx, dd_entries, Box::new(|_, _| EventResponses::default()))
        .with_max_expanded_height(10)
        .with_end_x(col1_width.clone().minus(1.into()))
        .at(x_min, y);
    let y = DynVal::y_after(&dropdown, y_padding.clone());
    el.add_element(Box::new(dropdown.label(ctx, "dropdown-list:")));
    el.add_element(Box::new(dropdown));

    let rbs = RadioButtons::new(
        ctx,
        vec![" wotz".to_string(), " op".to_string(), " dok".to_string()],
    )
    .at(x_min + 1, y.clone());
    el.add_element(Box::new(
        rbs.label(ctx, "radio buttons:")
            .at(x_min, y.minus(1.into())),
    ));
    let y = DynVal::y_after(&rbs, y_padding.clone());
    el.add_element(Box::new(rbs));

    let toggle = Toggle::new(ctx, " ★ ".to_string(), " ⏾ ".to_string()).at(x_min, y);
    let y = DynVal::y_after(&toggle, y_padding.clone());
    el.add_element(Box::new(toggle.label(ctx, "toggle:")));
    el.add_element(Box::new(toggle));

    let cb = Checkbox::new(ctx).at(x_min, y);
    let y = DynVal::y_after(&cb, y_padding.clone());
    el.add_element(Box::new(cb.label(ctx, " checkbox")));
    el.add_element(Box::new(cb));

    let button_click_fn = Box::new(move |_, _| EventResponses::default());
    let button = Button::new(ctx, "button", button_click_fn).at(x_min, y);
    el.add_element(Box::new(button));

    let y = DynVal::new_fixed(3);
    let x = col1_width.plus(2.into());
    let end_x = x.plus(col2_width.clone()).minus(1.into());
    let lb_height = DynVal::new_flex(0.13).with_min(5);

    let lb_entries = (1..=10)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();
    let listbox = ListBox::new(ctx, lb_entries, Box::new(|_, _| EventResponses::default()))
        .with_selection_mode(ctx, listbox::SelectionMode::UpTo(3))
        .with_dyn_height(ctx, lb_height.clone())
        .with_scrollbar(ctx)
        .at(x.clone(), y.clone())
        .with_end_x(end_x.clone());
    el.add_element(Box::new(
        listbox.label(ctx, "listbox (ex. \nselect up to 3):"),
    ));
    let y = DynVal::y_after(&listbox, y_padding.plus(1.into()).clone());
    el.add_element(Box::new(listbox));

    let ntb = NumbersTextBox::new(ctx, 0f64)
        .with_min(0.0)
        .with_max(1.0)
        .with_decimal_places(2)
        .with_dyn_width(col2_width.clone().div(2.))
        .at(x.clone(), y.clone());
    let y = DynVal::y_after(&ntb, y_padding.clone());
    el.add_element(Box::new(ntb.clone()));
    el.add_element(Box::new(
        ntb.label(ctx, "numbers-textbox:\n(linked to slider)"),
    ));

    let ntb_ = ntb.clone();
    let slider = Slider::new_basic_block(ctx)
        .with_gradient(ctx, Color::AQUA, Color::ORANGE)
        .at(x.clone(), y.clone())
        .with_end_x(end_x)
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            ntb_.set_value(p);
            EventResponses::default()
        }));
    let slider_ = slider.clone();
    ntb.set_value_changed_hook(Box::new(move |v| {
        slider_.set_position(v);
        EventResponses::default()
    }));
    let y = DynVal::y_after(&slider, 0.09).minus(1.into());
    el.add_element(Box::new(slider.label(ctx, "slider:")));
    el.add_element(Box::new(slider));

    let dial_entries = (0..8).map(|i| (i, " ")).collect::<Vec<(usize, &str)>>();

    let dial1 = Dial::new_ultra_compact(ctx, dial_entries.clone()).at(x.clone(), y.clone());
    let x_dial2 = x.plus(8.into());
    let dial2 = Dial::new_ultra_compact(ctx, dial_entries.clone()).at(x_dial2, y);

    let y = DynVal::y_after(&dial1, 0);
    let x_smile = x.plus(3.into());
    let happy = r#"
\________/"#;
    let excit = r#" ________
 \______/"#;
    let sad = r#"
/‾‾‾‾‾‾‾‾\"#;
    let smile_label = Label::new(ctx, happy).at(x_smile, y);

    let smile_label_ = smile_label.clone();
    let dial2_ = dial2.clone();
    dial1.set_fn(Box::new(move |ctx, _, pos, _| {
        let _ = dial2_.set_position(&ctx, pos, false);
        match pos {
            2 => smile_label_.set_text(excit.to_string()),
            4 => smile_label_.set_text(sad.to_string()),
            _ => smile_label_.set_text(happy.to_string()),
        }
        EventResponses::default()
    }));

    let smile_label_ = smile_label.clone();
    let dial1_ = dial1.clone();
    dial2.set_fn(Box::new(move |ctx, _, pos, _| {
        let _ = dial1_.set_position(&ctx, pos, false);
        match pos {
            2 => smile_label_.set_text(excit.to_string()),
            4 => smile_label_.set_text(sad.to_string()),
            _ => smile_label_.set_text(happy.to_string()),
        }
        EventResponses::default()
    }));

    el.add_element(Box::new(dial1));
    el.add_element(Box::new(dial2));
    el.add_element(Box::new(smile_label));

    let x = x.plus(col2_width).plus(1.into());
    let y = DynVal::new_fixed(4);
    let tb_width = col3_width.clone().minus(3.into());
    let tb_height = DynVal::new_flex(0.25).with_min(8);

    let tb = TextBox::new(ctx, "")
        .with_text_when_empty("enter text here")
        .with_dyn_width(tb_width)
        .with_dyn_height(tb_height.clone())
        .with_bottom_scrollbar(ctx)
        .with_left_scrollbar(ctx)
        .with_line_numbers(ctx)
        .editable(ctx)
        .with_no_wordwrap(ctx)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(
        tb.label(ctx, "basic textbox:\n(try right-clicking \non some text)"),
    ));
    let y = DynVal::y_after(&tb, DynVal::new_flex(0.09).with_min(1)).minus(1.into());
    el.add_element(Box::new(tb));

    let desc_text = "This is selectable pane, try\n\
                     clicking/scrolling around with\n\
                     the mouse! try using tabs to\n\
                     switch between different\n\
                     widgets, other keys (arrow,\n\
                     enter, esc, etc.) will also\n\
                     interact with widgets.";
    let description = Label::new(ctx, desc_text);
    el.add_element(Box::new(
        Bordered::new_basic(
            ctx,
            Box::new(description),
            Style::transparent().with_fg(Color::WHITE),
        )
        .with_dyn_height(DynVal::new_fixed(9))
        .with_dyn_width(DynVal::new_fixed(32))
        .at(x, y.clone()),
    ));

    Box::new(el)
}
