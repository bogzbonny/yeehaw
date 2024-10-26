use {
    crate::{
        Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event, EventResponses,
        ParentPane, Priority, ReceivableEventChanges, Size, SortingHat, Parent,
    },
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct VerticalStack {
    pub pane: ParentPane,
    #[allow(clippy::type_complexity)]
    pub els: Rc<RefCell<Vec<Rc<RefCell<dyn Element>>>>>,
}

impl VerticalStack {
    const KIND: &'static str = "vertical_stack";

    pub fn new(hat: &SortingHat) -> Self {
        Self {
            pane: ParentPane::new(hat, Self::KIND),
            els: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn new_with_kind(hat: &SortingHat, kind: &'static str) -> Self {
        Self {
            pane: ParentPane::new(hat, kind),
            els: Rc::new(RefCell::new(Vec::new())),
        }
    }

    // add an element to the end of the stack resizing the other elements
    // in order to fit the new element
    pub fn push(&self, ctx: &Context, el: Rc<RefCell<dyn Element>>) {
        Self::sanitize_el_location(&el);
        self.els.borrow_mut().push(el.clone());
        self.normalize_locations(ctx);
        self.pane.add_element(el);
    }

    pub fn insert(&self, ctx: &Context, idx: usize, el: Rc<RefCell<dyn Element>>) {
        Self::sanitize_el_location(&el);
        self.els.borrow_mut().insert(idx, el.clone());
        self.normalize_locations(ctx);
        self.pane.add_element(el);
    }

    pub fn remove(&self, ctx: &Context, idx: usize) {
        let el = self.els.borrow_mut().remove(idx);
        self.normalize_locations(ctx);
        self.pane.remove_element(&el.borrow().id());
    }

    pub fn clear(&self) {
        self.els.borrow_mut().clear();
        self.pane.clear_elements();
    }

    pub fn len(&self) -> usize {
        self.els.borrow().len()
    }

    pub fn get(&self, idx: usize) -> Option<Rc<RefCell<dyn Element>>> {
        self.els.borrow().get(idx).cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.els.borrow().is_empty()
    }

    // get the average value of the elements in the stack
    // this is useful for pushing new elements with an even size
    // to the other elements
    pub fn avg_height(&self, ctx: &Context) -> DynVal {
        let els = self.els.borrow();
        if els.is_empty() {
            return 1.0.into();
        }
        let virtual_size = 1000;
        let virtual_context =
            Context::new(Size::new(virtual_size, virtual_size), ctx.dur_since_launch);
        let avg = els
            .iter()
            .map(|el| {
                el.borrow()
                    .get_dyn_location_set()
                    .borrow()
                    .get_height_val(&virtual_context)
            })
            .sum::<usize>() as f64
            / els.len() as f64;
        let avg_flex = avg / virtual_size as f64;
        DynVal::new_flex(avg_flex)
    }

    fn sanitize_el_location(el: &Rc<RefCell<dyn Element>>) {
        let mut loc = el.borrow().get_dyn_location_set().borrow().clone();

        // ignore the x-dimension everything must fit fully
        loc.set_start_x(0.0.into()); // 0
        loc.set_end_x(1.0.into()); // 100%
        *el.borrow_mut().get_dyn_location_set().borrow_mut() = loc; // set loc without triggering hooks
    }

    // normalize all the locations within the stack
    pub fn normalize_locations(&self, ctx: &Context) {
        let mut heights: Vec<DynVal> = self
            .els
            .borrow()
            .iter()
            .map(|el| el.borrow().get_dyn_location_set().borrow().get_dyn_height())
            .collect();

        Self::normalize_heights_to_context(ctx, &mut heights);

        // set all the locations based on the heights
        self.adjust_locations_for_heights(&heights);
    }

    // incrementally change the flex value of each of the existing heights (evenly), until
    // the context height is reached. max out at 30 iterations.
    pub fn normalize_heights_to_context(ctx: &Context, heights: &mut [DynVal]) {
        adjust_els_to_fit_ctx_size(ctx.get_height(), heights);
    }

    // adjust all the locations based on the heights
    pub fn adjust_locations_for_heights(&self, heights: &[DynVal]) {
        let mut y = DynVal::new_fixed(0);
        for (el, height) in self.els.borrow().iter().zip(heights.iter()) {
            let mut loc = el.borrow().get_dyn_location_set().borrow().clone();
            loc.set_start_y(y.clone());
            loc.set_dyn_height(height.clone());
            *el.borrow_mut().get_dyn_location_set().borrow_mut() = loc; // set loc without triggering hooks
            y = y.plus(height.clone());
        }
    }
}

#[derive(Clone)]
pub struct HorizontalStack {
    pub pane: ParentPane,
    #[allow(clippy::type_complexity)]
    pub els: Rc<RefCell<Vec<Rc<RefCell<dyn Element>>>>>,
}

impl HorizontalStack {
    const KIND: &'static str = "horizontal_stack";

    pub fn new(hat: &SortingHat) -> Self {
        Self {
            pane: ParentPane::new(hat, Self::KIND),
            els: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn with_height(self, h: DynVal) -> Self {
        self.pane.pane.set_dyn_height(h);
        self
    }

    pub fn with_width(self, w: DynVal) -> Self {
        self.pane.pane.set_dyn_width(w);
        self
    }

    // add an element to the end of the stack resizing the other elements
    // in order to fit the new element
    pub fn push(&self, ctx: &Context, el: Rc<RefCell<dyn Element>>) {
        Self::sanitize_el_location(&el);
        self.els.borrow_mut().push(el.clone());
        self.normalize_locations(ctx);
        self.pane.add_element(el);
    }

    pub fn insert(&self, ctx: &Context, idx: usize, el: Rc<RefCell<dyn Element>>) {
        Self::sanitize_el_location(&el);
        self.els.borrow_mut().insert(idx, el.clone());
        self.normalize_locations(ctx);
        self.pane.add_element(el);
    }

    pub fn remove(&self, ctx: &Context, idx: usize) {
        let el = self.els.borrow_mut().remove(idx);
        self.normalize_locations(ctx);
        self.pane.remove_element(&el.borrow().id());
    }

    pub fn clear(&self) {
        self.els.borrow_mut().clear();
        self.pane.clear_elements();
    }

    pub fn len(&self) -> usize {
        self.els.borrow().len()
    }

    pub fn get(&self, idx: usize) -> Option<Rc<RefCell<dyn Element>>> {
        self.els.borrow().get(idx).cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.els.borrow().is_empty()
    }

    // get the average value of the elements in the stack
    // this is useful for pushing new elements with an even size
    // to the other elements
    pub fn avg_width(&self, ctx: &Context) -> DynVal {
        let els = self.els.borrow();
        if els.is_empty() {
            return 1.0.into();
        }
        let virtual_size = 1000;
        let virtual_context =
            Context::new(Size::new(virtual_size, virtual_size), ctx.dur_since_launch);
        let avg = els
            .iter()
            .map(|el| {
                el.borrow()
                    .get_dyn_location_set()
                    .borrow()
                    .get_width_val(&virtual_context)
            })
            .sum::<usize>() as f64
            / els.len() as f64;
        let avg_flex = avg / virtual_size as f64;
        DynVal::new_flex(avg_flex)
    }

    fn sanitize_el_location(el: &Rc<RefCell<dyn Element>>) {
        let mut loc = el.borrow().get_dyn_location_set().borrow().clone();

        // ignore the y-dimension everything must fit fully
        loc.set_start_y(0.0.into()); // 0
        loc.set_end_y(1.0.into()); // 100%
        *el.borrow_mut().get_dyn_location_set().borrow_mut() = loc; // set loc without triggering hooks
    }

    // normalize all the locations within the stack
    pub fn normalize_locations(&self, ctx: &Context) {
        let mut widths: Vec<DynVal> = self
            .els
            .borrow()
            .iter()
            .map(|el| el.borrow().get_dyn_location_set().borrow().get_dyn_width())
            .collect();

        Self::normalize_widths_to_context(ctx, &mut widths);

        // set all the locations based on the widths
        self.adjust_locations_for_widths(&widths);
    }

    // incrementally change the flex value of each of the existing widths (evenly), until
    // the context width is reached. max out at 30 iterations.
    pub fn normalize_widths_to_context(ctx: &Context, widths: &mut [DynVal]) {
        adjust_els_to_fit_ctx_size(ctx.get_width(), widths);
    }

    // adjust all the locations based on the widths
    pub fn adjust_locations_for_widths(&self, widths: &[DynVal]) {
        let mut x = DynVal::new_fixed(0);
        for (el, width) in self.els.borrow().iter().zip(widths.iter()) {
            let mut loc = el.borrow().get_dyn_location_set().borrow().clone();
            loc.set_start_x(x.clone());
            loc.set_dyn_width(width.clone());
            *el.borrow_mut().get_dyn_location_set().borrow_mut() = loc; // set loc without triggering hooks
            x = x.plus(width.clone());
        }
    }
}

// incrementally change the flex value of each of the existing element vals (either height or
// width), until the total context size is reached. max out at 30 iterations. flex changes are
// applied additively evenly to all elements (as opposed to multiplicatively).
//
// ctx_size is either the height or width of the context
// vals is either element heights or widths to be adjusted
fn adjust_els_to_fit_ctx_size(ctx_size: u16, vals: &mut [DynVal]) {
    vals.iter_mut().for_each(|h| h.flatten_internal());
    for _i in 0..30 {
        let total_size: i32 = vals.iter().map(|h| h.get_val(ctx_size)).sum();
        if total_size == ctx_size as i32 {
            break;
        }
        let total_static: i32 = vals.iter().map(|h| h.get_val(0)).sum();
        let total_flex: i32 = total_size - total_static;
        if total_flex == 0 {
            break;
        }

        let next_change = (ctx_size as i32 - total_size) as f64 / (ctx_size as f64);
        for h in vals.iter_mut() {
            let h_flex = h.get_flex_val_portion_for_ctx(ctx_size);
            let h_flex_change = next_change * h_flex as f64 / total_flex as f64;
            h.flex += h_flex_change;
        }
    }
}

impl Element for VerticalStack {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.pane.receive_event(ctx, ev.clone())
    }
    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.pane.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn Parent>) {
        self.pane.set_upward_propagator(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.pane.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.pane.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.pane.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.pane.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.pane.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.pane.get_visible()
    }
}

impl Element for HorizontalStack {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.pane.receive_event(ctx, ev.clone())
    }
    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.pane.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn Parent>) {
        self.pane.set_upward_propagator(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.pane.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.pane.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.pane.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.pane.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.pane.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.pane.get_visible()
    }
}
