mod shared;
use {
    shared::{colors::*, editor::*, image::*, widgets::*},
    yeehaw::*,
};

/// It is recommended to run this example with the `--release` flag (there is a lot going on)
///
/// NOTE this example requires steam locomotive (`sl`) to be installed if you want
/// to see the train. Additionally it should be run from this showcase directory
/// for the showcase-tab to work (which runs `cargo run --release --example showcase`).

#[tokio::main]
async fn main() -> Result<(), Error> {
    // uncomment the following line to enable logging
    //yeehaw::log::reset_log_file("./debug_test.log".to_string());
    std::env::set_var("RUST_BACKTRACE", "1");

    let (mut tui, ctx) = Tui::new()?;
    let main = PaneScrollable::new_expanding(&ctx, 135, 40);
    let limiter = PaneLimiter::new(Box::new(main.clone()), 135, 40);
    //let limiter = PaneLimiter::new(Box::new(main.clone()), 1000, 1000);
    let main_vs = VerticalStackFocuser::new(&ctx);
    main.add_element(Box::new(main_vs.clone()));

    // adding the menu bar and menu items

    let mb = MenuBar::top_menu_bar(&ctx).at(0, 0);
    for i in 0..3 {
        mb.add_item(&ctx, format!("upper/item-{i}"), None);
        mb.add_item(&ctx, format!("menu/item-{i}"), None);
        mb.add_item(&ctx, format!("bar/item-{i}"), None);
    }
    for j in 0..10 {
        let i = 3;
        mb.add_item(&ctx, format!("menu/item-{i}/sub-item-{j}"), None);
        mb.add_item(&ctx, format!("bar/item-{i}/sub-item-{j}"), None);
    }
    for j in 0..10 {
        for k in 0..3 {
            let i = 3;
            mb.add_item(
                &ctx,
                format!("upper/item-{i}/sub-{i}-item-{j}/sub-{i}-{j}-item-{k}"),
                None,
            );
        }
    }
    main_vs.push(Box::new(mb));

    let header_pane = ParentPaneOfSelectable::new(&ctx)
        .with_dyn_height(DynVal::new_fixed(7))
        .with_focused(false);
    main_vs.push(Box::new(header_pane.clone()));

    let gr = Gradient::x_grad_rainbow(&ctx, 5).with_angle(60.);
    let mtext = FigletText::new(
        &ctx,
        "YEEEHAW!!",
        figlet_rs::FIGfont::from_content(std::include_str!("../../assets/figlet/ANSI_Shadow.flf"))
            .expect("missing asset"),
    )
    .with_min_height()
    .with_style(Style::default().with_fg(Color::Gradient(gr)))
    .at(0, DynVal::new_fixed(1));
    header_pane.add_element(Box::new(mtext));

    let main_ = main.clone();
    let ctx_ = ctx.clone();
    let button = Button::new(&ctx, "do not\nclick me")
        .with_fn(Box::new(move |_, _| {
            bsod(&ctx_, main_.pane.clone());
            EventResponses::default()
        }))
        .at(DynVal::new_flex(0.9), DynVal::new_flex(0.3));
    header_pane.add_element(Box::new(button));

    let central_pane = HorizontalStackFocuser::new(&ctx);
    main_vs.push(Box::new(central_pane.clone()));
    let left_pane = VerticalStackFocuser::new(&ctx)
        .with_dyn_width(0.63)
        .with_bg(Color::BLACK);
    central_pane.push(Box::new(left_pane.clone()));

    left_pane.push(window_generation_zone(
        &ctx,
        Box::new(main_vs.pane.pane.clone()),
    ));

    //let train_pane = DebugSizePane::new(&ctx);
    let train_pane = TerminalPane::new(&ctx)?;
    train_pane.pane.set_dyn_height(0.7);
    train_pane.pane.set_focused(false);
    train_pane.execute_command("for i in {1..200}; do sl -l; done ; exit");
    train_pane.disable_cursor();
    left_pane.push(Box::new(train_pane));

    let tabs = Tabs::new(&ctx);
    tabs.pane.set_focused(false);
    let colors_tab = colors_demo(&ctx);
    let widgets_tab = widgets_demo(&ctx);
    let editor_tab = editor_demo(&ctx);
    let image_tab = image_demo(&ctx);
    let el_term = TerminalPane::new(&ctx)?;

    let showcase = TerminalPane::new(&ctx)?;
    showcase.set_env("YH_IMG_PROTOCOL", "1"); // TODO set to json
    showcase.pane.set_focused(false);

    let showcase_ = showcase.clone();
    let on_showcase_open_fn = Some(Box::new(move || {
        let command = "cargo run --release --example showcase";
        showcase_.execute_command(command);
    }) as Box<dyn FnOnce()>);

    tabs.push(colors_tab, "colors");
    tabs.push(widgets_tab, "widgets");
    tabs.push(editor_tab, "$EDITOR");
    tabs.push(image_tab, "images");
    tabs.push(Box::new(el_term), "terminal");
    tabs.push_with_on_open_fn(Box::new(showcase), "showcase", on_showcase_open_fn);
    tabs.select(0);
    central_pane.push(Box::new(tabs));

    //tui.run(Box::new(main)).await
    tui.run(Box::new(limiter)).await
}

pub fn bsod(ctx: &Context, main_pane: ParentPane) {
    let text = "

A problem has been detected and Windows has been shut down to prevent damage to your computer.

IRQL_NOT_LESS_OR_EQUAL

If this is the first time you've seen this Stop error screen, restart your computer. If this screen
appears again, follow these steps:

Check to make sure any new hardware or software is properly installed. If this is a new
installation, ask your hardware or software manufacturer for any Windows updates you might need.

If problems continue, disable or remove any newly installed hardware or software. Disable BIOS
memory options such as caching or shadowing. If you need to use Safe Mode to remove or disable
components, restart your computer, press F8 to select Advanced Startup Options, and then select
Safe Mode. 

Technical information:

*** STOP: 0x0000000A (0x00000004, 0x00000002, 0x00000000, 0x804F8677)

*** tuvix_memorial.exe - Address 804F8677 base at 804D7000, DateStamp 31a01870

Beginning dump of physical memory
Physical memory dump complete.
Contact your system administrator or technical support group for further assistance.
";

    let bsod = ParentPane::new(ctx, "bsod").with_bg(Color::BLUE);
    let text = Label::new(ctx, text).at(1, 1);

    // we need to send an exit command down to close the terminals...
    // TODO this should be handled automatically within clear_elements
    // just requires refactoring the context in.
    let _ = main_pane.receive_event(ctx, Event::Exit);
    bsod.add_element(Box::new(text.clone()));
    main_pane.clear_elements();
    main_pane.add_element(Box::new(bsod));
}

pub fn window_generation_zone(
    ctx: &Context, windows_generated_in: Box<ParentPane>,
) -> Box<dyn Element> {
    let sc = PaneScrollable::new_expanding(ctx, 50, 16);
    let el = ParentPaneOfSelectable::new(ctx);
    sc.add_element(Box::new(el.clone()));
    let bordered = Bordered::new_resizer(
        ctx,
        Box::new(sc.clone()),
        Style::transparent().with_fg(Color::WHITE),
    )
    .with_dyn_height(1.5);

    let l = Label::new(ctx, "window generation zone");
    el.add_element(Box::new(l));

    let dial_type = Dial::new_spacious(
        ctx,
        vec![
            (0, "Basic"),
            (1, "Fixed-Size"),
            (2, "Min-Size"),
            (3, "Terminal"),
        ],
    )
    .at(0, 3);
    let label = dial_type.label(ctx, "content:");
    el.add_element(Box::new(label));
    el.add_element(Box::new(dial_type.clone()));

    let dial_border = Dial::new_spacious(
        ctx,
        vec![
            (0, "None"),
            (1, "Basic"),
            (2, "Large"),
            (3, "Tight"),
            (6, "Double-Line"),
            (4, "Scrollbars"),
            (5, "Line-Scrollbars"),
            (7, "Text"),
            (8, "Resizer"),
            (9, "Mover"),
        ],
    )
    .at(DynVal::new_flex(0.35).with_min(19), 3);
    let label = dial_border.label(ctx, "border options:");
    el.add_element(Box::new(label));
    el.add_element(Box::new(dial_border.clone()));

    let shadow_cb = Checkbox::new(ctx).at(1, 8);
    let label = shadow_cb.label(ctx, " shadow");
    el.add_element(Box::new(shadow_cb.clone()));
    el.add_element(Box::new(label));

    let alpha_slider = Slider::new_basic_block(ctx)
        .with_gradient(ctx, Color::BLUE, Color::AQUA)
        .with_position(0.9)
        .with_dyn_width(DynVal::new_flex(0.4))
        .at(1, 11);

    let label = alpha_slider.label(ctx, "background alpha:");
    el.add_element(Box::new(label));
    el.add_element(Box::new(alpha_slider.clone()));

    let counter = Rc::new(RefCell::new(0));

    let ctx_ = ctx.clone();
    let counter_ = counter.clone();

    let generate_window_fn = Box::new(move |_, _| {
        let title = format!("Pane {}", *counter_.borrow());

        let alpha = (alpha_slider.get_position() * 255.0) as u8;

        let bg = Color::new_with_alpha(150, 150, 155, alpha);
        let fg = Color::new_with_alpha(150, 150, 155, alpha);
        let sty = Style::default().with_bg(bg.clone()).with_fg(fg.clone());
        let def_ch = DrawCh::new(ChPlus::Transparent, sty.clone());

        let el: Box<dyn Element> = match dial_type.get_value().as_str() {
            "Basic" => Box::new(
                DebugSizePane::new(&ctx_)
                    .with_text(title.clone())
                    .with_text_style(sty.clone().with_fg(Color::BLACK))
                    .with_dyn_width(DynVal::FULL)
                    .with_dyn_height(DynVal::FULL)
                    .with_default_ch(def_ch.clone()),
            ),
            "Fixed-Size" => {
                let el = DebugSizePane::new(&ctx_)
                    .with_text(title.clone())
                    .with_text_style(sty.clone().with_fg(Color::BLACK))
                    .with_dyn_width(DynVal::FULL)
                    .with_dyn_height(DynVal::FULL)
                    .with_default_ch(def_ch.clone());
                let sc_pane = PaneScrollable::new(&ctx_, 50, 25);
                sc_pane.add_element(Box::new(el));
                Box::new(sc_pane)
            }
            "Min-Size" => {
                let el = DebugSizePane::new(&ctx_)
                    .with_text(title.clone())
                    .with_text_style(sty.clone().with_fg(Color::BLACK))
                    .with_dyn_width(DynVal::FULL)
                    .with_dyn_height(DynVal::FULL)
                    .with_default_ch(def_ch.clone());
                let sc_pane = PaneScrollable::new_expanding(&ctx_, 50, 25);
                sc_pane.add_element(Box::new(el));
                Box::new(sc_pane)
            }
            "Terminal" => Box::new(TerminalPane::new(&ctx_).unwrap().with_focused(true)),
            _ => panic!("missing type implementation"),
        };

        let mut corner_resizer = false;
        let el: Box<dyn Element> = match dial_border.get_value().as_str() {
            "None" => {
                corner_resizer = true;
                el
            }
            "Basic" => Box::new(Bordered::new_basic(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Large" => Box::new(Bordered::new_large(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Tight" => Box::new(Bordered::new_tight(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Double-Line" => Box::new(Bordered::new_double(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Scrollbars" => {
                corner_resizer = true;
                Box::new(Bordered::new_borderless_with_scrollbars_and_thin_left(
                    &ctx_,
                    el,
                    sty.clone().with_fg(Color::WHITE),
                ))
            }
            "Line-Scrollbars" => Box::new(Bordered::new_resizer_with_scrollbars(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Text" => Box::new(
                Bordered::new_basic(&ctx_, el, sty.clone().with_fg(Color::BLACK))
                    .with_title("upper")
                    .with_bottom_right_text("lower")
                    .with_left_top_text("left")
                    .with_right_bottom_text("right"),
            ),
            "Resizer" => Box::new(Bordered::new_resizer(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            "Mover" => Box::new(Bordered::new_mover(
                &ctx_,
                el,
                sty.clone().with_fg(Color::BLACK),
            )),
            _ => panic!("missing type implementation"),
        };

        *counter_.borrow_mut() += 1;
        let mut window = WindowPane::new(&ctx_, el, &title)
            .with_dyn_height(DynVal::new_fixed(20))
            .with_dyn_width(DynVal::new_fixed(30))
            .at(DynVal::new_fixed(10), DynVal::new_fixed(10));

        if corner_resizer {
            window.set_corner_resizer(&ctx_);
        }

        let window: Box<dyn Element> = if shadow_cb.is_checked() {
            let shadow_color = Color::new_with_alpha(100, 100, 100, 150);
            Box::new(Shadowed::thick_with_color(
                Box::new(window.clone()),
                shadow_color,
            ))
        } else {
            Box::new(window)
        };

        let inner_resps = vec![
            EventResponse::BringToFront,
            EventResponse::UnfocusOthers,
            EventResponse::Focus,
        ];
        let resp = EventResponse::NewElement(window, Some(inner_resps.into()));
        windows_generated_in
            .pane
            .send_responses_upward(&ctx_, resp.into());

        EventResponses::default()
    });

    let button = Button::new(ctx, "generate")
        .with_fn(generate_window_fn)
        .at(1, 13);
    el.add_element(Box::new(button));

    Box::new(bordered)
}
