use {
    crate::{
        Context, DrawCh, DrawChPos, DrawChs2D, DynLocation, DynLocationSet, DynVal, Element,
        ElementID, Event, EventResponse, EventResponses, Parent, Priority, ReceivableEventChanges,
        SelfReceivableEvents, Style, ZIndex,
    },
    std::{
        collections::HashMap,
        {
            cell::{Ref, RefCell},
            rc::Rc,
        },
    },
};

/// Pane is a pane element which other objects can embed and build off
/// of. It defines the basic draw functionality of a pane.
#[derive(Clone)]
pub struct Pane {
    kind: Rc<RefCell<&'static str>>,

    /// element-id as assigned by the sorting-hat
    id: Rc<RefCell<String>>,

    attributes: Rc<RefCell<HashMap<String, Vec<u8>>>>,

    /// The SelfEvs are NOT handled by the standard pane. The element inheriting the
    /// standard pane is expected to handle all SelfReceivableEvents in the
    /// ReceiveEvent function. The standard pane is only responsible for
    /// listing the receivable events when Receivable() is called
    pub self_evs: Rc<RefCell<SelfReceivableEvents>>,

    /// This elements "overall" reference priority
    ///
    /// TODO this is only used currently by the ParentPane,
    /// consider moving this field into the ParentPane, if nothing
    /// else ever uses it.
    pub element_priority: Rc<RefCell<Priority>>,

    pub parent: Rc<RefCell<Option<Box<dyn Parent>>>>,

    #[allow(clippy::type_complexity)]
    pub hooks:
        Rc<RefCell<HashMap<String, Vec<(ElementID, Box<dyn FnMut(&str, Box<dyn Element>)>)>>>>,

    /// The pane's Content need not be the dimensions provided within
    /// the Location, however the Content will simply be cut off if it exceeds
    /// any dimension of the Location. If the Content is less than the dimensions
    /// of the Location all extra characters will be filled with the DefaultCh.
    /// The location of where to begin drawing from within the Content can be
    /// offset using content_view_offset_x and content_view_offset_y
    pub content: Rc<RefCell<DrawChs2D>>,
    pub default_ch: Rc<RefCell<DrawCh>>,
    pub content_view_offset_x: Rc<RefCell<usize>>,
    pub content_view_offset_y: Rc<RefCell<usize>>,

    /// scaleable values of x, y, width, and height in the parent context
    /// NOTE use getters/setters to ensure hook calls
    loc: Rc<RefCell<DynLocationSet>>,
    visible: Rc<RefCell<bool>>,
}

impl Pane {
    /// NOTE kind is a name for the kind of pane, typically a different kind will be applied
    /// to the standard pane, as the standard pane is only boilerplate.
    pub const KIND: &'static str = "standard_pane";

    pub fn new(ctx: &Context, kind: &'static str) -> Pane {
        Pane {
            kind: Rc::new(RefCell::new(kind)),
            id: Rc::new(RefCell::new(ctx.hat.create_element_id(kind))),
            attributes: Rc::new(RefCell::new(HashMap::new())),
            self_evs: Rc::new(RefCell::new(SelfReceivableEvents::default())),
            element_priority: Rc::new(RefCell::new(Priority::Unfocused)),
            parent: Rc::new(RefCell::new(None)),
            hooks: Rc::new(RefCell::new(HashMap::new())),
            content: Rc::new(RefCell::new(DrawChs2D::default())),
            default_ch: Rc::new(RefCell::new(DrawCh::default())),
            content_view_offset_x: Rc::new(RefCell::new(0)),
            content_view_offset_y: Rc::new(RefCell::new(0)),
            loc: Rc::new(RefCell::new(DynLocationSet::full())),
            visible: Rc::new(RefCell::new(true)),
        }
    }

    pub fn with_kind(self, kind: &'static str) -> Pane {
        *self.kind.borrow_mut() = kind;
        self
    }

    pub fn at(self, x: DynVal, y: DynVal) -> Pane {
        self.set_at(x, y);
        self
    }

    pub fn set_at(&self, x: DynVal, y: DynVal) {
        self.loc.borrow_mut().l.set_at(x, y);
    }

    pub fn set_kind(&self, kind: &'static str) {
        *self.kind.borrow_mut() = kind;
    }

    pub fn focused(self) -> Pane {
        *self.element_priority.borrow_mut() = Priority::Focused;
        self
    }

    pub fn unfocused(self) -> Pane {
        *self.element_priority.borrow_mut() = Priority::Unfocused;
        self
    }

    pub fn with_z(self, z: ZIndex) -> Pane {
        self.loc.borrow_mut().set_z(z);
        self
    }

    pub fn with_start_x(self, x: DynVal) -> Pane {
        self.loc.borrow_mut().l.set_start_x(x);
        self
    }

    pub fn with_start_y(self, y: DynVal) -> Pane {
        self.loc.borrow_mut().l.set_start_y(y);
        self
    }

    pub fn set_start_x(&self, x: DynVal) {
        self.loc.borrow_mut().l.set_start_x(x);
    }

    pub fn set_start_y(&self, y: DynVal) {
        self.loc.borrow_mut().l.set_start_y(y);
    }

    pub fn set_end_x(&self, x: DynVal) {
        self.loc.borrow_mut().l.set_end_x(x);
    }

    pub fn set_end_y(&self, y: DynVal) {
        self.loc.borrow_mut().l.set_end_y(y);
    }

    pub fn get_start_x(&self, ctx: &Context) -> i32 {
        self.loc.borrow().l.get_start_x(ctx)
    }

    pub fn get_start_y(&self, ctx: &Context) -> i32 {
        self.loc.borrow().l.get_start_y(ctx)
    }

    pub fn get_end_x(&self, ctx: &Context) -> i32 {
        self.loc.borrow().l.get_end_x(ctx)
    }

    pub fn get_end_y(&self, ctx: &Context) -> i32 {
        self.loc.borrow().l.get_end_y(ctx)
    }

    pub fn get_dyn_start_x(&self) -> DynVal {
        self.loc.borrow().l.start_x.clone()
    }

    pub fn get_dyn_start_y(&self) -> DynVal {
        self.loc.borrow().l.start_y.clone()
    }

    pub fn get_dyn_end_x(&self) -> DynVal {
        self.loc.borrow().l.end_x.clone()
    }

    pub fn get_dyn_end_y(&self) -> DynVal {
        self.loc.borrow().l.end_y.clone()
    }

    pub fn get_height(&self, ctx: &Context) -> usize {
        self.loc.borrow().l.height(ctx)
    }

    pub fn get_width(&self, ctx: &Context) -> usize {
        self.loc.borrow().l.width(ctx)
    }

    pub fn with_dyn_height(self, h: DynVal) -> Pane {
        self.loc.borrow_mut().l.set_dyn_height(h);
        self
    }

    pub fn with_dyn_width(self, w: DynVal) -> Pane {
        self.loc.borrow_mut().l.set_dyn_width(w);
        self
    }

    pub fn set_dyn_height(&self, h: DynVal) {
        self.loc.borrow_mut().l.set_dyn_height(h);
    }

    pub fn set_dyn_width(&self, w: DynVal) {
        self.loc.borrow_mut().l.set_dyn_width(w);
    }

    pub fn get_dyn_height(&self) -> DynVal {
        self.loc.borrow().l.get_dyn_height()
    }

    pub fn get_dyn_width(&self) -> DynVal {
        self.loc.borrow().l.get_dyn_width()
    }

    pub fn set_z(&self, z: ZIndex) {
        self.loc.borrow_mut().set_z(z);
    }

    pub fn with_dyn_location(self, l: DynLocation) -> Pane {
        self.loc.borrow_mut().l = l;
        self
    }

    pub fn get_dyn_location(&self) -> DynLocation {
        self.loc.borrow().l.clone()
    }

    pub fn set_dyn_location(&self, l: DynLocation) {
        self.loc.borrow_mut().l = l;
    }

    pub fn with_content(self, content: DrawChs2D) -> Pane {
        *self.content.borrow_mut() = content;
        self
    }

    pub fn set_content(&self, content: DrawChs2D) {
        *self.content.borrow_mut() = content;
    }

    pub fn set_content_from_string<S: Into<String>>(&self, s: S) {
        *self.content.borrow_mut() = DrawChs2D::from_string(s.into(), self.get_style());
    }

    /// sets content from string
    pub fn set_content_from_string_with_style(&self, ctx: &Context, s: &str, sty: Style) {
        let lines = s.split('\n');
        let mut rs: Vec<Vec<char>> = Vec::new();

        let mut width = self.get_width(ctx);
        let mut height = self.get_height(ctx);
        for line in lines {
            if width < line.len() {
                width = line.len();
            }
            rs.push(line.chars().collect());
        }
        if height < rs.len() {
            height = rs.len();
        }

        // initialize the content with blank characters
        // of the height and width of the widget
        *self.content.borrow_mut() = DrawChs2D::new_empty_of_size(width, height, sty.clone());

        // now fill in with actual content
        for y in 0..height {
            for x in 0..width {
                let r = if y < rs.len() && x < rs[y].len() {
                    rs[y][x]
                } else {
                    continue;
                };
                let dch = DrawCh::new(r, sty.clone());
                self.content.borrow_mut().0[y][x] = dch;
            }
        }
    }

    pub fn set_content_style(&self, sty: Style) {
        self.content.borrow_mut().change_all_styles(sty);
    }

    pub fn content_width(&self) -> usize {
        self.content.borrow().width()
    }

    pub fn content_height(&self) -> usize {
        self.content.borrow().height()
    }

    pub fn scroll_up(&self, ctx: &Context) {
        let view_offset_y = *self.content_view_offset_y.borrow();
        self.set_content_y_offset(ctx, view_offset_y.saturating_sub(1));
    }

    pub fn scroll_down(&self, ctx: &Context) {
        let view_offset_y = *self.content_view_offset_y.borrow();
        self.set_content_y_offset(ctx, view_offset_y + 1);
    }

    pub fn scroll_left(&self, ctx: &Context) {
        let view_offset_x = *self.content_view_offset_x.borrow();
        self.set_content_x_offset(ctx, view_offset_x.saturating_sub(1));
    }

    pub fn scroll_right(&self, ctx: &Context) {
        let view_offset_x = *self.content_view_offset_x.borrow();
        self.set_content_x_offset(ctx, view_offset_x + 1);
    }

    pub fn with_default_ch(self, ch: DrawCh) -> Pane {
        *self.default_ch.borrow_mut() = ch;
        self
    }

    pub fn with_style(self, style: Style) -> Pane {
        self.default_ch.borrow_mut().style = style;
        self
    }

    pub fn set_style(&self, style: Style) {
        self.default_ch.borrow_mut().style = style;
    }

    pub fn get_style(&self) -> Style {
        self.default_ch.borrow().style.clone()
    }

    pub fn set_default_ch(&self, ch: DrawCh) {
        *self.default_ch.borrow_mut() = ch;
    }

    pub fn with_self_receivable_events(self, evs: SelfReceivableEvents) -> Pane {
        *self.self_evs.borrow_mut() = evs;
        self
    }

    pub fn set_self_receivable_events(&self, evs: SelfReceivableEvents) {
        *self.self_evs.borrow_mut() = evs;
    }

    // -----------------------

    pub fn get_element_priority(&self) -> Priority {
        *self.element_priority.borrow()
    }

    /// NOTE this name was chosen to distinguish itself from propagate_responses_upward
    pub fn send_responses_upward(&self, ctx: &Context, resps: EventResponses) {
        if let Some(parent) = self.parent.borrow().as_ref() {
            if let Some(parent_ctx) = ctx.parent_context() {
                parent.propagate_responses_upward(parent_ctx, &self.id(), resps);
            }
        }
    }

    pub fn has_parent(&self) -> bool {
        self.parent.borrow().is_some()
    }

    /// focus all prioritized events
    pub fn focus(&self, ctx: &Context) {
        let rec = self.change_priority(Priority::Focused);
        if self.has_parent() {
            let resps = EventResponse::ReceivableEventChanges(rec);
            self.send_responses_upward(ctx, resps.into());
        }
    }

    /// defocus all prioritized events
    pub fn unfocus(&self, ctx: &Context) {
        let rec = self.change_priority(Priority::Unfocused);
        if self.has_parent() {
            let resps = EventResponse::ReceivableEventChanges(rec);
            self.send_responses_upward(ctx, resps.into());
        }
    }

    /// correct_offsets_to_view_position changes the content offsets within the
    /// pane in order to bring the given view position into view.
    pub fn correct_offsets_to_view_position(&self, ctx: &Context, x: usize, y: usize) {
        let view_offset_y = *self.content_view_offset_y.borrow();
        let view_offset_x = *self.content_view_offset_x.borrow();

        // set y offset if cursor out of bounds
        if y >= view_offset_y + self.get_height(ctx) {
            //debug!("cor1");
            self.set_content_y_offset(ctx, y - self.get_height(ctx) + 1);
        } else if y < view_offset_y {
            //debug!("cor2");
            self.set_content_y_offset(ctx, y);
        }

        // correct the offset if the offset is now showing lines that don't exist in
        // the content
        if view_offset_y + self.get_height(ctx) > self.content_height() {
            //debug!("cor3");
            self.set_content_y_offset(ctx, self.content_height());
        }

        // set x offset if cursor out of bounds
        if x >= view_offset_x + self.get_width(ctx) {
            self.set_content_x_offset(ctx, x - self.get_width(ctx) + 1);
        } else if x < view_offset_x {
            self.set_content_x_offset(ctx, x);
        }

        // correct the offset if the offset is now showing characters to the right
        // which don't exist in the content.
        if view_offset_x + self.get_width(ctx) > self.content_width() {
            self.set_content_x_offset(ctx, self.content_width());
        }
    }

    ///// correct_offsets_to_view_position changes the content offsets within the
    ///// pane in order to bring the given view position into view.
    //pub fn correct_offsets_to_view_position(&self, ctx: &Context, x: usize, y: usize) {
    //    let view_offset_y = *self.content_view_offset_y.borrow();
    //    let view_offset_x = *self.content_view_offset_x.borrow();
    //    let height = ctx.s.height as usize;
    //    let width = ctx.s.width as usize;

    //    debug!("corr for x: {} y: {}", x, y);

    //    // set y offset if cursor out of bounds
    //    if y >= view_offset_y + height {
    //        debug!("cor1, setting to {}", y - height + 1);
    //        self.set_content_y_offset(ctx, y - height + 1);
    //    } else if y < view_offset_y {
    //        debug!("cor2");
    //        self.set_content_y_offset(ctx, y);
    //    }

    //    // correct the offset if the offset is now showing lines that don't exist in
    //    // the content
    //    if view_offset_y + height > self.content_height() {
    //        self.set_content_y_offset(ctx, height);
    //    }

    //    // set x offset if cursor out of bounds
    //    if x >= view_offset_x + width {
    //        self.set_content_x_offset(ctx, x - width + 1);
    //    } else if x < view_offset_x {
    //        self.set_content_x_offset(ctx, x);
    //    }

    //    // correct the offset if the offset is now showing characters to the right
    //    // which don't exist in the content.
    //    if view_offset_x + width > self.content_width() {
    //        self.set_content_x_offset(ctx, self.content_width());
    //    }
    //}
}

impl Element for Pane {
    fn kind(&self) -> &'static str {
        *self.kind.borrow()
    }

    fn id(&self) -> ElementID {
        self.id.borrow().clone()
    }

    /// Receivable returns the event keys and commands that can
    /// be received by this element along with their priorities
    fn receivable(&self) -> SelfReceivableEvents {
        self.self_evs.borrow().clone()
    }

    ///                                                       (captured, resp          )
    fn receive_event_inner(&self, _ctx: &Context, _ev: Event) -> (bool, EventResponses) {
        (false, EventResponses::default())
    }

    /// ChangePriority returns a priority change request to its parent organizer so
    /// as to update the priority of all commands registered to this element.
    /// The element iterates through its registered cmds/evCombos, and returns a
    /// priority change request for each one.
    fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
        // update the priority of all registered events
        for pef in self.self_evs.borrow_mut().iter_mut() {
            pef.1 = p;
        }
        *self.element_priority.borrow_mut() = p;
        self.self_evs.borrow().to_receivable_event_changes()
    }

    /// Drawing compiles all of the DrawChPos necessary to draw this element
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        let mut chs = vec![];

        let (xmin, xmax, ymin, ymax) = if let Some(vis_region) = ctx.visible_region {
            (
                // take the intersection of the visibile region and the elements region
                (vis_region.start_x as usize).max(0),
                (vis_region.end_x as usize).min(ctx.s.width as usize),
                (vis_region.start_y as usize).max(0),
                (vis_region.end_y as usize).min(ctx.s.height as usize),
            )
        } else {
            (0, ctx.s.width as usize, 0, ctx.s.height as usize)
        };

        let view_offset_y = *self.content_view_offset_y.borrow();
        let view_offset_x = *self.content_view_offset_x.borrow();
        let default_ch = self.default_ch.borrow().clone();
        let content_height = self.content.borrow().height();
        let content_width = self.content.borrow().width();

        // convert the Content to DrawChPos
        for y in ymin..ymax {
            for x in xmin..xmax {
                let offset_y = y + view_offset_y;
                let offset_x = x + view_offset_x;

                // if the offset isn't pushing all the content out of view,
                // assign the next ch to be the one at the offset in the Content
                // matrix
                let ch_out = if offset_y < content_height && offset_x < content_width {
                    self.content.borrow().0[offset_y][offset_x].clone()
                } else {
                    default_ch.clone()
                };

                // convert the DrawCh to a DrawChPos
                chs.push(DrawChPos::new(ch_out, x as u16, y as u16))
            }
        }
        chs
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.attributes.borrow().get(key).cloned()
    }

    fn set_attribute_inner(&self, key: &str, value: Vec<u8>) {
        self.attributes.borrow_mut().insert(key.to_string(), value);
    }

    fn set_parent(&self, parent: Box<dyn Parent>) {
        *self.parent.borrow_mut() = Some(parent);
    }

    fn set_hook(
        &self,
        kind: &str,
        el_id: ElementID,
        //                  kind, hooked element
        hook: Box<dyn FnMut(&str, Box<dyn Element>)>,
    ) {
        self.hooks
            .borrow_mut()
            .entry(kind.to_string())
            .or_default()
            .push((el_id, hook));
    }

    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        if let Some(hook) = self.hooks.borrow_mut().get_mut(kind) {
            hook.retain(|(el_id_, _)| *el_id_ != el_id);
        };
    }

    /// remove all hooks for the element with the given id
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        for (_, hook) in self.hooks.borrow_mut().iter_mut() {
            hook.retain(|(el_id_, _)| *el_id_ != el_id);
        }
    }

    /// calls all the hooks of the provided kind
    fn call_hooks_of_kind(&self, kind: &str) {
        for (kind_, v) in self.hooks.borrow_mut().iter_mut() {
            if kind == kind_ {
                for (_, hook) in v.iter_mut() {
                    hook(kind, Box::new(self.clone()));
                }
            }
        }
    }

    fn get_dyn_location_set(&self) -> Ref<DynLocationSet> {
        self.loc.borrow()
    }
    fn get_visible(&self) -> bool {
        *self.visible.borrow()
    }

    fn get_ref_cell_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.loc.clone()
    }
    fn get_ref_cell_visible(&self) -> Rc<RefCell<bool>> {
        self.visible.clone()
    }

    fn set_content_x_offset(&self, ctx: &Context, x: usize) {
        let content_width = self.content.borrow().width();
        let view_width = self.loc.borrow().get_width_val(ctx);
        let x = if x > content_width.saturating_sub(view_width) {
            content_width.saturating_sub(view_width)
        } else {
            x
        };
        *self.content_view_offset_x.borrow_mut() = x;
    }

    fn set_content_y_offset(&self, ctx: &Context, y: usize) {
        let content_height = self.content.borrow().height();
        let view_height = self.loc.borrow().get_height_val(ctx);
        let y = if y > content_height.saturating_sub(view_height) {
            content_height.saturating_sub(view_height)
        } else {
            y
        };
        *self.content_view_offset_y.borrow_mut() = y;
    }

    fn get_content_x_offset(&self) -> usize {
        *self.content_view_offset_x.borrow()
    }
    fn get_content_y_offset(&self) -> usize {
        *self.content_view_offset_y.borrow()
    }
    fn get_content_width(&self) -> usize {
        self.content.borrow().width()
    }
    fn get_content_height(&self) -> usize {
        self.content.borrow().height()
    }
}
