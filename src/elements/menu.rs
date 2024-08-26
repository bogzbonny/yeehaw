use {
    crate::{
        element::ExtraLocationsRequest, element::ReceivableEventChanges, Context, DrawCh,
        DrawChPos, Element, ElementID, Event, EventResponse, EventResponses, Location, LocationSet,
        ParentPane, Priority, RgbColour, SortingHat, StandardPane, Style, UpwardPropagator, ZIndex,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::collections::HashMap,
    std::{cell::RefCell, rc::Rc},
};

// TODO add :command hints on the right hand side for command menu items
//      kind of like mac hotkey hints
// TODO add keyboard interation
// TODO multiline menu items

#[derive(Clone)]
pub struct MenuBar {
    pane: ParentPane,
    horizontal_bar: Rc<RefCell<bool>>, // is the bar horizontal or vertical
    menu_items: Rc<RefCell<HashMap<ElementID, MenuItem>>>,
    menu_items_order: Rc<RefCell<Vec<MenuItem>>>, // order of each menu item
    primary_hover_open: Rc<RefCell<bool>>,        // hover open for first level of menu items
    primary_has_show_arrow: Rc<RefCell<bool>>, // whether or not primary menu items show the expand arrow
    primary_open_dir: Rc<RefCell<OpenDirection>>, // used only for the first level of menu items
    secondary_open_dir: Rc<RefCell<OpenDirection>>, // used for all other levels of menu items
    menu_style: Rc<RefCell<MenuStyle>>,
    self_destruct_on_external_click: Rc<RefCell<bool>>,
}

impl MenuBar {
    const KIND: &'static str = "menu_bar";
    const Z_INDEX: ZIndex = -100; // very front

    pub fn top_bar(hat: &SortingHat) -> Self {
        MenuBar {
            pane: ParentPane::new(hat, MenuBar::KIND),
            horizontal_bar: Rc::new(RefCell::new(true)),
            menu_items: Rc::new(RefCell::new(HashMap::new())),
            menu_items_order: Rc::new(RefCell::new(vec![])),
            primary_hover_open: Rc::new(RefCell::new(false)),
            primary_has_show_arrow: Rc::new(RefCell::new(false)),
            primary_open_dir: Rc::new(RefCell::new(OpenDirection::Down)),
            secondary_open_dir: Rc::new(RefCell::new(OpenDirection::Right)),
            menu_style: Rc::new(RefCell::new(MenuStyle::default())),
            self_destruct_on_external_click: Rc::new(RefCell::new(false)),
        }
    }

    pub fn right_click_menu(hat: &SortingHat) -> Self {
        MenuBar {
            pane: ParentPane::new(hat, MenuBar::KIND),
            horizontal_bar: Rc::new(RefCell::new(false)),
            menu_items: Rc::new(RefCell::new(HashMap::new())),
            menu_items_order: Rc::new(RefCell::new(vec![])),
            primary_hover_open: Rc::new(RefCell::new(true)),
            primary_has_show_arrow: Rc::new(RefCell::new(true)),
            primary_open_dir: Rc::new(RefCell::new(OpenDirection::Right)),
            secondary_open_dir: Rc::new(RefCell::new(OpenDirection::Right)),
            menu_style: Rc::new(RefCell::new(MenuStyle::default())),
            self_destruct_on_external_click: Rc::new(RefCell::new(true)),
        }
    }

    pub fn with_menu_style(self, style: MenuStyle) -> Self {
        *self.menu_style.borrow_mut() = style;
        self
    }

    // unselectable item as decoration
    pub fn add_decor(&self, hat: &SortingHat, menu_path: String) {
        let mp = MenuPath(menu_path);
        self.ensure_folders(hat, mp.clone());
        let item = MenuItem::new(hat).with_unselectable();
        self.add_item_inner(item);
    }

    pub fn add_item(
        &self, hat: &SortingHat, menu_path: String,
        click_fn: Option<Box<dyn FnMut(Context) -> EventResponses>>,
    ) {
        let mp = MenuPath(menu_path);
        self.ensure_folders(hat, mp.clone());
        let item = MenuItem::new(hat).with_click_fn(click_fn);
        self.add_item_inner(item);
    }

    // ensure or create all folders leading to the final menu path
    pub fn ensure_folders(&self, hat: &SortingHat, menu_path: MenuPath) {
        let folders = menu_path.folders();
        for i in 0..folders.len() {
            let folder_path = folders[..i].join("/");
            if !self.contains_menu_item(MenuPath(folder_path.clone())) {
                let path = MenuPath(folder_path);
                let item = MenuItem::new_folder(hat, path);
                self.add_item_inner(item);
            }
        }
    }

    // the furthest location of the primary menu element
    pub fn max_primary_x(&self) -> i32 {
        let mut max_x = 0;
        for item in self.menu_items_order.borrow().iter() {
            if item.is_primary() {
                let loc = self
                    .pane
                    .eo
                    .borrow()
                    .must_get_locations(item.id())
                    .l
                    .clone();
                if loc.end_x > max_x {
                    max_x = loc.end_x;
                }
            }
        }
        max_x
    }
    pub fn max_primary_y(&self) -> i32 {
        let mut max_y = 0;
        for item in self.menu_items_order.borrow().iter() {
            if item.is_primary() {
                let loc = self
                    .pane
                    .eo
                    .borrow()
                    .must_get_locations(item.id())
                    .l
                    .clone();
                if loc.end_y > max_y {
                    max_y = loc.end_y;
                }
            }
        }
        max_y
    }

    fn add_item_inner(&self, item: MenuItem) {
        self.menu_items.borrow_mut().insert(item.id(), item.clone());
        self.menu_items_order.borrow_mut().push(item.clone());
        let is_primary = item.is_primary();
        let (loc, vis) = if is_primary {
            let item_width = item.min_width(
                &self.menu_style.borrow(),
                *self.primary_has_show_arrow.borrow(),
            ) as i32;
            let loc = if *self.horizontal_bar.borrow() {
                let x = self.max_primary_x();
                Location::new(x, x + item_width, 0, 1)
            } else {
                let y = self.max_primary_y();
                Location::new(0, item_width, y + 1, y + 1)
            };
            let ls = LocationSet::new(loc, vec![], Self::Z_INDEX);
            (ls, true)
        } else {
            (LocationSet::default(), false)
        };
        self.pane
            .eo
            .borrow_mut()
            .add_element(Rc::new(RefCell::new(item)), None, loc, vis);

        // correct widths if non-horizontal if this was a new primary being added
        if is_primary && !*self.horizontal_bar.borrow() {
            // get all the primary menu items
            let mut primary_items = vec![];
            for it in self.menu_items_order.borrow().iter() {
                if it.is_primary() {
                    primary_items.push(it.clone());
                }
            }

            // get the longest text
            let m_sty = self.menu_style.borrow();
            let primary_arrow = *self.primary_has_show_arrow.borrow();
            let max_width = primary_items
                .iter()
                .map(|it| it.min_width(&m_sty, primary_arrow))
                .max()
                .unwrap_or(0);

            // adjust all the widths in the element organizer
            for it in primary_items {
                let mut loc = self.pane.eo.borrow().must_get_locations(it.id()).l.clone();
                loc.set_width(max_width);
                let mut eo = self.pane.eo.borrow_mut();
                (*eo).update_el_primary_location(it.id(), loc);
            }
        }
    }

    pub fn contains_menu_item(&self, path: MenuPath) -> bool {
        let mi = self.menu_items_order.borrow();
        for item in mi.iter() {
            if *item.path.borrow() == path {
                return true;
            }
        }
        false
    }

    pub fn receive_mouse_event(
        &self, ctx: &Context, ev: crossterm::event::MouseEvent,
    ) -> (bool, EventResponses) {
        //let (click_x, click_y) = (ev.column, ev.row);
        let Some((el_id, mut resps)) = self.pane.eo.borrow_mut().mouse_event_process(&ev) else {
            return (true, EventResponses::default());
        };

        // get the menu item
        let menu_items = self.menu_items.borrow();
        let Some(item) = menu_items.get(&el_id) else {
            return (true, resps);
        };

        if *item.is_folder.borrow() {
            let (open_dir, hover_open) = if item.is_primary() {
                (
                    *self.primary_open_dir.borrow(),
                    *self.primary_hover_open.borrow(),
                )
            } else {
                (*self.secondary_open_dir.borrow(), true)
            };

            if !hover_open
                && !matches!(
                    ev.kind,
                    MouseEventKind::Up(_) | MouseEventKind::Down(_) | MouseEventKind::Drag(_)
                )
            {
                return (true, resps);
            }

            self.expand_folder(ctx, item, open_dir);

            // update extra locations for parent eo.Locations
            resps.push(
                EventResponse::default()
                    .with_extra_locations(ExtraLocationsRequest::new(self.extra_locations())),
            );
        }
        (true, resps)
    }

    pub fn receive_external_mouse_event(
        &self, _ctx: &Context, _ev: crossterm::event::MouseEvent,
    ) -> (bool, EventResponses) {
        // close all non-primary menus
        let mut eo = self.pane.eo.borrow_mut();
        let menu_items = self.menu_items.borrow();
        for (id, item) in menu_items.iter() {
            if !item.is_primary() {
                (*eo).update_el_visibility(id.clone(), false);
            }
        }

        let resp = if *self.self_destruct_on_external_click.borrow() {
            EventResponse::default().with_destruct()
        } else {
            EventResponse::default()
        };

        (true, resp.into())
    }

    pub fn extra_locations(&self) -> Vec<Location> {
        let mut locs = vec![];
        for (_, loc) in self.pane.eo.borrow().locations.iter() {
            locs.push(loc.l.clone());
        }
        locs
    }

    // expands all the sub-items of the provided item
    pub fn expand_folder(&self, _ctx: &Context, item: &MenuItem, dir: OpenDirection) {
        // get the immediate sub items of item
        let mut sub_items = vec![];
        let item_mp = (*item.path.borrow()).clone();
        for it in self.menu_items_order.borrow().iter() {
            if item_mp.is_immediate_parent_of(&it.path.borrow()) {
                sub_items.push(it.clone());
            }
        }

        // get the longest text
        let m_sty = self.menu_style.borrow();
        let primary_arrow = *self.primary_has_show_arrow.borrow();
        let max_width = sub_items
            .iter()
            .map(|it| it.min_width(&m_sty, primary_arrow))
            .max()
            .unwrap_or(0);

        // set all the locations in the element organizer

        // TODO adjust the open direction if there isn't enough space.
        let mut loc = self
            .pane
            .eo
            .borrow()
            .must_get_locations(item.id())
            .l
            .clone();
        let item_width = loc.get_size().width;
        for it in sub_items {
            // adjust for the next location
            match dir {
                OpenDirection::Up => {
                    loc.adjust_location_by(0, -1);
                }
                OpenDirection::Down => {
                    loc.adjust_location_by(0, 1);
                }
                OpenDirection::Left => {
                    loc.adjust_location_by(-(max_width as i32), 0);
                }
                OpenDirection::Right => {
                    loc.adjust_location_by(item_width as i32, 0);
                }
            };
            loc.set_width(max_width);
            let mut eo = self.pane.eo.borrow_mut();
            (*eo).update_el_primary_location(it.id(), loc.clone());
            (*eo).update_el_visibility(it.id(), true);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct MenuStyle {
    folder_arrow: String, // also including any left padding between the arrow and the menu item
    left_padding: usize,
    right_padding: usize,
    unselected_style: Style,
    selected_style: Style,
    disabled_style: Style,
}

impl Default for MenuStyle {
    fn default() -> Self {
        MenuStyle {
            folder_arrow: " ❯".to_string(),
            left_padding: 1,
            right_padding: 1,
            unselected_style: Style::default(),
            selected_style: Style::default().with_bg(RgbColour::BLUE),
            disabled_style: Style::default().with_bg(RgbColour::GREY13),
        }
    }
}

// direction which menu items prefer to open. If there is not enough space
// in the preferred direction, the menu will open in the opposite direction
#[derive(Clone, Copy)]
pub enum OpenDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone)]
pub struct MenuItem {
    pane: StandardPane,
    path: Rc<RefCell<MenuPath>>, // name displayed in the menu
    selectable: Rc<RefCell<bool>>,
    is_selected: Rc<RefCell<bool>>, // is the item currently selected
    is_folder: Rc<RefCell<bool>>,   // is the item a folder
    #[allow(clippy::type_complexity)]
    click_fn: Rc<RefCell<Option<Box<dyn FnMut(Context) -> EventResponses>>>>,
}

impl MenuItem {
    pub const KIND: &'static str = "menu_item";

    pub fn new(hat: &SortingHat) -> Self {
        MenuItem {
            pane: StandardPane::new(hat, MenuItem::KIND),
            path: Rc::new(RefCell::new(MenuPath("".to_string()))),
            selectable: Rc::new(RefCell::new(true)),
            is_selected: Rc::new(RefCell::new(false)),
            is_folder: Rc::new(RefCell::new(false)),
            click_fn: Rc::new(RefCell::new(None)),
        }
    }

    pub fn new_folder(hat: &SortingHat, path: MenuPath) -> Self {
        let item = MenuItem::new(hat);
        *item.is_folder.borrow_mut() = true;
        *item.path.borrow_mut() = path;
        item
    }

    pub fn with_click_fn(
        self, click_fn: Option<Box<dyn FnMut(Context) -> EventResponses>>,
    ) -> Self {
        *self.click_fn.borrow_mut() = click_fn;
        self
    }

    pub fn with_unselectable(self) -> Self {
        *self.selectable.borrow_mut() = false;
        self
    }

    pub fn with_selectable(self) -> Self {
        *self.selectable.borrow_mut() = true;
        self
    }

    pub fn select(&self) {
        *self.is_selected.borrow_mut() = true;
    }

    pub fn unselect(&self) {
        *self.is_selected.borrow_mut() = false;
    }

    pub fn is_primary(&self) -> bool {
        self.path.borrow().is_root()
    }

    // if the primary menu items are folders, they have an arrow if primary_show_arrow=true
    pub fn min_width(&self, sty: &MenuStyle, primary_show_arrow: bool) -> usize {
        let folder_len = if *self.is_folder.borrow()
            && (!self.is_primary() || (self.is_primary() && primary_show_arrow))
        {
            sty.folder_arrow.chars().count()
        } else {
            0
        };
        self.path.borrow().name().chars().count()
            + sty.left_padding
            + sty.right_padding
            + folder_len
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

impl Element for MenuBar {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.pane.receive_event(ctx, ev.clone());
        match ev {
            Event::Mouse(me) => self.receive_mouse_event(ctx, me),
            Event::ExternalMouse(me) => self.receive_external_mouse_event(ctx, me),
            _ => (false, EventResponses::default()),
        }
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        if !ctx.visible {
            return vec![];
        }

        let menu_style_bz = match serde_json::to_vec(&*self.menu_style.borrow()) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                return vec![];
            }
        };

        let mut out = vec![];

        // draw each menu item
        let eo = self.pane.eo.borrow();
        for (id, el) in eo.elements.iter() {
            // offset pos to location
            let loc = eo.must_get_locations(id.to_string());

            let s = loc.l.get_size();
            let c = Context::new(s, eo.visibility[id]).with_metadata(menu_style_bz.clone());
            let dcps = el.borrow().drawing(&c);

            for mut dcp in dcps {
                dcp.adjust_by_location(&loc.l);
                out.push(dcp);
            }
        }
        out
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.pane.pane.set_upward_propagator(up)
    }
}

impl Element for MenuItem {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.pane.receive_event(ctx, ev.clone());
        match ev {
            Event::Mouse(me) => {
                self.select();
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    if let Some(ref mut click_fn) = *self.click_fn.borrow_mut() {
                        return (true, click_fn(ctx.clone()));
                    }
                }
                (true, EventResponses::default())
            }
            Event::ExternalMouse(_) => {
                self.unselect();
                (true, EventResponses::default())
            }
            _ => (true, EventResponses::default()),
        }
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        if !ctx.visible {
            return vec![];
        }

        let Some(ref md) = ctx.metadata else {
            return vec![];
        };

        let m_sty: MenuStyle = match serde_json::from_slice(md) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                return vec![];
            }
        };

        let sty = match (*self.is_selected.borrow(), *self.selectable.borrow()) {
            (true, _) => m_sty.selected_style,
            (false, true) => m_sty.unselected_style,
            (false, false) => m_sty.disabled_style,
        };

        let (mut x, mut out) = MenuItem::draw_padding(m_sty.left_padding, 0, sty, vec![]);

        // draw name
        let name = self.path.borrow().name().to_string();
        let name_chs = DrawChPos::new_from_string(name, x as u16, 0, sty);
        x += name_chs.len();
        out.extend(name_chs);

        let arrow_text =
            if *self.is_folder.borrow() { m_sty.folder_arrow.clone() } else { "".to_string() };

        // add filler space
        while x < ctx.s.width as usize - m_sty.right_padding - arrow_text.chars().count() {
            let dc = DrawCh::new(' ', false, sty);
            let dcp = DrawChPos::new(dc, x as u16, 0);
            out.push(dcp);
            x += 1;
        }

        // draw folder arrow
        let arrow_chs = DrawChPos::new_from_string(arrow_text, x as u16, 0, sty);
        x += arrow_chs.len();

        // add right padding
        let (_, out) = MenuItem::draw_padding(m_sty.right_padding, x, sty, out);
        out
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.pane.set_upward_propagator(up)
    }
}

// MenuPath is a path of menu items within a menu tree.
// For example:
//
//	"nwmod/cool_stuff/blaze"
//
// represents a menu item which lives in the top-level menu "nwmod" within the
// sub-menu "cool_stuff" with the name "blaze".
#[derive(Clone, PartialEq)]
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

    // check if other is the the same as the parent
    // but then includes one more item
    pub fn is_immediate_parent_of(&self, other: &MenuPath) -> bool {
        let parent_path1 = self.0.clone();
        let parent_path2 = other.0.clone();
        let split1: Vec<&str> = self.0.split('/').collect();
        let split2: Vec<&str> = other.0.split('/').collect();
        parent_path2.starts_with(&parent_path1) && split2.len() == split1.len() + 1
    }
}
