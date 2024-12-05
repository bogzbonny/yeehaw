use {
    crate::{
        Color, Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Parent, ParentPane, Priority, ReceivableEventChanges, SelfReceivableEvents,
        Style, VerticalStack,
    },
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
    pub els: Rc<RefCell<Vec<Box<dyn Element>>>>,
    /// the tab names
    pub names: Rc<RefCell<Vec<String>>>,

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

impl TabsTop {
    const KIND: &'static str = "tabs_top";

    #[allow(clippy::type_complexity)]
    pub fn new(ctx: &Context, els: Rc<RefCell<Vec<Box<dyn Element>>>>, names: Vec<String>) -> Self {
        let tt = Self {
            pane: ParentPane::new(ctx, Self::KIND),
            els,
            names: Rc::new(RefCell::new(names)),
            selected: Rc::new(RefCell::new(None)),
            tab_prefix: Rc::new(RefCell::new(String::new())),
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
        for name in self.names.borrow().iter() {
            let mut full_name = prefix.clone();
            full_name.push_str(name);
            full_name.push_str(&suffix);
            names.push(full_name);
        }
        names
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
                                return self.pane.receive_event(ctx, ev);
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
        for (i, name_) in self.names.borrow().iter().enumerate() {
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
    pub els: Rc<RefCell<Vec<Box<dyn Element>>>>,
}

impl Tabs {
    const KIND: &'static str = "tabs";

    pub fn new(ctx: &Context) -> Self {
        let tabs_top = TabsTop::new(ctx, Rc::new(RefCell::new(Vec::new())), Vec::new());
        let pane = VerticalStack::new(ctx);
        pane.pane.pane.set_kind(Self::KIND);
        pane.push(Box::new(tabs_top.clone()));
        Self {
            pane,
            tabs_top,
            els: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// add an element to the end of the stack resizing the other elements
    /// in order to fit the new element
    pub fn push<S: Into<String>>(&self, el: Box<dyn Element>, name: S) {
        Self::sanitize_el_location(&*el);
        self.els.borrow_mut().push(el.clone());
        self.tabs_top.names.borrow_mut().push(name.into());
    }

    pub fn insert<S: Into<String>>(&self, idx: usize, el: Box<dyn Element>, name: S) {
        Self::sanitize_el_location(&*el);
        self.els.borrow_mut().insert(idx, el.clone());
        self.tabs_top.names.borrow_mut().insert(idx, name.into());
    }

    pub fn remove(&self, idx: usize) {
        self.els.borrow_mut().remove(idx);
    }

    pub fn clear(&self) {
        self.els.borrow_mut().clear();
    }

    fn sanitize_el_location(el: &dyn Element) {
        let mut loc = el.get_dyn_location_set().clone();
        loc.set_start_x(0.0.into()); // 0
        loc.set_end_x(1.0.into()); // 100%
        loc.set_start_y(0.0.into()); // 0
        loc.set_end_y(1.0.into()); // 100%
        el.set_dyn_location_set(loc); // set loc without triggering hooks
    }

    pub fn set_tab_view_pane(&self, idx: Option<usize>) {
        // the second element (1) is the tab view pane
        if let Some(el) = self.pane.get(1) {
            el.set_visible(false);
            self.pane.remove(1);
        }
        if let Some(idx) = idx {
            if let Some(el) = self.els.borrow().get(idx) {
                el.set_visible(true);
                self.pane.push(el.clone());
            }
        }
    }

    pub fn select(&self, idx: usize) {
        *self.tabs_top.selected.borrow_mut() = Some(idx);
        self.set_tab_view_pane(Some(idx));
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Tabs {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let start_selected = *self.tabs_top.selected.borrow();
        let out = self.pane.receive_event(ctx, ev.clone());
        let end_selected = *self.tabs_top.selected.borrow();
        if start_selected != end_selected {
            self.set_tab_view_pane(end_selected);
        }
        out
    }
}
