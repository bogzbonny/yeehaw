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
    //yeehaw::debug::set_log_file("./debug_test.log".to_string());
    //yeehaw::debug::clear();
    std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();

    let time_gr = vec![
        (std::time::Duration::from_secs(0), Color::RED),
        (std::time::Duration::from_secs(1), Color::GREEN),
        (std::time::Duration::from_secs(2), Color::BLUE),
    ];
    let t1 = Color::TimeGradient(TimeGradient::new(
        std::time::Duration::from_secs(3),
        time_gr,
    ));

    let time_gr = vec![
        (std::time::Duration::from_secs(0), Color::GREEN),
        (std::time::Duration::from_secs(1), Color::BLUE),
        (std::time::Duration::from_secs(2), Color::RED),
    ];
    let t2 = Color::TimeGradient(TimeGradient::new(
        std::time::Duration::from_secs(3),
        time_gr,
    ));

    let time_gr = vec![
        (std::time::Duration::from_secs(0), Color::BLUE),
        (std::time::Duration::from_secs(1), Color::RED),
        (std::time::Duration::from_secs(2), Color::GREEN),
    ];
    let t3 = Color::TimeGradient(TimeGradient::new(
        std::time::Duration::from_secs(3),
        time_gr,
    ));

    //let grad = vec![
    //    (DynVal::new_fixed(0), Color::RED),
    //    (DynVal::new_flex(0.5), Color::GREEN),
    //    (DynVal::new_flex(1.), Color::BLUE),
    //];

    //let grad2 = vec![
    //    (DynVal::new_fixed(0), Color::ORANGE),
    //    (DynVal::new_fixed(15), Color::BLUE),
    //    (DynVal::new_fixed(30), Color::BLACK),
    //];
    //let g2 = Color::Gradient(Gradient::new(grad2, 90.));

    let grad = vec![
        //(DynVal::new_fixed(0), t1),
        (DynVal::new_fixed(0), t1),
        (DynVal::new_fixed(15), t2),
        (DynVal::new_fixed(30), t3),
    ];

    let el_bg = Color::Gradient(Gradient::new(grad, 0.));

    let el = WidgetPane::new(&hat).with_bg_color(el_bg);
    Cui::new(Rc::new(RefCell::new(el)))?.run().await
}
