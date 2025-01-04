use yeehaw::*;

pub fn image_demo(ctx: &Context) -> Box<dyn Element> {
    let bg: Color = Gradient::x_grad_rainbow(ctx, 10).into();
    let el = ParentPaneOfSelectable::new(ctx).with_bg(bg.clone());

    // include the file bytes as a static slice
    let img_x = 2;
    let img_bz = std::include_bytes!("../../../assets/tuvix.jpg");
    let dyn_img = image::load_from_memory(img_bz).unwrap();
    let img_pane = ImageViewer::new(ctx, dyn_img, Color::BLACK)
        .unwrap()
        .with_dyn_width(61)
        .with_dyn_height(20)
        .at(img_x, 4);

    let above_text = "Images can be easily displayed in the terminal using various\n\
         image viewing protocols depending on the terminal being used.\n\
         (thanks you ratatui_image). #neverforget #tuvix";
    el.add_element(Box::new(
        img_pane
            .label(ctx, above_text)
            .with_fg(Color::BLACK)
            .with_bg(Color::WHITE.with_alpha(&ctx.color_store, 123)),
    ));
    el.add_element(Box::new(img_pane));
    Box::new(el)
}
