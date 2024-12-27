use {std::time::Duration, yeehaw::*};

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
    let limiter = PaneLimiter::new(Box::new(main.clone()), 140, 40);
    //let limiter = PaneLimiter::new(Box::new(main.clone()), 1000, 1000);
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

    let gr = Gradient::x_grad_rainbow(&ctx, 5).with_angle(60.);
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
    train_pane.execute_command("for i in {1..20}; do sl -l; done ; exit");
    train_pane.disable_cursor();
    left_pane.push(Box::new(train_pane));

    let tabs = Tabs::new(&ctx);
    tabs.pane.set_focused(false);
    let colors_tab = colors_demo(&ctx);
    let widgets_tab = widgets_demo(&ctx);
    let el3 = DebugSizePane::new(&ctx)
        .with_bg(Color::GREEN)
        .with_text("tab 3".to_string());
    let el4 = DebugSizePane::new(&ctx)
        .with_bg(Color::PINK)
        .with_text("tab 4".to_string());
    let el_term = TerminalPane::new(&ctx)?;

    let showcase = TerminalPane::new(&ctx)?;
    showcase.pane.set_focused(false);

    let showcase_ = showcase.clone();
    let on_showcase_open_fn = Some(Box::new(move || {
        let command = "cargo run --release --example showcase";
        showcase_.execute_command(command);
    }) as Box<dyn FnOnce()>);

    tabs.push(colors_tab, "colors");
    tabs.push(widgets_tab, "widgets");
    tabs.push(Box::new(el3), "$EDITOR");
    tabs.push(Box::new(el4), "images");
    tabs.push(Box::new(el_term), "terminal");
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
    let label = dial_type.label(ctx, "content:");
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
    let label = dial_border.label(ctx, "border options:");
    el.add_element(Box::new(label));
    el.add_element(Box::new(dial_border.clone()));

    let shadow_cb = Checkbox::new(ctx).at(1, 8);
    let label = shadow_cb.label(ctx, " shadow");
    el.add_element(Box::new(shadow_cb.clone()));
    el.add_element(Box::new(label));

    let alpha_slider = Slider::new_basic_block(ctx)
        .with_gradient(ctx, Color::BLUE, Color::AQUA)
        .with_position(0.9)
        .with_width(DynVal::new_flex(0.4))
        .at(1, 11);

    let label = alpha_slider.label(ctx, "background alpha:");
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
    let dd_width = DynVal::default()
        .plus_max_of(DynVal::new_flex(0.2))
        .plus_max_of(DynVal::new_fixed(12));
    let dropdown = DropdownList::new(ctx, dd_entries, Box::new(|_, _| EventResponses::default()))
        .with_max_expanded_height(10)
        .with_width(dd_width.clone())
        .at(x_min, y_min);
    el.add_element(Box::new(dropdown.label(ctx, "dropdown-list:")));
    el.add_element(Box::new(dropdown));

    y_min += 3;
    let y = DynVal::new_flex_with_min_fixed(0.25, y_min);
    let rbs = RadioButtons::new(
        ctx,
        vec![" wotz".to_string(), " op".to_string(), " dok".to_string()],
    )
    .at(x_min + 1, y.clone());
    el.add_element(Box::new(
        rbs.label(ctx, "radio buttons:")
            .at(x_min, y.minus(1.into())),
    ));
    el.add_element(Box::new(rbs));

    y_min += 5;
    let y = DynVal::new_flex_with_min_fixed(0.4, y_min);
    let toggle = Toggle::new(ctx, " ★ ".to_string(), " ⏾ ".to_string()).at(x_min, y);
    el.add_element(Box::new(toggle.label(ctx, "toggle:")));
    el.add_element(Box::new(toggle));

    y_min += 3;
    let y = DynVal::new_flex_with_min_fixed(0.6, y_min);
    let cb = Checkbox::new(ctx).at(x_min, y);
    el.add_element(Box::new(cb.label(ctx, " checkbox-1")));
    el.add_element(Box::new(cb));

    y_min += 2;
    let y = DynVal::new_flex_with_min_fixed(0.6, y_min).plus(2.into());
    let cb2 = Checkbox::new(ctx).at(x_min, y);
    el.add_element(Box::new(cb2.label(ctx, " checkbox-2")));
    el.add_element(Box::new(cb2));

    y_min += 2;
    let y = DynVal::new_flex_with_min_fixed(0.8, y_min);
    let button_click_fn = Box::new(move |_, _| EventResponses::default());
    let button = Button::new(ctx, "button", button_click_fn).at(x_min, y);
    el.add_element(Box::new(button));

    let y = DynVal::new_fixed(3);
    let x = DynVal::default()
        .plus_max_of(0.25.into())
        .plus_max_of(dd_width.plus(3.into()));
    let lb_width = DynVal::default()
        .plus_max_of(DynVal::new_flex(0.15))
        .plus_max_of(DynVal::new_fixed(16));
    let lb_height = DynVal::default()
        .plus_max_of(DynVal::new_flex(0.13))
        .plus_max_of(DynVal::new_fixed(5));

    let lb_entries = (1..=10)
        .map(|i| format!("entry {}", i))
        .collect::<Vec<String>>();
    let listbox = ListBox::new(ctx, lb_entries, Box::new(|_, _| EventResponses::default()))
        .with_selection_mode(ctx, listbox::SelectionMode::UpTo(3))
        .with_width(ctx, lb_width.clone())
        .with_height(ctx, lb_height.clone())
        .with_scrollbar(ctx)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(
        listbox.label(ctx, "listbox (ex. \nselect up to 3):"),
    ));
    el.add_element(Box::new(listbox));

    let y = y.plus(lb_height).plus(3.into());
    let ntb_width = DynVal::default()
        .plus_max_of(DynVal::new_flex(0.10))
        .plus_max_of(DynVal::new_fixed(8));
    let ntb = NumbersTextBox::new(ctx, 0f64)
        .with_min(0.0)
        .with_max(1.0)
        .with_decimal_places(2)
        .with_width(ntb_width)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(ntb.clone()));
    el.add_element(Box::new(
        ntb.label(ctx, "numbers-textbox:\n(linked to slider)"),
    ));

    let y = y.plus(3.into());
    let ntb_ = ntb.clone();
    let slider = Slider::new_basic_block(ctx)
        .with_gradient(ctx, Color::AQUA, Color::ORANGE)
        .with_width(lb_width.clone())
        .at(x.clone(), y.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            ntb_.change_value(p);
            EventResponses::default()
        }));
    let slider_ = slider.clone();
    ntb.set_value_changed_hook(Box::new(move |v| {
        slider_.set_position(v);
        EventResponses::default()
    }));
    el.add_element(Box::new(slider.label(ctx, "slider:")));
    el.add_element(Box::new(slider));

    let dial_entries = (0..8).map(|i| (i, " ")).collect::<Vec<(usize, &str)>>();

    let y = y.plus(2.into());
    let dial1 = Dial::new_ultra_compact(ctx, dial_entries.clone()).at(x.clone(), y.clone());
    let x_dial2 = x.plus(8.into());
    let dial2 = Dial::new_ultra_compact(ctx, dial_entries.clone()).at(x_dial2, y.clone());

    let y = y.plus(4.into());
    let x_smile = x.plus(3.into());
    let happy = r#"
\________/"#;
    let excit = r#" ________
 \______/"#;
    let sad = r#"
/‾‾‾‾‾‾‾‾\"#;
    let smile_label = Label::new(ctx, happy).at(x_smile, y.clone());

    let smile_label_ = smile_label.clone();
    let dial2_ = dial2.clone();
    dial1.set_fn(Box::new(move |ctx, _, pos, _| {
        let _ = dial2_.set_position(&ctx, pos);
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
        let _ = dial1_.set_position(&ctx, pos);
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

    //let smile_label = Label::new(ctx, happy).at(x_smile, y.clone());

    let x = x.plus(lb_width).plus(4.into());
    let y = DynVal::new_fixed(4);
    let tb_width = DynVal::default()
        .plus_max_of(DynVal::new_flex(0.25))
        .plus_max_of(DynVal::new_fixed(12));
    let tb_height = DynVal::default()
        .plus_max_of(DynVal::new_flex(0.25))
        .plus_max_of(DynVal::new_fixed(8));

    let tb = TextBox::new(ctx, "")
        .with_text_when_empty("enter text here")
        .with_width(tb_width)
        .with_height(tb_height.clone())
        .with_bottom_scrollbar(ctx)
        .with_left_scrollbar(ctx)
        .with_line_numbers(ctx)
        .editable(ctx)
        .with_no_wordwrap(ctx)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(
        tb.label(ctx, "basic textbox:\n(try right-clicking \non some text)"),
    ));
    el.add_element(Box::new(tb));

    let y = y.plus(tb_height).plus(2.into());
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

pub fn colors_demo(ctx: &Context) -> Box<dyn Element> {
    let el = ParentPaneOfSelectable::new(ctx).with_bg(Color::DARK_OLIVE_GREEN);
    //pub const DARK_OLIVE_GREEN:        Color = Color::new(85, 107, 47);
    //let el = ParentPaneOfSelectable::new(ctx).with_bg(Color::new(50, 50, 50));
    //let el = ParentPaneOfSelectable::new(ctx).with_bg(Color::MEDIUM_SEA_GREEN);
    //let el = ParentPaneOfSelectable::new(ctx).with_bg(Color::PALE_VIOLET_RED);
    //let el = ParentPaneOfSelectable::new(ctx).with_bg(Color::STEEL_BLUE);
    //let el = ParentPaneOfSelectable::new(ctx).with_bg(Color::NAVY);

    let x = DynVal::new_fixed(1);
    let y = DynVal::new_fixed(1);
    let desc_text = "Within yeehaw, a standard\n\
                     color can be an RGBA or a\n\
                     gradient changing by position\n\
                     or time (or both!).";
    let description = Label::new(ctx, desc_text).at(x.clone(), y.clone());
    let y = DynVal::y_after(&description, 1);
    el.add_element(Box::new(description));

    let x_tog = DynVal::new_fixed(18);
    let toggle = Toggle::new(ctx, "  fg  ".to_string(), "  bg  ".to_string()).at(x_tog, y.clone());
    let y = DynVal::y_after(&toggle, 0);
    el.add_element(Box::new(toggle.clone()));

    let dial_color = Dial::new_spacious(
        ctx,
        vec![
            (0, "Solid"),
            (1, "Time-Gradient"),
            (2, "Radial-Gradient"),
            (3, "Linear-Gradient"),
            (4, "Radial-Time"),
            (5, "Linear-Time"),
            (6, "Tiles"),
        ],
    )
    .with_label_color(ctx, Color::GREY22)
    .at(x.clone(), y.clone());
    let y = DynVal::y_after(&dial_color, 1);
    el.add_element(Box::new(dial_color.label(ctx, "color kind:")));
    el.add_element(Box::new(dial_color.clone()));

    let dd_x = x.plus(7.into());
    let color_dd = DropdownList::new(
        ctx,
        vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"],
        Box::new(|_, _| EventResponses::default()),
    )
    .with_width(5.into())
    .with_max_expanded_height(8)
    .at(dd_x, y.clone());
    let y = DynVal::y_after(&color_dd, 0);
    el.add_element(Box::new(color_dd.label_left(ctx, "color (")));
    el.add_element(Box::new(color_dd.label_right(ctx, "):")));
    el.add_element(Box::new(color_dd.clone()));

    let x_nb = x.plus(3.into());
    let ntb_width = DynVal::new_fixed(8);
    let r_ntb = NumbersTextBox::new(ctx, 128usize)
        .with_min(0)
        .with_max(255)
        .with_width(ntb_width)
        .at(x_nb.clone(), y.clone());
    el.add_element(Box::new(r_ntb.clone()));
    el.add_element(Box::new(r_ntb.label_left_top(ctx, "R: ")));
    let x_slider = x_nb.plus(9.into());
    let slider_width = DynVal::new_fixed(17);
    let ntb_ = r_ntb.clone();
    let r_slider = Slider::new_basic_block(ctx)
        .with_color(Color::RED)
        .with_width(slider_width.clone())
        .at(x_slider.clone(), y.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 255.) as usize;
            ntb_.change_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&r_slider, 0);
    el.add_element(Box::new(r_slider.clone()));

    let x_nb = x.plus(3.into());
    let ntb_width = DynVal::new_fixed(8);
    let g_ntb = NumbersTextBox::new(ctx, 128usize)
        .with_min(0)
        .with_max(255)
        .with_width(ntb_width)
        .at(x_nb.clone(), y.clone());
    el.add_element(Box::new(g_ntb.clone()));
    el.add_element(Box::new(g_ntb.label_left_top(ctx, "G: ")));
    let x_slider = x_nb.plus(9.into());
    let slider_width = DynVal::new_fixed(17);
    let ntb_ = g_ntb.clone();
    let g_slider = Slider::new_basic_block(ctx)
        .with_color(Color::GREEN)
        .with_width(slider_width.clone())
        .at(x_slider.clone(), y.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 255.) as usize;
            ntb_.change_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&g_slider, 0);
    el.add_element(Box::new(g_slider.clone()));

    let x_nb = x.plus(3.into());
    let ntb_width = DynVal::new_fixed(8);
    let b_ntb = NumbersTextBox::new(ctx, 128usize)
        .with_min(0)
        .with_max(255)
        .with_width(ntb_width)
        .at(x_nb.clone(), y.clone());
    el.add_element(Box::new(b_ntb.clone()));
    el.add_element(Box::new(b_ntb.label_left_top(ctx, "B: ")));
    let x_slider = x_nb.plus(9.into());
    let slider_width = DynVal::new_fixed(17);
    let ntb_ = b_ntb.clone();
    let b_slider = Slider::new_basic_block(ctx)
        .with_color(Color::BLUE)
        .with_width(slider_width.clone())
        .at(x_slider.clone(), y.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 255.) as usize;
            ntb_.change_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&b_slider, 0);
    el.add_element(Box::new(b_slider.clone()));

    let x_nb = x.plus(3.into());
    let ntb_width = DynVal::new_fixed(8);
    let a_ntb = NumbersTextBox::new(ctx, 128usize)
        .with_min(0)
        .with_max(255)
        .with_width(ntb_width)
        .at(x_nb.clone(), y.clone());
    el.add_element(Box::new(a_ntb.clone()));
    el.add_element(Box::new(a_ntb.label_left_top(ctx, "A: ")));
    let x_slider = x_nb.plus(9.into());
    let slider_width = DynVal::new_fixed(17);
    let ntb_ = a_ntb.clone();
    let a_slider = Slider::new_basic_block(ctx)
        .with_color(Color::AQUA)
        .with_width(slider_width.clone())
        .at(x_slider.clone(), y.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 255.) as usize;
            ntb_.change_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&a_slider, 1);
    el.add_element(Box::new(a_slider.clone()));

    r_slider.set_position(0.5);
    g_slider.set_position(0.5);
    b_slider.set_position(0.5);
    a_slider.set_position(0.5);

    let dd_x = x.plus(22.into());
    let max_gr_colors_dd = DropdownList::new(
        ctx,
        vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"],
        Box::new(|_, _| EventResponses::default()),
    )
    .with_width(5.into())
    .with_max_expanded_height(8)
    .at(dd_x, y.clone());
    let y = DynVal::y_after(&max_gr_colors_dd, 2);
    el.add_element(Box::new(
        max_gr_colors_dd.label_left(ctx, "# of gradient colors: "),
    ));
    el.add_element(Box::new(max_gr_colors_dd.clone()));

    let ntb_width = DynVal::new_fixed(8);
    let dist_ntb = NumbersTextBox::new(ctx, 50usize)
        .with_min(0)
        .with_max(100)
        .with_width(ntb_width)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(dist_ntb.clone()));
    el.add_element(Box::new(dist_ntb.label(ctx, "gradient color distance:")));
    let x_slider = x.plus(9.into());
    let slider_width = DynVal::new_fixed(20);
    let ntb_ = dist_ntb.clone();
    let dist_slider = Slider::new_basic_block(ctx)
        .with_color(Color::AQUA)
        .with_width(slider_width.clone())
        .at(x_slider.clone(), y.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 100.) as usize;
            ntb_.change_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&dist_slider, 2);
    el.add_element(Box::new(dist_slider.clone()));

    let ntb_width = DynVal::new_fixed(8);
    let angle_ntb = NumbersTextBox::new(ctx, 0f64)
        .with_min(0.)
        .with_max(360.)
        .with_decimal_places(2)
        .with_width(ntb_width)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(angle_ntb.clone()));
    el.add_element(Box::new(angle_ntb.label(ctx, "linear_gradient angle:")));
    let x_slider = x.plus(9.into());
    let slider_width = DynVal::new_fixed(20);
    let ntb_ = angle_ntb.clone();
    let angle_slider = Slider::new_basic_block(ctx)
        .with_color(Color::AQUA)
        .with_width(slider_width.clone())
        .at(x_slider.clone(), y.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = p * 360.;
            ntb_.change_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&angle_slider, 2);
    el.add_element(Box::new(angle_slider.clone()));

    let ntb_width = DynVal::new_fixed(8);
    let time_ntb = NumbersTextBox::new(ctx, 1000usize)
        .with_min(0)
        .with_max(5000)
        .with_width(ntb_width)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(time_ntb.clone()));
    el.add_element(Box::new(time_ntb.label(ctx, "time gradient ms:")));
    let x_slider = x.plus(9.into());
    let slider_width = DynVal::new_fixed(20);
    let ntb_ = time_ntb.clone();
    let time_slider = Slider::new_basic_block(ctx)
        .with_color(Color::AQUA)
        .with_width(slider_width.clone())
        .at(x_slider.clone(), y.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 5000.) as usize;
            ntb_.change_value(p);
            EventResponses::default()
        }));
    el.add_element(Box::new(time_slider.clone()));

    // ------------------------------

    let y_art = DynVal::new_fixed(0);
    let x_art = DynVal::x_after(&toggle, 1);
    let bg_art = ParentPane::new(ctx, "art");
    let fg_art = Label::new(ctx, "");
    bg_art.add_element(Box::new(fg_art.clone()));

    let art = Bordered::new_tight(
        ctx,
        Box::new(bg_art.clone()),
        Style::transparent().with_fg(Color::WHITE),
    )
    .with_dyn_height(DynVal::new_fixed(26))
    .with_dyn_width(DynVal::new_fixed(51))
    .at(x_art.clone(), y_art.clone());
    let y_dial_fg_art = DynVal::y_after(&art, 1);
    let x_dial_fg_art = x_art.plus(12.into());

    el.add_element(Box::new(art));

    let dial_fg_art = Dial::new_spacious(
        ctx,
        vec![
            (1, "None"),
            (2, "Butterfly"),
            (3, "Spiral"),
            (8, "Rust-Logo"),
            (9, "Saturn"),
            (10, "Chompy"),
        ],
    )
    .with_position(2)
    .with_label_color(ctx, Color::GREY22)
    .at(x_dial_fg_art, y_dial_fg_art);
    el.add_element(Box::new(dial_fg_art.label_left_top(ctx, "fg text:")));
    el.add_element(Box::new(dial_fg_art.clone()));

    let state = ColorsDemoState {
        fg: Rc::new(RefCell::new(ColorsDemoColor::default_fg(ctx))),
        bg: Rc::new(RefCell::new(ColorsDemoColor::default_bg(ctx))),
        updating: Rc::new(RefCell::new(false)),
        drawing_fg: fg_art,
        drawing_bg: bg_art,
        toggle: toggle.clone(),
        dial_fg_art: dial_fg_art.clone(),
        dial_color_kind: dial_color.clone(),
        color_dd: color_dd.clone(),
        max_gr_colors_dd: max_gr_colors_dd.clone(),
        dist_ntb: dist_ntb.clone(),
        angle_ntb: angle_ntb.clone(),
        time_ntb: time_ntb.clone(),
        r_ntb: r_ntb.clone(),
        g_ntb: g_ntb.clone(),
        b_ntb: b_ntb.clone(),
        a_ntb: a_ntb.clone(),
    };

    let state_ = state.clone();
    dial_fg_art.set_fn(Box::new(move |_, _, _, _| {
        state_.update_drawing();
        EventResponses::default()
    }));

    let state_ = state.clone();
    toggle.set_fn(Box::new(move |ctx, _| {
        state_.update_for_toggle_change(&ctx);
        EventResponses::default()
    }));

    let state_ = state.clone();
    dial_color.set_fn(Box::new(move |ctx, _, _, _| {
        state_.update_for_color_dial_change(&ctx);
        EventResponses::default()
    }));

    let state_ = state.clone();
    color_dd.set_fn(Box::new(move |_, _| {
        state_.update_for_color_dd_change();
        state_.update_for_minor_changes();
        EventResponses::default()
    }));

    let state_ = state.clone();
    max_gr_colors_dd.set_fn(Box::new(move |_, _| {
        state_.update_for_minor_changes();
        EventResponses::default()
    }));

    let state_ = state.clone();
    let dist_slider_ = dist_slider.clone();
    dist_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 100.;
        dist_slider_.set_position(v);
        state_.update_for_minor_changes();
        EventResponses::default()
    }));

    let state_ = state.clone();
    let angle_slider_ = angle_slider.clone();
    angle_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v / 360.;
        angle_slider_.set_position(v);
        state_.update_for_minor_changes();
        EventResponses::default()
    }));

    let state_ = state.clone();
    let time_slider_ = time_slider.clone();
    time_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 5000.;
        time_slider_.set_position(v);
        state_.update_for_minor_changes();
        EventResponses::default()
    }));

    let state_ = state.clone();
    let r_slider_ = r_slider.clone();
    r_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 255.;
        r_slider_.set_position(v);
        state_.update_for_minor_changes();
        EventResponses::default()
    }));

    let state_ = state.clone();
    let g_slider_ = g_slider.clone();
    g_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 255.;
        g_slider_.set_position(v);
        state_.update_for_minor_changes();
        EventResponses::default()
    }));

    let state_ = state.clone();
    let b_slider_ = b_slider.clone();
    b_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 255.;
        b_slider_.set_position(v);
        state_.update_for_minor_changes();
        EventResponses::default()
    }));

    let state_ = state.clone();
    let a_slider_ = a_slider.clone();
    a_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 255.;
        a_slider_.set_position(v);
        state_.update_for_minor_changes();
        EventResponses::default()
    }));

    state.update_for_toggle_change(ctx);
    Box::new(el)
}

#[derive(Clone)]
pub struct ColorsDemoState {
    pub fg: Rc<RefCell<ColorsDemoColor>>,
    pub bg: Rc<RefCell<ColorsDemoColor>>,

    // whether or not the state is currently be updated (do not draw)
    pub updating: Rc<RefCell<bool>>,

    pub drawing_fg: Label,
    pub drawing_bg: ParentPane,

    pub toggle: Toggle,
    pub dial_fg_art: Dial,
    pub dial_color_kind: Dial,
    pub color_dd: DropdownList,
    pub max_gr_colors_dd: DropdownList,
    pub dist_ntb: NumbersTextBox<usize>,
    pub angle_ntb: NumbersTextBox<f64>,
    pub time_ntb: NumbersTextBox<usize>,
    pub r_ntb: NumbersTextBox<usize>,
    pub g_ntb: NumbersTextBox<usize>,
    pub b_ntb: NumbersTextBox<usize>,
    pub a_ntb: NumbersTextBox<usize>,
}

#[derive(Default)]
pub struct ColorsDemoColor {
    pub kind: ColorsDemoColorKind,
    // all inner states are kept so they may be returned too.
    // Vec<Color> is kept so that gradients can have memory of colors
    // beyond the max gradient colors.
    pub solid_state: Color,
    pub time_gradient_state: (TimeGradient, Vec<Color>),
    pub radial_gradient_state: (RadialGradient, Vec<Color>),
    pub linear_gradient_state: (Gradient, Vec<Color>),
    pub radial_time_state: (RadialGradient, Vec<TimeGradient>, Vec<Color>),
    pub linear_time_state: (Gradient, Vec<TimeGradient>, Vec<Color>),
    pub tiles_state: (Pattern, Color, Color),
}

#[derive(Clone, Copy, Default)]
pub enum ColorsDemoColorKind {
    #[default]
    Solid,
    TimeGradient,
    RadialGradient,
    LinearGradient,
    RadialTime,
    LinearTime,
    Tiles,
}

impl ColorsDemoState {
    /// updates all the sliders/tbs for a dial change
    pub fn update_for_toggle_change(&self, ctx: &Context) {
        let kind =
            if self.toggle.is_left() { self.fg.borrow().kind } else { self.bg.borrow().kind };

        let v = match kind {
            ColorsDemoColorKind::Solid => "Solid",
            ColorsDemoColorKind::TimeGradient => "Time-Gradient",
            ColorsDemoColorKind::RadialGradient => "Radial-Gradient",
            ColorsDemoColorKind::LinearGradient => "Linear-Gradient",
            ColorsDemoColorKind::RadialTime => "Radial-Time",
            ColorsDemoColorKind::LinearTime => "Linear-Time",
            ColorsDemoColorKind::Tiles => "Tiles",
        };
        // setting the dial with trigger update_for_color_dial_change
        let _ = self.dial_color_kind.set_value(ctx, v);
    }

    /// updates all the sliders/tbs for a dial change
    pub fn update_for_color_dial_change(&self, ctx: &Context) {
        {
            self.updating.replace(true);
            let mut demo_color =
                if self.toggle.is_left() { self.fg.borrow_mut() } else { self.bg.borrow_mut() };
            match self.dial_color_kind.get_value().as_str() {
                "Solid" => {
                    debug!("Solid hit");
                    demo_color.kind = ColorsDemoColorKind::Solid;
                    self.color_dd.set_selected(0);
                    self.color_dd.pane.disable();
                    self.max_gr_colors_dd.pane.disable();
                    self.max_gr_colors_dd.set_selected(0);
                    self.update_for_color_dd_change_from_demo_color("Solid", &demo_color);
                }
                "Time-Gradient" => {
                    demo_color.kind = ColorsDemoColorKind::TimeGradient;
                    self.max_gr_colors_dd.pane.enable();
                    let max = demo_color.time_gradient_state.0.len(ctx) - 1;
                    self.color_dd.pane.enable();
                    let entries = (1..=max).map(|i| i.to_string()).collect::<Vec<String>>();
                    self.color_dd.set_entries(entries);
                    self.color_dd.set_selected(0);
                    self.max_gr_colors_dd.set_selected(max - 1);
                    self.angle_ntb.tb.pane.disable();
                    self.dist_ntb.tb.pane.disable();
                    self.update_for_color_dd_change_from_demo_color("Time-Gradient", &demo_color);
                }
                "Radial-Gradient" => {
                    demo_color.kind = ColorsDemoColorKind::RadialGradient;
                    self.color_dd.pane.enable();
                    self.max_gr_colors_dd.pane.enable();
                }
                "Linear-Gradient" => {
                    demo_color.kind = ColorsDemoColorKind::LinearGradient;
                    self.color_dd.pane.enable();
                    self.max_gr_colors_dd.pane.enable();
                }
                "Radial-Time" => {
                    demo_color.kind = ColorsDemoColorKind::RadialTime;
                    self.color_dd.pane.enable();
                    self.max_gr_colors_dd.pane.enable();
                }
                "Linear-Time" => {
                    demo_color.kind = ColorsDemoColorKind::LinearTime;
                    self.color_dd.pane.enable();
                    self.max_gr_colors_dd.pane.enable();
                }
                "Tiles" => {
                    demo_color.kind = ColorsDemoColorKind::Tiles;
                    self.color_dd.pane.disable();
                    self.max_gr_colors_dd.pane.disable();
                }
                _ => unreachable!(),
            }
            self.updating.replace(false);
        }

        self.update_drawing();
    }

    pub fn update_for_color_dd_change(&self) {
        let demo_color = if self.toggle.is_left() { self.fg.borrow() } else { self.bg.borrow() };
        self.update_for_color_dd_change_from_demo_color(
            self.dial_color_kind.get_value().as_str(),
            &demo_color,
        );
    }

    pub fn update_for_color_dd_change_from_demo_color(
        &self, dial_color_kind: &str, demo_color: &ColorsDemoColor,
    ) {
        let dd_i = self.color_dd.get_selected();

        let c = match dial_color_kind {
            "Solid" => demo_color.solid_state.clone(),
            "Time-Gradient" => demo_color
                .time_gradient_state
                .1
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Radial-Gradient" => demo_color
                .radial_gradient_state
                .1
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Linear-Gradient" => demo_color
                .linear_gradient_state
                .1
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Radial-Time" => demo_color
                .radial_time_state
                .2
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Linear-Time" => demo_color
                .linear_time_state
                .2
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Tiles" => {
                if dd_i == 0 {
                    demo_color.tiles_state.1.clone()
                } else {
                    demo_color.tiles_state.2.clone()
                }
            }
            _ => unreachable!(),
        };

        let Color::Rgba(rgba) = c else {
            return;
        };
        self.r_ntb.change_value(rgba.r as usize);
        self.g_ntb.change_value(rgba.g as usize);
        self.b_ntb.change_value(rgba.b as usize);
        self.a_ntb.change_value(rgba.a as usize);
    }

    /// updates for any smaller-changes (sliders/tbs)
    pub fn update_for_minor_changes(&self) {
        self.update_drawing();
    }

    const SPIRAL: &'static str = r#"
  ██████████████████████████████████████████████
  ██                                          ██
  ██  ██████████████████████████████████████  ██
  ██  ██                                  ██  ██
  ██  ██  ██████████████████████████████  ██  ██
  ██  ██  ██                          ██  ██  ██
  ██  ██  ██  ██████████████████████  ██  ██  ██
  ██  ██  ██  ██                  ██  ██  ██  ██
  ██  ██  ██  ██  ██████████████  ██  ██  ██  ██
  ██  ██  ██  ██  ██          ██  ██  ██  ██  ██
  ██  ██  ██  ██  ██  ██████  ██  ██  ██  ██  ██
  ██  ██  ██  ██  ██  ██  ██  ██  ██  ██  ██  ██
  ██  ██  ██  ██  ██      ██  ██  ██  ██  ██  ██
  ██  ██  ██  ██  ██████████  ██  ██  ██  ██  ██
  ██  ██  ██  ██              ██  ██  ██  ██  ██
  ██  ██  ██  ██████████████████  ██  ██  ██  ██
  ██  ██  ██                      ██  ██  ██  ██
  ██  ██  ██████████████████████████  ██  ██  ██
  ██  ██                              ██  ██  ██
  ██  ██████████████████████████████████  ██  ██
  ██                                      ██  ██
  ██████████████████████████████████████████  ██"#;

    const BUTTERFLY: &'static str = r#"                                 , 
                                 ;o\ 
                                 ;Ob`. 
                                ;OOOOb`. 
                               ;OOOOOY" ) 
                              ;OOOO' ,%%) 
                          \  /OOO ,%%%%,%\ 
                           |:  ,%%%%%%;%%/ 
                           ||,%%%%%%%%%%/ 
                           ;|%%%%%%%%%'/`-'"`. 
                          /: %%%%%%%%'/ c$$$$.`. 
             `.______     \ \%%%%%%%'/.$$YF"Y$: ) 
           _________ "`.\`\o \`%%' ,',$F,.   $F ) 
  ___,--""'dOOO;,:%%`-._ o_,O_   ,',"',d88)  '  ) 
"'. YOOOOOOO';%%%%%%%%%;`-O   )_     ,X888F   _/ 
   \ YOOOO',%%%%%%%%%%Y    \__;`),-.  `""F  ,' 
    \ `" ,%%%%%%%%%%,' _,-   \-' \_ `------' 
     \_ %%%%',%%%%%_,-' ,;    ( _,-\ 
       `-.__`%%',-' .c$$'     |\-_,-\ 
            `""; ,$$$',md8oY  : `\_,') 
              ( ,$$$F `88888  ;   `--' 
               \`$$(    `""' / 
                \`"$$c'   _,' 
 -hrr-           `.____,-' "#;

    const SATURN: &'static str = r#"




                                          _.oo.
                  _.u[[/;:,.         .odMMMMMM'
               .o888UU[[[/;:-.  .o@P^    MMM^
              oN88888UU[[[/;::-.        dP^
             dNMMNN888UU[[[/;:--.   .o@P^
            ,MMMMMMN888UU[[/;::-. o@^
            NNMMMNN888UU[[[/~.o@P^
            888888888UU[[[/o@^-..
           oI8888UU[[[/o@P^:--..
        .@^  YUU[[[/o@^;::---..
      oMP     ^/o@P^;:::---..
   .dMMM    .o@^ ^;::---...
  dMMMMMMM@^`       `^^^^
 YMMMUP^
  ^^
 
 unknown"#;

    // TODO there are nicer ways to center... being lazy
    const RUST_LOGO: &'static str = r#"







                    △ △ △ △ △
                  ╭─────o─────╮
                ◁ │ ███████   │ ▷
                ◁ o  ██   ██  o ▷
                ◁ │  ██████   │ ▷
                ◁ │  ██   ██  │ ▷
                ◁ │ ████  ███ │ ▷
                  ╰──o─────o──╯
                    ▽ ▽ ▽ ▽ ▽    
"#;

    // TODO there are nicer ways to center... being lazy
    const CHOMPY: &'static str = r#"
    
    
    
    
    
    
    
                        CHOMPPY ANGRYYY!
                        DO NOT TEST CHOMPPY!
                _____  /
              _/o\ /o \
             /        |
             v v v v  |
              ^ ^ ^ ^ |
              \.......|
"#;

    /// updates the drawing of the art
    pub fn update_drawing(&self) {
        if *self.updating.borrow() {
            return;
        }

        let fg = self.fg.borrow().get_color();
        let bg = self.bg.borrow().get_color();

        let text = match self.dial_fg_art.get_value().as_str() {
            "Butterfly" => ColorsDemoState::BUTTERFLY,
            "Spiral" => ColorsDemoState::SPIRAL,
            "Rust-Logo" => ColorsDemoState::RUST_LOGO,
            "Saturn" => ColorsDemoState::SATURN,
            "Chompy" => ColorsDemoState::CHOMPY,
            _ => "",
        };
        self.drawing_fg.set_text(text);
        self.drawing_fg.set_style(Style::transparent().with_fg(fg));
        self.drawing_bg.set_bg(bg);
    }
}

impl ColorsDemoColor {
    pub fn default_fg(ctx: &Context) -> ColorsDemoColor {
        let time_gr_colors = vec![Color::MIDNIGHT_BLUE, Color::WHITE, Color::PINK];
        let time_gr = TimeGradient::new_loop(ctx, Duration::from_secs(1), time_gr_colors.clone());

        let radial_gr_colors = vec![Color::MIDNIGHT_BLUE, Color::WHITE, Color::PINK];
        let radial_gr = RadialGradient::new_basic_circle(
            ctx,
            (0.5.into(), 0.5.into()),
            5.into(),
            radial_gr_colors.clone(),
        );
        let linear_gr_colors = vec![
            Color::VIOLET,
            Color::INDIGO,
            Color::BLUE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::RED,
        ];
        let linear_gr = Gradient::x_grad_rainbow(ctx, 5);
        let radial_time_colors = vec![Color::MIDNIGHT_BLUE, Color::WHITE, Color::PINK];
        let radial_time_gr = RadialGradient::new_basic_circle_time_loop(
            ctx,
            (0.5.into(), 0.5.into()),
            Duration::from_secs(1),
            5.into(),
            radial_time_colors.clone(),
        );
        let linear_time_colors = vec![
            Color::VIOLET,
            Color::INDIGO,
            Color::BLUE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::RED,
        ];
        let linear_time_gr = Gradient::x_grad_rainbow_time_loop(ctx, 5, Duration::from_secs(1));
        let tiles_colors = (Color::WHITE, Color::BLUE);
        let tiles = Pattern::new_sqr_tiles(ctx, 5, tiles_colors.0.clone(), tiles_colors.1.clone());
        ColorsDemoColor {
            kind: ColorsDemoColorKind::Solid,
            solid_state: Color::WHITE,
            time_gradient_state: (time_gr, time_gr_colors),
            radial_gradient_state: (radial_gr, radial_gr_colors),
            linear_gradient_state: (linear_gr, linear_gr_colors),
            radial_time_state: (radial_time_gr.0, radial_time_gr.1, radial_time_colors),
            linear_time_state: (linear_time_gr.0, linear_time_gr.1, linear_time_colors),
            tiles_state: (tiles, tiles_colors.0, tiles_colors.1),
        }
    }

    pub fn default_bg(ctx: &Context) -> ColorsDemoColor {
        let time_gr_colors = vec![Color::MIDNIGHT_BLUE, Color::WHITE, Color::PINK];
        let time_gr = TimeGradient::new_loop(ctx, Duration::from_secs(1), time_gr_colors.clone());

        let radial_gr_colors = vec![Color::MIDNIGHT_BLUE, Color::WHITE, Color::PINK];
        let radial_gr = RadialGradient::new_basic_circle(
            ctx,
            (0.5.into(), 0.5.into()),
            5.into(),
            radial_gr_colors.clone(),
        );
        let linear_gr_colors = vec![
            Color::VIOLET,
            Color::INDIGO,
            Color::BLUE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::RED,
        ];
        let linear_gr = Gradient::x_grad_rainbow(ctx, 5);
        let radial_time_colors = vec![Color::MIDNIGHT_BLUE, Color::WHITE, Color::PINK];
        let radial_time_gr = RadialGradient::new_basic_circle_time_loop(
            ctx,
            (0.5.into(), 0.5.into()),
            Duration::from_secs(1),
            5.into(),
            radial_time_colors.clone(),
        );
        let linear_time_colors = vec![
            Color::VIOLET,
            Color::INDIGO,
            Color::BLUE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::RED,
        ];
        let linear_time_gr = Gradient::x_grad_rainbow_time_loop(ctx, 5, Duration::from_secs(1));
        let tiles_colors = (Color::WHITE, Color::BLUE);
        let tiles = Pattern::new_sqr_tiles(ctx, 5, tiles_colors.0.clone(), tiles_colors.1.clone());
        ColorsDemoColor {
            kind: ColorsDemoColorKind::Solid,
            solid_state: Color::CRIMSON,
            time_gradient_state: (time_gr, time_gr_colors),
            radial_gradient_state: (radial_gr, radial_gr_colors),
            linear_gradient_state: (linear_gr, linear_gr_colors),
            radial_time_state: (radial_time_gr.0, radial_time_gr.1, radial_time_colors),
            linear_time_state: (linear_time_gr.0, linear_time_gr.1, linear_time_colors),
            tiles_state: (tiles, tiles_colors.0, tiles_colors.1),
        }
    }

    pub fn get_color(&self) -> Color {
        match self.kind {
            ColorsDemoColorKind::Solid => self.solid_state.clone(),
            ColorsDemoColorKind::TimeGradient => self.time_gradient_state.0.clone().into(),
            ColorsDemoColorKind::RadialGradient => self.radial_gradient_state.0.clone().into(),
            ColorsDemoColorKind::LinearGradient => self.linear_gradient_state.0.clone().into(),
            ColorsDemoColorKind::RadialTime => self.radial_time_state.0.clone().into(),
            ColorsDemoColorKind::LinearTime => self.linear_time_state.0.clone().into(),
            ColorsDemoColorKind::Tiles => self.tiles_state.0.clone().into(),
        }
    }
}
