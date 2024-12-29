use crate::*;

// currently resizing stacks makes the resized dimention static for the two elements which have
// changed dimension values during a resize.

#[derive(Clone)]
pub struct VerticalStackFocuser {
    pub pane: VerticalStack,
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for VerticalStackFocuser {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        // this code is important for when this element is the highest level main element
        let mut resps = focuser::focus_on_click(&ev, self.get_focused());
        let (captured, resps_) = self.pane.receive_event(ctx, ev.clone());
        resps.extend(resps_);
        (captured, resps)
    }
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl VerticalStackFocuser {
    pub fn new(ctx: &Context) -> Self {
        Self {
            pane: VerticalStack::new(ctx),
        }
    }

    /// add an element to the end of the stack resizing the other elements
    /// in order to fit the new element
    pub fn push(&self, el: Box<dyn Element>) {
        let el = Box::new(Focuser::new(el));
        self.pane.push(el);
    }

    pub fn insert(&self, idx: usize, el: Box<dyn Element>) {
        let el = Box::new(Focuser::new(el));
        self.pane.insert(idx, el);
    }

    pub fn with_min_resize_height(self, min_resize_height: usize) -> Self {
        self.pane.set_min_resize_height(min_resize_height);
        self
    }
}

#[derive(Clone)]
pub struct HorizontalStackFocuser {
    pub pane: HorizontalStack,
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for HorizontalStackFocuser {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        // this code is important for when this element is the highest level main element
        let mut resps = focuser::focus_on_click(&ev, self.get_focused());
        let (captured, resps_) = self.pane.receive_event(ctx, ev.clone());
        resps.extend(resps_);
        (captured, resps)
    }
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl HorizontalStackFocuser {
    pub fn new(ctx: &Context) -> Self {
        Self {
            pane: HorizontalStack::new(ctx),
        }
    }

    /// add an element to the end of the stack resizing the other elements
    /// in order to fit the new element
    pub fn push(&self, el: Box<dyn Element>) {
        let el = Box::new(Focuser::new(el));
        self.pane.push(el);
    }

    pub fn insert(&self, idx: usize, el: Box<dyn Element>) {
        let el = Box::new(Focuser::new(el));
        self.pane.insert(idx, el);
    }

    pub fn with_min_resize_width(self, min_resize_width: usize) -> Self {
        self.pane.set_min_resize_width(min_resize_width);
        self
    }
}

#[derive(Clone)]
pub struct VerticalStack {
    pub pane: ParentPane,
    #[allow(clippy::type_complexity)]
    pub els: Rc<RefCell<Vec<Box<dyn Element>>>>,
    pub last_size: Rc<RefCell<Size>>,

    /// minimum height allowable by resizes
    pub min_resize_height: Rc<RefCell<usize>>,

    pub is_dirty: Rc<RefCell<bool>>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl VerticalStack {
    const KIND: &'static str = "vertical_stack";

    pub fn new(ctx: &Context) -> Self {
        Self {
            pane: ParentPane::new(ctx, Self::KIND),
            els: Rc::new(RefCell::new(Vec::new())),
            last_size: Rc::new(RefCell::new(Size::new(0, 0))),
            min_resize_height: Rc::new(RefCell::new(1)),
            is_dirty: Rc::new(RefCell::new(true)),
        }
    }

    pub fn new_with_kind(ctx: &Context, kind: &'static str) -> Self {
        Self {
            pane: ParentPane::new(ctx, kind),
            els: Rc::new(RefCell::new(Vec::new())),
            last_size: Rc::new(RefCell::new(Size::new(0, 0))),
            min_resize_height: Rc::new(RefCell::new(1)),
            is_dirty: Rc::new(RefCell::new(true)),
        }
    }

    pub fn with_min_resize_height(self, min_resize_height: usize) -> Self {
        self.set_min_resize_height(min_resize_height);
        self
    }

    pub fn set_min_resize_height(&self, min_resize_height: usize) {
        *self.min_resize_height.borrow_mut() = min_resize_height;
    }

    /// add an element to the end of the stack resizing the other elements
    /// in order to fit the new element
    pub fn push(&self, el: Box<dyn Element>) {
        Self::sanitize_el_location(&*el);
        self.els.borrow_mut().push(el.clone());
        self.is_dirty.replace(true);
        self.pane.add_element(el)
    }

    pub fn pop(&self) {
        let el = self.els.borrow_mut().pop();
        if let Some(el) = el {
            self.is_dirty.replace(true);
            self.pane.remove_element(&el.id())
        }
    }

    pub fn insert(&self, idx: usize, el: Box<dyn Element>) {
        Self::sanitize_el_location(&*el);
        self.els.borrow_mut().insert(idx, el.clone());
        self.is_dirty.replace(true);
        self.pane.add_element(el)
    }

    pub fn remove(&self, idx: usize) {
        let el = self.els.borrow_mut().remove(idx);
        self.is_dirty.replace(true);
        self.pane.remove_element(&el.id());
    }

    pub fn clear(&self) {
        self.els.borrow_mut().clear();
        self.is_dirty.replace(true);
        self.pane.clear_elements();
    }

    pub fn len(&self) -> usize {
        self.els.borrow().len()
    }

    pub fn get(&self, idx: usize) -> Option<Box<dyn Element>> {
        self.els.borrow().get(idx).cloned()
    }

    pub fn first(&self) -> Option<Box<dyn Element>> {
        self.els.borrow().first().cloned()
    }

    pub fn last(&self) -> Option<Box<dyn Element>> {
        self.els.borrow().last().cloned()
    }

    pub fn get_index(&self, id: &ElementID) -> Option<usize> {
        self.els.borrow().iter().position(|el| &el.id() == id)
    }

    pub fn is_empty(&self) -> bool {
        self.els.borrow().is_empty()
    }

    /// get the average value of the elements in the stack
    /// this is useful for pushing new elements with an even size
    /// to the other elements
    pub fn avg_height(&self, ctx: &Context) -> DynVal {
        let els = self.els.borrow();
        if els.is_empty() {
            return 1.0.into();
        }
        let virtual_size = 1000;
        let virtual_context = ctx.clone().with_size(Size::new(virtual_size, virtual_size));
        let avg = els
            .iter()
            .map(|el| el.get_dyn_location_set().get_height_val(&virtual_context))
            .sum::<usize>() as f64
            / els.len() as f64;
        let avg_flex = avg / virtual_size as f64;
        DynVal::new_flex(avg_flex)
    }

    fn sanitize_el_location(el: &dyn Element) {
        let mut loc = el.get_dyn_location_set().clone();

        // ignore the x-dimension everything must fit fully
        loc.set_start_x(0);
        loc.set_end_x(DynVal::FULL); // 100%
        el.set_dyn_location_set(loc); // set loc without triggering hooks
    }

    pub fn ensure_normalized_sizes(&self, ctx: &Context) {
        if *self.last_size.borrow() != ctx.size || self.is_dirty.replace(false) {
            self.normalize_locations(ctx);
            *self.last_size.borrow_mut() = ctx.size;
        }
    }

    /// normalize all the locations within the stack
    pub fn normalize_locations(&self, ctx: &Context) {
        let mut heights: Vec<DynVal> = self
            .els
            .borrow()
            .iter()
            .map(|el| el.get_dyn_location_set().get_dyn_height())
            .collect();

        self.normalize_heights_to_context(ctx, &mut heights);

        // set all the locations based on the heights
        self.adjust_locations_for_heights(&heights);

        // ensure that the first element is at 0.0 and the last element is at 1.0
        let Some(el) = self.first() else {
            return;
        };
        let l = el.get_dyn_location_set().clone().with_start_y(0.into());
        el.set_dyn_location_set(l);
        let Some(el) = self.last() else {
            return;
        };
        let l = el.get_dyn_location_set().clone().with_end_y(DynVal::FULL);
        el.set_dyn_location_set(l);
    }

    /// incrementally change the flex value of each of the existing heights (evenly), until
    /// the context height is reached. max out at 30 iterations.
    pub fn normalize_heights_to_context(&self, ctx: &Context, heights: &mut [DynVal]) {
        adjust_els_to_fit_ctx_size(ctx.get_height(), heights, *self.min_resize_height.borrow());
    }

    /// adjust all the locations based on the heights
    pub fn adjust_locations_for_heights(&self, heights: &[DynVal]) {
        let mut y = DynVal::new_fixed(0);
        for (el, height) in self.els.borrow().iter().zip(heights.iter()) {
            let mut loc = el.get_dyn_location_set().clone();
            loc.set_start_y(y.clone());
            loc.set_dyn_height(height.clone());
            el.set_dyn_location_set(loc); // set loc without triggering hooks
            y = y.plus(height.clone());
        }
    }
}

#[derive(Clone)]
pub struct HorizontalStack {
    pub pane: ParentPane,
    #[allow(clippy::type_complexity)]
    pub els: Rc<RefCell<Vec<Box<dyn Element>>>>,
    pub last_size: Rc<RefCell<Size>>,

    /// minimum width allowable by resizes
    pub min_resize_width: Rc<RefCell<usize>>,

    pub is_dirty: Rc<RefCell<bool>>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl HorizontalStack {
    const KIND: &'static str = "horizontal_stack";

    pub fn new(ctx: &Context) -> Self {
        Self {
            pane: ParentPane::new(ctx, Self::KIND),
            els: Rc::new(RefCell::new(Vec::new())),
            last_size: Rc::new(RefCell::new(Size::new(0, 0))),
            min_resize_width: Rc::new(RefCell::new(1)),
            is_dirty: Rc::new(RefCell::new(true)),
        }
    }

    pub fn new_with_kind(ctx: &Context, kind: &'static str) -> Self {
        Self {
            pane: ParentPane::new(ctx, kind),
            els: Rc::new(RefCell::new(Vec::new())),
            last_size: Rc::new(RefCell::new(Size::new(0, 0))),
            min_resize_width: Rc::new(RefCell::new(1)),
            is_dirty: Rc::new(RefCell::new(true)),
        }
    }

    pub fn with_min_resize_width(self, min_resize_width: usize) -> Self {
        self.set_min_resize_width(min_resize_width);
        self
    }

    pub fn set_min_resize_width(&self, min_resize_width: usize) {
        *self.min_resize_width.borrow_mut() = min_resize_width;
    }

    /// add an element to the end of the stack resizing the other elements
    /// in order to fit the new element
    pub fn push(&self, el: Box<dyn Element>) {
        Self::sanitize_el_location(&*el);
        self.els.borrow_mut().push(el.clone());
        self.is_dirty.replace(true);
        self.pane.add_element(el);
    }

    pub fn pop(&self) {
        let el = self.els.borrow_mut().pop();
        if let Some(el) = el {
            self.is_dirty.replace(true);
            self.pane.remove_element(&el.id())
        }
    }

    pub fn insert(&self, idx: usize, el: Box<dyn Element>) {
        Self::sanitize_el_location(&*el);
        self.els.borrow_mut().insert(idx, el.clone());
        self.is_dirty.replace(true);
        self.pane.add_element(el);
    }

    pub fn remove(&self, idx: usize) {
        let el = self.els.borrow_mut().remove(idx);
        self.is_dirty.replace(true);
        self.pane.remove_element(&el.id());
    }

    pub fn clear(&self) {
        self.els.borrow_mut().clear();
        self.pane.clear_elements();
    }

    pub fn len(&self) -> usize {
        self.els.borrow().len()
    }

    pub fn get(&self, idx: usize) -> Option<Box<dyn Element>> {
        self.els.borrow().get(idx).cloned()
    }

    pub fn first(&self) -> Option<Box<dyn Element>> {
        self.els.borrow().first().cloned()
    }

    pub fn last(&self) -> Option<Box<dyn Element>> {
        self.els.borrow().last().cloned()
    }

    pub fn get_index(&self, id: &ElementID) -> Option<usize> {
        self.els.borrow().iter().position(|el| &el.id() == id)
    }

    pub fn is_empty(&self) -> bool {
        self.els.borrow().is_empty()
    }

    /// get the average value of the elements in the stack
    /// this is useful for pushing new elements with an even size
    /// to the other elements
    pub fn avg_width(&self, ctx: &Context) -> DynVal {
        let els = self.els.borrow();
        if els.is_empty() {
            return 1.0.into();
        }
        let virtual_size = 1000;
        let virtual_context = ctx.clone().with_size(Size::new(virtual_size, virtual_size));
        let avg = els
            .iter()
            .map(|el| el.get_dyn_location_set().get_width_val(&virtual_context))
            .sum::<usize>() as f64
            / els.len() as f64;
        let avg_flex = avg / virtual_size as f64;
        DynVal::new_flex(avg_flex)
    }

    fn sanitize_el_location(el: &dyn Element) {
        let mut loc = el.get_dyn_location_set().clone();

        // ignore the y-dimension everything must fit fully
        loc.set_start_y(0);
        loc.set_end_y(DynVal::FULL); // 100%
        el.set_dyn_location_set(loc); // set loc without triggering hooks
    }

    pub fn ensure_normalized_sizes(&self, ctx: &Context) {
        if *self.last_size.borrow() != ctx.size || self.is_dirty.replace(false) {
            self.normalize_locations(ctx);
            *self.last_size.borrow_mut() = ctx.size;
        }
    }

    /// normalize all the locations within the stack
    pub fn normalize_locations(&self, ctx: &Context) {
        let mut widths: Vec<DynVal> = self
            .els
            .borrow()
            .iter()
            .map(|el| el.get_dyn_location_set().get_dyn_width())
            .collect();

        self.normalize_widths_to_context(ctx, &mut widths);

        // set all the locations based on the widths
        self.adjust_locations_for_widths(&widths);

        // ensure that the first element is at 0.0 and the last element is at 1.0
        let Some(el) = self.first() else {
            return;
        };
        let l = el.get_dyn_location_set().clone().with_start_x(0.into());
        el.set_dyn_location_set(l);
        let Some(el) = self.last() else {
            return;
        };
        let l = el.get_dyn_location_set().clone().with_end_x(DynVal::FULL);
        el.set_dyn_location_set(l);
    }

    /// incrementally change the flex value of each of the existing widths (evenly), until
    /// the context width is reached. max out at 30 iterations.
    pub fn normalize_widths_to_context(&self, ctx: &Context, widths: &mut [DynVal]) {
        adjust_els_to_fit_ctx_size(ctx.get_width(), widths, *self.min_resize_width.borrow());
    }

    /// adjust all the locations based on the widths
    pub fn adjust_locations_for_widths(&self, widths: &[DynVal]) {
        let mut x = DynVal::new_fixed(0);
        for (el, width) in self.els.borrow().iter().zip(widths.iter()) {
            let mut loc = el.get_dyn_location_set().clone();
            loc.set_start_x(x.clone());
            loc.set_dyn_width(width.clone());
            el.set_dyn_location_set(loc); // set loc without triggering hooks
            x = x.plus(width.clone());
        }
    }
}

/// incrementally change the flex value of each of the existing element vals (either height or
/// width), until the total context size is reached. max out at 30 iterations. flex changes are
/// applied additively evenly to all elements (as opposed to multiplicatively).
///
/// ctx.size is either the height or width of the context
/// vals is either element heights or widths to be adjusted
fn adjust_els_to_fit_ctx_size(ctx_size: u16, vals: &mut [DynVal], min_size: usize) {
    vals.iter_mut().for_each(|v| v.flatten_internal());
    for _i in 0..30 {
        let total_size: i32 = vals.iter().map(|v| v.get_val(ctx_size)).sum();
        if total_size == ctx_size as i32 {
            break;
        }
        let total_static: i32 = vals.iter().map(|v| v.get_val(0)).sum();
        let total_flex: i32 = total_size - total_static;
        if total_flex == 0 {
            break;
        }

        let next_change = (ctx_size as i32 - total_size) as f64 / (ctx_size as f64);
        for v in vals.iter_mut() {
            let v_flex = v.get_flex_val_portion_for_ctx(ctx_size);
            let v_flex_change = next_change * v_flex as f64 / total_flex as f64;
            v.flex += v_flex_change;
            if v.get_val(ctx_size) < min_size as i32 {
                // undo the change if it leads to a value of under the min size
                v.flex -= v_flex_change
            }
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for VerticalStack {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        //debug!("id: {}, focused: {}", self.id(), self.get_focused());
        self.ensure_normalized_sizes(ctx);
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());

        let mut resized = false;
        for resp in resps.iter_mut() {
            let (el_id, top_dy, bottom_dy) = match resp {
                EventResponse::Resize(r) => {
                    if r.top_dy == 0 && r.bottom_dy == 0 {
                        if r.left_dx != 0 || r.right_dx != 0 {
                            // allow the resize to continue to be passed to the next
                            // parent element to resize this stacks width
                            let mut r_ = r.clone();
                            r_.el_id = self.id();
                            *resp = EventResponse::Resize(r_);
                            continue;
                        }

                        // this is an empty resize event
                        *resp = EventResponse::None;
                        continue;
                    }
                    (&r.el_id, r.top_dy, r.bottom_dy)
                }
                _ => continue,
            };
            let (top_el, bottom_el, change_dy) = if top_dy != 0 {
                let Some(el_index) = self.get_index(el_id) else {
                    continue;
                };
                if el_index == 0 {
                    // ignore resizing the first element on the top
                    *resp = EventResponse::None;
                    continue;
                }
                let top_el = self.get(el_index - 1).expect("top_dy, missing top el");
                let bottom_el = self.get(el_index).expect("bottom_dy, missing bottom el");
                (top_el, bottom_el, top_dy)
            } else if bottom_dy != 0 {
                let Some(el_index) = self.get_index(el_id) else {
                    continue;
                };
                if el_index == self.len() - 1 {
                    // ignore resizing the last element on the bottom
                    *resp = EventResponse::None;
                    continue;
                }
                let top_el = self.get(el_index).expect("top_dy, missing top el");
                let bottom_el = self
                    .get(el_index + 1)
                    .expect("bottom_dy, missing bottom el");
                (top_el, bottom_el, bottom_dy)
            } else {
                *resp = EventResponse::None;
                continue;
            };

            let mut t_loc = top_el.get_dyn_location_set().clone();
            let mut b_loc = bottom_el.get_dyn_location_set().clone();

            // NOTE must set to a a fixed value (aka need to get the size for the pane DynVal
            // using ctx here. if we do not then the next pane position drag will be off
            let t_start_y = t_loc.get_start_y(ctx);
            let t_end_y = t_loc.get_end_y(ctx);
            let b_start_y = b_loc.get_start_y(ctx);
            let b_end_y = b_loc.get_end_y(ctx);

            let b_start_y_adj = b_start_y + change_dy;
            let t_end_y_adj = t_end_y + change_dy;

            let min_resize_height = *self.min_resize_height.borrow() as i32;
            if t_end_y_adj - t_start_y < min_resize_height
                || b_end_y - b_start_y_adj < min_resize_height
                || b_start_y_adj < 0
            {
                continue;
            }

            t_loc.set_end_y(t_end_y_adj);
            b_loc.set_start_y(b_start_y_adj);
            bottom_el.set_dyn_location_set(b_loc);
            top_el.set_dyn_location_set(t_loc);
            resized = true;

            *resp = EventResponse::None;
        }
        if resized {
            let (_, r) = self.pane.receive_event(ctx, Event::Resize);
            resps.extend(r);
            self.normalize_locations(ctx);
        }
        (captured, resps)
    }

    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        self.ensure_normalized_sizes(ctx);
        self.pane.drawing(ctx, force_update)
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for HorizontalStack {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        //debug!("id: {}, focused: {}", self.id(), self.get_focused());
        self.ensure_normalized_sizes(ctx);
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());

        let mut resized = false;
        for resp in resps.iter_mut() {
            let (el_id, left_dx, right_dx) = match resp {
                EventResponse::Resize(r) => {
                    if r.left_dx == 0 && r.right_dx == 0 {
                        if r.top_dy != 0 || r.bottom_dy != 0 {
                            // allow the resize to continue to be passed to the next
                            // parent element to resize this stacks height
                            let mut r_ = r.clone();
                            r_.el_id = self.id();
                            *resp = EventResponse::Resize(r_);
                            continue;
                        }

                        // this is an empty resize event
                        *resp = EventResponse::None;
                        continue;
                    }
                    (&r.el_id, r.left_dx, r.right_dx)
                }
                _ => continue,
            };
            let (left_el, right_el, change_dx) = if left_dx != 0 {
                let Some(el_index) = self.get_index(el_id) else {
                    continue;
                };
                if el_index == 0 {
                    // ignore resizing the first element on the left
                    *resp = EventResponse::None;
                    continue;
                }
                let left_el = self.get(el_index - 1).expect("left_dx, missing left el");
                let right_el = self.get(el_index).expect("right_dx, missing right el");
                (left_el, right_el, left_dx)
            } else if right_dx != 0 {
                let Some(el_index) = self.get_index(el_id) else {
                    continue;
                };
                if el_index == self.len() - 1 {
                    // ignore resizing the last element on the right
                    *resp = EventResponse::None;
                    continue;
                }
                let left_el = self.get(el_index).expect("left_dx, missing left el");
                let right_el = self.get(el_index + 1).expect("right_dx, missing right el");
                (left_el, right_el, right_dx)
            } else {
                *resp = EventResponse::None;
                continue;
            };

            let mut l_loc = left_el.get_dyn_location_set().clone();
            let mut r_loc = right_el.get_dyn_location_set().clone();

            // NOTE must set to a a fixed value (aka need to get the size for the pane DynVal
            // using ctx here. if we do not then the next pane position drag will be off
            let l_start_x = l_loc.get_start_x(ctx);
            let l_end_x = l_loc.get_end_x(ctx);
            let r_start_x = r_loc.get_start_x(ctx);
            let r_end_x = r_loc.get_end_x(ctx);

            let r_start_x_adj = r_start_x + change_dx;
            let l_end_x_adj = l_end_x + change_dx;

            let min_resize_width = *self.min_resize_width.borrow() as i32;
            if l_end_x_adj - l_start_x < min_resize_width
                || r_end_x - r_start_x_adj < min_resize_width
                || r_start_x_adj < 0
            {
                continue;
            }

            l_loc.set_end_x(l_end_x_adj);
            r_loc.set_start_x(r_start_x_adj);
            right_el.set_dyn_location_set(r_loc);
            left_el.set_dyn_location_set(l_loc);
            resized = true;

            *resp = EventResponse::None;
        }
        if resized {
            let (_, r) = self.pane.receive_event(ctx, Event::Resize);
            resps.extend(r);
            self.normalize_locations(ctx);
        }
        (captured, resps)
    }
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        self.ensure_normalized_sizes(ctx);
        self.pane.drawing(ctx, force_update)
    }
}

// -----------------------------------------------------------------

/// The StackTr is used to ensure feature parity between the VerticalStack and HorizontalStack
#[allow(dead_code)]
trait StackTr {
    const KIND: &'static str;
    fn new(ctx: &Context) -> Self;
    fn new_with_kind(ctx: &Context, kind: &'static str) -> Self;
    fn push(&self, el: Box<dyn Element>);
    fn insert(&self, idx: usize, el: Box<dyn Element>);
    fn remove(&self, idx: usize);
    fn clear(&self);
    fn len(&self) -> usize;
    fn get(&self, idx: usize) -> Option<Box<dyn Element>>;
    fn is_empty(&self) -> bool;
    fn with_style(self, style: Style) -> Self;
    fn with_transparent(self) -> Self;
    fn sanitize_el_location(el: &dyn Element);
    fn ensure_normalized_sizes(&self, ctx: &Context);
    fn normalize_locations(&self, ctx: &Context);
}

impl StackTr for VerticalStack {
    const KIND: &'static str = "vertical_stack";
    fn new(ctx: &Context) -> Self {
        VerticalStack::new(ctx)
    }
    fn new_with_kind(ctx: &Context, kind: &'static str) -> Self {
        VerticalStack::new_with_kind(ctx, kind)
    }
    fn push(&self, el: Box<dyn Element>) {
        VerticalStack::push(self, el)
    }
    fn insert(&self, idx: usize, el: Box<dyn Element>) {
        VerticalStack::insert(self, idx, el)
    }
    fn remove(&self, idx: usize) {
        VerticalStack::remove(self, idx)
    }
    fn clear(&self) {
        VerticalStack::clear(self)
    }
    fn len(&self) -> usize {
        VerticalStack::len(self)
    }
    fn get(&self, idx: usize) -> Option<Box<dyn Element>> {
        VerticalStack::get(self, idx)
    }
    fn is_empty(&self) -> bool {
        VerticalStack::is_empty(self)
    }
    fn with_style(self, style: Style) -> Self {
        VerticalStack::with_style(self, style)
    }
    fn with_transparent(self) -> Self {
        VerticalStack::with_transparent(self)
    }
    fn sanitize_el_location(el: &dyn Element) {
        VerticalStack::sanitize_el_location(el)
    }
    fn ensure_normalized_sizes(&self, ctx: &Context) {
        VerticalStack::ensure_normalized_sizes(self, ctx)
    }
    fn normalize_locations(&self, ctx: &Context) {
        VerticalStack::normalize_locations(self, ctx)
    }
}

impl StackTr for HorizontalStack {
    const KIND: &'static str = "horizontal_stack";
    fn new(ctx: &Context) -> Self {
        HorizontalStack::new(ctx)
    }
    fn new_with_kind(ctx: &Context, kind: &'static str) -> Self {
        HorizontalStack::new_with_kind(ctx, kind)
    }
    fn push(&self, el: Box<dyn Element>) {
        HorizontalStack::push(self, el)
    }
    fn insert(&self, idx: usize, el: Box<dyn Element>) {
        HorizontalStack::insert(self, idx, el)
    }
    fn remove(&self, idx: usize) {
        HorizontalStack::remove(self, idx)
    }
    fn clear(&self) {
        HorizontalStack::clear(self)
    }
    fn len(&self) -> usize {
        HorizontalStack::len(self)
    }
    fn get(&self, idx: usize) -> Option<Box<dyn Element>> {
        HorizontalStack::get(self, idx)
    }
    fn is_empty(&self) -> bool {
        HorizontalStack::is_empty(self)
    }
    fn with_style(self, style: Style) -> Self {
        HorizontalStack::with_style(self, style)
    }
    fn with_transparent(self) -> Self {
        HorizontalStack::with_transparent(self)
    }
    fn sanitize_el_location(el: &dyn Element) {
        HorizontalStack::sanitize_el_location(el)
    }
    fn ensure_normalized_sizes(&self, ctx: &Context) {
        HorizontalStack::ensure_normalized_sizes(self, ctx)
    }
    fn normalize_locations(&self, ctx: &Context) {
        HorizontalStack::normalize_locations(self, ctx)
    }
}
