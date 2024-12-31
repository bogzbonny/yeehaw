use {
    crate::*,
    crossterm::event::{KeyModifiers, MouseEventKind},
    rayon::prelude::*,
};

/// PaneScrollable is a simple pane which exhibits offsets for the content.
/// The size of the view is fixed, determined by the view_height and view_width.
/// Additionally mouse scroll functionality is provided.
#[derive(Clone)]
pub struct PaneScrollable {
    pub pane: ParentPane,
    content_width: Rc<RefCell<usize>>,
    content_height: Rc<RefCell<usize>>,
    content_offset_x: Rc<RefCell<usize>>,
    content_offset_y: Rc<RefCell<usize>>,

    /// if true, then the pane will grow to fill the width of the parent
    /// when the parent pane is larger than `content_width`
    pub expand_to_fill_width: Rc<RefCell<bool>>,

    /// if true, then the pane will grow to fill the height of the parent
    /// when the parent pane is larger than `content_height`
    pub expand_to_fill_height: Rc<RefCell<bool>>,

    /// how many characters to scroll on a scroll event, if None, then disable scroll
    pub scroll_rate: Rc<RefCell<Option<i16>>>,

    last_draw_details: Rc<RefCell<Option<PSDrawDetails>>>,
}

struct PSDrawDetails {
    x_off: usize,
    y_off: usize,
    max_x: usize,
    max_y: usize,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl PaneScrollable {
    pub const KIND: &'static str = "pane_scrollable";

    pub fn new(ctx: &Context, width: usize, height: usize) -> Self {
        Self {
            pane: ParentPane::new(ctx, Self::KIND).with_transparent(),
            content_width: Rc::new(RefCell::new(width)),
            content_height: Rc::new(RefCell::new(height)),
            content_offset_x: Rc::new(RefCell::new(0)),
            content_offset_y: Rc::new(RefCell::new(0)),
            expand_to_fill_width: Rc::new(RefCell::new(false)),
            expand_to_fill_height: Rc::new(RefCell::new(false)),
            scroll_rate: Rc::new(RefCell::new(Some(1))),
            last_draw_details: Rc::new(RefCell::new(None)),
        }
    }

    /// Create a scrollable pane which expands to fill the parent pane when the parent pane is
    /// larger than the width and height provided.
    pub fn new_expanding(ctx: &Context, width: usize, height: usize) -> Self {
        Self {
            pane: ParentPane::new(ctx, Self::KIND).with_transparent(),
            content_width: Rc::new(RefCell::new(width)),
            content_height: Rc::new(RefCell::new(height)),
            content_offset_x: Rc::new(RefCell::new(0)),
            content_offset_y: Rc::new(RefCell::new(0)),
            expand_to_fill_width: Rc::new(RefCell::new(true)),
            expand_to_fill_height: Rc::new(RefCell::new(true)),
            scroll_rate: Rc::new(RefCell::new(Some(1))),
            last_draw_details: Rc::new(RefCell::new(None)),
        }
    }

    pub fn add_element(&self, el: Box<dyn Element>) {
        self.pane.add_element(el.clone())
    }

    pub fn remove_element(&self, el_id: &ElementID) {
        self.pane.eo.remove_element(el_id)
    }

    pub fn clear_elements(&self) {
        self.pane.eo.clear_elements()
    }

    // ------------------------------------

    pub fn inner_draw_region(&self, dr: &DrawRegion) -> DrawRegion {
        let mut inner_dr = dr.clone();

        inner_dr.size.width = self.get_content_width(Some(dr)) as u16;
        inner_dr.size.height = self.get_content_height(Some(dr)) as u16;
        //debug!(
        //    "dr: \twidth: {}, \theight: {}, inner_dr: \twidth: {}, \theight: {}",
        //    dr.size.width, dr.size.height, inner_dr.size.width, inner_dr.size.height
        //);
        let x1 = *self.content_offset_x.borrow() as u16;
        let y1 = *self.content_offset_y.borrow() as u16;
        let x2 = x1 + dr.size.width;
        let y2 = y1 + dr.size.height;
        //debug!(
        //    "visible region: \n\tx1: {}, \n\tx2: {}, \n\ty1: {}, \n\ty2: {}",
        //    x1, x2, y1, y2
        //);
        let visible_region = Loc::new(x1, x2, y1, y2);
        inner_dr.visible_region = Some(visible_region);
        inner_dr
    }

    pub fn get_width_val(&self, dr: &DrawRegion) -> usize {
        self.pane.get_dyn_location_set().get_width_val(dr)
    }

    pub fn get_height_val(&self, dr: &DrawRegion) -> usize {
        self.pane.get_dyn_location_set().get_height_val(dr)
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for PaneScrollable {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let mut adj_ev = ev.clone();
        if let Event::Mouse(me) = &mut adj_ev {
            // adjust the pos of the mouse event
            me.column += *self.content_offset_x.borrow() as i32;
            me.row += *self.content_offset_y.borrow() as i32;
            me.dr = self.inner_draw_region(&me.dr);
        }

        if let Event::ExternalMouse(me) = &mut adj_ev {
            // adjust the pos of the mouse event
            me.column += *self.content_offset_x.borrow() as i32;
            me.row += *self.content_offset_y.borrow() as i32;
            me.dr = self.inner_draw_region(&me.dr);
        }

        let (mut captured, resps) = self.pane.receive_event(ctx, adj_ev);
        if captured {
            return (captured, resps);
        }

        if let Event::Mouse(me) = ev {
            let Some(sc_rate) = *self.scroll_rate.borrow() else {
                return (captured, resps);
            };

            let scroll = match me.kind {
                MouseEventKind::ScrollDown if me.modifiers == KeyModifiers::NONE => {
                    Some((0i16, sc_rate))
                }
                MouseEventKind::ScrollUp if me.modifiers == KeyModifiers::NONE => {
                    Some((0, -sc_rate))
                }
                MouseEventKind::ScrollDown if me.modifiers == KeyModifiers::SHIFT => {
                    Some((sc_rate, 0))
                }
                MouseEventKind::ScrollUp if me.modifiers == KeyModifiers::SHIFT => {
                    Some((-sc_rate, 0))
                }
                MouseEventKind::ScrollLeft => Some((-sc_rate, 0)),
                MouseEventKind::ScrollRight => Some((sc_rate, 0)),
                _ => None,
            };
            if let Some((dx, dy)) = scroll {
                if dx != 0 {
                    let start_x = *self.content_offset_x.borrow();

                    let x = if dx < 0 {
                        start_x.saturating_sub((-dx) as usize)
                    } else {
                        start_x + dx as usize
                    };
                    self.set_content_x_offset(Some(&me.dr), x);
                    let end_x = *self.content_offset_x.borrow();
                    if start_x != end_x {
                        captured = true;
                    }
                    return (captured, resps);
                }
                if dy != 0 {
                    let start_y = *self.content_offset_y.borrow();

                    let y = if dy < 0 {
                        self.content_offset_y
                            .borrow()
                            .saturating_sub((-dy) as usize)
                    } else {
                        *self.content_offset_y.borrow() + dy as usize
                    };
                    self.set_content_y_offset(Some(&me.dr), y);

                    let end_y = *self.content_offset_y.borrow();
                    if start_y != end_y {
                        captured = true;
                    }
                    return (captured, resps);
                }
            }
        }
        (captured, resps)
    }

    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        let x_off = *self.content_offset_x.borrow();
        let y_off = *self.content_offset_y.borrow();
        let max_x = x_off + self.get_content_width(Some(dr));
        let max_y = y_off + self.get_content_height(Some(dr));

        let scope_changed =
            if let Some(last_draw_details) = self.last_draw_details.borrow().as_ref() {
                !(last_draw_details.x_off == x_off
                    && last_draw_details.y_off == y_off
                    && last_draw_details.max_x == max_x
                    && last_draw_details.max_y == max_y)
            } else {
                true
            };
        // update the last draw details
        *self.last_draw_details.borrow_mut() = Some(PSDrawDetails {
            x_off,
            y_off,
            max_x,
            max_y,
        });

        let force_update = force_update || scope_changed;

        let inner_dr = self.inner_draw_region(dr);
        let mut upds = self.pane.drawing(ctx, &inner_dr, force_update);

        // NOTE computational bottleneck, use rayon
        upds.par_iter_mut().for_each(|upd| {
            match upd.action {
                DrawAction::Update(ref mut dcps) | DrawAction::Extend(ref mut dcps) => {
                    dcps.par_iter_mut().for_each(|dcp| {
                        if (dcp.x as usize) < x_off
                            || (dcp.y as usize) < y_off
                            || (dcp.x as usize) >= max_x
                            || (dcp.y as usize) >= max_y
                        {
                            // it'd be better to delete, but we can't delete from a parallel iterator
                            // also using a filter here is slower that this
                            dcp.ch = DrawCh::skip();
                            (dcp.x, dcp.y) = (0, 0);
                        } else {
                            dcp.x = (dcp.x as usize - x_off) as u16;
                            dcp.y = (dcp.y as usize - y_off) as u16;
                            dcp.add_to_offset_colors(-(x_off as i32), -(y_off as i32));
                        }
                    })
                }
                DrawAction::Remove => {}
                DrawAction::ClearAll => {}
            }
        });

        upds
    }

    fn set_content_x_offset(&self, dr: Option<&DrawRegion>, x: usize) {
        let size = if let Some(dr) = dr { dr.size } else { *self.get_last_size() };
        let offset = self.get_content_width(dr).saturating_sub(size.width.into());
        let offset = if x > offset { offset } else { x };
        *self.content_offset_x.borrow_mut() = offset
    }

    fn set_content_y_offset(&self, dr: Option<&DrawRegion>, y: usize) {
        let size = if let Some(dr) = dr { dr.size } else { *self.get_last_size() };
        let offset = self
            .get_content_height(dr)
            .saturating_sub(size.height.into());
        let offset = if y > offset { offset } else { y };
        *self.content_offset_y.borrow_mut() = offset
    }

    fn get_content_x_offset(&self) -> usize {
        *self.content_offset_x.borrow()
    }
    fn get_content_y_offset(&self) -> usize {
        *self.content_offset_y.borrow()
    }
    fn get_content_width(&self, dr: Option<&DrawRegion>) -> usize {
        let size = if let Some(dr) = dr { dr.size } else { *self.get_last_size() };
        if *self.expand_to_fill_width.borrow() && size.width as usize > *self.content_width.borrow()
        {
            size.width as usize
        } else {
            *self.content_width.borrow()
        }
    }
    fn get_content_height(&self, dr: Option<&DrawRegion>) -> usize {
        let size = if let Some(dr) = dr { dr.size } else { *self.get_last_size() };
        if *self.expand_to_fill_height.borrow()
            && size.height as usize > *self.content_height.borrow()
        {
            size.height as usize
        } else {
            *self.content_height.borrow()
        }
    }
}
