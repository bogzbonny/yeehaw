use {
    //std::env,
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        //debug,
        widgets::{
            megafonts, widget_button::ButtonSides, Button, Checkbox, DropdownList, Label, ListBox,
            Megatext, NumbersTextBox, RadioButtons, TextBox, Toggle, WBStyles,
        },
        Color,
        Context,
        Cui,
        DynVal,
        Element,
        Error,
        EventResponses,
        Gradient,
        SortingHat,
        TimeGradient,
        WidgetPane,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./widget_test.log".to_string());
    //yeehaw::debug::clear();
    //std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();

    //let x_length = 10;
    //let x_grad = vec![
    //    (DynVal::new_fixed(0), Color::RED),
    //    (DynVal::new_fixed(x_length), Color::GREEN),
    //    (DynVal::new_fixed(2 * x_length), Color::RED),
    //];
    //let y_length = 10;
    //let y_grad = vec![
    //    (DynVal::new_fixed(0), Color::TRANSPARENT),
    //    (DynVal::new_fixed(y_length), Color::WHITE),
    //    (DynVal::new_fixed(2 * y_length), Color::TRANSPARENT),
    //    //(DynVal::new_fixed(0), Color::GREY),
    //    //(
    //    //    DynVal::new_fixed(y_length),
    //    //    Color::new_with_alpha(100, 200, 100, 255),
    //    //),
    //    //(DynVal::new_fixed(2 * y_length), Color::GREY),
    //];
    //let el_bg = Color::Gradient(Gradient::new(x_grad, y_grad));

    let time_gr = vec![
        (std::time::Duration::from_secs(0), Color::BLUE),
        (std::time::Duration::from_secs(2), Color::GREEN),
        (std::time::Duration::from_secs(4), Color::BLUE),
    ];
    let time_gr = Color::TimeGradient(TimeGradient::new(
        std::time::Duration::from_secs(4),
        time_gr,
    ));

    let x_grad = vec![
        (DynVal::new_fixed(0), Color::RED),
        (DynVal::new_fixed(20), time_gr),
        (DynVal::new_fixed(40), Color::RED),
    ];
    //let el_bg1 = Color::Gradient(Gradient::new(x_grad.clone(), vec![]));
    let el_bg1 = Color::Gradient(Gradient::new(x_grad.clone(), vec![]));
    let y_grad = vec![
        (DynVal::new_flex(0.), Color::WHITE),
        (DynVal::new_flex(0.5), el_bg1.clone()),
        //(DynVal::new_flex(0.5), Color::BLUE),
        (DynVal::new_flex(1.), Color::WHITE),
    ];
    let el_bg = Color::Gradient(Gradient::new(vec![], y_grad));

    let el = WidgetPane::new(&hat).with_bg_color(el_bg);
    //let ctx = Context::new_context_for_screen();

    Cui::new(Rc::new(RefCell::new(el)))?.run().await
}
