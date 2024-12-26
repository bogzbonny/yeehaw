use {
    crate::{
        Color, Context, DrawCh, DrawChPos, DrawChs2D, DrawUpdate, DynLocation, DynLocationSet,
        DynVal, Element, ElementID, Event, EventResponses, Loc, Parent, ReceivableEvents, Size,
        Style, ZIndex,
    },
    std::{
        collections::HashMap,
        {
            cell::{Ref, RefCell, RefMut},
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

    /// events which this element can receive while focused
    pub rec_evs_focused: Rc<RefCell<ReceivableEvents>>,

    /// events which this element can receive while focused or unfocused
    pub rec_evs_always: Rc<RefCell<ReceivableEvents>>,

    /// Element focus
    pub focused: Rc<RefCell<bool>>,

    pub parent: Rc<RefCell<Option<Box<dyn Parent>>>>,

    #[allow(clippy::type_complexity)]
    pub hooks:
        Rc<RefCell<HashMap<String, Vec<(ElementID, Box<dyn FnMut(&str, Box<dyn Element>)>)>>>>,

    content: Rc<RefCell<DrawChs2D>>,
    is_content_dirty: Rc<RefCell<bool>>,
    last_size: Rc<RefCell<Size>>,
    last_visible_region: Rc<RefCell<Option<Loc>>>,

    pub default_ch: Rc<RefCell<DrawCh>>,
    pub content_view_offset_x: Rc<RefCell<usize>>,
    pub content_view_offset_y: Rc<RefCell<usize>>,

    /// scaleable values of x, y, width, and height in the parent context
    /// NOTE use getters/setters to ensure hook calls
    loc: Rc<RefCell<DynLocationSet>>,
    visible: Rc<RefCell<bool>>,

    /// allows a pane to overflow its bounds during drawing. Useful for menus
    overflow: Rc<RefCell<bool>>,
}

impl Pane {
    /// NOTE kind is a name for the kind of pane, typically a different kind will be applied
    /// to the standard pane, as the standard pane is only boilerplate.
    pub const KIND: &'static str = "pane";

    pub fn new(ctx: &Context, kind: &'static str) -> Pane {
        Pane {
            kind: Rc::new(RefCell::new(kind)),
            id: Rc::new(RefCell::new(ctx.hat.create_element_id(kind))),
            attributes: Rc::new(RefCell::new(HashMap::new())),
            rec_evs_focused: Rc::new(RefCell::new(ReceivableEvents::default())),
            rec_evs_always: Rc::new(RefCell::new(ReceivableEvents::default())),
            focused: Rc::new(RefCell::new(false)),
            parent: Rc::new(RefCell::new(None)),
            hooks: Rc::new(RefCell::new(HashMap::new())),
            content: Rc::new(RefCell::new(DrawChs2D::default())),
            is_content_dirty: Rc::new(RefCell::new(true)),
            last_size: Rc::new(RefCell::new(ctx.size)),
            last_visible_region: Rc::new(RefCell::new(ctx.visible_region)),
            default_ch: Rc::new(RefCell::new(DrawCh::default())),
            content_view_offset_x: Rc::new(RefCell::new(0)),
            content_view_offset_y: Rc::new(RefCell::new(0)),
            loc: Rc::new(RefCell::new(DynLocationSet::full())),
            visible: Rc::new(RefCell::new(true)),
            overflow: Rc::new(RefCell::new(false)),
        }
    }

    pub fn with_kind(self, kind: &'static str) -> Pane {
        *self.kind.borrow_mut() = kind;
        self
    }

    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, x: D, y: D2) -> Pane {
        self.set_at(x.into(), y.into());
        self
    }

    pub fn set_at(&self, x: DynVal, y: DynVal) {
        self.loc.borrow_mut().l.set_at(x, y);
    }

    pub fn set_kind(&self, kind: &'static str) {
        *self.kind.borrow_mut() = kind;
    }

    pub fn with_overflow(self) -> Pane {
        *self.overflow.borrow_mut() = true;
        self
    }

    pub fn set_overflow(&self) {
        *self.overflow.borrow_mut() = true;
    }

    pub fn with_focused(self, focused: bool) -> Pane {
        self.set_focused(focused);
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

    pub fn set_start_x<D: Into<DynVal>>(&self, x: D) {
        self.loc.borrow_mut().l.set_start_x(x.into());
    }

    pub fn set_start_y<D: Into<DynVal>>(&self, y: D) {
        self.loc.borrow_mut().l.set_start_y(y.into());
    }

    pub fn set_end_x<D: Into<DynVal>>(&self, x: D) {
        self.loc.borrow_mut().l.set_end_x(x.into());
    }

    pub fn set_end_y<D: Into<DynVal>>(&self, y: D) {
        self.loc.borrow_mut().l.set_end_y(y.into());
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

    pub fn with_dyn_height<D: Into<DynVal>>(self, h: D) -> Pane {
        self.loc.borrow_mut().l.set_dyn_height(h.into());
        self
    }

    pub fn with_dyn_width<D: Into<DynVal>>(self, w: D) -> Pane {
        self.loc.borrow_mut().l.set_dyn_width(w.into());
        self
    }

    pub fn set_dyn_height<D: Into<DynVal>>(&self, h: D) {
        self.loc.borrow_mut().l.set_dyn_height(h.into());
    }

    pub fn set_dyn_width<D: Into<DynVal>>(&self, w: D) {
        self.loc.borrow_mut().l.set_dyn_width(w.into());
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
        self.set_content(content);
        self
    }

    pub fn set_content_from_string<S: Into<String>>(&self, s: S) {
        self.set_content(DrawChs2D::from_string(s.into(), self.get_style()));
    }

    /// sets content from string
    pub fn set_content_from_string_with_style(&self, ctx: &Context, s: &str, sty: Style) {
        *self.is_content_dirty.borrow_mut() = true;
        let lines = s.split('\n');
        let mut rs: Vec<Vec<char>> = Vec::new();

        let mut width = ctx.size.width as usize;
        let mut height = ctx.size.height as usize;
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
        *self.is_content_dirty.borrow_mut() = true;
    }

    /// set the content, if its different from the current content. If the
    /// content is the same then the dirty flag is not set. NOTE that checking
    /// if the content is the same is an expensive comparison so this function should
    /// not be used if the content is known to be different.
    pub fn set_content_if_diff(&self, content: DrawChs2D) {
        if *self.content.borrow() != content {
            self.set_content(content);
        }
    }

    /// The pane's Content need not be the dimensions provided within
    /// the Location, however the Content will simply be cut off if it exceeds
    /// any dimension of the Location. If the Content is less than the dimensions
    /// of the Location all extra characters will be filled with the DefaultCh.
    /// The location of where to begin drawing from within the Content can be
    /// offset using content_view_offset_x and content_view_offset_y
    pub fn set_content(&self, content: DrawChs2D) {
        *self.content.borrow_mut() = content;
        *self.is_content_dirty.borrow_mut() = true;
    }

    pub fn get_content(&self) -> Ref<DrawChs2D> {
        self.content.borrow()
    }

    pub fn get_content_mut(&self) -> RefMut<DrawChs2D> {
        *self.is_content_dirty.borrow_mut() = true;
        self.content.borrow_mut()
    }

    pub fn content_width(&self) -> usize {
        self.content.borrow().width()
    }

    pub fn content_height(&self) -> usize {
        self.content.borrow().height()
    }

    pub fn content_size(&self) -> Size {
        Size::new(self.content_width() as u16, self.content_height() as u16)
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
        self.is_content_dirty.replace(true);
        self.default_ch.borrow_mut().style = style;
    }

    pub fn get_style(&self) -> Style {
        self.default_ch.borrow().style.clone()
    }

    pub fn with_bg(self, bg: Color) -> Pane {
        self.is_content_dirty.replace(true);
        self.default_ch.borrow_mut().style.set_bg(bg);
        self
    }

    pub fn set_bg(&self, bg: Color) {
        self.is_content_dirty.replace(true);
        self.default_ch.borrow_mut().style.set_bg(bg);
    }

    pub fn with_fg(self, fg: Color) -> Pane {
        self.is_content_dirty.replace(true);
        self.default_ch.borrow_mut().style.set_fg(fg);
        self
    }

    pub fn set_fg(&self, fg: Color) {
        self.is_content_dirty.replace(true);
        self.default_ch.borrow_mut().style.set_fg(fg);
    }

    pub fn set_default_ch(&self, ch: DrawCh) {
        self.is_content_dirty.replace(true);
        *self.default_ch.borrow_mut() = ch;
    }

    pub fn with_transparent(self) -> Self {
        self.set_transparent();
        self
    }

    pub fn set_transparent(&self) {
        let ch = DrawCh::transparent();
        self.set_default_ch(ch);
    }

    pub fn with_focused_receivable_events(self, evs: ReceivableEvents) -> Pane {
        *self.rec_evs_focused.borrow_mut() = evs;
        self
    }

    pub fn set_focused_receivable_events(&self, evs: ReceivableEvents) {
        *self.rec_evs_focused.borrow_mut() = evs;
    }

    pub fn with_always_receivable_events(self, evs: ReceivableEvents) -> Pane {
        *self.rec_evs_always.borrow_mut() = evs;
        self
    }

    pub fn set_always_receivable_events(&self, evs: ReceivableEvents) {
        *self.rec_evs_always.borrow_mut() = evs;
    }

    // -----------------------

    /// NOTE this name was chosen to distinguish itself from propagate_responses_upward
    pub fn send_responses_upward(&self, ctx: &Context, resps: EventResponses) {
        if let Some(parent) = self.parent.borrow().as_ref() {
            let parent_ctx = ctx.must_get_parent_context();
            parent.propagate_responses_upward(parent_ctx, &self.id(), resps);
        }
    }

    pub fn has_parent(&self) -> bool {
        self.parent.borrow().is_some()
    }

    /// correct_offsets_to_view_position changes the content offsets within the
    /// pane in order to bring the given view position into view.
    pub fn correct_offsets_to_view_position(&self, ctx: &Context, x: usize, y: usize) {
        let view_offset_y = *self.content_view_offset_y.borrow();
        let view_offset_x = *self.content_view_offset_x.borrow();
        let height = ctx.size.height as usize;
        let width = ctx.size.width as usize;

        // set y offset if cursor out of bounds
        if y >= view_offset_y + height {
            self.set_content_y_offset(ctx, y - height + 1);
        } else if y < view_offset_y {
            self.set_content_y_offset(ctx, y);
        }

        // correct the offset if the offset is now showing lines that don't exist in
        // the content
        if view_offset_y + height > self.content_height() {
            self.set_content_y_offset(ctx, height);
        }

        // set x offset if cursor out of bounds
        if x >= view_offset_x + width {
            self.set_content_x_offset(ctx, x - width + 1);
        } else if x < view_offset_x {
            self.set_content_x_offset(ctx, x);
        }

        // correct the offset if the offset is now showing characters to the right
        // which don't exist in the content.
        if view_offset_x + width > self.content_width() {
            self.set_content_x_offset(ctx, self.content_width());
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

    fn can_receive(&self, ev: &Event) -> bool {
        (*self.focused.borrow() && self.rec_evs_focused.borrow().contains_match(ev))
            || self.rec_evs_always.borrow().contains_match(ev)
    }

    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>> {
        if *self.focused.borrow() {
            vec![self.rec_evs_focused.clone(), self.rec_evs_always.clone()]
        } else {
            vec![self.rec_evs_always.clone()]
        }
    }

    ///                                                       (captured, resp          )
    fn receive_event_inner(&self, _ctx: &Context, _ev: Event) -> (bool, EventResponses) {
        (false, EventResponses::default())
    }

    /// ChangePriority returns a priority change request to its parent organizer so
    /// as to update the priority of all commands registered to this element.
    /// The element iterates through its registered cmds/evCombos, and returns a
    /// priority change request for each one.
    fn set_focused(&self, focused: bool) {
        *self.focused.borrow_mut() = focused;
    }

    fn get_focused(&self) -> bool {
        *self.focused.borrow()
    }

    /// Drawing compiles all of the DrawChPos necessary to draw this element
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        if !force_update
            && !*self.is_content_dirty.borrow()
            && *self.last_size.borrow() == ctx.size
            && *self.last_visible_region.borrow() == ctx.visible_region
        {
            return Vec::with_capacity(0);
        }

        self.is_content_dirty.replace(false);
        self.last_size.replace(ctx.size);
        self.last_visible_region.replace(ctx.visible_region);

        let mut chs = vec![];

        let (xmin, xmax, ymin, ymax) = if let Some(vis_region) = ctx.visible_region {
            (
                // take the intersection of the visibile region and the elements region
                (vis_region.start_x as usize).max(0),
                (vis_region.end_x as usize).min(ctx.size.width as usize),
                (vis_region.start_y as usize).max(0),
                (vis_region.end_y as usize).min(ctx.size.height as usize),
            )
        } else {
            (0, ctx.size.width as usize, 0, ctx.size.height as usize)
        };

        let view_offset_y = *self.content_view_offset_y.borrow();
        let view_offset_x = *self.content_view_offset_x.borrow();
        let default_ch = self.default_ch.borrow().clone();
        //let content_height = self.content.borrow().height();
        //let content_width = self.content.borrow().width();

        // convert the Content to DrawChPos
        for y in ymin..ymax {
            for x in xmin..xmax {
                let offset_y = y + view_offset_y;
                let offset_x = x + view_offset_x;

                // if the offset isn't pushing all the content out of view,
                // assign the next ch to be the one at the offset in the Content
                // matrix
                let ch_out = self
                    .content
                    .borrow()
                    .0
                    .get(offset_y)
                    .and_then(|row| row.get(offset_x))
                    .unwrap_or(&default_ch)
                    .clone();

                // convert the DrawCh to a DrawChPos
                chs.push(DrawChPos::new(ch_out, x as u16, y as u16))
            }
        }
        DrawUpdate::update(chs).into()
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
    fn get_ref_cell_overflow(&self) -> Rc<RefCell<bool>> {
        self.overflow.clone()
    }

    fn set_content_x_offset(&self, ctx: &Context, x: usize) {
        let content_width = self.content.borrow().width();
        let view_width = ctx.size.width as usize;
        let x = if x > content_width.saturating_sub(view_width) {
            content_width.saturating_sub(view_width)
        } else {
            x
        };
        *self.content_view_offset_x.borrow_mut() = x;
    }

    fn set_content_y_offset(&self, ctx: &Context, y: usize) {
        let content_height = self.content.borrow().height();
        let view_height = ctx.size.height as usize;
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
    fn get_content_width(&self, _: &Context) -> usize {
        self.content.borrow().width()
    }
    fn get_content_height(&self, _: &Context) -> usize {
        self.content.borrow().height()
    }
}
