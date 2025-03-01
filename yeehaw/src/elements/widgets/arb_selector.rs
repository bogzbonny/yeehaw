use {
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
    std::ops::{Deref, DerefMut},
};

// basic usage:
//#[rustfmt::skip]
//let drawing_base = "  H__A  \n".to_string()
//                 + "G ╱  ╲ B\n"
//                 + "F ╲__╱ C\n"
//                 + "  E  D  ";
//let positions = "GHHHAAAB\n\
//                 GGGHABBB\n\
//                 FFFEDCCC\n\
//                 FEEEDDDC";
//let sel_changes = vec![
//    ('°', 4, 1),
//    ('∘', 5, 1),
//    ('°', 5, 2),
//    ('∘', 4, 2),
//    ('∘', 3, 2),
//    ('°', 2, 2),
//    ('∘', 2, 1),
//    ('°', 3, 1),
//];
//let dial = ArbSelector::new_with_uniform_style(
//    &ctx,
//    reg_sty,
//    drawing_base.to_string(),
//    positions.to_string(),
//    sel_changes,
//)

/// Arbitrary Selector is a selector object which can be used to construct
/// cool selectors with arbitrary selection positions such as dials.
#[derive(Clone)]
pub struct ArbSelector {
    pub pane: SelectablePane,

    pub is_dirty: Rc<RefCell<bool>>,

    /// position of the selector
    pub position: Rc<RefCell<usize>>,

    pub max_position: Rc<RefCell<usize>>,

    /// all positions that the selector can be in
    /// these positions may have gaps (eg 2, 4, 6)
    pub positions: Rc<RefCell<Vec<usize>>>,

    pub drawing_base: Rc<RefCell<DrawChs2D>>,

    /// y, x, position
    /// if the position is NONE then no selection change is to be made if that position is selected
    pub positions_map: Rc<RefCell<Vec<Vec<Option<usize>>>>>,

    /// changes to be made on selection of a position, usize is the position
    pub selection_changes: Rc<RefCell<Vec<(usize, SelChanges)>>>,

    /// activated when mouse is clicked down while over button
    pub clicked_down: Rc<RefCell<bool>>,

    // called for each adjustment
    pub select_fn: Rc<RefCell<SelectFn>>,
}

#[derive(Default)]
pub struct SelChanges(Vec<DrawChPos>);

impl Deref for SelChanges {
    type Target = Vec<DrawChPos>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SelChanges {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type SelectFn = Box<dyn FnMut(Context, &ArbSelector, usize) -> EventResponses>;

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl ArbSelector {
    const KIND: &'static str = "arb_selector";

    pub fn default_receivable_events() -> ReceivableEvents {
        ReceivableEvents(vec![
            (KB::KEY_LEFT.into()),
            (KB::KEY_RIGHT.into()),
            (KB::KEY_H.into()),
            (KB::KEY_L.into()),
        ])
    }

    /// Create a new ArbSelector with the given drawing style and base drawing
    /// and the positions map. The positions map is a 2D array of letters where 'A' is the
    /// 1st position, 'B' is the 2nd position, etc.
    pub fn new_with_uniform_style<S: Into<String>>(
        ctx: &Context, drawing_sty: Style, drawing_base: S, positions_map: S,
        selection_changes: Vec<(char, u16, u16)>,
    ) -> Self {
        let drawing_base = DrawChs2D::from_string(drawing_base.into(), drawing_sty.clone());
        let positions_map = positions_map
            .into()
            .lines()
            .map(|line| {
                line.chars()
                    .map(
                        |ch|
                        // parse the string to usize starting with A=0, B=1, etc.
                        if ch == '0' {
                            None
                        } else {
                            Some(ch as usize - 'A' as usize)
                        },
                    )
                    .collect()
            })
            .collect();
        let selection_changes = selection_changes
            .into_iter()
            .enumerate()
            .map(|(i, (ch, x, y))| {
                (
                    i,
                    SelChanges(vec![DrawChPos::new(
                        DrawCh::new(ch, drawing_sty.clone()),
                        x,
                        y,
                    )]),
                )
            })
            .collect();
        Self::new_inner(ctx, drawing_base, positions_map, selection_changes)
    }

    // TODO it would be good to combine drawing_base and positions_map into a single type
    // so that no incorrect dimensions can be input
    //
    // selection changes: usize is the position
    pub fn new_inner(
        ctx: &Context, drawing_base: DrawChs2D, positions_map: Vec<Vec<Option<usize>>>,
        selection_changes: Vec<(usize, SelChanges)>,
    ) -> Self {
        // verify that the drawing base and positions map are the same size
        let base_height = drawing_base.height();
        let base_width = drawing_base.width();
        let pos_height = positions_map.len();
        let pos_width = positions_map[0].len();
        if base_height != pos_height || base_width != pos_width {
            log_err!(
                "Drawing base and positions map must be the same size. Base: {}x{}, Positions: {}x{}",
                base_height, base_width, pos_height, pos_width
            );
        }

        let mut min_pos = None;
        let mut positions = Vec::new();

        // get the max position
        let mut max_pos = 0;
        for row in positions_map.iter() {
            for &pos in row.iter() {
                if let Some(pos) = pos {
                    max_pos = max_pos.max(pos);
                    if !positions.contains(&pos) {
                        positions.push(pos);
                    }
                    match min_pos {
                        Some(min) => {
                            if pos < min {
                                min_pos.replace(pos);
                            }
                        }
                        None => {
                            min_pos.replace(pos);
                        }
                    }
                }
            }
        }
        let min_pos = min_pos.unwrap_or(0);

        let pane = SelectablePane::new(ctx, Self::KIND)
            .with_focused_receivable_events(Self::default_receivable_events())
            .with_styles(SelStyles::opaque(ctx))
            .with_dyn_width(DynVal::new_fixed(base_width as i32))
            .with_dyn_height(DynVal::new_fixed(base_height as i32));

        let t = ArbSelector {
            pane,
            is_dirty: Rc::new(RefCell::new(true)),
            position: Rc::new(RefCell::new(min_pos)),
            max_position: Rc::new(RefCell::new(max_pos)),
            positions: Rc::new(RefCell::new(positions)),
            drawing_base: Rc::new(RefCell::new(drawing_base)),
            positions_map: Rc::new(RefCell::new(positions_map)),
            selection_changes: Rc::new(RefCell::new(selection_changes)),
            clicked_down: Rc::new(RefCell::new(false)),
            select_fn: Rc::new(RefCell::new(Box::new(|_, _, _| EventResponses::default()))),
        };

        let t_ = t.clone();
        t.pane
            .set_post_hook_for_set_selectability(Box::new(move |_, _| {
                t_.is_dirty.replace(true);
            }));
        t
    }

    pub fn positions_string_to_map(positions: &str) -> Vec<Vec<Option<usize>>> {
        positions
            .lines()
            .map(|line| {
                line.chars()
                    .map(
                        |ch|
                        // parse the string to usize starting with A=0, B=1, etc.
                        if ch == '0' {
                            None
                        } else {
                            Some(ch as usize - 'A' as usize)
                        },
                    )
                    .collect()
            })
            .collect()
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.pane.set_styles(styles);
        self
    }

    pub fn with_fn(self, select_fn: SelectFn) -> Self {
        self.set_fn(select_fn);
        self
    }

    pub fn set_fn(&self, select_fn: SelectFn) {
        *self.select_fn.borrow_mut() = select_fn;
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, loc_x: D, loc_y: D2) -> Self {
        self.pane.set_at(loc_x.into(), loc_y.into());
        self
    }

    pub fn with_position(self, pos: usize) -> Self {
        self.set_position(pos);
        self
    }

    // ----------------------------------------------

    pub fn get_position(&self) -> usize {
        *self.position.borrow()
    }

    pub fn set_position(&self, mut pos: usize) {
        let max_pos = *self.max_position.borrow();
        pos = pos.min(max_pos);
        *self.position.borrow_mut() = pos;
        self.is_dirty.replace(true);
    }

    pub fn has_position(&self, pos: usize) -> bool {
        self.positions.borrow().contains(&pos)
    }

    pub fn get_pos_from_x_y(&self, x: i32, y: i32) -> Option<usize> {
        let pos_map = self.positions_map.borrow();
        if y < 0 || x < 0 {
            return None;
        }
        let x = x as usize;
        let y = y as usize;
        pos_map[y][x]
    }

    pub fn perform_selection(&self, ctx: &Context, pos: usize) -> EventResponses {
        // check if the pos has changed
        if pos == self.get_position() {
            return EventResponses::default();
        }
        self.set_position(pos);
        let mut select_fn = self.select_fn.borrow_mut();
        select_fn(ctx.clone(), self, pos)
    }

    pub fn increment_position(&self, ctx: &Context) -> EventResponses {
        let mut pos = self.get_position();
        loop {
            if pos < *self.max_position.borrow() {
                pos += 1;
            } else {
                pos = 0;
            }
            if self.has_position(pos) {
                break;
            }
        }
        self.perform_selection(ctx, pos)
    }

    pub fn decrement_position(&self, ctx: &Context) -> EventResponses {
        let mut pos = self.get_position();
        loop {
            if pos > 0 {
                pos -= 1;
            } else {
                pos = *self.max_position.borrow();
            }
            if self.has_position(pos) {
                break;
            }
        }

        self.perform_selection(ctx, pos)
    }

    pub fn update_content(&self, ctx: &Context) {
        let pos = *self.position.borrow();

        let mut content = self.drawing_base.borrow().clone();
        let sel_changes = self.selection_changes.borrow();
        let updates = sel_changes.iter().find_map(
            |(ref pos_, ref changes)| {
                if *pos_ == pos {
                    Some(changes)
                } else {
                    None
                }
            },
        );

        if let Some(updates) = updates {
            for update in updates.0.iter() {
                content.set_ch(update.x.into(), update.y.into(), update.ch.clone());
            }
        }

        //update overlay styles everywhere
        let sty = self.pane.get_current_style();
        content.overlay_all_styles(ctx, &sty);

        self.pane.set_content(content);
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for ArbSelector {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        if captured {
            return (true, resps);
        }
        if self.pane.get_selectability() == Selectability::Unselectable {
            return (false, resps);
        }
        let sel = self.pane.get_selectability();
        match ev {
            Event::KeyCombo(ke) => {
                if sel != Selectability::Selected || ke.is_empty() {
                    return (false, resps);
                }
                match true {
                    _ if ke[0] == KB::KEY_LEFT || ke[0] == KB::KEY_H => {
                        let resps_ = self.decrement_position(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ if ke[0] == KB::KEY_RIGHT || ke[0] == KB::KEY_L => {
                        let resps_ = self.increment_position(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    _ => {}
                }
                return (false, resps);
            }
            Event::Mouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollRight
                        if sel == Selectability::Selected =>
                    {
                        let resps_ = self.decrement_position(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    MouseEventKind::ScrollDown | MouseEventKind::ScrollLeft
                        if sel == Selectability::Selected =>
                    {
                        let resps_ = self.increment_position(ctx);
                        resps.extend(resps_);
                        return (true, resps);
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, resps);
                    }
                    MouseEventKind::Drag(MouseButton::Left)
                    | MouseEventKind::Up(MouseButton::Left)
                        if clicked_down =>
                    {
                        let pos = self.get_pos_from_x_y(me.column, me.row);
                        if let Some(pos) = pos {
                            let resps_ = self.perform_selection(ctx, pos);
                            resps.extend(resps_);
                        }
                        return (true, resps);
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }
                return (false, resps);
            }
            Event::Resize => {
                self.is_dirty.replace(true);
                return (false, resps);
            }
            _ => {}
        }
        (false, resps)
    }
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        if self.is_dirty.replace(false) || force_update {
            self.update_content(ctx);
        }
        self.pane.drawing(ctx, dr, force_update)
    }
}
