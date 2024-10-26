use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        //debug,
        widgets::Button,
        Context,
        Cui,
        DebugSizePane,
        DynVal,
        Error,
        EventResponse,
        ParentPane,
        SortingHat,
        WindowPane,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let ctx = Context::new_context_for_screen_no_dur();

    let pp = ParentPane::new(&hat, "parent_pane");

    let counter = Rc::new(RefCell::new(0));

    let hat_ = hat.clone();
    //let pp_ = pp.clone();
    let add_button_click_fn = Box::new(move |ctx_| {
        let title = format!("Pane {}", *counter.borrow());
        let el = DebugSizePane::new(&hat_).with_text(title.clone());
        *counter.borrow_mut() += 1;
        let window = WindowPane::new(&hat_, &ctx_, Rc::new(RefCell::new(el)), &title)
            .with_width(DynVal::new_fixed(20))
            .with_height(DynVal::new_fixed(10))
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10));

        let resp = EventResponse::NewElement(Rc::new(RefCell::new(window)));
        resp.into()
    });

    let add_button =
        Button::new(&hat, &ctx, "add_pane", add_button_click_fn).at(1.into(), 1.into());
    pp.add_element(Rc::new(RefCell::new(add_button)));

    Cui::new(Rc::new(RefCell::new(pp)))?.run().await
}
