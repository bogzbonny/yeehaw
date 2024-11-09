use {
    crate::{
        Context, DrawCh, DrawChPos, DrawChs2D, DynLocation, DynLocationSet, DynVal, Element,
        ElementID, Event, EventResponse, EventResponses, Parent, Priority, ReceivableEventChanges,
        SelfReceivableEvents, Style, ZIndex,
    },
    std::{
        collections::HashMap,
        {cell::RefCell, rc::Rc},
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
    pub default_line: Rc<RefCell<Vec<DrawCh>>>,
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
            default_line: Rc::new(RefCell::new(vec![])),
            content_view_offset_x: Rc::new(RefCell::new(0)),
            content_view_offset_y: Rc::new(RefCell::new(0)),
            loc: Rc::new(RefCell::new(DynLocationSet::new_full())),
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

    pub fn with_default_line(self, line: Vec<DrawCh>) -> Pane {
        *self.default_line.borrow_mut() = line;
        self
    }

    pub fn with_content_view_offset(self, x: usize, y: usize) -> Pane {
        *self.content_view_offset_x.borrow_mut() = x;
        *self.content_view_offset_y.borrow_mut() = y;
        self
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

        // convert the Content to DrawChPos
        for y in ymin..ymax {
            for x in xmin..xmax {
                // default ch being added next is the DefaultCh
                let mut ch_out = self.default_ch.borrow().clone();

                let offset_y = y + *self.content_view_offset_y.borrow();
                let offset_x = x + *self.content_view_offset_x.borrow();

                // if the offset isn't pushing all the content out of view,
                // assign the next ch to be the one at the offset in the Content
                // matrix
                if offset_y < self.content.borrow().0.len()
                    && offset_x < self.content.borrow().0[offset_y].len()
                {
                    ch_out = self.content.borrow().0[offset_y][offset_x].clone();
                }

                // if y is greater than the height of the visible content,
                // trigger a default line.
                // NOTE: height of the visible content is the height of the
                // content minus the offset
                if y > self.content.borrow().0.len() {
                    if x < self.default_line.borrow().len() {
                        ch_out = self.default_line.borrow()[x].clone();
                    } else {
                        ch_out = self.default_ch.borrow().clone();
                    }
                }

                // convert the DrawCh to a DrawChPos
                chs.push(DrawChPos::new(ch_out, x as u16, y as u16))
            }
        }
        chs
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.attributes.borrow().get(key).cloned()
    }

    fn set_attribute(&self, key: &str, value: Vec<u8>) {
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

    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.loc.clone()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.visible.clone()
    }

    fn set_content_x_offset(&self, ctx: &Context, x: usize) {
        // need these +1 or else will overscroll
        // - due to x/y being 0 indexed
        let content_width = self.content.borrow().width();
        let view_width = self.loc.borrow().get_width_val(ctx);
        let x = if x > content_width.saturating_sub(view_width + 1) {
            content_width.saturating_sub(view_width + 1)
        } else {
            x
        };
        *self.content_view_offset_x.borrow_mut() = x;
    }

    fn set_content_y_offset(&self, ctx: &Context, y: usize) {
        // need these +1 or else will overscroll
        // - due to x/y being 0 indexed
        let content_height = self.content.borrow().height();
        let view_height = self.loc.borrow().get_height_val(ctx);
        let y = if y > content_height.saturating_sub(view_height + 1) {
            content_height.saturating_sub(view_height + 1)
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
