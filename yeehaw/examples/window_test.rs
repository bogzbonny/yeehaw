use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        //debug,
        widgets::{Button, HorizontalSBPositions, VerticalSBPositions},
        ChPlus,
        Color,
        Cui,
        DebugSizePane,
        DrawCh,
        DynVal,
        Error,
        EventResponse,
        PaneWithScrollbars,
        ParentPane,

        Style,
        TerminalPane,
        WindowPane,
        *,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::debug::set_log_file("./debug_test.log".to_string());
    yeehaw::debug::clear();
    //unsafe { std::env::set_var("RUST_BACKTRACE", "1") };

    let (mut cui, ctx) = Cui::new()?;

    let pp = ParentPane::new(&ctx, "parent_pane")
        .with_dyn_height(1.0.into())
        .with_dyn_width(1.0.into())
        .with_bg_color(Color::BLUE);

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
            .with_width(DynVal::new_flex(1.))
            .with_height(DynVal::new_flex(1.))
            .with_default_ch(def_ch)
            .with_style(Style::default().with_bg(bg).with_fg(Color::BLACK));

        *counter_.borrow_mut() += 1;
        let window = WindowPane::new(&ctx_, Box::new(el), &title)
            .with_corner_adjuster(&ctx_)
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
        let def_ch = DrawCh::new(
            ChPlus::Transparent,
            Style::default().with_bg(bg.clone()).with_fg(fg),
        );

        let el = DebugSizePane::new(&ctx_)
            .with_text(title.clone())
            .with_width(DynVal::new_flex(1.))
            .with_height(DynVal::new_flex(1.))
            .with_default_ch(def_ch)
            .with_style(Style::default().with_bg(bg).with_fg(Color::BLACK));

        let sc_pane = PaneWithScrollbars::new(
            &ctx_,
            50,
            50,
            HorizontalSBPositions::Below,
            VerticalSBPositions::ToTheRight,
        );

        //let sc_pane = PaneScrollable::new(&hat_, 50, 50);

        sc_pane.add_element(Box::new(el));

        *counter_.borrow_mut() += 1;
        let window = WindowPane::new(&ctx_, Box::new(sc_pane), &title)
            .with_corner_adjuster(&ctx_)
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30))
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10));

        let shadow_color = Color::new_with_alpha(50, 50, 50, 100);
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
            .with_width(DynVal::new_flex(1.))
            .with_height(DynVal::new_flex(1.));

        *counter_.borrow_mut() += 1;
        let window = WindowPane::new(&ctx_, Box::new(el), &title)
            .with_corner_adjuster(&ctx_)
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

    let add_button = Button::new(&ctx, "add_window", add_button_click_fn).at(1.into(), 1.into());
    let add_button2 =
        Button::new(&ctx, "add_window_scrollable", add_button_scr_click_fn).at(15.into(), 1.into());
    let add_button3 =
        Button::new(&ctx, "add_terminal_window", add_term_click_fn).at(40.into(), 1.into());
    pp.add_element(Box::new(add_button));
    pp.add_element(Box::new(add_button2));
    pp.add_element(Box::new(add_button3));

    cui.run(Box::new(pp)).await
}
