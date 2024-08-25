use {
    crate::{
        element::ReceivableEventChanges, Context, DrawCh, DrawChPos, Element, ElementID, Event,
        EventResponses, Priority, RgbColour, SortingHat, StandardPane, Style, UpwardPropagator,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{
        ops::{Deref, DerefMut},
        {cell::RefCell, rc::Rc},
    },
};

#[derive(Clone)]
pub struct MenuItem {
    base: StandardPane,
    menu_path: Rc<RefCell<MenuPath>>, // name displayed in the menu
    selectible: Rc<RefCell<bool>>,    // false for lines/decor
    is_selected: Rc<RefCell<bool>>,   // is the item currently selected
    left_padding: Rc<RefCell<usize>>,
    right_padding: Rc<RefCell<usize>>,

    pub clicked_fn: Rc<RefCell<dyn FnMut(Context) -> EventResponses>>,
    unselected_style: Rc<RefCell<Style>>,
    selected_style: Rc<RefCell<Style>>,
    disabled_style: Rc<RefCell<Style>>,
}

impl MenuItem {
    pub const KIND: &'static str = "menu_item";

    // default padding for menu items (both for left and right)
    pub const DEFAULT_PADDING: usize = 1;

    pub fn new(hat: &SortingHat, menu_path: String) -> Self {
        let menu_path = MenuPath(menu_path);

        Self {
            base: StandardPane::new(hat, Self::KIND),
            menu_path: Rc::new(RefCell::new(menu_path)),
            selectible: Rc::new(RefCell::new(true)),
            is_selected: Rc::new(RefCell::new(false)),
            left_padding: Rc::new(RefCell::new(Self::DEFAULT_PADDING)),
            right_padding: Rc::new(RefCell::new(Self::DEFAULT_PADDING)),
            clicked_fn: Rc::new(RefCell::new(|_| EventResponses::default())),
            unselected_style: Rc::new(RefCell::new(Style::default())),
            selected_style: Rc::new(RefCell::new(Style::default().with_bg(RgbColour::BLUE))),
            disabled_style: Rc::new(RefCell::new(Style::default().with_bg(RgbColour::GREY13))),
        }
    }

    pub fn with_unselectible(self) -> Self {
        *self.selectible.borrow_mut() = false;
        self
    }

    pub fn with_selectible(self) -> Self {
        *self.selectible.borrow_mut() = true;
        self
    }

    pub fn with_clicked_fn(
        mut self, clicked_fn: Box<dyn FnMut(Context) -> EventResponses>,
    ) -> Self {
        self.clicked_fn = Rc::new(RefCell::new(clicked_fn));
        self
    }

    pub fn with_unselected_style(self, style: Style) -> Self {
        *self.unselected_style.borrow_mut() = style;
        self
    }

    pub fn with_selected_style(self, style: Style) -> Self {
        *self.selected_style.borrow_mut() = style;
        self
    }

    pub fn with_disabled_style(self, style: Style) -> Self {
        *self.disabled_style.borrow_mut() = style;
        self
    }

    pub fn with_styles(self, unselected: Style, selected: Style, disabled: Style) -> Self {
        *self.unselected_style.borrow_mut() = unselected;
        *self.selected_style.borrow_mut() = selected;
        *self.disabled_style.borrow_mut() = disabled;
        self
    }

    pub fn with_padding(self, left: usize, right: usize) -> Self {
        *self.left_padding.borrow_mut() = left;
        *self.right_padding.borrow_mut() = right;
        self
    }

    pub fn with_selected(self) -> Self {
        *self.is_selected.borrow_mut() = true;
        self
    }

    pub fn with_unselected(self) -> Self {
        *self.is_selected.borrow_mut() = false;
        self
    }

    pub fn select(&self) {
        *self.is_selected.borrow_mut() = true;
    }

    pub fn unselect(&self) {
        *self.is_selected.borrow_mut() = false;
    }

    // is_root checks returns true if menu item has no parents
    pub fn is_root(&self) -> bool {
        !self.menu_path.borrow().0.contains('/')
    }

    // is_parent_of checks if the given menu path is a parent of this menu path
    pub fn is_parent_of(&self, mi: &MenuItem) -> bool {
        self.menu_path.borrow().is_parent_of(&mi.menu_path.borrow())
    }

    // is_sibling_of checks if the given menu item is a sibling of this menu item
    pub fn is_sibling_of(&self, mi: &MenuItem) -> bool {
        self.menu_path
            .borrow()
            .is_sibling_of(&mi.menu_path.borrow())
    }

    // total_padding returns the total padding for both sides
    pub fn total_padding(&self) -> usize {
        *self.left_padding.borrow() + *self.right_padding.borrow()
    }

    // len_if_root returns the length of this item with padding if it were a root item
    pub fn len_if_root(&self) -> usize {
        self.menu_path.borrow().name().len() + self.total_padding()
    }

    // draw_padding draws DrawChPos's for padding and return updated DrawChPos array
    // along with updated x position
    pub fn draw_padding(
        padding: usize, mut x: usize, style: Style, dcps: Vec<DrawChPos>,
    ) -> (usize, Vec<DrawChPos>) {
        let mut dcps = dcps;
        for _ in 0..padding {
            let dc = DrawCh::new(' ', false, style);
            dcps.push(DrawChPos::new(dc, x as u16, 0));
            x += 1;
        }
        (x, dcps)
    }
}

impl Element for MenuItem {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }

    /*
    // ReceiveMouseEvent handles mouse events sent to this element
    func (mi *MenuItem) ReceiveMouseEvent(ctx yh.Context, ev *tcell.EventMouse) yh.EventResponse {

        if ev.Buttons() == tcell.Button1 && mi.fn != nil {
            return mi.fn(ctx)
        }

        return yh.EventResponse{}
    }

    func (m *MenuItem) ExternalMouseEvent(ctx yh.Context, ev *tcell.EventMouse) yh.EventResponse {
        m.isSelected = false
        return yh.EventResponse{}
    }
    */

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    return (true, self.clicked_fn.borrow_mut()(ctx.clone()));
                }
            }
            Event::ExternalMouse(_) => {
                self.unselect();
                return (true, EventResponses::default());
            }
            _ => {}
        }
        (false, EventResponses::default())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        if !ctx.visible {
            return vec![];
        }

        // TODO rewrite this! Each menu item is it's own element. they should
        // just all line up to look like coherent menus

        let sty = match (*self.is_selected.borrow(), *self.selectible.borrow()) {
            (true, _) => *self.selected_style.borrow(),
            (false, true) => *self.unselected_style.borrow(),
            (false, false) => *self.disabled_style.borrow(),
        };

        let (mut x, mut out) = MenuItem::draw_padding(*self.left_padding.borrow(), 0, sty, vec![]);

        // Draw sub items
        for ch in self.menu_path.borrow().name().chars() {
            let dc = DrawCh::new(ch, false, sty);
            let dcp = DrawChPos::new(dc, x as u16, 0);
            out.push(dcp);
            x += 1;
        }

        // add filler space
        while x < ctx.s.width as usize - 1 {
            let dc = DrawCh::new(' ', false, sty);
            let dcp = DrawChPos::new(dc, x as u16, 0);
            out.push(dcp);
            x += 1;
        }

        // add right padding
        let (_, out) = MenuItem::draw_padding(*self.right_padding.borrow(), x, sty, out);

        out
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.base.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.base.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.base.set_upward_propagator(up)
    }
}

pub struct MenuItems(Vec<MenuItem>);

impl Deref for MenuItems {
    type Target = Vec<MenuItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MenuItems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MenuItems {
    // LargestMenuItemNameAndPaddingLen returns the length of the longest menu item name
    pub fn largest_menu_item_name_and_padding_len(&self) -> usize {
        let mut longest = 0;
        for item in &self.0 {
            // TODO check for sub items in sub items because then a sub menu
            // indicator has to be added to the length
            // TODO also check for commands and add a command indicator to the
            // length
            // TODO add padding in here too
            let name_len = item.menu_path.borrow().name().len();
            let l = name_len + item.total_padding();
            if l > longest {
                longest = l;
            }
        }
        longest
    }
}

// MenuPath is a path of menu items within a menu tree.
// For example:
//
//	"nwmod/cool_stuff/blaze"
//
// represents a menu item which lives in the top-level menu "nwmod" within the
// sub-menu "cool_stuff" with the name "blaze".
pub struct MenuPath(String);

impl MenuPath {
    // Name returns the name of the menu item at the end of the menu path
    pub fn name(&self) -> &str {
        let split: Vec<&str> = self.0.split('/').collect();
        if !split.is_empty() {
            split[split.len() - 1]
        } else {
            ""
        }
    }

    // Folders returns the folders of the menu path
    pub fn folders(&self) -> Vec<&str> {
        let split: Vec<&str> = self.0.split('/').collect();
        if !split.is_empty() {
            split[..split.len() - 1].to_vec()
        } else {
            vec![]
        }
    }

    pub fn is_root(&self) -> bool {
        !self.0.contains('/')
    }

    pub fn is_parent_of(&self, other: &MenuPath) -> bool {
        let parent_path1 = self.0.clone();
        let parent_path2 = other.0.clone();
        parent_path2.starts_with(&parent_path1)
    }

    pub fn is_sibling_of(&self, other: &MenuPath) -> bool {
        let parent_path1 = self.0.clone();
        let parent_path2 = other.0.clone();
        let parent_path1 = parent_path1.split('/').collect::<Vec<&str>>();
        let parent_path2 = parent_path2.split('/').collect::<Vec<&str>>();
        if parent_path1.len() == 1 || parent_path2.len() == 1 {
            return false;
        }
        let parent_path1 = parent_path1[..parent_path1.len() - 1].join("/");
        let parent_path2 = parent_path2[..parent_path2.len() - 1].join("/");
        parent_path1 == parent_path2
    }
}
