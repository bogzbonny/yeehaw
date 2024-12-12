use yeehaw::*;

/// NOTE this example requires steam locomotive (`sl`) to be installed if you want
/// to see the train. Additionally it should be run from this showcase directory
/// for the showcase-tab to work (which runs `cargo run --release --example showcase`).

#[tokio::main]
async fn main() -> Result<(), Error> {
    // uncomment the following line to enable logging
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let main = VerticalStack::new(&ctx);

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
    let _ = main.push(Box::new(mb));

    let header_pane = ParentPaneOfSelectable::new(&ctx)
        .with_dyn_height(DynVal::new_fixed(7))
        .with_unfocused(&ctx);
    let _ = main.push(Box::new(header_pane.clone()));

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
    let _ = header_pane.add_element(Box::new(mtext));

    let button = Button::new(
        &ctx,
        "do not\nclick me",
        Box::new(|_, _| EventResponses::default()),
    )
    .at(DynVal::new_flex(0.9), DynVal::new_flex(0.3));
    let _ = header_pane.add_element(Box::new(button));

    let central_pane = HorizontalStack::new(&ctx);
    let _ = main.push(Box::new(central_pane.clone()));
    let left_pane = VerticalStack::new(&ctx).with_width(DynVal::new_flex(0.7));
    let right_pane = VerticalStack::new(&ctx);
    let _ = central_pane.push(Box::new(left_pane.clone()));
    let _ = central_pane.push(Box::new(right_pane.clone()));
    let _ = left_pane.push(window_generation_zone(&ctx, Box::new(main.pane.clone())));

    let train_pane = DebugSizePane::new(&ctx);
    //let train_pane = TerminalPane::new(&ctx)?;
    //train_pane.pane.set_dyn_height(DynVal::new_flex(0.7));
    //train_pane.pane.set_unfocused(&ctx);
    //train_pane.disable_cursor();
    //train_pane.execute_command("for i in {1..7}; do sl -l; done ; exit");
    let _ = left_pane.push(Box::new(train_pane));

    let tabs = Tabs::new(&ctx);
    let el1 = DebugSizePane::new(&ctx)
        .with_bg(Color::RED)
        .with_text("tab 1".to_string());
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
    showcase.pane.set_unfocused(&ctx);

    let showcase_ = showcase.clone();
    let on_showcase_open_fn = Some(Box::new(move || {
        let command = "cargo run --release --example showcase";
        showcase_.execute_command(command);
    }) as Box<dyn FnOnce()>);

    let _ = tabs.push(Box::new(el1), "widgets");
    let _ = tabs.push(Box::new(el2), "colors");
    let _ = tabs.push(Box::new(el3), "images");
    let _ = tabs.push(Box::new(el4), "file-nav");
    let _ = tabs.push(Box::new(el5), "terminal");
    let _ = tabs.push_with_on_open_fn(Box::new(showcase), "showcase", on_showcase_open_fn);
    let _ = tabs.select(0);
    let _ = right_pane.push(Box::new(tabs));

    tui.run(Box::new(main)).await
}

pub fn window_generation_zone(
    ctx: &Context, windows_generated_in: Box<ParentPane>,
) -> Box<dyn Element> {
    let sc = PaneScrollable::new_expanding(ctx, 50, 10);
    let el = ParentPaneOfSelectable::new(ctx);
    let _ = sc.add_element(Box::new(el.clone()));
    let bordered = Bordered::new_resizer(
        ctx,
        Box::new(sc.clone()),
        Style::transparent().with_fg(Color::WHITE),
    )
    .with_dyn_height(1.5);

    let l = Label::new(ctx, "window generation zone");
    let _ = el.add_element(Box::new(l));

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
    let label =
        Label::new_for_el(ctx, dial_type.get_dyn_location_set().l.clone(), "content:").underlined();
    let _ = el.add_element(Box::new(label));

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
    let label = Label::new_for_el(
        ctx,
        dial_border.get_dyn_location_set().l.clone(),
        "border options:",
    )
    .underlined();
    let _ = el.add_element(Box::new(label));

    let shadow_cb = Checkbox::new(ctx).at(1, 8);
    let cb_label = Label::new_for_el(ctx, shadow_cb.get_dyn_location_set().l.clone(), "shadow");

    let alpha_slider = Slider::new_basic_block(ctx)
        .with_position(0.9)
        .with_width(DynVal::new_flex(0.4))
        .at(1, 11);
    let label = Label::new_for_el(
        ctx,
        alpha_slider.get_dyn_location_set().l.clone(),
        "background alpha:",
    )
    .underlined();
    let _ = el.add_element(Box::new(label));
    let _ = el.add_element(Box::new(alpha_slider.clone()));

    let counter = Rc::new(RefCell::new(0));

    let mut ctx_ = ctx.clone();
    let counter_ = counter.clone();
    let shadow_cb_ = shadow_cb.clone();
    let windows_generated_in_ = windows_generated_in.clone();
    let dial_type_ = dial_type.clone();
    let dial_border_ = dial_border.clone();
    let generate_window_fn = Box::new(move |_, _| {
        ctx_.size.width = 30;
        ctx_.size.height = 20;
        let title = format!("Pane {}", *counter_.borrow());

        let alpha = (alpha_slider.get_position() * 255.0) as u8;

        let bg = Color::new_with_alpha(150, 150, 155, alpha);
        let fg = Color::new_with_alpha(150, 150, 155, alpha);
        let sty = Style::default().with_bg(bg.clone()).with_fg(fg.clone());
        let def_ch = DrawCh::new(ChPlus::Transparent, sty.clone());

        let el: Box<dyn Element> = match dial_type_.get_value().as_str() {
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
                let _ = sc_pane.add_element(Box::new(el));
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
                let _ = sc_pane.add_element(Box::new(el));
                Box::new(sc_pane)
            }
            "Terminal" => Box::new(TerminalPane::new(&ctx_).unwrap()),
            _ => panic!("missing type implementation"),
        };

        let el: Box<dyn Element> = match dial_border_.get_value().as_str() {
            "None" => el,
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
            "Scrollbars" => Box::new(Bordered::new_borderless_with_scrollbars_and_thin_left(
                &ctx_,
                el,
                sty.clone().with_fg(Color::WHITE),
            )),
            "Line-Scrollbars" => Box::new(Bordered::new_resizer_with_scrollbars(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Text" => Box::new(
                Bordered::new_basic(&ctx_, el, sty.clone().with_fg(Color::BLACK))
                    .with_title("upper")
                    .with_bottom_right_text("lower")
                    .with_left_top_text("left-top")
                    .with_right_bottom_text("right-bottom"),
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
        let window = WindowPane::new(&ctx_, el, &title)
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30))
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10));

        let window: Box<dyn Element> = if shadow_cb_.is_checked() {
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
        windows_generated_in_
            .pane
            .send_responses_upward(&ctx_, resp.into());

        EventResponses::default()
    });

    let button = Button::new(ctx, "generate", generate_window_fn).at(1, 13);

    let _ = el.add_element(Box::new(dial_type));
    let _ = el.add_element(Box::new(dial_border));
    let _ = el.add_element(Box::new(shadow_cb));
    let _ = el.add_element(Box::new(cb_label));
    let _ = el.add_element(Box::new(button));

    Box::new(bordered)
}
