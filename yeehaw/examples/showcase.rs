use yeehaw::*;

/// NOTE this example requires steam locomotive (`sl`) to be installed if you want
/// to see the train. Additionally it should be run from this showcase directory
/// for the showcase-tab to work (which runs `cargo run --release --example showcase`).

#[tokio::main]
async fn main() -> Result<(), Error> {
    // uncomment the following line to enable logging
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let main = PaneScrollable::new_expanding(&ctx, 123, 35);
    let limiter = PaneLimiter::new(Box::new(main.clone()), 123, 35);
    let main_vs = VerticalStackFocuser::new(&ctx);
    main.add_element(Box::new(main_vs.clone()));

    // adding the menu bar and menu items

    let mb = MenuBar::top_menu_bar(&ctx).at(0, 0);
    for i in 0..3 {
        mb.add_item(&ctx, format!("upper/item-{i}"), None);
        mb.add_item(&ctx, format!("menu/item-{i}"), None);
        mb.add_item(&ctx, format!("bar/item-{i}"), None);
    }
    for j in 0..10 {
        let i = 3;
        mb.add_item(&ctx, format!("menu/item-{i}/sub-item-{j}"), None);
        mb.add_item(&ctx, format!("bar/item-{i}/sub-item-{j}"), None);
    }
    for j in 0..10 {
        for k in 0..3 {
            let i = 3;
            mb.add_item(
                &ctx,
                format!("upper/item-{i}/sub-{i}-item-{j}/sub-{i}-{j}-item-{k}"),
                None,
            );
        }
    }
    main_vs.push(Box::new(mb));

    let header_pane = ParentPaneOfSelectable::new(&ctx)
        .with_dyn_height(DynVal::new_fixed(7))
        .with_focused(false);
    main_vs.push(Box::new(header_pane.clone()));

    let gr = Gradient::x_grad_rainbow(5).with_angle(60.);
    let mtext = FigletText::new(
        &ctx,
        "YEEEHAW!!",
        figlet_rs::FIGfont::from_content(std::include_str!("../../assets/figlet/ANSI_Shadow.flf"))
            .expect("missing asset"),
    )
    .with_min_height()
    .with_style(Style::default().with_fg(Color::Gradient(gr)))
    .at(0, DynVal::new_fixed(1));
    header_pane.add_element(Box::new(mtext));

    let button = Button::new(
        &ctx,
        "do not\nclick me",
        Box::new(|_, _| EventResponses::default()),
    )
    .at(DynVal::new_flex(0.9), DynVal::new_flex(0.3));
    header_pane.add_element(Box::new(button));

    let central_pane = HorizontalStackFocuser::new(&ctx);
    main_vs.push(Box::new(central_pane.clone()));
    let left_pane = VerticalStackFocuser::new(&ctx)
        .with_dyn_width(0.7)
        .with_bg(Color::BLACK);
    central_pane.push(Box::new(left_pane.clone()));

    // need to generate the context for the main_vs pane
    // for upward propogation of events from the main_vs element
    let main_vs_ctx = ctx.child_context(&main_vs.get_dyn_location());

    left_pane.push(window_generation_zone(
        &main_vs_ctx,
        Box::new(main_vs.pane.pane.clone()),
    ));

    //let train_pane = DebugSizePane::new(&ctx);
    let train_pane = TerminalPane::new(&ctx)?;
    train_pane.pane.set_dyn_height(0.7);
    train_pane.pane.set_focused(false);
    train_pane.execute_command("for i in {1..1}; do sl -l; done ; exit");
    train_pane.disable_cursor();
    left_pane.push(Box::new(train_pane));

    let tabs = Tabs::new(&ctx);
    tabs.pane.set_focused(false);
    let widgets_tab = widgets_demo(&ctx);
    //let widgets_tab = Box::new(
    //    DebugSizePane::new(&ctx)
    //        .with_bg(Color::RED)
    //        .with_text("widgets".to_string()),
    //);
    let el2 = DebugSizePane::new(&ctx)
        .with_bg(Color::BLUE)
        .with_text("tab 2".to_string());
    let el3 = DebugSizePane::new(&ctx)
        .with_bg(Color::GREEN)
        .with_text("tab 3".to_string());
    let el4 = DebugSizePane::new(&ctx)
        .with_bg(Color::PINK)
        .with_text("tab 4".to_string());
    let el5 = TerminalPane::new(&ctx)?;

    let showcase = TerminalPane::new(&ctx)?;
    showcase.pane.set_focused(false);

    let showcase_ = showcase.clone();
    let on_showcase_open_fn = Some(Box::new(move || {
        let command = "cargo run --release --example showcase";
        showcase_.execute_command(command);
    }) as Box<dyn FnOnce()>);

    tabs.push(widgets_tab, "widgets");
    tabs.push(Box::new(el2), "colors");
    tabs.push(Box::new(el3), "images");
    tabs.push(Box::new(el4), "file-nav");
    tabs.push(Box::new(el5), "terminal");
    tabs.push_with_on_open_fn(Box::new(showcase), "showcase", on_showcase_open_fn);
    tabs.select(0);
    central_pane.push(Box::new(tabs));

    //tui.run(Box::new(main)).await
    tui.run(Box::new(limiter)).await
}

pub fn window_generation_zone(
    ctx: &Context, windows_generated_in: Box<ParentPane>,
) -> Box<dyn Element> {
    let sc = PaneScrollable::new_expanding(ctx, 50, 16);
    let el = ParentPaneOfSelectable::new(ctx);
    sc.add_element(Box::new(el.clone()));
    let bordered = Bordered::new_resizer(
        ctx,
        Box::new(sc.clone()),
        Style::transparent().with_fg(Color::WHITE),
    )
    .with_dyn_height(1.5);

    let l = Label::new(ctx, "window generation zone");
    el.add_element(Box::new(l));

    let dial_type = Dial::new_spacious(
        ctx,
        vec![
            (0, "Basic"),
            (1, "Fixed-Size"),
            (2, "Min-Size"),
            (3, "Terminal"),
        ],
    )
    .at(0, 3);
    let label = dial_type.label(ctx, "content:").underlined();
    el.add_element(Box::new(label));
    el.add_element(Box::new(dial_type.clone()));

    let dial_border = Dial::new_spacious(
        ctx,
        vec![
            (0, "None"),
            (1, "Basic"),
            (2, "Large"),
            (3, "Tight"),
            (6, "Double-Line"),
            (4, "Scrollbars"),
            (5, "Line-Scrollbars"),
            (7, "Text"),
            (8, "Resizer"),
            (9, "Mover"),
        ],
    )
    .at(DynVal::new_flex_with_min_fixed(0.35, 19), 3);
    let label = dial_border.label(ctx, "border options:").underlined();
    el.add_element(Box::new(label));
    el.add_element(Box::new(dial_border.clone()));

    let shadow_cb = Checkbox::new(ctx).at(1, 8);
    let label = shadow_cb.label(ctx, "shadow");
    el.add_element(Box::new(shadow_cb.clone()));
    el.add_element(Box::new(label));

    let alpha_slider = Slider::new_basic_block(ctx)
        .with_gradient(Color::BLUE, Color::AQUA)
        .with_position(0.9)
        .with_width(DynVal::new_flex(0.4))
        .at(1, 11);

    let label = alpha_slider.label(ctx, "background alpha:").underlined();
    el.add_element(Box::new(label));
    el.add_element(Box::new(alpha_slider.clone()));

    let counter = Rc::new(RefCell::new(0));

    let mut ctx_ = ctx.clone();
    let counter_ = counter.clone();

    let generate_window_fn = Box::new(move |_, _| {
        ctx_.size.width = 30;
        ctx_.size.height = 20;
        let title = format!("Pane {}", *counter_.borrow());

        let alpha = (alpha_slider.get_position() * 255.0) as u8;

        let bg = Color::new_with_alpha(150, 150, 155, alpha);
        let fg = Color::new_with_alpha(150, 150, 155, alpha);
        let sty = Style::default().with_bg(bg.clone()).with_fg(fg.clone());
        let def_ch = DrawCh::new(ChPlus::Transparent, sty.clone());

        let el: Box<dyn Element> = match dial_type.get_value().as_str() {
            "Basic" => Box::new(
                DebugSizePane::new(&ctx_)
                    .with_text(title.clone())
                    .with_text_style(sty.clone().with_fg(Color::BLACK))
                    .with_dyn_width(DynVal::FULL)
                    .with_dyn_height(DynVal::FULL)
                    .with_default_ch(def_ch.clone()),
            ),
            "Fixed-Size" => {
                let el = DebugSizePane::new(&ctx_)
                    .with_text(title.clone())
                    .with_text_style(sty.clone().with_fg(Color::BLACK))
                    .with_dyn_width(DynVal::FULL)
                    .with_dyn_height(DynVal::FULL)
                    .with_default_ch(def_ch.clone());
                let sc_pane = PaneScrollable::new(&ctx_, 50, 25);
                sc_pane.add_element(Box::new(el));
                Box::new(sc_pane)
            }
            "Min-Size" => {
                let el = DebugSizePane::new(&ctx_)
                    .with_text(title.clone())
                    .with_text_style(sty.clone().with_fg(Color::BLACK))
                    .with_dyn_width(DynVal::FULL)
                    .with_dyn_height(DynVal::FULL)
                    .with_default_ch(def_ch.clone());
                let sc_pane = PaneScrollable::new_expanding(&ctx_, 50, 25);
                sc_pane.add_element(Box::new(el));
                Box::new(sc_pane)
            }
            "Terminal" => Box::new(TerminalPane::new(&ctx_).unwrap()),
            _ => panic!("missing type implementation"),
        };

        let mut corner_resizer = false;
        let el: Box<dyn Element> = match dial_border.get_value().as_str() {
            "None" => {
                corner_resizer = true;
                el
            }
            "Basic" => Box::new(Bordered::new_basic(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Large" => Box::new(Bordered::new_large(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Tight" => Box::new(Bordered::new_tight(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Double-Line" => Box::new(Bordered::new_double(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Scrollbars" => {
                corner_resizer = true;
                Box::new(Bordered::new_borderless_with_scrollbars_and_thin_left(
                    &ctx_,
                    el,
                    sty.clone().with_fg(Color::WHITE),
                ))
            }
            "Line-Scrollbars" => Box::new(Bordered::new_resizer_with_scrollbars(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Text" => Box::new(
                Bordered::new_basic(&ctx_, el, sty.clone().with_fg(Color::BLACK))
                    .with_title("upper")
                    .with_bottom_right_text("lower")
                    .with_left_top_text("left")
                    .with_right_bottom_text("right"),
            ),
            "Resizer" => Box::new(Bordered::new_resizer(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Mover" => Box::new(Bordered::new_mover(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            _ => panic!("missing type implementation"),
        };

        *counter_.borrow_mut() += 1;
        let mut window = WindowPane::new(&ctx_, el, &title)
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30))
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10));

        if corner_resizer {
            window.set_corner_resizer(&ctx_);
        }

        let window: Box<dyn Element> = if shadow_cb.is_checked() {
            let shadow_color = Color::new_with_alpha(100, 100, 100, 150);
            Box::new(Shadowed::thick_with_color(
                &ctx_,
                Box::new(window.clone()),
                shadow_color,
            ))
        } else {
            Box::new(window)
        };

        let inner_resps = vec![
            EventResponse::BringToFront,
            EventResponse::UnfocusOthers,
            EventResponse::Focus,
        ];
        let resp = EventResponse::NewElement(window, Some(inner_resps.into()));
        windows_generated_in
            .pane
            .send_responses_upward(&ctx_, resp.into());

        EventResponses::default()
    });

    let button = Button::new(ctx, "generate", generate_window_fn).at(1, 13);
    el.add_element(Box::new(button));

    Box::new(bordered)
}

pub fn widgets_demo(ctx: &Context) -> Box<dyn Element> {
    let el = ParentPaneOfSelectable::new(ctx).with_bg(Color::MIDNIGHT_BLUE);

    // fill dd entries with 20 items
    let dd_entries = (1..=20)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();

    let x_min = 1;
    let mut y_min = 2;
    let dropdown = DropdownList::new(ctx, dd_entries, Box::new(|_, _| EventResponses::default()))
        .with_max_expanded_height(10)
        .with_width(
            DynVal::default()
                .plus_max_of(DynVal::new_flex(0.2))
                .plus_max_of(DynVal::new_fixed(12)),
        )
        .at(x_min, y_min);
    el.add_element(Box::new(dropdown.label(ctx, "dropdown-list:")));
    el.add_element(Box::new(dropdown));

    y_min += 3;
    let y = DynVal::new_flex_with_min_fixed(0.1, y_min);
    let rbs = RadioButtons::new(
        ctx,
        vec![" wotz".to_string(), " op".to_string(), " dok".to_string()],
    )
    .at(x_min, y);
    el.add_element(Box::new(rbs.label(ctx, "radio buttons:")));
    el.add_element(Box::new(rbs));

    y_min += 5;
    let y = DynVal::new_flex_with_min_fixed(0.15, y_min);
    let toggle = Toggle::new(ctx, " ★ ".to_string(), " ⏾ ".to_string()).at(x_min, y);
    el.add_element(Box::new(toggle.label(ctx, "toggle:")));
    el.add_element(Box::new(toggle));

    y_min += 3;
    let y = DynVal::new_flex_with_min_fixed(0.2, y_min);
    let cb = Checkbox::new(ctx).at(x_min, y);
    let cb_label = Label::new_for_el(ctx, cb.get_dyn_location_set().l.clone(), "check me");
    el.add_element(Box::new(cb));
    el.add_element(Box::new(cb_label));

    y_min += 2;
    let y = DynVal::new_flex_with_min_fixed(0.20, y_min);
    let cb2 = Checkbox::new(ctx).at(x_min, y);
    let cb2_label = Label::new_for_el(ctx, cb2.get_dyn_location_set().l.clone(), "check me");
    el.add_element(Box::new(cb2));
    el.add_element(Box::new(cb2_label));

    //---------------------------

    let l1 = Label::new(ctx, "some label");
    let l = l1.clone().at(DynVal::new_flex(0.5), DynVal::new_flex(0.5));
    el.add_element(Box::new(l));

    let button_click_fn = Box::new(move |_, _| {
        let t = l1.get_text();
        let t = t + "0";
        l1.set_text(t);
        EventResponses::default()
    });
    let button = Button::new(ctx, "click me", button_click_fn)
        .with_description("a button!".to_string())
        .at(DynVal::new_flex(0.25), DynVal::new_flex(0.25));
    let button_label =
        Label::new_for_el(ctx, button.get_dyn_location_set().l.clone(), "button-label");
    el.add_element(Box::new(button));
    el.add_element(Box::new(button_label));

    let button = Button::new(
        ctx,
        "do not\nclick me",
        Box::new(|_, _| EventResponses::default()),
    )
    .with_description("a button!".to_string())
    .at(DynVal::new_flex(0.25), DynVal::new_flex(0.15));
    el.add_element(Box::new(button));

    let button2 = Button::new(ctx, "button2", Box::new(|_, _| EventResponses::default()))
        .with_description("a button!".to_string())
        .with_sides(ButtonSides::default())
        .at(DynVal::new_flex(0.25), DynVal::new_flex(0.29));
    el.add_element(Box::new(button2));

    let button3 = Button::new(ctx, "b", Box::new(|_, _| EventResponses::default()))
        .with_description("a button!".to_string())
        .with_micro_shadow(ButtonMicroShadow::default())
        .at(DynVal::new_flex(0.25), DynVal::new_flex(0.10));
    el.add_element(Box::new(button3));

    let ld_entries = (1..=10)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();

    use yeehaw::elements::listbox::SelectionMode;
    let listbox = ListBox::new(ctx, ld_entries, Box::new(|_, _| EventResponses::default()))
        .with_selection_mode(ctx, SelectionMode::UpTo(3))
        .with_width(
            ctx,
            DynVal::default()
                .plus_max_of(DynVal::new_flex(0.15))
                .plus_max_of(DynVal::new_fixed(12)),
        )
        .with_height(
            ctx,
            DynVal::default()
                .plus_max_of(DynVal::new_flex(0.13))
                .plus_max_of(DynVal::new_fixed(5)),
        )
        .with_scrollbar(ctx)
        .at(DynVal::new_flex(0.5), DynVal::new_flex(0.1));
    el.add_element(Box::new(listbox));

    let tb = TextBox::new(
        ctx,
        "012345678901234567890123456789\
        \n1\n2\n3\n4\n5\n6\n7\n8\n9\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9",
        //\n1",
    )
    //.with_width(DynVal::new_fixed(20))
    //.with_height(DynVal::new_fixed(10))
    .with_width(DynVal::new_flex(0.1))
    .with_height(DynVal::new_flex(0.2))
    //.with_top_scrollbar(&ctx)
    .with_bottom_scrollbar(ctx)
    .with_left_scrollbar(ctx)
    .with_line_numbers(ctx)
    //.with_right_scrollbar(&ctx)
    .editable(ctx)
    .with_no_wordwrap(ctx)
    .at(DynVal::new_flex(0.35), DynVal::new_flex(0.1));
    el.add_element(Box::new(tb));

    let tb_with_grey = TextBox::new(ctx, "")
        .with_width(DynVal::new_fixed(18))
        .with_height(DynVal::new_fixed(1))
        .editable(ctx)
        .with_text_when_empty("enter text here")
        .with_no_wordwrap(ctx)
        .at(DynVal::new_flex(0.25), DynVal::new_fixed(3));
    el.add_element(Box::new(tb_with_grey));

    let ntb = NumbersTextBox::new(ctx, 0f64)
        .with_min(-10.0)
        .with_max(10.0)
        .at(DynVal::new_flex(0.75), DynVal::new_flex(0.5));
    el.add_element(Box::new(ntb.clone()));

    let ntb_ = ntb.clone();
    let slider = Slider::new_basic_block(ctx)
        .with_gradient(Color::AQUA, Color::ORANGE)
        .with_width(DynVal::new_flex(0.1))
        .at(DynVal::new_flex(0.8), DynVal::new_flex(0.5))
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            ntb_.change_value(p);
            EventResponses::default()
        }));
    el.add_element(Box::new(slider));

    let dial1 = Dial::new_semi_compact(
        ctx,
        vec![
            (0, "OptionA"),
            (1, "OptionB"),
            (2, "OptionC"),
            (3, "OptionD"),
            (4, "OptionE"),
            (5, "OptionF"),
            (6, "OptionG"),
            (7, "OptionH"),
            (8, "OptionI"),
            (9, "OptionJ"),
            (10, "OptionK"),
            (11, "OptionL"),
        ],
    )
    .at(DynVal::new_flex(0.6), DynVal::new_flex(0.7));
    el.add_element(Box::new(dial1));
    Box::new(el)
}
