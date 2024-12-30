use {std::time::Duration, yeehaw::*};

pub fn colors_demo(ctx: &Context) -> Box<dyn Element> {
    let el = ParentPaneOfSelectable::new(ctx).with_bg(Color::DARK_OLIVE_GREEN);

    let art_width = DynVal::new_fixed(51);
    let art_height = DynVal::new_fixed(26);
    let min_first_col = DynVal::new_fixed(30);
    let max_first_col = DynVal::new_fixed(55); // my iq
    let total_height = DynVal::new_fixed(29);
    let excess_height = DynVal::FULL.minus(total_height).div(2.);
    let excess_width = DynVal::FULL.minus(art_width.clone()).minus(1.into());

    let first_col_end = excess_width.with_max(max_first_col).with_min(min_first_col);

    let y = excess_height.clone().with_min(1);
    let x = DynVal::new_fixed(1);

    let desc_text = "Within yeehaw, a standard\n\
                     color can be an RGBA or a\n\
                     gradient changing by position\n\
                     or time (or both!).";
    let description = Label::new(ctx, desc_text).at(x.clone(), y.clone());
    let y = DynVal::y_after(&description, 1);
    el.add_element(Box::new(description));

    //let x_tog = DynVal::new_fixed(18);
    let x_tog = first_col_end.minus(12.into());
    let toggle = Toggle::new(ctx, "  fg  ".to_string(), "  bg  ".to_string()).at(x_tog, y.clone());
    let y = DynVal::y_after(&toggle, 0);
    el.add_element(Box::new(toggle.clone()));

    let dial_color = Dial::new_spacious(
        ctx,
        vec![
            (0, "Solid"),
            (1, "Time-Gradient"),
            (2, "Radial-Gradient"),
            (3, "Linear-Gradient"),
            (4, "Radial-Time"),
            (5, "Linear-Time"),
            (6, "Tiles"),
        ],
    )
    .with_label_color(ctx, Color::GREY22)
    .at(x.clone(), y.clone());
    let y = DynVal::y_after(&dial_color, 1);
    el.add_element(Box::new(dial_color.label(ctx, "color kind:")));
    el.add_element(Box::new(dial_color.clone()));

    let dd_x = x.plus(7.into());
    let color_dd = DropdownList::new(
        ctx,
        vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"],
        Box::new(|_, _| EventResponses::default()),
    )
    .with_dyn_width(5.into())
    .with_max_expanded_height(8)
    .at(dd_x, y.clone());
    let y = DynVal::y_after(&color_dd, 0);
    el.add_element(Box::new(color_dd.label_left(ctx, "color (")));
    el.add_element(Box::new(color_dd.label_right(ctx, "):")));
    el.add_element(Box::new(color_dd.clone()));

    let x_nb = x.plus(3.into());
    let ntb_width = DynVal::new_fixed(8);
    let r_ntb = NumbersTextBox::new(ctx, 128usize)
        .with_min(0)
        .with_max(255)
        .with_dyn_width(ntb_width)
        .at(x_nb.clone(), y.clone());
    el.add_element(Box::new(r_ntb.clone()));
    el.add_element(Box::new(r_ntb.label_left_top(ctx, "R: ")));
    let x_slider = x_nb.plus(9.into());
    let ntb_ = r_ntb.clone();
    let r_slider = Slider::new_basic_block(ctx)
        .with_color(Color::RED)
        .at(x_slider.clone(), y.clone())
        .with_end_x(first_col_end.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 255.) as usize;
            ntb_.set_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&r_slider, 0);
    el.add_element(Box::new(r_slider.clone()));

    let x_nb = x.plus(3.into());
    let ntb_width = DynVal::new_fixed(8);
    let g_ntb = NumbersTextBox::new(ctx, 128usize)
        .with_min(0)
        .with_max(255)
        .with_dyn_width(ntb_width)
        .at(x_nb.clone(), y.clone());
    el.add_element(Box::new(g_ntb.clone()));
    el.add_element(Box::new(g_ntb.label_left_top(ctx, "G: ")));
    let x_slider = x_nb.plus(9.into());
    let ntb_ = g_ntb.clone();
    let g_slider = Slider::new_basic_block(ctx)
        .with_color(Color::GREEN)
        .at(x_slider.clone(), y.clone())
        .with_end_x(first_col_end.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 255.) as usize;
            ntb_.set_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&g_slider, 0);
    el.add_element(Box::new(g_slider.clone()));

    let x_nb = x.plus(3.into());
    let ntb_width = DynVal::new_fixed(8);
    let b_ntb = NumbersTextBox::new(ctx, 128usize)
        .with_min(0)
        .with_max(255)
        .with_dyn_width(ntb_width)
        .at(x_nb.clone(), y.clone());
    el.add_element(Box::new(b_ntb.clone()));
    el.add_element(Box::new(b_ntb.label_left_top(ctx, "B: ")));
    let x_slider = x_nb.plus(9.into());
    let ntb_ = b_ntb.clone();
    let b_slider = Slider::new_basic_block(ctx)
        .with_color(Color::BLUE)
        .at(x_slider.clone(), y.clone())
        .with_end_x(first_col_end.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 255.) as usize;
            ntb_.set_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&b_slider, 0);
    el.add_element(Box::new(b_slider.clone()));

    let x_nb = x.plus(3.into());
    let ntb_width = DynVal::new_fixed(8);
    let a_ntb = NumbersTextBox::new(ctx, 128usize)
        .with_min(0)
        .with_max(255)
        .with_dyn_width(ntb_width)
        .at(x_nb.clone(), y.clone());
    el.add_element(Box::new(a_ntb.clone()));
    el.add_element(Box::new(a_ntb.label_left_top(ctx, "A: ")));
    let x_slider = x_nb.plus(9.into());
    let ntb_ = a_ntb.clone();
    let a_slider = Slider::new_basic_block(ctx)
        .with_color(Color::AQUA)
        .at(x_slider.clone(), y.clone())
        .with_end_x(first_col_end.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 255.) as usize;
            ntb_.set_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&a_slider, 1);
    el.add_element(Box::new(a_slider.clone()));

    r_slider.set_position(0.5);
    g_slider.set_position(0.5);
    b_slider.set_position(0.5);
    a_slider.set_position(0.5);

    let dd_x = x.plus(22.into());
    let max_gr_colors_dd = DropdownList::new(
        ctx,
        vec!["2", "3", "4", "5", "6", "7", "8", "9", "10"],
        Box::new(|_, _| EventResponses::default()),
    )
    .with_dyn_width(5.into())
    .with_max_expanded_height(8)
    .at(dd_x, y.clone());
    let y = DynVal::y_after(&max_gr_colors_dd, 2);
    el.add_element(Box::new(
        max_gr_colors_dd.label_left(ctx, "# of gradient colors: "),
    ));
    el.add_element(Box::new(max_gr_colors_dd.clone()));

    let ntb_width = DynVal::new_fixed(8);
    let dist_ntb = NumbersTextBox::new(ctx, 50usize)
        .with_min(1)
        .with_max(20)
        .with_dyn_width(ntb_width)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(dist_ntb.clone()));
    el.add_element(Box::new(dist_ntb.label(ctx, "gradient color distance:")));
    let x_slider = x.plus(9.into());
    let ntb_ = dist_ntb.clone();
    let dist_slider = Slider::new_basic_block(ctx)
        .with_color(Color::AQUA)
        .at(x_slider.clone(), y.clone())
        .with_end_x(first_col_end.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 20.) as usize;
            ntb_.set_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&dist_slider, 2);
    el.add_element(Box::new(dist_slider.clone()));

    let ntb_width = DynVal::new_fixed(8);
    let angle_ntb = NumbersTextBox::new(ctx, 0f64)
        .with_min(0.)
        .with_max(360.)
        .with_decimal_places(2)
        .with_dyn_width(ntb_width)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(angle_ntb.clone()));
    el.add_element(Box::new(
        angle_ntb.label(ctx, "linear gradient angle (deg):"),
    ));
    let x_slider = x.plus(9.into());
    let ntb_ = angle_ntb.clone();
    let angle_slider = Slider::new_basic_block(ctx)
        .with_color(Color::AQUA)
        .at(x_slider.clone(), y.clone())
        .with_end_x(first_col_end.clone())
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = p * 360.;
            ntb_.set_value(p);
            EventResponses::default()
        }));
    let y = DynVal::y_after(&angle_slider, 2);
    el.add_element(Box::new(angle_slider.clone()));

    let ntb_width = DynVal::new_fixed(8);
    let time_ntb = NumbersTextBox::new(ctx, 1000usize)
        .with_min(100)
        .with_max(2000)
        .with_dyn_width(ntb_width)
        .at(x.clone(), y.clone());
    el.add_element(Box::new(time_ntb.clone()));
    el.add_element(Box::new(time_ntb.label(ctx, "time gradient ms:")));
    let x_slider = x.plus(9.into());
    let ntb_ = time_ntb.clone();
    let time_slider = Slider::new_basic_block(ctx)
        .with_color(Color::AQUA)
        .at(x_slider.clone(), y.clone())
        .with_end_x(first_col_end)
        .with_fn(Box::new(move |_, sl| {
            let p = sl.get_position();
            let p = (p * 2000.) as usize;
            ntb_.set_value(p);
            EventResponses::default()
        }));
    el.add_element(Box::new(time_slider.clone()));

    // ------------------------------

    let y_art = excess_height.with_min(1).minus(1.into());
    let x_art = DynVal::x_after(&toggle, 1);
    let bg_art = ParentPane::new(ctx, "art");
    let fg_art = Label::new(ctx, "");
    bg_art.add_element(Box::new(fg_art.clone()));

    let art = Bordered::new_tight(
        ctx,
        Box::new(bg_art.clone()),
        Style::transparent().with_fg(Color::WHITE),
    )
    .with_dyn_height(art_height)
    .with_dyn_width(art_width)
    .at(x_art.clone(), y_art.clone());
    let y_dial_fg_art = DynVal::y_after(&art, 1);
    let x_dial_fg_art = x_art.plus(12.into());

    el.add_element(Box::new(art));

    let dial_fg_art = Dial::new_spacious(
        ctx,
        vec![
            (1, "None"),
            (2, "Butterfly"),
            (3, "Spiral"),
            (8, "Rust-Logo"),
            (9, "Saturn"),
            (10, "Chompy"),
        ],
    )
    .with_position(2)
    .with_label_color(ctx, Color::GREY22)
    .at(x_dial_fg_art, y_dial_fg_art);
    el.add_element(Box::new(dial_fg_art.label_left_top(ctx, "fg text:")));
    el.add_element(Box::new(dial_fg_art.clone()));

    let state = ColorsDemoState {
        fg: Rc::new(RefCell::new(ColorsDemoColor::default_fg(ctx))),
        bg: Rc::new(RefCell::new(ColorsDemoColor::default_bg(ctx))),
        updating: Rc::new(RefCell::new(false)),
        drawing_fg: fg_art,
        drawing_bg: bg_art,
        toggle: toggle.clone(),
        dial_fg_art: dial_fg_art.clone(),
        dial_color_kind: dial_color.clone(),
        color_dd: color_dd.clone(),
        max_gr_colors_dd: max_gr_colors_dd.clone(),
        dist_ntb: dist_ntb.clone(),
        angle_ntb: angle_ntb.clone(),
        time_ntb: time_ntb.clone(),
        r_ntb: r_ntb.clone(),
        g_ntb: g_ntb.clone(),
        b_ntb: b_ntb.clone(),
        a_ntb: a_ntb.clone(),
        dist_slider: dist_slider.clone(),
        angle_slider: angle_slider.clone(),
        time_slider: time_slider.clone(),
    };

    let state_ = state.clone();
    dial_fg_art.set_fn(Box::new(move |_, _, _, _| {
        state_.update_drawing();
        EventResponses::default()
    }));

    let state_ = state.clone();
    toggle.set_fn(Box::new(move |ctx, _| {
        state_.update_for_toggle_change(&ctx);
        EventResponses::default()
    }));

    let state_ = state.clone();
    dial_color.set_fn(Box::new(move |ctx, _, _, _| {
        state_.update_for_color_dial_change(&ctx);
        EventResponses::default()
    }));

    let state_ = state.clone();
    let ctx_ = ctx.clone();
    color_dd.set_fn(Box::new(move |_, _| {
        if *state_.updating.borrow() {
            return EventResponses::default();
        }
        state_.update_for_color_dd_change();
        state_.update_for_minor_changes(&ctx_);
        EventResponses::default()
    }));

    let state_ = state.clone();
    let color_dd_ = color_dd.clone();
    max_gr_colors_dd.set_fn(Box::new(move |ctx, value| {
        let max = value.parse::<usize>().ok().unwrap_or(1);
        let entries = (1..=max).map(|i| i.to_string()).collect::<Vec<String>>();
        color_dd_.set_entries(entries);
        let _ = color_dd_.set_selected(&ctx, 0);
        state_.update_for_minor_changes(&ctx);
        EventResponses::default()
    }));

    let state_ = state.clone();
    let dist_slider_ = dist_slider.clone();
    let ctx_ = ctx.clone();
    dist_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 20.;
        dist_slider_.set_position(v);
        state_.update_for_minor_changes(&ctx_);
        EventResponses::default()
    }));

    let state_ = state.clone();
    let angle_slider_ = angle_slider.clone();
    let ctx_ = ctx.clone();
    angle_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v / 360.;
        angle_slider_.set_position(v);
        state_.update_for_minor_changes(&ctx_);
        EventResponses::default()
    }));

    let state_ = state.clone();
    let time_slider_ = time_slider.clone();
    let ctx_ = ctx.clone();
    time_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 2000.;
        time_slider_.set_position(v);
        state_.update_for_minor_changes(&ctx_);
        EventResponses::default()
    }));

    let state_ = state.clone();
    let r_slider_ = r_slider.clone();
    let ctx_ = ctx.clone();
    r_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 255.;
        r_slider_.set_position(v);
        state_.update_for_minor_changes(&ctx_);
        EventResponses::default()
    }));

    let state_ = state.clone();
    let g_slider_ = g_slider.clone();
    let ctx_ = ctx.clone();
    g_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 255.;
        g_slider_.set_position(v);
        state_.update_for_minor_changes(&ctx_);
        EventResponses::default()
    }));

    let state_ = state.clone();
    let b_slider_ = b_slider.clone();
    let ctx_ = ctx.clone();
    b_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 255.;
        b_slider_.set_position(v);
        state_.update_for_minor_changes(&ctx_);
        EventResponses::default()
    }));

    let state_ = state.clone();
    let a_slider_ = a_slider.clone();
    let ctx_ = ctx.clone();
    a_ntb.set_value_changed_hook(Box::new(move |v| {
        let v = v as f64 / 255.;
        a_slider_.set_position(v);
        state_.update_for_minor_changes(&ctx_);
        EventResponses::default()
    }));

    //state.update_for_toggle_change(ctx);
    toggle.toggle_right(ctx);
    let _ = dial_fg_art.set_value(ctx, "Butterfly", true);
    let _ = dial_color.set_value(ctx, "Linear-Time", true);

    Box::new(el)
}

#[derive(Clone)]
pub struct ColorsDemoState {
    pub fg: Rc<RefCell<ColorsDemoColor>>,
    pub bg: Rc<RefCell<ColorsDemoColor>>,

    // whether or not the state is currently be updated (do not draw)
    pub updating: Rc<RefCell<bool>>,

    pub drawing_fg: Label,
    pub drawing_bg: ParentPane,

    pub toggle: Toggle,
    pub dial_fg_art: Dial,
    pub dial_color_kind: Dial,
    pub color_dd: DropdownList,
    pub max_gr_colors_dd: DropdownList,
    pub dist_ntb: NumbersTextBox<usize>,
    pub angle_ntb: NumbersTextBox<f64>,
    pub time_ntb: NumbersTextBox<usize>,
    pub r_ntb: NumbersTextBox<usize>,
    pub g_ntb: NumbersTextBox<usize>,
    pub b_ntb: NumbersTextBox<usize>,
    pub a_ntb: NumbersTextBox<usize>,

    pub dist_slider: Slider,
    pub angle_slider: Slider,
    pub time_slider: Slider,
}

#[derive(Default)]
pub struct ColorsDemoColor {
    pub kind: ColorsDemoColorKind,
    // all inner states are kept so they may be returned too.
    // Vec<Color> is kept so that gradients can have memory of colors
    // beyond the max gradient colors.
    pub solid_state: Color,
    pub time_gradient_state: (TimeGradient, Vec<Color>),
    pub radial_gradient_state: (RadialGradient, Vec<Color>),
    pub linear_gradient_state: (Gradient, Vec<Color>),
    pub radial_time_state: (RadialGradient, Vec<TimeGradient>, Vec<Color>),
    pub linear_time_state: (Gradient, Vec<TimeGradient>, Vec<Color>),
    pub tiles_state: (Pattern, Color, Color),
}

#[derive(Clone, Copy, Default)]
pub enum ColorsDemoColorKind {
    #[default]
    Solid,
    TimeGradient,
    RadialGradient,
    LinearGradient,
    RadialTime,
    LinearTime,
    Tiles,
}

impl ColorsDemoState {
    /// updates all the sliders/tbs for a dial change
    pub fn update_for_toggle_change(&self, ctx: &Context) {
        let kind =
            if self.toggle.is_left() { self.fg.borrow().kind } else { self.bg.borrow().kind };

        let v = match kind {
            ColorsDemoColorKind::Solid => "Solid",
            ColorsDemoColorKind::TimeGradient => "Time-Gradient",
            ColorsDemoColorKind::RadialGradient => "Radial-Gradient",
            ColorsDemoColorKind::LinearGradient => "Linear-Gradient",
            ColorsDemoColorKind::RadialTime => "Radial-Time",
            ColorsDemoColorKind::LinearTime => "Linear-Time",
            ColorsDemoColorKind::Tiles => "Tiles",
        };
        // setting the dial with trigger update_for_color_dial_change
        let _ = self.dial_color_kind.set_value(ctx, v, true);
    }

    /// updates all the sliders/tbs for a dial change
    pub fn update_for_color_dial_change(&self, ctx: &Context) {
        {
            self.updating.replace(true);
            let mut demo_color =
                if self.toggle.is_left() { self.fg.borrow_mut() } else { self.bg.borrow_mut() };
            match self.dial_color_kind.get_value().as_str() {
                "Solid" => {
                    demo_color.kind = ColorsDemoColorKind::Solid;

                    self.angle_ntb.tb.pane.disable();
                    self.angle_slider.pane.disable();
                    self.dist_ntb.tb.pane.disable();
                    self.dist_slider.pane.disable();
                    self.time_ntb.tb.pane.disable();
                    self.time_slider.pane.disable();
                    self.color_dd.pane.disable();
                    self.max_gr_colors_dd.pane.disable();

                    let _ = self.color_dd.set_selected(ctx, 0);
                    let _ = self.max_gr_colors_dd.set_selected(ctx, 0);
                    self.update_for_color_dd_change_from_demo_color("Solid", &demo_color);
                }
                "Time-Gradient" => {
                    demo_color.kind = ColorsDemoColorKind::TimeGradient;

                    self.angle_ntb.tb.pane.disable();
                    self.angle_slider.pane.disable();
                    self.dist_ntb.tb.pane.disable();
                    self.dist_slider.pane.disable();

                    self.max_gr_colors_dd.pane.enable();
                    self.color_dd.pane.enable();
                    self.time_ntb.tb.pane.enable();
                    self.time_slider.pane.enable();

                    let max = demo_color.time_gradient_state.0.len(ctx) - 1;
                    let _ = self
                        .max_gr_colors_dd
                        .set_selected_str(ctx, &max.to_string());
                    self.update_for_color_dd_change_from_demo_color("Time-Gradient", &demo_color);

                    // update the time
                    let time = demo_color.time_gradient_state.0.get_grad(ctx)[1]
                        .0
                        .as_millis() as usize;
                    self.time_ntb.set_value(time);
                }
                "Radial-Gradient" => {
                    demo_color.kind = ColorsDemoColorKind::RadialGradient;
                    self.angle_ntb.tb.pane.disable();
                    self.angle_slider.pane.disable();
                    self.time_ntb.tb.pane.disable();
                    self.time_slider.pane.disable();

                    self.dist_ntb.tb.pane.enable();
                    self.dist_slider.pane.enable();
                    self.color_dd.pane.enable();
                    self.max_gr_colors_dd.pane.enable();

                    let max = demo_color.radial_gradient_state.0.len(ctx);
                    let _ = self
                        .max_gr_colors_dd
                        .set_selected_str(ctx, &max.to_string());
                    self.update_for_color_dd_change_from_demo_color("Radial-Gradient", &demo_color);

                    // update the dist
                    let dist = demo_color.radial_gradient_state.0.get_grad(ctx)[1].0.fixed;
                    self.dist_ntb.set_value(dist as usize);
                }
                "Linear-Gradient" => {
                    demo_color.kind = ColorsDemoColorKind::LinearGradient;

                    self.time_ntb.tb.pane.disable();
                    self.time_slider.pane.disable();

                    self.angle_ntb.tb.pane.enable();
                    self.angle_slider.pane.enable();
                    self.dist_ntb.tb.pane.enable();
                    self.dist_slider.pane.enable();
                    self.color_dd.pane.enable();
                    self.max_gr_colors_dd.pane.enable();

                    let max = demo_color.linear_gradient_state.0.len(ctx) - 1;
                    let _ = self
                        .max_gr_colors_dd
                        .set_selected_str(ctx, &max.to_string());
                    self.update_for_color_dd_change_from_demo_color("Linear-Gradient", &demo_color);

                    // update the dist
                    let dist = demo_color.linear_gradient_state.0.get_grad(ctx)[1].0.fixed;
                    self.dist_ntb.set_value(dist as usize);

                    // update the angle
                    let angle = demo_color.linear_gradient_state.0.angle_deg;
                    self.angle_ntb.set_value(angle);
                }
                "Radial-Time" => {
                    demo_color.kind = ColorsDemoColorKind::RadialTime;

                    self.angle_ntb.tb.pane.disable();
                    self.angle_slider.pane.disable();

                    self.time_ntb.tb.pane.enable();
                    self.time_slider.pane.enable();
                    self.dist_ntb.tb.pane.enable();
                    self.dist_slider.pane.enable();
                    self.color_dd.pane.enable();
                    self.max_gr_colors_dd.pane.enable();

                    let max = demo_color.radial_time_state.0.len(ctx);
                    let _ = self
                        .max_gr_colors_dd
                        .set_selected_str(ctx, &max.to_string());
                    self.update_for_color_dd_change_from_demo_color("Radial-Time", &demo_color);

                    // update the time
                    let grad = demo_color.radial_time_state.0.get_grad(ctx);
                    let Color::TimeGradient(ref tg) = grad[1].1 else {
                        return;
                    };
                    let time = tg.get_grad(ctx)[1].0.as_millis() as usize;
                    self.time_ntb.set_value(time);

                    // update the dist
                    let dist = grad[1].0.fixed;
                    self.dist_ntb.set_value(dist as usize);
                }
                "Linear-Time" => {
                    demo_color.kind = ColorsDemoColorKind::LinearTime;
                    self.angle_ntb.tb.pane.enable();
                    self.angle_slider.pane.enable();
                    self.time_ntb.tb.pane.enable();
                    self.time_slider.pane.enable();
                    self.dist_ntb.tb.pane.enable();
                    self.dist_slider.pane.enable();
                    self.color_dd.pane.enable();
                    self.max_gr_colors_dd.pane.enable();

                    let max = demo_color.linear_time_state.0.len(ctx) - 1;
                    let _ = self
                        .max_gr_colors_dd
                        .set_selected_str(ctx, &max.to_string());
                    self.update_for_color_dd_change_from_demo_color("Linear-Time", &demo_color);

                    // update the time
                    let grad = demo_color.linear_time_state.0.get_grad(ctx);
                    let Color::TimeGradient(ref tg) = grad[1].1 else {
                        return;
                    };
                    let time = tg.get_grad(ctx)[1].0.as_millis() as usize;
                    self.time_ntb.set_value(time);

                    // update the dist
                    let dist = grad[1].0.fixed;
                    self.dist_ntb.set_value(dist as usize);

                    // update the angle
                    let angle = demo_color.linear_gradient_state.0.angle_deg;
                    self.angle_ntb.set_value(angle);
                }
                "Tiles" => {
                    demo_color.kind = ColorsDemoColorKind::Tiles;
                    self.color_dd.pane.enable();
                    self.dist_ntb.tb.pane.enable();
                    self.dist_slider.pane.enable();

                    self.max_gr_colors_dd.pane.disable();
                    self.angle_ntb.tb.pane.disable();
                    self.angle_slider.pane.disable();
                    self.time_ntb.tb.pane.disable();
                    self.time_slider.pane.disable();

                    // update the dist
                    let pat = demo_color.tiles_state.0.get_pattern(ctx);
                    let dist = pat[1].len() / 2;
                    self.dist_ntb.set_value(dist);

                    let _ = self.max_gr_colors_dd.set_selected_str(ctx, "2");
                    self.update_for_color_dd_change_from_demo_color("Tiles", &demo_color);
                }
                _ => unreachable!(),
            }
            self.updating.replace(false);
        }

        self.update_drawing();
    }

    pub fn update_for_color_dd_change(&self) {
        let demo_color = if self.toggle.is_left() { self.fg.borrow() } else { self.bg.borrow() };
        self.update_for_color_dd_change_from_demo_color(
            self.dial_color_kind.get_value().as_str(),
            &demo_color,
        );
    }

    pub fn update_for_color_dd_change_from_demo_color(
        &self, dial_color_kind: &str, demo_color: &ColorsDemoColor,
    ) {
        let dd_i = self.color_dd.get_selected();

        let c = match dial_color_kind {
            "Solid" => demo_color.solid_state.clone(),
            "Time-Gradient" => demo_color
                .time_gradient_state
                .1
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Radial-Gradient" => demo_color
                .radial_gradient_state
                .1
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Linear-Gradient" => demo_color
                .linear_gradient_state
                .1
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Radial-Time" => demo_color
                .radial_time_state
                .2
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Linear-Time" => demo_color
                .linear_time_state
                .2
                .get(dd_i)
                .cloned()
                .unwrap_or_default(),
            "Tiles" => {
                if dd_i == 0 {
                    demo_color.tiles_state.1.clone()
                } else {
                    demo_color.tiles_state.2.clone()
                }
            }
            _ => unreachable!(),
        };

        let Color::Rgba(rgba) = c else {
            return;
        };
        let og_updating = self.updating.replace(true);
        self.r_ntb.set_value(rgba.r as usize);
        self.g_ntb.set_value(rgba.g as usize);
        self.b_ntb.set_value(rgba.b as usize);
        self.a_ntb.set_value(rgba.a as usize);
        self.updating.replace(og_updating);
    }

    /// updates for any smaller-changes (sliders/tbs)
    pub fn update_for_minor_changes(&self, ctx: &Context) {
        if *self.updating.borrow() {
            return;
        }

        {
            let mut demo_color =
                if self.toggle.is_left() { self.fg.borrow_mut() } else { self.bg.borrow_mut() };
            let r = self.r_ntb.get_value();
            let g = self.g_ntb.get_value();
            let b = self.b_ntb.get_value();
            let a = self.a_ntb.get_value();
            let dd_i = self.color_dd.get_selected();
            let max_i = self
                .max_gr_colors_dd
                .get_selected_str()
                .parse::<usize>()
                .unwrap();
            debug!("max_i: {}", max_i);
            match self.dial_color_kind.get_value().as_str() {
                "Solid" => {
                    demo_color.solid_state =
                        Color::new_with_alpha(r as u8, g as u8, b as u8, a as u8);
                }
                "Time-Gradient" => {
                    let colors = &mut demo_color.time_gradient_state.1;
                    if max_i >= colors.len() {
                        colors.resize(max_i, Color::new(0, 0, 0));
                    }
                    colors[dd_i] = Color::new_with_alpha(r as u8, g as u8, b as u8, a as u8);

                    let time = self.time_ntb.get_value();
                    let time = Duration::from_millis(time as u64);

                    let trunc_colors = colors.iter().take(max_i).cloned().collect::<Vec<Color>>();

                    let gr = TimeGradient::new_loop(ctx, time, trunc_colors);
                    demo_color.time_gradient_state.0 = gr;
                }
                "Radial-Gradient" => {
                    let colors = &mut demo_color.radial_gradient_state.1;
                    if max_i >= colors.len() {
                        colors.resize(max_i, Color::new(0, 0, 0));
                    }
                    colors[dd_i] = Color::new_with_alpha(r as u8, g as u8, b as u8, a as u8);

                    let trunc_colors = colors.iter().take(max_i).cloned().collect::<Vec<Color>>();
                    debug!("trunc_colors len: {}", trunc_colors.len());

                    let dist = self.dist_ntb.get_value();
                    let gr = RadialGradient::new_basic_circle(
                        ctx,
                        (0.5.into(), 0.5.into()),
                        dist.into(),
                        trunc_colors,
                    );
                    demo_color.radial_gradient_state.0 = gr;
                }
                "Linear-Gradient" => {
                    let colors = &mut demo_color.linear_gradient_state.1;
                    if max_i >= colors.len() {
                        colors.resize(max_i, Color::new(0, 0, 0));
                    }
                    colors[dd_i] = Color::new_with_alpha(r as u8, g as u8, b as u8, a as u8);

                    let trunc_colors = colors.iter().take(max_i).cloned().collect::<Vec<Color>>();

                    let dist = self.dist_ntb.get_value();
                    let angle = self.angle_ntb.get_value();
                    let gr = Gradient::new_grad_repeater(ctx, trunc_colors, dist, angle);
                    demo_color.linear_gradient_state.0 = gr;
                }
                "Radial-Time" => {
                    let colors = &mut demo_color.radial_time_state.2;
                    if max_i >= colors.len() {
                        colors.resize(max_i, Color::new(0, 0, 0));
                    }
                    colors[dd_i] = Color::new_with_alpha(r as u8, g as u8, b as u8, a as u8);

                    let trunc_colors = colors.iter().take(max_i).cloned().collect::<Vec<Color>>();

                    let dist = self.dist_ntb.get_value();

                    let time = self.time_ntb.get_value();
                    let time = Duration::from_millis(time as u64);

                    let radial_time_gr = RadialGradient::new_basic_circle_time_loop(
                        ctx,
                        (0.5.into(), 0.5.into()),
                        time,
                        dist.into(),
                        trunc_colors,
                    );
                    demo_color.radial_time_state.0 = radial_time_gr.0;
                    demo_color.radial_time_state.1 = radial_time_gr.1;
                }
                "Linear-Time" => {
                    let colors = &mut demo_color.linear_time_state.2;
                    if max_i >= colors.len() {
                        colors.resize(max_i, Color::new(0, 0, 0));
                    }
                    colors[dd_i] = Color::new_with_alpha(r as u8, g as u8, b as u8, a as u8);

                    let trunc_colors = colors.iter().take(max_i).cloned().collect::<Vec<Color>>();

                    let dist = self.dist_ntb.get_value();

                    let time = self.time_ntb.get_value();
                    let time = Duration::from_millis(time as u64);

                    let angle = self.angle_ntb.get_value();

                    let linear_time_gr =
                        Gradient::new_grad_repeater_time_loop(ctx, trunc_colors, dist, time, angle);
                    demo_color.linear_time_state.0 = linear_time_gr.0;
                    demo_color.linear_time_state.1 = linear_time_gr.1;
                }
                "Tiles" => {
                    let color = Color::new_with_alpha(r as u8, g as u8, b as u8, a as u8);
                    if dd_i == 0 {
                        demo_color.tiles_state.1 = color;
                    } else {
                        demo_color.tiles_state.2 = color;
                    }

                    let dist = self.dist_ntb.get_value();

                    let tiles = Pattern::new_sqr_tiles(
                        ctx,
                        dist,
                        demo_color.tiles_state.1.clone(),
                        demo_color.tiles_state.2.clone(),
                    );
                    demo_color.tiles_state.0 = tiles;
                }

                _ => (),
            };
        }
        self.update_drawing();
    }

    const SPIRAL: &'static str = r#"
  ██████████████████████████████████████████████
  ██                                          ██
  ██  ██████████████████████████████████████  ██
  ██  ██                                  ██  ██
  ██  ██  ██████████████████████████████  ██  ██
  ██  ██  ██                          ██  ██  ██
  ██  ██  ██  ██████████████████████  ██  ██  ██
  ██  ██  ██  ██                  ██  ██  ██  ██
  ██  ██  ██  ██  ██████████████  ██  ██  ██  ██
  ██  ██  ██  ██  ██          ██  ██  ██  ██  ██
  ██  ██  ██  ██  ██  ██████  ██  ██  ██  ██  ██
  ██  ██  ██  ██  ██  ██  ██  ██  ██  ██  ██  ██
  ██  ██  ██  ██  ██      ██  ██  ██  ██  ██  ██
  ██  ██  ██  ██  ██████████  ██  ██  ██  ██  ██
  ██  ██  ██  ██              ██  ██  ██  ██  ██
  ██  ██  ██  ██████████████████  ██  ██  ██  ██
  ██  ██  ██                      ██  ██  ██  ██
  ██  ██  ██████████████████████████  ██  ██  ██
  ██  ██                              ██  ██  ██
  ██  ██████████████████████████████████  ██  ██
  ██                                      ██  ██
  ██████████████████████████████████████████  ██"#;

    const BUTTERFLY: &'static str = r#"                                 , 
                                 ;o\ 
                                 ;Ob`. 
                                ;OOOOb`. 
                               ;OOOOOY" ) 
                              ;OOOO' ,%%) 
                          \  /OOO ,%%%%,%\ 
                           |:  ,%%%%%%;%%/ 
                           ||,%%%%%%%%%%/ 
                           ;|%%%%%%%%%'/`-'"`. 
                          /: %%%%%%%%'/ c$$$$.`. 
             `.______     \ \%%%%%%%'/.$$YF"Y$: ) 
           _________ "`.\`\o \`%%' ,',$F,.   $F ) 
  ___,--""'dOOO;,:%%`-._ o_,O_   ,',"',d88)  '  ) 
"'. YOOOOOOO';%%%%%%%%%;`-O   )_     ,X888F   _/ 
   \ YOOOO',%%%%%%%%%%Y    \__;`),-.  `""F  ,' 
    \ `" ,%%%%%%%%%%,' _,-   \-' \_ `------' 
     \_ %%%%',%%%%%_,-' ,;    ( _,-\ 
       `-.__`%%',-' .c$$'     |\-_,-\ 
            `""; ,$$$',md8oY  : `\_,') 
              ( ,$$$F `88888  ;   `--' 
               \`$$(    `""' / 
                \`"$$c'   _,' 
 -hrr-           `.____,-' "#;

    const SATURN: &'static str = r#"




                                          _.oo.
                  _.u[[/;:,.         .odMMMMMM'
               .o888UU[[[/;:-.  .o@P^    MMM^
              oN88888UU[[[/;::-.        dP^
             dNMMNN888UU[[[/;:--.   .o@P^
            ,MMMMMMN888UU[[/;::-. o@^
            NNMMMNN888UU[[[/~.o@P^
            888888888UU[[[/o@^-..
           oI8888UU[[[/o@P^:--..
        .@^  YUU[[[/o@^;::---..
      oMP     ^/o@P^;:::---..
   .dMMM    .o@^ ^;::---...
  dMMMMMMM@^`       `^^^^
 YMMMUP^
  ^^
 
 unknown"#;
    // TODO there are nicer ways to center... being lazy
    const RUST_LOGO: &'static str = r#"







                    △ △ △ △ △
                  ╭─────o─────╮
                ◁ │ ███████   │ ▷
                ◁ o  ██   ██  o ▷
                ◁ │  ██████   │ ▷
                ◁ │  ██   ██  │ ▷
                ◁ │ ████  ███ │ ▷
                  ╰──o─────o──╯
                    ▽ ▽ ▽ ▽ ▽    
"#;

    // TODO there are nicer ways to center... being lazy
    const CHOMPY: &'static str = r#"
    
    
    
    
    
    
    
                        CHOMPPY ANGRYYY!
                        DO NOT TEST CHOMPPY!
                _____  /
              _/o\ /o \
             /        |
             v v v v  |
              ^ ^ ^ ^ |
              \.......|
"#;

    /// updates the drawing of the art
    pub fn update_drawing(&self) {
        if *self.updating.borrow() {
            return;
        }

        let fg = self.fg.borrow().get_color();
        let bg = self.bg.borrow().get_color();

        let text = match self.dial_fg_art.get_value().as_str() {
            "Butterfly" => ColorsDemoState::BUTTERFLY,
            "Spiral" => ColorsDemoState::SPIRAL,
            "Rust-Logo" => ColorsDemoState::RUST_LOGO,
            "Saturn" => ColorsDemoState::SATURN,
            "Chompy" => ColorsDemoState::CHOMPY,
            _ => "",
        };
        self.drawing_fg.set_text(text);
        let fg_sty = Style::transparent()
            .with_fg(fg)
            .with_fg_transp_src(FgTranspSrc::LowerBg);
        self.drawing_fg.set_style(fg_sty);
        self.drawing_bg.set_bg(bg.clone());
        self.drawing_bg.set_fg(bg);
    }
}

impl ColorsDemoColor {
    pub fn default_fg(ctx: &Context) -> ColorsDemoColor {
        let three_colors = vec![Color::RED, Color::GREEN, Color::BLUE];
        let time_gr = TimeGradient::new_loop(ctx, Duration::from_secs(1), three_colors.clone());

        let radial_gr = RadialGradient::new_basic_circle(
            ctx,
            (0.5.into(), 0.5.into()),
            15.into(),
            three_colors.clone(),
        );
        let linear_gr_colors = vec![
            Color::VIOLET,
            Color::INDIGO,
            Color::BLUE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::RED,
        ];
        let linear_gr = Gradient::new_y_grad_repeater(ctx, linear_gr_colors.clone(), 5);

        let radial_time_gr = RadialGradient::new_basic_circle_time_loop(
            ctx,
            (0.5.into(), 0.5.into()),
            Duration::from_secs(1),
            15.into(),
            three_colors.clone(),
        );
        let linear_time_colors = vec![
            Color::VIOLET,
            Color::INDIGO,
            Color::BLUE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::RED,
        ];
        let linear_time_gr = Gradient::new_y_grad_repeater_time_loop(
            ctx,
            linear_time_colors.clone(),
            5,
            Duration::from_secs(1),
        );
        let tiles_colors = (Color::WHITE, Color::BLUE);
        let tiles = Pattern::new_sqr_tiles(ctx, 5, tiles_colors.0.clone(), tiles_colors.1.clone());
        ColorsDemoColor {
            kind: ColorsDemoColorKind::Solid,
            solid_state: Color::WHITE,
            time_gradient_state: (time_gr, three_colors.clone()),
            radial_gradient_state: (radial_gr, three_colors.clone()),
            linear_gradient_state: (linear_gr, linear_gr_colors),
            radial_time_state: (radial_time_gr.0, radial_time_gr.1, three_colors),
            linear_time_state: (linear_time_gr.0, linear_time_gr.1, linear_time_colors),
            tiles_state: (tiles, tiles_colors.0, tiles_colors.1),
        }
    }

    pub fn default_bg(ctx: &Context) -> ColorsDemoColor {
        let c_store = ctx.color_store;
        let three_colors = vec![
            Color::RED.darken(&c_store),
            Color::GREEN.darken(&c_store),
            Color::BLUE.darken(&c_store),
        ];
        let time_gr = TimeGradient::new_loop(ctx, Duration::from_secs(1), three_colors.clone());

        let radial_gr = RadialGradient::new_basic_circle(
            ctx,
            (0.5.into(), 0.5.into()),
            15.into(),
            three_colors.clone(),
        );
        let linear_gr_colors = vec![
            Color::VIOLET,
            Color::INDIGO,
            Color::BLUE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::RED,
        ];
        let linear_gr = Gradient::new_x_grad_repeater(ctx, linear_gr_colors.clone(), 5);
        let radial_time_gr = RadialGradient::new_basic_circle_time_loop(
            ctx,
            (0.5.into(), 0.5.into()),
            Duration::from_secs(1),
            15.into(),
            three_colors.clone(),
        );
        let linear_time_colors = vec![
            Color::VIOLET.darken(&c_store),
            Color::INDIGO.darken(&c_store),
            Color::AQUA.darken(&c_store),
            Color::GREEN.darken(&c_store),
        ];
        let linear_time_gr = Gradient::new_grad_repeater_time_loop(
            ctx,
            linear_time_colors.clone(),
            12,
            Duration::from_millis(1500),
            0.,
        );
        let tiles_colors = (Color::WHITE, Color::BLUE);
        let tiles = Pattern::new_sqr_tiles(ctx, 5, tiles_colors.0.clone(), tiles_colors.1.clone());
        ColorsDemoColor {
            kind: ColorsDemoColorKind::Solid,
            solid_state: Color::BLACK,
            time_gradient_state: (time_gr, three_colors.clone()),
            radial_gradient_state: (radial_gr, three_colors.clone()),
            linear_gradient_state: (linear_gr, linear_gr_colors),
            radial_time_state: (radial_time_gr.0, radial_time_gr.1, three_colors.clone()),
            linear_time_state: (linear_time_gr.0, linear_time_gr.1, linear_time_colors),
            tiles_state: (tiles, tiles_colors.0, tiles_colors.1),
        }
    }

    pub fn get_color(&self) -> Color {
        match self.kind {
            ColorsDemoColorKind::Solid => self.solid_state.clone(),
            ColorsDemoColorKind::TimeGradient => self.time_gradient_state.0.clone().into(),
            ColorsDemoColorKind::RadialGradient => self.radial_gradient_state.0.clone().into(),
            ColorsDemoColorKind::LinearGradient => self.linear_gradient_state.0.clone().into(),
            ColorsDemoColorKind::RadialTime => self.radial_time_state.0.clone().into(),
            ColorsDemoColorKind::LinearTime => self.linear_time_state.0.clone().into(),
            ColorsDemoColorKind::Tiles => self.tiles_state.0.clone().into(),
        }
    }
}
