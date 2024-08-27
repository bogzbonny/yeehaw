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
    activated: Rc<RefCell<bool>>, // the bar must first be activated with a click before any expansion
    primary_has_show_arrow: Rc<RefCell<bool>>, // whether or not primary menu items show the expand arrow
    primary_open_dir: Rc<RefCell<OpenDirection>>, // used only for the first level of menu items
    secondary_open_dir: Rc<RefCell<OpenDirection>>, // used for all other levels of menu items
    menu_style: Rc<RefCell<MenuStyle>>,
    make_invisible_on_closedown: Rc<RefCell<bool>>, // useful for right click menu
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
            unselected_style: Style::default().with_fg(RgbColour::WHITE),
            selected_style: Style::default()
                .with_bg(RgbColour::BLUE)
                .with_fg(RgbColour::WHITE),
            disabled_style: Style::default()
                .with_bg(RgbColour::GREY13)
                .with_fg(RgbColour::WHITE),
        }
    }
}

// direction which menu items prefer to open. If there is not enough space
// in the preferred direction, the menu will open in the opposite direction
#[derive(Clone, Copy)]
pub enum OpenDirection {
    Up,
    Down,
    LeftThenDown,
    RightThenDown,
    LeftThenUp,
    RightThenUp,
}

impl MenuBar {
    const KIND: &'static str = "menu_bar";
    const Z_INDEX: ZIndex = -100; // very front
    const MENU_STYLE_MD_KEY: &'static str = "menu_style";

    pub fn top_menu_bar(hat: &SortingHat) -> Self {
        MenuBar {
            pane: ParentPane::new(hat, MenuBar::KIND),
            horizontal_bar: Rc::new(RefCell::new(true)),
            menu_items: Rc::new(RefCell::new(HashMap::new())),
            menu_items_order: Rc::new(RefCell::new(vec![])),
            activated: Rc::new(RefCell::new(false)),
            primary_has_show_arrow: Rc::new(RefCell::new(false)),
            primary_open_dir: Rc::new(RefCell::new(OpenDirection::Down)),
            secondary_open_dir: Rc::new(RefCell::new(OpenDirection::RightThenDown)),
            menu_style: Rc::new(RefCell::new(MenuStyle::default())),
            make_invisible_on_closedown: Rc::new(RefCell::new(false)),
        }
    }

    pub fn right_click_menu(hat: &SortingHat) -> Self {
        MenuBar {
            pane: ParentPane::new(hat, MenuBar::KIND),
            horizontal_bar: Rc::new(RefCell::new(false)),
            menu_items: Rc::new(RefCell::new(HashMap::new())),
            menu_items_order: Rc::new(RefCell::new(vec![])),
            activated: Rc::new(RefCell::new(true)), // right click menus are always activated
            primary_has_show_arrow: Rc::new(RefCell::new(true)),
            primary_open_dir: Rc::new(RefCell::new(OpenDirection::RightThenDown)),
            secondary_open_dir: Rc::new(RefCell::new(OpenDirection::RightThenDown)),
            menu_style: Rc::new(RefCell::new(MenuStyle::default())),
            make_invisible_on_closedown: Rc::new(RefCell::new(true)),
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
        let item = MenuItem::new(hat, mp).with_unselectable();
        self.add_item_inner(item);
    }

    pub fn add_item(
        &self, hat: &SortingHat, menu_path: String,
        click_fn: Option<Box<dyn FnMut(Context) -> EventResponses>>,
    ) {
        let mp = MenuPath(menu_path);
        self.ensure_folders(hat, mp.clone());
        let item = MenuItem::new(hat, mp).with_click_fn(click_fn);
        self.add_item_inner(item);
    }

    pub fn with_item(
        self, hat: &SortingHat, menu_path: String,
        click_fn: Option<Box<dyn FnMut(Context) -> EventResponses>>,
    ) -> Self {
        self.add_item(hat, menu_path, click_fn);
        self
    }

    pub fn set_items(&self, hat: &SortingHat, items: Vec<MenuItem>) {
        for item in items {
            self.ensure_folders(hat, item.path.borrow().clone());
            self.add_item_inner(item);
        }
    }

    // ensure or create all folders leading to the final menu path
    pub fn ensure_folders(&self, hat: &SortingHat, menu_path: MenuPath) {
        let folders = menu_path.folders();
        for i in 0..folders.len() {
            let folder_path = folders[..=i].join("/");
            if !self.contains_menu_item(MenuPath(folder_path.clone())) {
                let path = MenuPath(folder_path);
                let item = MenuItem::new_folder(hat, path);
                self.add_item_inner(item);
            }
        }
    }

    // the furthest location of the primary menu element
    pub fn max_primary_x(&self) -> Option<i32> {
        let mut max_x = None;
        for item in self.menu_items_order.borrow().iter() {
            if item.is_primary() {
                let loc = self.pane.eo.get_location(&item.id()).expect("missing el").l;
                if let Some(mx) = max_x {
                    if loc.end_x > mx {
                        max_x = Some(loc.end_x);
                    }
                } else {
                    max_x = Some(loc.end_x);
                }
            }
        }
        max_x
    }
    pub fn max_primary_y(&self) -> Option<i32> {
        let mut max_y = None;
        for item in self.menu_items_order.borrow().iter() {
            if item.is_primary() {
                let loc = self.pane.eo.get_location(&item.id()).expect("missing el").l;
                if let Some(my) = max_y {
                    if loc.end_y > my {
                        max_y = Some(loc.end_y);
                    }
                } else {
                    max_y = Some(loc.end_y);
                }
            }
        }
        max_y
    }

    fn add_item_inner(&self, item: MenuItem) {
        let is_primary = item.is_primary();
        if is_primary && !*self.primary_has_show_arrow.borrow() {
            *item.show_folder_arrow.borrow_mut() = false;
        }
        let (loc, vis) = if is_primary {
            let item_width = item.min_width(
                &self.menu_style.borrow(),
                *self.primary_has_show_arrow.borrow(),
            ) as i32;
            let loc = if *self.horizontal_bar.borrow() {
                let x = self.max_primary_x();
                let x = if let Some(x) = x { x + 1 } else { 0 };
                Location::new(x, x + item_width - 1, 0, 0)
            } else {
                let y = self.max_primary_y();
                let y = if let Some(y) = y { y + 1 } else { 0 };
                Location::new(0, item_width - 1, y, y)
            };
            let ls = LocationSet::new(loc, vec![], Self::Z_INDEX);
            (ls, true)
        } else {
            (LocationSet::default(), false)
        };
        self.pane
            .eo
            .add_element(Rc::new(RefCell::new(item.clone())), None, loc, vis);
        self.menu_items.borrow_mut().insert(item.id(), item.clone());
        self.menu_items_order.borrow_mut().push(item);

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
                let mut loc = self.pane.eo.get_location(&it.id()).expect("missing item").l;
                loc.set_width(max_width);
                self.pane.eo.update_el_primary_location(it.id(), loc);
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

    pub fn get_menu_item_from_path(&self, path: MenuPath) -> Option<MenuItem> {
        let mi = self.menu_items_order.borrow();
        for item in mi.iter() {
            if *item.path.borrow() == path {
                return Some(item.clone());
            }
        }
        None
    }

    pub fn receive_mouse_event(
        &self, ctx: &Context, ev: crossterm::event::MouseEvent,
    ) -> (bool, EventResponses) {
        // must check if bar is activated
        let clicked = matches!(
            ev.kind,
            MouseEventKind::Up(_) | MouseEventKind::Down(_) | MouseEventKind::Drag(_)
        );
        if clicked {
            *self.activated.borrow_mut() = true;
        }

        if !*self.activated.borrow() {
            return (true, EventResponses::default());
        }

        let mep = self.pane.eo.mouse_event_process(&ev);
        let Some((el_id, mut resps)) = mep else {
            if clicked {
                return self.closedown();
            }
            return (true, EventResponses::default());
        };

        // get the menu item
        let menu_items = self.menu_items.borrow();
        let Some(item) = menu_items.get(&el_id) else {
            return (true, resps);
        };

        self.collapse_non_primary();
        self.expand_up_to_item(ctx, item);

        if *item.is_folder.borrow() {
            let open_dir = if item.is_primary() {
                *self.primary_open_dir.borrow()
            } else {
                *self.secondary_open_dir.borrow()
            };

            self.expand_folder(ctx, item, open_dir);

            // update extra locations for parent eo.Locations
        }
        resps.push(
            EventResponse::default()
                .with_extra_locations(ExtraLocationsRequest::new(self.extra_locations())),
        );
        (true, resps)
    }

    pub fn receive_external_mouse_event(
        &self, _ctx: &Context, _ev: crossterm::event::MouseEvent,
    ) -> (bool, EventResponses) {
        self.closedown()
    }

    // closedown routine
    pub fn closedown(&self) -> (bool, EventResponses) {
        *self.activated.borrow_mut() = false;
        let make_invis = *self.make_invisible_on_closedown.borrow();

        let menu_items = self.menu_items.borrow();
        for (id, item) in menu_items.iter() {
            // close all non-primary menus
            if !item.is_primary() || make_invis {
                self.pane.eo.update_el_visibility(id.clone(), false);
            } else {
                item.unselect();
            }
        }

        // update extra locations for parent eo
        let resps: EventResponses = EventResponse::default()
            .with_extra_locations(ExtraLocationsRequest::new(self.extra_locations()))
            .into();
        (true, resps)
    }

    // useful for right click menu
    pub fn make_primary_visible(&self) -> EventResponses {
        let menu_items = self.menu_items.borrow();
        for (id, item) in menu_items.iter() {
            if item.is_primary() {
                self.pane.eo.update_el_visibility(id.clone(), true);
            }
        }

        // update extra locations for parent eo
        EventResponse::default()
            .with_extra_locations(ExtraLocationsRequest::new(self.extra_locations()))
            .into()
    }

    pub fn extra_locations(&self) -> Vec<Location> {
        let mut locs = vec![];
        for details in self.pane.eo.els.borrow().values() {
            locs.push(details.loc.l.clone());
        }
        locs
    }

    pub fn collapse_non_primary(&self) {
        let menu_items = self.menu_items.borrow();
        for (id, item) in menu_items.iter() {
            // close all non-primary menus
            if !item.is_primary() {
                self.pane.eo.update_el_visibility(id.clone(), false);
            }
        }
    }

    // expands all folders required to make the item visible
    pub fn expand_up_to_item(&self, ctx: &Context, item: &MenuItem) {
        let path = item.path.borrow();
        let folders = path.folders();
        for i in 0..folders.len() {
            let folder_path = folders[..=i].join("/");
            let folder_mp = MenuPath(folder_path);
            if let Some(item) = self.get_menu_item_from_path(folder_mp) {
                let open_dir = if item.is_primary() {
                    *self.primary_open_dir.borrow()
                } else {
                    *self.secondary_open_dir.borrow()
                };
                self.expand_folder(ctx, &item, open_dir);
            };
        }
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
        let mut loc = self.pane.eo.get_location(&item.id()).expect("missing el").l;
        let item_width = loc.get_size().width;
        for (i, it) in sub_items.iter().enumerate() {
            // adjust for the next location
            match dir {
                OpenDirection::Up => {
                    loc.adjust_location_by(0, -1);
                }
                OpenDirection::Down => {
                    loc.adjust_location_by(0, 1);
                }
                OpenDirection::LeftThenDown => {
                    if i == 0 {
                        loc.adjust_location_by(-(max_width as i32), 0);
                    } else {
                        loc.adjust_location_by(0, 1);
                    }
                }
                OpenDirection::RightThenDown => {
                    if i == 0 {
                        loc.adjust_location_by(item_width as i32, 0);
                    } else {
                        loc.adjust_location_by(0, 1);
                    }
                }
                OpenDirection::LeftThenUp => {
                    if i == 0 {
                        loc.adjust_location_by(-(max_width as i32), 0);
                    } else {
                        loc.adjust_location_by(0, -1);
                    }
                }
                OpenDirection::RightThenUp => {
                    if i == 0 {
                        loc.adjust_location_by(item_width as i32, 0);
                    } else {
                        loc.adjust_location_by(0, -1);
                    }
                }
            };
            loc.set_width(max_width);
            self.pane
                .eo
                .update_el_primary_location(it.id(), loc.clone());
            self.pane.eo.update_el_visibility(it.id(), true);
        }
    }
}

// ---------------------

#[derive(Clone)]
pub struct MenuItem {
    pane: StandardPane,
    path: Rc<RefCell<MenuPath>>, // name displayed in the menu
    selectable: Rc<RefCell<bool>>,
    is_selected: Rc<RefCell<bool>>, // is the item currently selected
    is_folder: Rc<RefCell<bool>>,   // is the item a folder
    show_folder_arrow: Rc<RefCell<bool>>, // show the folder arrow, false for primary horizontal bar
    #[allow(clippy::type_complexity)]
    click_fn: Rc<RefCell<Option<Box<dyn FnMut(Context) -> EventResponses>>>>,
}

impl MenuItem {
    pub const KIND: &'static str = "menu_item";

    pub fn new(hat: &SortingHat, path: MenuPath) -> Self {
        MenuItem {
            pane: StandardPane::new(hat, MenuItem::KIND),
            path: Rc::new(RefCell::new(path)),
            selectable: Rc::new(RefCell::new(true)),
            is_selected: Rc::new(RefCell::new(false)),
            is_folder: Rc::new(RefCell::new(false)),
            show_folder_arrow: Rc::new(RefCell::new(true)),
            click_fn: Rc::new(RefCell::new(None)),
        }
    }

    pub fn new_folder(hat: &SortingHat, path: MenuPath) -> Self {
        let item = MenuItem::new(hat, path);
        *item.is_folder.borrow_mut() = true;
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

        let mut out = self.pane.drawing(ctx);

        // draw each menu item
        for el_details in self.pane.eo.els.borrow().values() {
            // offset pos to location
            let s = el_details.loc.l.get_size();
            let c = Context::new(s, el_details.vis)
                .with_metadata(Self::MENU_STYLE_MD_KEY.to_string(), menu_style_bz.clone());
            let dcps = el_details.el.borrow().drawing(&c);

            for mut dcp in dcps {
                dcp.adjust_by_location(&el_details.loc.l);
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

        let Some(ref md) = ctx.get_metadata(MenuBar::MENU_STYLE_MD_KEY) else {
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

        let arrow_text = if *self.is_folder.borrow()
            && (!self.is_primary() || (self.is_primary() && *self.show_folder_arrow.borrow()))
        {
            m_sty.folder_arrow
        } else {
            "".to_string()
        };

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
        out.extend(arrow_chs);

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

// -----------------------------------------------------------------------
// MenuPath is a path of menu items within a menu tree.
// For example:
//
//	"nwmod/cool_stuff/blaze"
//
// represents a menu item which lives in the top-level menu "nwmod" within the
// sub-menu "cool_stuff" with the name "blaze".
#[derive(Clone, PartialEq, Debug)]
pub struct MenuPath(pub String);

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
