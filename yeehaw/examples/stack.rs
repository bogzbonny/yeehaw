use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::log::reset_log_file("./debug_test.log".to_string());
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let top = VerticalStack::new(&ctx);
    let sel_pane = ParentPaneOfSelectable::new(&ctx).with_dyn_height(3);
    let hstack = HorizontalStack::new(&ctx)
        .with_min_resize_width(2)
        .with_dyn_height(DynVal::FULL);
    let vstack = VerticalStack::new(&ctx)
        .with_min_resize_height(2)
        .with_dyn_height(DynVal::FULL);
    top.push(Box::new(sel_pane.clone()));
    top.push(Box::new(vstack.clone()));

    let top_ = top.clone();
    let vstack_ = vstack.clone();
    let hstack_ = hstack.clone();
    let toggle = Toggle::new(&ctx, " vertical ", " horizontal ")
        .at(28, 1)
        .with_fn(Box::new(move |_, tog| {
            top_.pop();
            if tog.selected() == " horizontal " {
                top_.push(Box::new(hstack_.clone()));
            } else {
                top_.push(Box::new(vstack_.clone()));
            };
            EventResponses::default()
        }));

    let toggle_ = toggle.clone();
    let hstack_ = hstack.clone();
    let vstack_ = vstack.clone();
    let remove_button_click_fn = Box::new(move |_, _| {
        if toggle_.selected() == " horizontal " {
            if !hstack_.is_empty() {
                hstack_.remove(hstack_.len() - 1);
            }
        } else if !vstack_.is_empty() {
            vstack_.remove(vstack_.len() - 1);
        }
        EventResponses::default()
    });
    let remove_button = Button::new(&ctx, "remove_pane")
        .with_fn(remove_button_click_fn)
        .at(13, 1);

    let toggle_ = toggle.clone();
    let hstack_ = hstack.clone();
    let vstack_ = vstack.clone();
    let add_button_click_fn = Box::new(move |_, ctx_| {
        let sty = Style::default().with_fg(Color::WHITE).with_bg(Color::BLACK);
        if toggle_.selected() == " horizontal " {
            // NOTE this is the button context, but the size doesn't matter for these operations
            //let el: Box<dyn Element> = if hstack_.is_empty() {
            //    Box::new(
            //        DebugSizePane::new(&ctx_)
            //            .with_style(sty)
            //            .with_dyn_width(hstack_.avg_width(&ctx_)),
            //    )
            //} else {
            //    Box::new(
            //        Bordered::new_left_resizer(
            //            &ctx_,
            //            Box::new(DebugSizePane::new(&ctx_).with_style(sty)),
            //            Style::default(),
            //        )
            //        .with_dyn_width(hstack_.avg_width(&ctx_)),
            //    )
            //};

            let el = Box::new(
                Bordered::new_resizer(
                    &ctx_,
                    Box::new(DebugSizePane::new(&ctx_).with_style(sty)),
                    Style::default(),
                )
                .with_dyn_width(hstack_.avg_width(&ctx_)),
            );

            hstack_.push(el);
        } else {
            //let el: Box<dyn Element> = if vstack_.is_empty() {
            //    Box::new(
            //        DebugSizePane::new(&ctx_)
            //            .with_style(sty)
            //            .with_dyn_height(vstack_.avg_height(&ctx_)),
            //    )
            //} else {
            //    Box::new(
            //        Bordered::new_top_resizer(
            //            &ctx_,
            //            Box::new(DebugSizePane::new(&ctx_).with_style(sty)),
            //            Style::default(),
            //        )
            //        .with_dyn_height(vstack_.avg_height(&ctx_)),
            //    )
            //};

            let el = Box::new(
                Bordered::new_resizer(
                    &ctx_,
                    Box::new(DebugSizePane::new(&ctx_).with_style(sty)),
                    Style::default(),
                )
                .with_dyn_height(vstack_.avg_height(&ctx_)),
            );
            vstack_.push(el);
        }

        EventResponses::default()
    });
    let add_button = Button::new(&ctx, "add_pane")
        .with_fn(add_button_click_fn)
        .at(1, 1);

    sel_pane.add_element(Box::new(add_button));
    sel_pane.add_element(Box::new(remove_button));
    sel_pane.add_element(Box::new(toggle));

    tui.run(Box::new(top)).await
}
