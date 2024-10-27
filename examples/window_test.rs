use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        widgets::Button, ChPlus, Color, Context, Cui, DebugSizePane, DrawCh, DynVal, Error,
        EventResponse, ParentPane, SortingHat, Style, WindowPane,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    yeehaw::debug::set_log_file("./debug_test.log".to_string());
    yeehaw::debug::clear();
    std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let ctx = Context::new_context_for_screen_no_dur();

    let pp = ParentPane::new(&hat, "parent_pane")
        .with_dyn_height(1.0.into())
        .with_dyn_width(1.0.into());

    let counter = Rc::new(RefCell::new(0));

    let hat_ = hat.clone();
    //let pp_ = pp.clone();
    let add_button_click_fn = Box::new(move |_, mut ctx_: Context| {
        let title = format!("Pane {}", *counter.borrow());
        let bg = Color::new_with_alpha(150, 150, 155, 150);
        let fg = Color::new_with_alpha(150, 150, 155, 150);
        let def_ch = DrawCh::new(
            ChPlus::Transparent,
            Style::default().with_bg(bg.clone()).with_fg(fg),
        );

        let el = DebugSizePane::new(&hat_)
            .with_text(title.clone())
            .with_width(DynVal::new_flex(1.))
            .with_height(DynVal::new_flex(1.))
            .with_default_ch(def_ch)
            .with_style(Style::default().with_bg(bg).with_fg(Color::BLACK));

        *counter.borrow_mut() += 1;
        ctx_.s.width = 30;
        ctx_.s.height = 20;
        let window = WindowPane::new(&hat_, &ctx_, Rc::new(RefCell::new(el)), &title)
            .with_corner_adjuster(&hat_, &ctx_)
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10))
            .with_height(DynVal::new_fixed(20))
            .with_width(DynVal::new_fixed(30));

        let resp = EventResponse::NewElement(Rc::new(RefCell::new(window)));
        resp.into()
    });

    let add_button =
        Button::new(&hat, &ctx, "add_window", add_button_click_fn).at(1.into(), 1.into());
    pp.add_element(Rc::new(RefCell::new(add_button)));

    Cui::new(Rc::new(RefCell::new(pp)))?.run().await
}
