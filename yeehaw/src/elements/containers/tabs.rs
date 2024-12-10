use {
    crate::*,
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
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
    }

    pub fn insert<S: Into<String>>(&self, idx: usize, el: ElementID, name: S) {
        self.els.borrow_mut().insert(idx, (el, name.into(), None));
    }

    pub fn remove(&self, idx: usize) -> ElementID {
        let (el_id, _, _) = self.els.borrow_mut().remove(idx);
        el_id
    }

    pub fn clear(&self) {
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
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        // set the names of the tabs
        let mut chs = self.pane.drawing(ctx);
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
        chs
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
        let _ = pane.push(Box::new(tabs_top.clone()));
        let _ = pane.push(Box::new(lower.clone()));
        Self {
            pane,
            tabs_top,
            lower,
        }
    }

    #[must_use]
    /// add an element to the end of the stack resizing the other elements
    /// in order to fit the new element
    pub fn push<S: Into<String>>(&self, el: Box<dyn Element>, name: S) -> EventResponses {
        self.push_with_on_open_fn(el, name, None)
    }

    #[must_use]
    /// add an element to the end of the stack resizing the other elements
    /// in order to fit the new element
    pub fn push_with_on_open_fn<S: Into<String>>(
        &self, el: Box<dyn Element>, name: S, on_open_fn: OnOpenFn,
    ) -> EventResponses {
        Self::sanitize_el_location(&*el);
        self.tabs_top.push(el.id(), name.into(), on_open_fn);
        let mut resps = EventResponses::default();
        let resp = self.lower.add_element(el.clone());
        resps.push(resp);
        let idx = self.tabs_top.els.borrow().len() - 1;
        if self.tabs_top.selected.borrow().is_none() {
            self.tabs_top.selected.replace(Some(idx));
            let resps_ = self.set_tab_view_pane(None, Some(idx));
            resps.extend(resps_);
        } else {
            el.set_visible(false);
            let _ = el.change_priority(Priority::Unfocused);
        }
        resps
    }

    #[must_use]
    pub fn insert<S: Into<String>>(
        &self, idx: usize, el: Box<dyn Element>, name: S,
    ) -> EventResponses {
        Self::sanitize_el_location(&*el);

        let mut resps = EventResponses::default();
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
                    let resp_ = old_el.change_priority(Priority::Unfocused);
                    resps.push(resp_.into());
                }
            }
        }

        self.tabs_top.insert(idx, el.id(), name.into());
        let resp = self.lower.add_element(el.clone());
        resps.push(resp);
        if self.tabs_top.selected.borrow().is_none() {
            self.tabs_top.selected.replace(Some(idx));
            let resps_ = self.set_tab_view_pane(None, Some(idx));
            resps.extend(resps_);
        } else if *self.tabs_top.selected.borrow() == Some(idx) {
            el.set_visible(true);
        } else {
            el.set_visible(false);
            let _ = el.change_priority(Priority::Unfocused);
        }
        resps
    }

    #[must_use]
    pub fn remove(&self, idx: usize) -> EventResponse {
        let el_id = self.tabs_top.remove(idx);
        self.lower.remove_element(&el_id)
    }

    pub fn clear(&self) -> EventResponse {
        self.tabs_top.clear();
        self.lower.clear_elements()
    }

    fn sanitize_el_location(el: &dyn Element) {
        let mut loc = el.get_dyn_location_set().clone();
        loc.set_start_x(0);
        loc.set_end_x(DynVal::FULL);
        loc.set_start_y(0);
        loc.set_end_y(DynVal::FULL);
        el.set_dyn_location_set(loc); // set loc without triggering hooks
    }

    #[must_use]
    pub fn set_tab_view_pane(
        &self, old_idx: Option<usize>, new_idx: Option<usize>,
    ) -> EventResponses {
        let mut resps = EventResponses::default();
        if let Some(idx) = old_idx {
            if let Some((old_id, _, _)) = self.tabs_top.els.borrow().get(idx) {
                let resp = self.lower.eo.hide_element(old_id);
                resps.push(resp);
            }
        }

        if let Some(idx) = new_idx {
            if let Some((new_id, _, on_open_fn)) = self.tabs_top.els.borrow_mut().get_mut(idx) {
                let resp = self.lower.eo.unhide_element(new_id);
                resps.push(resp);
                if let Some(on_open_fn) = on_open_fn.take() {
                    on_open_fn();
                }
            }
        }
        resps
    }

    #[must_use]
    pub fn select(&self, idx: usize) -> EventResponses {
        let start_selected = *self.tabs_top.selected.borrow();
        *self.tabs_top.selected.borrow_mut() = Some(idx);
        self.set_tab_view_pane(start_selected, Some(idx))
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Tabs {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let start_selected = *self.tabs_top.selected.borrow();
        let (captured, mut resps) = self.pane.receive_event(ctx, ev.clone());
        let end_selected = *self.tabs_top.selected.borrow();
        if start_selected != end_selected {
            let resps_ = self.set_tab_view_pane(start_selected, end_selected);
            resps.extend(resps_);
        }
        (captured, resps)
    }
}
