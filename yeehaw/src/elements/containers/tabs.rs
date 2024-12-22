use {
    crate::*,
    crossterm::event::{MouseButton, MouseEventKind},
};

/// TODO dragging tabs
/// TODO allow for buttons beside the tabs
/// TODO tab commands

/// The upper tabs pane
#[derive(Clone)]
pub struct TabsTop {
    pub pane: ParentPane,
    #[allow(clippy::type_complexity)]

    /// All the elements, their names, and their open functions
    //                      (ElementID, name,   on_open_fn)
    pub els: Rc<RefCell<Vec<(ElementID, String, OnOpenFn)>>>,

    /// the selected tab
    pub selected: Rc<RefCell<Option<usize>>>,
    /// the prefix for the tab names
    pub tab_prefix: Rc<RefCell<String>>,
    /// the suffix for the tab names
    pub tab_suffix: Rc<RefCell<String>>,
    /// the style for the highlighted tab
    pub highlight_style: Rc<RefCell<Style>>,
    /// the style for the normal tabs
    pub normal_style: Rc<RefCell<Style>>,

    pub is_dirty: Rc<RefCell<bool>>,
    pub cached_drawing: Rc<RefCell<Vec<DrawChPos>>>,
}

pub type OnOpenFn = Option<Box<dyn FnOnce()>>;

impl TabsTop {
    const KIND: &'static str = "tabs_top";

    #[allow(clippy::type_complexity)]
    pub fn new(ctx: &Context, els: Rc<RefCell<Vec<(ElementID, String, OnOpenFn)>>>) -> Self {
        let tt = Self {
            pane: ParentPane::new(ctx, Self::KIND),
            els,
            selected: Rc::new(RefCell::new(None)),
            tab_prefix: Rc::new(RefCell::new(" ".to_string())),
            tab_suffix: Rc::new(RefCell::new(" ".to_string())),
            highlight_style: Rc::new(RefCell::new(
                Style::default_const()
                    .with_bg(Color::LIGHT_BLUE)
                    .with_fg(Color::BLACK),
            )),
            normal_style: Rc::new(RefCell::new(
                Style::default_const()
                    .with_bg(Color::LIGHT_YELLOW)
                    .with_fg(Color::BLACK),
            )),
            is_dirty: Rc::new(RefCell::new(true)),
            cached_drawing: Rc::new(RefCell::new(Vec::new())),
        };

        // set the height/width of the tabs top
        tt.pane.pane.set_dyn_height(DynVal::new_fixed(1));
        tt.pane.pane.set_dyn_width(DynVal::FULL);
        tt
    }

    /// get the names with the prefix/suffixes
    pub fn get_full_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let prefix = self.tab_prefix.borrow().clone();
        let suffix = self.tab_suffix.borrow().clone();
        for (_, name, _) in self.els.borrow().iter() {
            let mut full_name = prefix.clone();
            full_name.push_str(name);
            full_name.push_str(&suffix);
            names.push(full_name);
        }
        names
    }

    pub fn push<S: Into<String>>(&self, el: ElementID, name: S, on_open_fn: OnOpenFn) {
        self.els.borrow_mut().push((el, name.into(), on_open_fn));
        self.is_dirty.replace(true);
    }

    pub fn insert<S: Into<String>>(&self, idx: usize, el: ElementID, name: S) {
        self.els.borrow_mut().insert(idx, (el, name.into(), None));
        self.is_dirty.replace(true);
    }

    pub fn remove(&self, idx: usize) -> ElementID {
        let (el_id, _, _) = self.els.borrow_mut().remove(idx);
        self.is_dirty.replace(true);
        el_id
    }

    pub fn clear(&self) {
        self.is_dirty.replace(true);
        self.els.borrow_mut().clear();
    }

    pub fn get_selected_id(&self) -> Option<ElementID> {
        let i = *self.selected.borrow();
        if let Some(i) = i {
            self.els.borrow().get(i).map(|(id, _, _)| id.clone())
        } else {
            None
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for TabsTop {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        #[allow(clippy::single_match)]
        match ev {
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    let x = me.column as usize;
                    let y = me.row as usize;
                    if y == 0 {
                        let mut pos = 0usize;
                        for (i, name) in self.get_full_names().iter().enumerate() {
                            let name_len = name.chars().count();
                            if x >= pos && x < pos + name_len {
                                *self.selected.borrow_mut() = Some(i);
                                let (_, resps) = self.pane.receive_event(ctx, ev);
                                self.is_dirty.replace(true);
                                return (true, resps);
                            }
                            pos += name_len;
                        }
                    }
                }
            }
            _ => {}
        }
        self.pane.receive_event(ctx, ev)
    }
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        // set the names of the tabs
        let mut upds = self.pane.drawing(ctx, force_update);
        if !force_update && !*self.is_dirty.borrow() {
            return upds;
        }
        let mut chs = Vec::new();
        let mut pos = 0usize;
        for (i, name_) in self
            .els
            .borrow()
            .iter()
            .map(|(_, name, _)| name)
            .enumerate()
        {
            let mut name = self.tab_prefix.borrow().clone();
            name.push_str(name_);
            name.push_str(&self.tab_suffix.borrow());

            let style = if let Some(sel) = *self.selected.borrow() {
                if i == sel {
                    self.highlight_style.borrow().clone()
                } else {
                    self.normal_style.borrow().clone()
                }
            } else {
                self.normal_style.borrow().clone()
            };
            let name_len = name.chars().count();
            let name_chs = DrawChPos::new_from_string(name, pos as u16, 0, style);
            pos += name_len;
            chs.extend(name_chs);
        }
        self.is_dirty.replace(false);
        let upd = DrawUpdate::update(chs);
        upds.push(upd);
        upds
    }
}

#[derive(Clone)]
pub struct Tabs {
    pub pane: VerticalStack,
    pub tabs_top: TabsTop,
    #[allow(clippy::type_complexity)]
    pub lower: ParentPane,
}

impl Tabs {
    const KIND: &'static str = "tabs";
    const LOWER_KIND: &'static str = "tabs_lower";

    pub fn new(ctx: &Context) -> Self {
        let tabs_top = TabsTop::new(ctx, Rc::new(RefCell::new(Vec::new())));
        let pane = VerticalStack::new(ctx);
        pane.pane.pane.set_kind(Self::KIND);
        let lower = ParentPane::new(ctx, Self::LOWER_KIND);
        pane.push(Box::new(tabs_top.clone()));
        pane.push(Box::new(lower.clone()));
        Self {
            pane,
            tabs_top,
            lower,
        }
    }

    /// add an element to the end of the stack resizing the other elements
    /// in order to fit the new element
    pub fn push<S: Into<String>>(&self, el: Box<dyn Element>, name: S) {
        self.push_with_on_open_fn(el, name, None)
    }

    /// add an element to the end of the stack resizing the other elements
    /// in order to fit the new element
    pub fn push_with_on_open_fn<S: Into<String>>(
        &self, el: Box<dyn Element>, name: S, on_open_fn: OnOpenFn,
    ) {
        Self::sanitize_el_location(&*el);
        self.tabs_top.push(el.id(), name.into(), on_open_fn);
        self.lower.add_element(el.clone());
        let idx = self.tabs_top.els.borrow().len() - 1;
        if self.tabs_top.selected.borrow().is_none() {
            self.tabs_top.selected.replace(Some(idx));
            self.set_tab_view_pane(None, Some(idx));
        } else {
            el.set_visible(false);
            el.set_focused(false);
        }
    }

    pub fn insert<S: Into<String>>(&self, idx: usize, el: Box<dyn Element>, name: S) {
        Self::sanitize_el_location(&*el);

        let mut unfocus_old = false;
        if let Some(old_idx) = *self.tabs_top.selected.borrow() {
            if idx == old_idx {
                unfocus_old = true;
            }
        }
        if unfocus_old {
            if let Some(old_id) = self.tabs_top.get_selected_id() {
                if let Some(old_el) = self.lower.get_element(&old_id) {
                    old_el.set_visible(false);
                    old_el.set_focused(false);
                }
            }
        }

        self.tabs_top.insert(idx, el.id(), name.into());
        self.lower.add_element(el.clone());
        if self.tabs_top.selected.borrow().is_none() {
            self.tabs_top.selected.replace(Some(idx));
            self.set_tab_view_pane(None, Some(idx));
        } else if *self.tabs_top.selected.borrow() == Some(idx) {
            el.set_visible(true);
        } else {
            el.set_visible(false);
            el.set_focused(false);
        }
    }

    pub fn remove(&self, idx: usize) {
        let el_id = self.tabs_top.remove(idx);
        self.lower.remove_element(&el_id);
    }

    pub fn clear(&self) {
        self.tabs_top.clear();
        self.lower.clear_elements();
    }

    fn sanitize_el_location(el: &dyn Element) {
        let mut loc = el.get_dyn_location_set().clone();
        loc.set_start_x(0);
        loc.set_end_x(DynVal::FULL);
        loc.set_start_y(0);
        loc.set_end_y(DynVal::FULL);
        el.set_dyn_location_set(loc); // set loc without triggering hooks
    }

    pub fn set_tab_view_pane(&self, old_idx: Option<usize>, new_idx: Option<usize>) {
        if let Some(idx) = old_idx {
            if let Some((old_id, _, _)) = self.tabs_top.els.borrow().get(idx) {
                self.lower.eo.hide_element(old_id);
            }
        }

        if let Some(idx) = new_idx {
            if let Some((new_id, _, on_open_fn)) = self.tabs_top.els.borrow_mut().get_mut(idx) {
                self.lower.eo.unhide_element(new_id);
                if let Some(on_open_fn) = on_open_fn.take() {
                    on_open_fn();
                }
            }
        }
    }

    pub fn select(&self, idx: usize) {
        let start_selected = *self.tabs_top.selected.borrow();
        *self.tabs_top.selected.borrow_mut() = Some(idx);
        self.set_tab_view_pane(start_selected, Some(idx));
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Tabs {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let start_selected = *self.tabs_top.selected.borrow();
        let (captured, resps) = self.pane.receive_event(ctx, ev.clone());
        let end_selected = *self.tabs_top.selected.borrow();
        if start_selected != end_selected {
            self.set_tab_view_pane(start_selected, end_selected);
        }
        (captured, resps)
    }
}
