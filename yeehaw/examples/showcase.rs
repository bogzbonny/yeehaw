use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::log::set_log_file("./debug_test.log".to_string());
    //yeehaw::log::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let el = VerticalStack::new(&ctx);
    let header_pane = ParentPaneOfSelectable::new(&ctx)
        .with_dyn_height(DynVal::new_fixed(10))
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
    .with_style(Style::default().with_fg(Color::Gradient(gr)))
    .at(DynVal::new_flex(0.), DynVal::new_flex(0.));
    header_pane.add_element(Box::new(mtext));

    println!("1");

    let central_pane = HorizontalStack::new(&ctx);
    el.push(Box::new(central_pane.clone()));
    let left_pane = VerticalStack::new(&ctx);
    let right_pane = VerticalStack::new(&ctx);
    central_pane.push(Box::new(left_pane.clone()));
    central_pane.push(Box::new(right_pane.clone()));
    let left_top = ParentPaneOfSelectable::new(&ctx).with_dyn_height(DynVal::new_flex(1.));
    left_pane.push(Box::new(left_top.clone()));
    left_pane.push(Box::new(DebugSizePane::new(&ctx).with_bg(Color::BLUE)));
    right_pane.push(Box::new(DebugSizePane::new(&ctx).with_bg(Color::RED)));

    let l1 = Label::new(&ctx, "window generation zone");
    let l = l1.clone().at(DynVal::new_flex(0.5), DynVal::new_flex(0.5));
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
    .at(DynVal::new_flex(0.6), DynVal::new_flex(0.7));
    left_top.add_element(Box::new(dial1));

    println!("end");
    tui.run(Box::new(el)).await
}
