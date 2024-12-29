use yeehaw::*;

pub fn image_demo(ctx: &Context) -> Box<dyn Element> {
    //let bg = Color::DARK_SLATE_GRAY;
    let bg: Color = Gradient::x_grad_rainbow(ctx, 10).into();

    let el = ParentPaneOfSelectable::new(ctx).with_bg(bg.clone());

    // include the file bytes as a static slice
    let img_x = 2;
    let img_bz = std::include_bytes!("../../../assets/tuvix.jpg");
    let dyn_img = image::load_from_memory(img_bz).unwrap();
    let img_pane = ImageViewer::new(ctx, dyn_img, bg)
        .unwrap()
        .with_dyn_width(61)
        .with_dyn_height(17)
        .at(img_x, 4);

    let above_text = "images can be easily displayed in the terminal using various\n\
         image viewing protocols depending on the terminal being used.\n\
         (thanks you ratatui_image)";
    let below_text = "#neverforget #tuvix";
    let y = DynVal::y_after(&img_pane, 0);
    let x = DynVal::x_after(&img_pane, 0).minus(below_text.chars().count().into());
    let label_below = Label::new(ctx, below_text)
        .with_fg(Color::BLACK)
        .bold()
        .at(x, y);

    let cctx = ctx.get_color_context();
    el.add_element(Box::new(
        img_pane
            .label(ctx, above_text)
            .with_fg(Color::BLACK)
            .with_bg(Color::WHITE.with_alpha(&cctx, 123)),
    ));
    el.add_element(Box::new(label_below));
    el.add_element(Box::new(img_pane));

    Box::new(el)
}
