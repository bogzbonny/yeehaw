use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // uncomment the following line to enable logging
    yeehaw::log::set_log_file("./debug_test.log".to_string());
    yeehaw::log::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let el = VerticalStack::new(&ctx);
    //let header_pane = ParentPaneOfSelectable::new(&ctx)
    let header_pane = ParentPane::new(&ctx, "header")
        .with_dyn_height(DynVal::new_fixed(7))
        .with_unfocused(&ctx);
    el.push(Box::new(header_pane.clone()));

    let mut gr = Gradient::x_grad_rainbow(5);
    gr.angle_deg = 60.;

    let mtext = FigletText::new(
        &ctx,
        "yyeeehaaaaawww",
        figlet_rs::FIGfont::from_content(std::include_str!("../../assets/figlet/ANSI_Shadow.flf"))
            .expect("missing asset"),
    )
    .with_min_height()
    .with_style(Style::default().with_fg(Color::Gradient(gr)))
    .at(0.into(), DynVal::new_fixed(1));
    header_pane.add_element(Box::new(mtext));

    let mb = MenuBar::top_menu_bar(&ctx)
        .with_height(1.into())
        .with_width(1.0.into());
    mb.pane.set_at(0.into(), 0.into());
    mb.add_item(&ctx, "upper/asdg/2222/3".to_string(), None);
    mb.add_item(&ctx, "menu/asdg/444ll/3adsf3/sdlkjf".to_string(), None);
    mb.add_item(&ctx, "bar/as33/222222/33".to_string(), None);
    mb.add_item(&ctx, "hello/yoyo/hi/asgd".to_string(), None);
    mb.add_item(&ctx, "world/yo".to_string(), None);
    mb.add_item(&ctx, "world/yosdfjldsffff/asdkjl".to_string(), None);
    header_pane.add_element(Box::new(mb));

    let central_pane = HorizontalStack::new(&ctx);
    el.push(Box::new(central_pane.clone()));
    let left_pane = VerticalStack::new(&ctx);
    let right_pane = VerticalStack::new(&ctx);
    central_pane.push(Box::new(left_pane.clone()));
    central_pane.push(Box::new(right_pane.clone()));
    let left_top = ParentPaneOfSelectable::new(&ctx).with_dyn_height(DynVal::new_flex(1.));
    left_pane.push(Box::new(Bordered::new_resizer(
        &ctx,
        Box::new(left_top.clone()),
        Style::default(),
    )));
    left_pane.push(Box::new(DebugSizePane::new(&ctx).with_bg(Color::BLUE)));
    right_pane.push(Box::new(DebugSizePane::new(&ctx).with_bg(Color::RED)));

    let l = Label::new(&ctx, "window generation zone");
    left_top.add_element(Box::new(l));

    let dial1 = Dial::new_spacious(
        &ctx,
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
    .at(DynVal::new_flex(0.), DynVal::new_flex(0.3));
    left_top.add_element(Box::new(dial1));

    println!("end");
    tui.run(Box::new(el)).await
}
