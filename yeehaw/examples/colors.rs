use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::log::reset_log_file("./debug_test.log".to_string());
    //std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;

    let time_gr = vec![
        (std::time::Duration::from_secs(0), Color::RED),
        (std::time::Duration::from_secs(1), Color::GREEN),
        (std::time::Duration::from_secs(2), Color::BLUE),
        (std::time::Duration::from_secs(3), Color::RED),
    ];
    let t1 = Color::TimeGradient(TimeGradient::new(
        std::time::Duration::from_secs(3),
        time_gr,
    ));

    let time_gr = vec![
        (std::time::Duration::from_secs(0), Color::GREEN),
        (std::time::Duration::from_secs(1), Color::BLUE),
        (std::time::Duration::from_secs(2), Color::RED),
        (std::time::Duration::from_secs(3), Color::GREEN),
    ];
    let t2 = Color::TimeGradient(TimeGradient::new(
        std::time::Duration::from_secs(3),
        time_gr,
    ));

    let time_gr = vec![
        (std::time::Duration::from_secs(0), Color::BLUE),
        (std::time::Duration::from_secs(1), Color::RED),
        (std::time::Duration::from_secs(2), Color::GREEN),
        (std::time::Duration::from_secs(3), Color::BLUE),
    ];
    let t3 = Color::TimeGradient(TimeGradient::new(
        std::time::Duration::from_secs(3),
        time_gr,
    ));

    //let grad = vec![
    //    (DynVal::new_fixed(0), Color::RED),
    //    (DynVal::new_flex(0.5), Color::GREEN),
    //    (DynVal::FULL, Color::BLUE),
    //];

    //let grad2 = vec![
    //    (DynVal::new_fixed(0), Color::ORANGE),
    //    (DynVal::new_fixed(15), Color::BLUE),
    //    (DynVal::new_fixed(30), Color::BLACK),
    //];
    //let g2 = Color::Gradient(Gradient::new(grad2, 90.));

    //let grad = vec![
    //    //(DynVal::new_fixed(0), t1),
    //    (DynVal::new_fixed(0), t1.clone()),
    //    (DynVal::new_fixed(15), t2),
    //    (DynVal::new_fixed(30), t3),
    //    (DynVal::new_fixed(45), t1),
    //];

    //let el_bg = Color::Gradient(Gradient::new(grad, 60.));

    let rgrad = vec![
        (DynVal::new_flex(0.), t1.clone()),
        (DynVal::new_flex(0.2), t2),
        (DynVal::new_flex(0.4), t3),
        (DynVal::new_flex(0.5), t1),
    ];

    let el_bg = Color::RadialGradient(RadialGradient {
        draw_size: None,
        offset: (0, 0),
        center: (0.25.into(), 0.5.into()),
        skew: (1., 1. / 0.55),
        grad: rgrad,
    });

    let el = ParentPane::new(&ctx, "color-test").with_bg_color(el_bg);

    tui.run(Box::new(el)).await
}
