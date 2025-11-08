use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut tui, ctx) = Tui::new()?;
    let main_el = ParentPane::new(&ctx, "main_element");

    let fig = FigletText::new(
        &ctx,
        "This is the worlds smallest figlet font!",
        figlet_rs::FIGfont::from_content(std::include_str!(
            "../../assets/figlet/worlds_smallest_figlet.flf"
        ))
        .expect("missing asset"),
    )
    .at(0.1, 0.3);

    main_el.add_element(Box::new(fig));
    tui.run(Box::new(main_el)).await
}
