use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::*,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::debug::set_log_file("./debug_test.log".to_string());
    yeehaw::debug::clear();
    //unsafe { std::env::set_var("RUST_BACKTRACE", "1") };

    let (mut tui, ctx) = Tui::new()?;

    let pp = ParentPane::new(&ctx, "parent_pane")
        .with_dyn_height(1.0.into())
        .with_dyn_width(1.0.into())
        .with_bg_color(Color::DARK_BLUE);

    let counter = Rc::new(RefCell::new(0));

    let mut ctx_ = ctx.clone();
    let counter_ = counter.clone();
    let add_button_click_fn = Box::new(move |_, _| {
        ctx_.s.width = 30;
        ctx_.s.height = 20;
        let title = format!("Pane {}", *counter_.borrow());
        let bg = Color::new_with_alpha(150, 150, 155, 150);
        let fg = Color::new_with_alpha(150, 150, 155, 150);
        let def_ch = DrawCh::new(
            ChPlus::Transparent,
            Style::default().with_bg(bg.clone()).with_fg(fg),
        );

        let el = DebugSizePane::new(&ctx_)
            .with_text(title.clone())
            .with_width(DynVal::full())
            .with_height(DynVal::full())
            .with_default_ch(def_ch)
            .with_style(Style::default().with_bg(bg).with_fg(Color::BLACK));

        *counter_.borrow_mut() += 1;
        let window = WindowPane::new(&ctx_, Box::new(el), &title)
            .with_corner_resizer(&ctx_)
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10))
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30));

        let inner_resps = vec![
            EventResponse::BringToFront,
            EventResponse::UnfocusOthers,
            EventResponse::Focus,
        ];
        EventResponse::NewElement(Box::new(window.clone()), Some(inner_resps.into())).into()
    });

    let mut ctx_ = ctx.clone();
    let counter_ = counter.clone();
    let add_button_scr_click_fn = Box::new(move |_, _| {
        ctx_.s.width = 30;
        ctx_.s.height = 20;
        let title = format!("Pane {}", *counter_.borrow());
        let bg = Color::new_with_alpha(150, 150, 155, 150);
        let fg = Color::new_with_alpha(150, 150, 155, 150);
        let sty = Style::default().with_bg(bg.clone()).with_fg(fg);
        let def_ch = DrawCh::new(ChPlus::Transparent, sty.clone());

        let el = DebugSizePane::new(&ctx_)
            .with_text(title.clone())
            .with_width(DynVal::full())
            .with_height(DynVal::full())
            .with_default_ch(def_ch)
            .with_style(Style::default().with_bg(bg).with_fg(Color::BLACK));

        //let sc_pane = PaneWithScrollbars::new(
        //    &ctx_,
        //    50,
        //    50,
        //    HorizontalSBPositions::Below,
        //    VerticalSBPositions::ToTheRight,
        //);

        let sc_pane = PaneScrollable::new(&ctx_, 50, 50);
        sc_pane.add_element(Box::new(el));
        let sc_pane = Bordered::new_borderless_with_scrollbars(&ctx_, Box::new(sc_pane), sty);

        *counter_.borrow_mut() += 1;
        let window = WindowPane::new(&ctx_, Box::new(sc_pane), &title)
            .with_corner_resizer(&ctx_)
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30))
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10));

        let shadow_color = Color::new_with_alpha(0, 0, 0, 150);
        let window_with_shadow =
            Shadowed::thick_with_color(&ctx_, Box::new(window.clone()), shadow_color);

        let inner_resps = vec![
            EventResponse::BringToFront,
            EventResponse::UnfocusOthers,
            EventResponse::Focus,
        ];
        EventResponse::NewElement(
            Box::new(window_with_shadow.clone()),
            Some(inner_resps.into()),
        )
        .into()
    });

    let mut ctx_ = ctx.clone();
    let counter_ = counter.clone();
    let add_term_click_fn = Box::new(move |_, _| {
        ctx_.s.width = 30;
        ctx_.s.height = 20;
        let title = format!("Pane {}", *counter_.borrow());
        let el = TerminalPane::new(&ctx_)
            .with_width(DynVal::full())
            .with_height(DynVal::full());

        *counter_.borrow_mut() += 1;
        let window = WindowPane::new(&ctx_, Box::new(el), &title)
            .with_corner_resizer(&ctx_)
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10))
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30));

        let inner_resps = vec![
            EventResponse::BringToFront,
            EventResponse::UnfocusOthers,
            EventResponse::Focus,
        ];
        EventResponse::NewElement(Box::new(window.clone()), Some(inner_resps.into())).into()
    });

    let mut ctx_ = ctx.clone();
    let counter_ = counter.clone();
    let add_button_bordered_resizer_fn = Box::new(move |_, _| {
        ctx_.s.width = 30;
        ctx_.s.height = 20;
        let title = format!("Bordered Pane {}", *counter_.borrow());
        let bg = Color::new_with_alpha(150, 150, 155, 150);
        let fg = Color::new_with_alpha(150, 150, 155, 150);
        let sty = Style::default().with_bg(bg.clone()).with_fg(fg);
        let def_ch = DrawCh::new(ChPlus::Transparent, sty.clone());

        let el = DebugSizePane::new(&ctx_)
            .with_text(title.clone())
            .with_width(DynVal::full())
            .with_height(DynVal::full())
            .with_default_ch(def_ch)
            .with_style(Style::default().with_bg(bg).with_fg(Color::BLACK));

        let bordered =
            Bordered::new_resizer(&ctx_, Box::new(el), sty.clone().with_fg(Color::BLACK))
                .with_title(&title);

        *counter_.borrow_mut() += 1;
        let window = WindowPane::new(&ctx_, Box::new(bordered), &title)
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10))
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30));

        let inner_resps = vec![
            EventResponse::BringToFront,
            EventResponse::UnfocusOthers,
            EventResponse::Focus,
        ];
        EventResponse::NewElement(Box::new(window.clone()), Some(inner_resps.into())).into()
    });

    let mut ctx_ = ctx.clone();
    let counter_ = counter.clone();
    let add_button_bordered_mover_fn = Box::new(move |_, _| {
        ctx_.s.width = 30;
        ctx_.s.height = 20;
        let title = format!("Bordered Pane {}", *counter_.borrow());
        let bg = Color::new_with_alpha(150, 150, 155, 150);
        let fg = Color::new_with_alpha(150, 150, 155, 150);
        let sty = Style::default().with_bg(bg.clone()).with_fg(fg);
        let def_ch = DrawCh::new(ChPlus::Transparent, sty.clone());

        let el = DebugSizePane::new(&ctx_)
            .with_text(title.clone())
            .with_width(DynVal::full())
            .with_height(DynVal::full())
            .with_default_ch(def_ch)
            .with_style(Style::default().with_bg(bg).with_fg(Color::BLACK));

        let bordered = Bordered::new_mover(&ctx_, Box::new(el), sty.clone().with_fg(Color::BLACK))
            .with_title(&title);

        *counter_.borrow_mut() += 1;
        let window = WindowPane::new(&ctx_, Box::new(bordered), &title)
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10))
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30));

        let inner_resps = vec![
            EventResponse::BringToFront,
            EventResponse::UnfocusOthers,
            EventResponse::Focus,
        ];
        EventResponse::NewElement(Box::new(window.clone()), Some(inner_resps.into())).into()
    });

    let mut ctx_ = ctx.clone();
    let counter_ = counter.clone();
    let add_button_bordered_scr_click_fn = Box::new(move |_, _| {
        ctx_.s.width = 30;
        ctx_.s.height = 20;
        let title = format!("Pane {}", *counter_.borrow());
        let bg = Color::new_with_alpha(150, 150, 155, 150);
        let fg = Color::new_with_alpha(150, 150, 155, 150);
        let sty = Style::default().with_bg(bg.clone()).with_fg(fg);
        let def_ch = DrawCh::new(ChPlus::Transparent, sty.clone());

        let el = DebugSizePane::new(&ctx_)
            .with_text(title.clone())
            .with_width(DynVal::full())
            .with_height(DynVal::full())
            .with_default_ch(def_ch)
            .with_style(Style::default().with_bg(bg).with_fg(Color::BLACK));

        let sc_pane = PaneScrollable::new(&ctx_, 50, 50);
        sc_pane.add_element(Box::new(el));

        let bordered = Bordered::new_resizer_with_scrollbars(
            &ctx_,
            Box::new(sc_pane),
            sty.clone().with_fg(Color::BLACK),
        )
        .with_title(&title);

        *counter_.borrow_mut() += 1;
        let window = WindowPane::new(&ctx_, Box::new(bordered), &title)
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30))
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10));

        let inner_resps = vec![
            EventResponse::BringToFront,
            EventResponse::UnfocusOthers,
            EventResponse::Focus,
        ];
        EventResponse::NewElement(Box::new(window.clone()), Some(inner_resps.into())).into()
    });

    let add_button = Button::new(&ctx, "add_window", add_button_click_fn).at(1.into(), 1.into());
    let add_button2 =
        Button::new(&ctx, "add_window_scrollable", add_button_scr_click_fn).at(15.into(), 1.into());
    let add_button3 =
        Button::new(&ctx, "add_terminal_window", add_term_click_fn).at(40.into(), 1.into());
    let add_button4 = Button::new(&ctx, "add_bordered_resizer", add_button_bordered_resizer_fn)
        .at(63.into(), 1.into());
    let add_button5 = Button::new(&ctx, "add_bordered_mover", add_button_bordered_mover_fn)
        .at(87.into(), 1.into());
    let add_button6 = Button::new(
        &ctx,
        "add_bordered_resizer_sbs",
        add_button_bordered_scr_click_fn,
    )
    .at(1.into(), 3.into());
    pp.add_element(Box::new(add_button));
    pp.add_element(Box::new(add_button2));
    pp.add_element(Box::new(add_button3));
    pp.add_element(Box::new(add_button4));
    pp.add_element(Box::new(add_button5));
    pp.add_element(Box::new(add_button6));

    tui.run(Box::new(pp)).await
}
