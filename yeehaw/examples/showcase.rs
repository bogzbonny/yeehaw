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
    //let main = PaneScrollable::new_expanding(&ctx, 90, 30);
    //let main_vstack = VerticalStack::new(&ctx);
    //let _ = main.add_element(Box::new(main_vstack.clone()));
    let main_vstack = VerticalStack::new(&ctx);

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
    let _ = main_vstack.push(Box::new(mb));

    let header_pane = ParentPaneOfSelectable::new(&ctx)
        .with_dyn_height(DynVal::new_fixed(7))
        .with_unfocused(&ctx);
    let _ = main_vstack.push(Box::new(header_pane.clone()));

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
    let _ = main_vstack.push(Box::new(central_pane.clone()));
    let left_pane = VerticalStack::new(&ctx).with_width(DynVal::new_flex(0.7));
    let right_pane = VerticalStack::new(&ctx);
    let _ = central_pane.push(Box::new(left_pane.clone()));
    let _ = central_pane.push(Box::new(right_pane.clone()));
    let _ = left_pane.push(window_generation_zone(&ctx));

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

    //tui.run(Box::new(main)).await
    tui.run(Box::new(main_vstack)).await
}

pub fn window_generation_zone(ctx: &Context) -> Box<dyn Element> {
    //let sc = PaneScrollable::new_expanding(ctx, 30, 10);
    let sc = PaneScrollable::new_expanding(ctx, 100, 100);
    let el = ParentPaneOfSelectable::new(ctx);
    let _ = sc.add_element(Box::new(el.clone()));
    let bordered =
        Bordered::new_resizer(ctx, Box::new(sc.clone()), Style::default()).with_dyn_height(1.5);
    //let el = ParentPaneOfSelectable::new(ctx);
    //let bordered =
    //    Bordered::new_resizer(ctx, Box::new(el.clone()), Style::default()).with_dyn_height(1.5);

    let l = Label::new(ctx, "window generation zone");
    let _ = el.add_element(Box::new(l));

    let dial1 = Dial::new_spacious(
        ctx,
        vec![
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
    .at(0, 7);

    let _ = el.add_element(Box::new(dial1));

    Box::new(bordered)
}
