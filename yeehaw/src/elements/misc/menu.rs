use {
    crate::{Keyboard as KB, *},
    crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEventKind},
    rayon::prelude::*,
    std::collections::HashMap,
};

// TODO refactor to cache instead of just returning updates every drawing
// TODO add :command hints on the right hand side for command menu items
//      kind of like mac hotkey hints
// TODO add keyboard interation
// TODO multiline menu items

#[derive(Clone)]
pub struct MenuBar {
    pub pane: ParentPane,
    /// is the bar horizontal or vertical
    horizontal_bar: Rc<RefCell<bool>>,
    menu_items: Rc<RefCell<HashMap<ElementID, MenuItem>>>,
    /// order of each menu item
    menu_items_order: Rc<RefCell<Vec<MenuItem>>>,
    /// the bar must first be activated with a click before any expansion
    activated: Rc<RefCell<bool>>,
    /// whether or not primary menu items show the expand arrow
    primary_has_show_arrow: Rc<RefCell<bool>>,
    /// used only for the first level of menu items
    primary_open_dir: Rc<RefCell<OpenDirection>>,
    /// used for all other levels of menu items
    secondary_open_dir: Rc<RefCell<OpenDirection>>,
    menu_style: Rc<RefCell<MenuStyle>>,
    /// useful for right click menu
    make_invisible_on_closedown: Rc<RefCell<bool>>,
    /// close the menubar on a click of a primary menu item
    close_on_primary_click: Rc<RefCell<bool>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct MenuStyle {
    folder_arrow: String,
    /// also including any left padding between the arrow and the menu item
    left_padding: usize,
    right_padding: usize,
    unselected_style: Style,
    selected_style: Style,
    disabled_style: Style,
}

impl Default for MenuStyle {
    fn default() -> Self {
        let selected_style = Style::default_const()
            .with_bg(Color::BLUE)
            .with_fg(Color::WHITE);
        let unselected_style = Style::default_const()
            .with_bg(Color::GREY10)
            .with_fg(Color::WHITE);
        let disabled_style = Style::default_const()
            .with_bg(Color::GREY20)
            .with_fg(Color::WHITE);
        MenuStyle {
            folder_arrow: " ❯".to_string(),
            left_padding: 1,
            right_padding: 1,
            unselected_style,
            selected_style,
            disabled_style,
        }
    }
}

/// direction which menu items prefer to open. If there is not enough space
/// in the preferred direction, the menu will open in the opposite direction
#[derive(Clone, Copy)]
pub enum OpenDirection {
    Up,
    Down,
    LeftThenDown,
    RightThenDown,
    LeftThenUp,
    RightThenUp,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl MenuBar {
    const KIND: &'static str = "menu_bar";
    const Z_INDEX: ZIndex = 100;
    /// very frontward
    const MENU_STYLE_MD_KEY: &'static str = "menu_style";

    pub fn default_receivable_events() -> ReceivableEvents {
        ReceivableEvents(vec![
            (KB::KEY_ESC.into()),
            (KB::KEY_ENTER.into()),
            (KB::KEY_DOWN.into()),
            (KB::KEY_UP.into()),
            (KB::KEY_LEFT.into()),
            (KB::KEY_RIGHT.into()),
        ])
    }

    pub fn top_menu_bar(ctx: &Context) -> Self {
        let menu_sty = MenuStyle::default();
        let pane = ParentPane::new(ctx, MenuBar::KIND)
            .with_z(MenuBar::Z_INDEX)
            .with_style(menu_sty.unselected_style.clone())
            .with_overflow()
            .with_focused_receivable_events(Self::default_receivable_events());

        MenuBar {
            pane,
            horizontal_bar: Rc::new(RefCell::new(true)),
            menu_items: Rc::new(RefCell::new(HashMap::new())),
            menu_items_order: Rc::new(RefCell::new(vec![])),
            activated: Rc::new(RefCell::new(false)),
            primary_has_show_arrow: Rc::new(RefCell::new(false)),
            primary_open_dir: Rc::new(RefCell::new(OpenDirection::Down)),
            secondary_open_dir: Rc::new(RefCell::new(OpenDirection::RightThenDown)),
            menu_style: Rc::new(RefCell::new(menu_sty)),
            make_invisible_on_closedown: Rc::new(RefCell::new(false)),
            close_on_primary_click: Rc::new(RefCell::new(true)),
        }
        .with_dyn_height(1)
        .with_dyn_width(DynVal::FULL)
    }

    pub fn right_click_menu(ctx: &Context) -> Self {
        let pane = ParentPane::new(ctx, MenuBar::KIND)
            .with_z(MenuBar::Z_INDEX)
            .with_overflow()
            .with_focused_receivable_events(Self::default_receivable_events());
        MenuBar {
            pane,
            horizontal_bar: Rc::new(RefCell::new(false)),
            menu_items: Rc::new(RefCell::new(HashMap::new())),
            menu_items_order: Rc::new(RefCell::new(vec![])),
            activated: Rc::new(RefCell::new(true)), // right click menus are always activated
            primary_has_show_arrow: Rc::new(RefCell::new(true)),
            primary_open_dir: Rc::new(RefCell::new(OpenDirection::RightThenDown)),
            secondary_open_dir: Rc::new(RefCell::new(OpenDirection::RightThenDown)),
            menu_style: Rc::new(RefCell::new(MenuStyle::default())),
            make_invisible_on_closedown: Rc::new(RefCell::new(true)),
            close_on_primary_click: Rc::new(RefCell::new(false)),
        }
    }

    pub fn with_menu_style(self, style: MenuStyle) -> Self {
        *self.menu_style.borrow_mut() = style;
        self
    }

    pub fn at<T: Into<DynVal>>(self, x: T, y: T) -> Self {
        self.pane.pane.set_at(x.into(), y.into());
        self
    }

    /// unselectable item as decoration
    pub fn add_decor(&self, ctx: &Context, menu_path: String) {
        let mp = MenuPath(menu_path);
        self.ensure_folders(ctx, mp.clone());
        let item = MenuItem::new(ctx, mp).with_unselectable();
        self.add_item_inner(item);
    }

    pub fn add_item(
        &self, ctx: &Context, menu_path: String,
        click_fn: Option<Box<dyn FnMut(Context) -> EventResponses>>,
    ) {
        let mp = MenuPath(menu_path);
        self.ensure_folders(ctx, mp.clone());
        let item = MenuItem::new(ctx, mp).with_fn(click_fn);
        self.add_item_inner(item);
    }

    pub fn with_item(
        self, ctx: &Context, menu_path: String,
        click_fn: Option<Box<dyn FnMut(Context) -> EventResponses>>,
    ) -> Self {
        self.add_item(ctx, menu_path, click_fn);
        self
    }

    pub fn set_items(&self, ctx: &Context, items: Vec<MenuItem>) {
        for item in items {
            self.ensure_folders(ctx, item.path.borrow().clone());
            self.add_item_inner(item);
        }
    }

    /// ensure or create all folders leading to the final menu path
    pub fn ensure_folders(&self, ctx: &Context, menu_path: MenuPath) {
        let folders = menu_path.folders();
        for i in 0..folders.len() {
            let folder_path = folders[..=i].join("/");
            if !self.contains_menu_item(MenuPath(folder_path.clone())) {
                let path = MenuPath(folder_path);
                let item = MenuItem::new_folder(ctx, path);
                self.add_item_inner(item);
            }
        }
    }

    /// the furthest location of the primary menu element
    pub fn max_primary_x(&self, dr: &DrawRegion) -> Option<i32> {
        let mut max_x = None;
        for item in self.menu_items_order.borrow().iter() {
            if item.is_primary() {
                let loc = self.pane.eo.get_location(&item.id()).expect("missing el").l;
                let end_x = loc.get_end_x(dr);
                if let Some(mx) = max_x {
                    if end_x > mx {
                        max_x = Some(end_x);
                    }
                } else {
                    max_x = Some(end_x);
                }
            }
        }
        max_x
    }
    pub fn max_primary_y(&self, dr: &DrawRegion) -> Option<i32> {
        let mut max_y = None;
        for item in self.menu_items_order.borrow().iter() {
            if item.is_primary() {
                let loc = self.pane.eo.get_location(&item.id()).expect("missing el").l;
                let end_y = loc.get_end_y(dr);
                if let Some(my) = max_y {
                    if end_y > my {
                        max_y = Some(end_y);
                    }
                } else {
                    max_y = Some(end_y);
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
        let dr = DrawRegion::default().with_size(*self.get_last_size());

        let (loc, vis) = if is_primary {
            let item_width = item.min_width(
                &self.menu_style.borrow(),
                *self.primary_has_show_arrow.borrow(),
            ) as i32;
            let loc = if *self.horizontal_bar.borrow() {
                let x = self.max_primary_x(&dr).unwrap_or(0); // returns max end_x which is exclusive (so don't +1)
                let x1 = DynVal::new_fixed(x);
                let x2 = DynVal::new_fixed(x + item_width);
                let y1 = DynVal::new_fixed(0);
                let y2 = DynVal::new_fixed(1);
                DynLocation::new(x1, x2, y1, y2)
            } else {
                let y = self.max_primary_y(&dr).unwrap_or(0); // returns max end_y which is exclusive (so don't +1)
                let y1 = DynVal::new_fixed(y);
                let y2 = DynVal::new_fixed(y + 1);
                let x1 = DynVal::new_fixed(0);
                let x2 = DynVal::new_fixed(item_width);
                DynLocation::new(x1, x2, y1, y2)
            };
            let ls = DynLocationSet::new(loc, vec![], Self::Z_INDEX);
            (ls, true)
        } else {
            (DynLocationSet::default().with_z(Self::Z_INDEX), false)
        };
        item.set_dyn_location_set(loc);
        item.set_visible(vis);

        // NOTE ignore the resps, as items should not have any resps
        // maybe fix one day
        self.pane.add_element(Box::new(item.clone()));

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
            let max_width = DynVal::new_fixed(max_width as i32);

            // adjust all the widths in the element organizer
            for it in primary_items {
                let mut loc = self.pane.eo.get_location(&it.id()).expect("missing item").l;
                loc.set_dyn_width(max_width.clone());
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

    pub fn receive_mouse_event(&self, ctx: &Context, ev: MouseEvent) -> (bool, EventResponses) {
        // must check if bar is activated
        let clicked = matches!(ev.kind, MouseEventKind::Down(_));
        let active_at_start = *self.activated.borrow();
        if !*self.activated.borrow() && clicked {
            *self.activated.borrow_mut() = true;
        }

        if !*self.activated.borrow() {
            return (true, EventResponses::default());
        }

        let mep = self
            .pane
            .eo
            .mouse_event_process(ctx, &ev, Box::new(self.pane.clone()));
        let (Some(el_id), resps) = mep else {
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

        // close the menu if there was a click on a non-folder item
        if matches!(ev.kind, MouseEventKind::Up(_))
            && !*item.is_folder.borrow()
            && *item.selectable.borrow()
        {
            return self.closedown();
        }

        // close if was a click on the primary bar
        let c = *self.close_on_primary_click.borrow();
        if c && active_at_start && clicked && item.is_primary() {
            return self.closedown();
        }

        // reopen everything to the item and its sub-folder
        self.collapse_non_primary();
        self.expand_up_to_item(item);

        if *item.is_folder.borrow() {
            let open_dir = if item.is_primary() {
                *self.primary_open_dir.borrow()
            } else {
                *self.secondary_open_dir.borrow()
            };

            self.expand_folder(item, open_dir);
        }
        self.update_extra_locations();
        (true, resps)
    }

    pub fn receive_external_mouse_event(
        &self, _ctx: &Context, ev: MouseEvent,
    ) -> (bool, EventResponses) {
        let clicked = matches!(
            ev.kind,
            MouseEventKind::Up(_) | MouseEventKind::Down(_) | MouseEventKind::Drag(_)
        );
        if clicked {
            return self.closedown();
        }
        (false, EventResponses::default())
    }

    /// closedown routine
    /// TODO cleanup, can remove EventResponses
    fn get_selected_item(&self) -> Option<MenuItem> {
        let menu_items = self.menu_items.borrow();
        for item in menu_items.values() {
            if *item.is_selected.borrow() {
                return Some(item.clone());
            }
        }
        None
    }

    pub fn closedown(&self) -> (bool, EventResponses) {
        *self.activated.borrow_mut() = false;
        let make_invis = *self.make_invisible_on_closedown.borrow();

        let menu_items = self.menu_items.borrow();
        for (_, item) in menu_items.iter() {
            // close all non-primary menus
            if !item.is_primary() || make_invis {
                item.set_visible(false);
            } else {
                item.unselect();
            }
        }

        // update extra locations for parent eo
        self.update_extra_locations();

        if make_invis {
            // make the actual menu bar element invisible in the parent eo
            self.set_visible(false);
        }
        (true, EventResponses::default())
    }

    /// useful for right click menu
    pub fn make_primary_visible(&self) {
        let menu_items = self.menu_items.borrow();
        for (_, item) in menu_items.iter() {
            if item.is_primary() {
                item.set_visible(true);
            }
        }

        // update extra locations for parent eo
        self.update_extra_locations();
    }

    /// useful for right click menu
    pub fn deselect_all(&self) {
        let menu_items = self.menu_items.borrow();
        for item in menu_items.values() {
            item.unselect();
        }
    }

    pub fn activate(&self) {
        *self.activated.borrow_mut() = true;

        // Select first primary item if none is selected
        if self.get_selected_item().is_none() {
            let menu_items = self.menu_items_order.borrow();
            if let Some(first_primary) = menu_items.iter().find(|item| item.is_primary()) {
                first_primary.select();
            }
        }
    }

    fn get_first_subitem(&self, parent_item: &MenuItem) -> Option<MenuItem> {
        let menu_items = self.menu_items_order.borrow();
        let parent_path = parent_item.path.borrow().clone();
        menu_items
            .iter()
            .find(|sub_item| parent_path.is_immediate_parent_of(&sub_item.path.borrow()))
            .cloned()
    }

    fn get_visible_siblings(&self, current_item: &MenuItem) -> Vec<MenuItem> {
        let current_path = current_item.path.borrow();
        let current_folders = current_path.folders();
        let menu_items = self.menu_items_order.borrow();
        menu_items
            .iter()
            .filter(|item| {
                item.get_visible() && {
                    let item_path = item.path.borrow();
                    let item_folders = item_path.folders();
                    current_folders == item_folders
                }
            })
            .cloned()
            .collect()
    }

    fn select_and_expand_item(&self, new_item: &MenuItem, old_item: Option<&MenuItem>) {
        if let Some(old_item) = old_item {
            old_item.unselect();
        }
        new_item.select();

        self.collapse_non_primary();
        self.expand_up_to_item(new_item);
        if *new_item.is_folder.borrow() {
            let open_dir = if new_item.is_primary() {
                *self.primary_open_dir.borrow()
            } else {
                *self.secondary_open_dir.borrow()
            };
            self.expand_folder(new_item, open_dir);
        }
        self.update_extra_locations();
    }

    fn handle_left_key(
        &self, current_item: Option<MenuItem>, is_primary: bool, is_horizontal: bool,
    ) -> (bool, EventResponses) {
        if is_horizontal && is_primary {
            self.select_prev_primary();
            return (true, EventResponses::default());
        }

        let Some(item) = current_item else {
            return (true, EventResponses::default());
        };

        if is_primary {
            return (true, EventResponses::default());
        }

        let current_path = item.path.borrow();
        let folders = current_path.folders();
        if folders.is_empty() {
            return (true, EventResponses::default());
        }

        let parent_path = folders.join("/");
        let Some(parent_item) = self.get_menu_item_from_path(MenuPath(parent_path)) else {
            return (true, EventResponses::default());
        };

        // Only return to primary items with left key in vertical menus
        if parent_item.is_primary() && is_horizontal {
            return (true, EventResponses::default());
        }

        self.select_and_expand_item(&parent_item, Some(&item));
        (true, EventResponses::default())
    }

    fn handle_right_key(
        &self, current_item: Option<MenuItem>, is_primary: bool, is_horizontal: bool,
    ) -> (bool, EventResponses) {
        if is_horizontal && is_primary {
            self.select_next_primary();
            return (true, EventResponses::default());
        }

        let Some(item) = current_item else {
            return (true, EventResponses::default());
        };

        if !*item.is_folder.borrow() {
            return (true, EventResponses::default());
        }

        self.expand_current_submenu();

        let Some(first_sub_item) = self.get_first_subitem(&item) else {
            return (true, EventResponses::default());
        };

        self.select_and_expand_item(&first_sub_item, Some(&item));
        (true, EventResponses::default())
    }

    fn handle_up_key(
        &self, current_item: Option<MenuItem>, is_primary: bool, is_horizontal: bool,
    ) -> (bool, EventResponses) {
        if is_primary {
            if !is_horizontal {
                self.select_prev_primary();
            }
            return (true, EventResponses::default());
        }

        let Some(current_item) = current_item else {
            return (true, EventResponses::default());
        };

        // Handle first sub-item in horizontal menus
        if is_horizontal {
            let current_path = current_item.path.borrow();
            let folders = current_path.folders();
            if !folders.is_empty() {
                let parent_path = folders.join("/");
                if let Some(parent_item) = self.get_menu_item_from_path(MenuPath(parent_path)) {
                    if parent_item.is_primary() {
                        let is_first_subitem = self
                            .get_first_subitem(&parent_item)
                            .map(|first_sub| first_sub.id() == current_item.id())
                            .unwrap_or(false);

                        if is_first_subitem {
                            self.select_and_expand_item(&parent_item, Some(&current_item));
                            return (true, EventResponses::default());
                        }
                    }
                }
            }
        }

        // Handle sibling navigation
        let visible_siblings = self.get_visible_siblings(&current_item);
        if visible_siblings.is_empty() {
            return (true, EventResponses::default());
        }

        let current_idx = visible_siblings
            .iter()
            .position(|item| item.id() == current_item.id())
            .unwrap_or(0);

        // Don't wrap around to the end
        if current_idx == 0 {
            return (true, EventResponses::default());
        }

        let new_item = visible_siblings[current_idx - 1].clone();
        self.select_and_expand_item(&new_item, Some(&current_item));
        (true, EventResponses::default())
    }

    fn handle_down_key(
        &self, current_item: Option<MenuItem>, is_primary: bool, is_horizontal: bool,
    ) -> (bool, EventResponses) {
        if is_primary {
            if is_horizontal {
                let Some(item) = current_item else {
                    return (true, EventResponses::default());
                };

                if !*item.is_folder.borrow() {
                    return (true, EventResponses::default());
                }

                self.expand_current_submenu();

                if let Some(first_sub_item) = self.get_first_subitem(&item) {
                    self.select_and_expand_item(&first_sub_item, Some(&item));
                }
            } else {
                self.select_next_primary();
            }
            return (true, EventResponses::default());
        }

        let Some(current_item) = current_item else {
            return (true, EventResponses::default());
        };

        let visible_siblings = self.get_visible_siblings(&current_item);
        if visible_siblings.is_empty() {
            return (true, EventResponses::default());
        }

        let current_idx = visible_siblings
            .iter()
            .position(|item| item.id() == current_item.id())
            .unwrap_or(0);

        // Don't wrap around to the beginning
        let new_idx = current_idx + 1;
        if new_idx >= visible_siblings.len() {
            return (true, EventResponses::default());
        }

        let new_item = visible_siblings[new_idx].clone();
        self.select_and_expand_item(&new_item, Some(&current_item));
        (true, EventResponses::default())
    }

    fn handle_enter_key(
        &self, current_item: Option<MenuItem>, is_primary: bool, is_horizontal: bool, ctx: &Context,
    ) -> (bool, EventResponses) {
        let Some(item) = current_item else {
            return (true, EventResponses::default());
        };

        if *item.is_folder.borrow() {
            self.expand_current_submenu();

            if is_horizontal && is_primary {
                if let Some(first_sub_item) = self.get_first_subitem(&item) {
                    self.select_and_expand_item(&first_sub_item, Some(&item));
                }
            }
            return (true, EventResponses::default());
        }

        if !*item.selectable.borrow() {
            return (true, EventResponses::default());
        }

        // For non-folder items, call their click function and close menu
        let resps = if let Some(ref mut click_fn) = *item.click_fn.borrow_mut() {
            click_fn(ctx.clone())
        } else {
            EventResponses::default()
        };

        // Deactivate and collapse everything
        *self.activated.borrow_mut() = false;
        self.collapse_non_primary();
        // Unselect all items
        let menu_items = self.menu_items.borrow();
        for item in menu_items.values() {
            item.unselect();
        }
        self.update_extra_locations();
        (true, resps)
    }

    pub fn receive_key_event(&self, ctx: &Context, ke: KeyEvent) -> (bool, EventResponses) {
        if !*self.activated.borrow() {
            return (true, EventResponses::default());
        }

        // Get current item info
        let current_item = self.get_selected_item();
        let is_primary = current_item.as_ref().map_or(true, |item| item.is_primary());
        let is_horizontal = *self.horizontal_bar.borrow();

        match ke.code {
            KeyCode::Left => self.handle_left_key(current_item, is_primary, is_horizontal),
            KeyCode::Right => self.handle_right_key(current_item, is_primary, is_horizontal),
            KeyCode::Up => self.handle_up_key(current_item, is_primary, is_horizontal),
            KeyCode::Down => self.handle_down_key(current_item, is_primary, is_horizontal),
            KeyCode::Enter => self.handle_enter_key(current_item, is_primary, is_horizontal, ctx),
            KeyCode::Esc => {
                *self.activated.borrow_mut() = false;
                self.collapse_non_primary();
                let menu_items = self.menu_items.borrow();
                for item in menu_items.values() {
                    item.unselect();
                }
                self.update_extra_locations();
                (true, EventResponses::default())
            }
            _ => (true, EventResponses::default()),
        }
    }

    fn select_prev_primary(&self) {
        let menu_items = self.menu_items_order.borrow();
        let primary_items: Vec<_> = menu_items.iter().filter(|item| item.is_primary()).collect();

        if primary_items.is_empty() {
            return;
        }

        let current_idx = if let Some(current_item) = self.get_selected_item() {
            current_item.unselect();
            primary_items
                .iter()
                .position(|item| item.id() == current_item.id())
                .unwrap_or(0)
        } else {
            0
        };

        let new_idx = if current_idx == 0 { primary_items.len() - 1 } else { current_idx - 1 };

        let new_item = primary_items[new_idx].clone();
        new_item.select();

        self.collapse_non_primary();
        if *new_item.is_folder.borrow() {
            self.expand_folder(&new_item, *self.primary_open_dir.borrow());
        }
        self.update_extra_locations();
    }

    fn select_next_primary(&self) {
        let menu_items = self.menu_items_order.borrow();
        let primary_items: Vec<_> = menu_items.iter().filter(|item| item.is_primary()).collect();

        if primary_items.is_empty() {
            return;
        }

        let current_idx = if let Some(current_item) = self.get_selected_item() {
            current_item.unselect();
            primary_items
                .iter()
                .position(|item| item.id() == current_item.id())
                .unwrap_or(0)
        } else {
            0
        };

        let new_idx = (current_idx + 1) % primary_items.len();
        let new_item = primary_items[new_idx].clone();
        new_item.select();

        self.collapse_non_primary();
        if *new_item.is_folder.borrow() {
            self.expand_folder(&new_item, *self.primary_open_dir.borrow());
        }
        self.update_extra_locations();
    }

    fn expand_current_submenu(&self) {
        if let Some(current_item) = self.get_selected_item() {
            if *current_item.is_folder.borrow() {
                let open_dir = if current_item.is_primary() {
                    *self.primary_open_dir.borrow()
                } else {
                    *self.secondary_open_dir.borrow()
                };
                self.expand_folder(&current_item, open_dir);
                self.update_extra_locations();
            }
        }
    }

    pub fn extra_locations(&self) -> Vec<DynLocation> {
        let bar_loc = self.get_dyn_location_set();
        let x = bar_loc.get_dyn_start_x();
        let y = bar_loc.get_dyn_start_y();

        let mut locs = vec![];
        for details in self.pane.eo.els.borrow().values() {
            if !*details.vis.borrow() {
                continue;
            }
            let mut item_loc = details.loc.borrow().l.clone();
            item_loc.adjust_location_by(x.clone(), y.clone());
            locs.push(item_loc);
        }
        locs
    }

    pub fn update_extra_locations(&self) {
        let extra = self.extra_locations();
        self.set_dyn_location_extra(extra);
    }

    pub fn collapse_non_primary(&self) {
        let menu_items = self.menu_items.borrow();
        for (_, item) in menu_items.iter() {
            // close all non-primary menus
            if !item.is_primary() {
                item.set_visible(false);
            }
        }
    }

    /// expands all folders required to make the item visible
    pub fn expand_up_to_item(&self, item: &MenuItem) {
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
                self.expand_folder(&item, open_dir);
            };
        }
    }

    /// expands all the sub-items of the provided item
    pub fn expand_folder(&self, item: &MenuItem, dir: OpenDirection) {
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
        let neg_max_width = DynVal::new_fixed(-(max_width as i32));
        let max_width = DynVal::new_fixed(max_width as i32);

        // set all the locations in the element organizer

        // TODO adjust the open direction if there isn't enough space.
        let mut loc = self.pane.eo.get_location(&item.id()).expect("missing el").l;
        let item_width = loc.get_dyn_width();
        for (i, it) in sub_items.iter().enumerate() {
            // adjust for the next location
            loc = loc.clone();
            match dir {
                OpenDirection::Up => {
                    loc.adjust_location_by_y(DynVal::new_fixed(-(i as i32)));
                }
                OpenDirection::Down => {
                    loc.adjust_location_by_y(DynVal::new_fixed(1));
                }
                OpenDirection::LeftThenDown => {
                    if i == 0 {
                        loc.adjust_location_by_x(neg_max_width.clone());
                    } else {
                        loc.adjust_location_by_y(DynVal::new_fixed(1));
                    }
                }
                OpenDirection::RightThenDown => {
                    if i == 0 {
                        loc.adjust_location_by_x(item_width.clone());
                    } else {
                        loc.adjust_location_by_y(DynVal::new_fixed(1));
                    }
                }
                OpenDirection::LeftThenUp => {
                    if i == 0 {
                        loc.adjust_location_by_x(neg_max_width.clone());
                    } else {
                        loc.adjust_location_by_y(DynVal::new_fixed(-1));
                    }
                }
                OpenDirection::RightThenUp => {
                    if i == 0 {
                        loc.adjust_location_by_x(item_width.clone());
                    } else {
                        loc.adjust_location_by_y(DynVal::new_fixed(-1));
                    }
                }
            };
            loc.set_dyn_width(max_width.clone());
            self.pane
                .eo
                .update_el_primary_location(it.id(), loc.clone());
            it.set_visible(true);
        }
    }
}

// ---------------------

#[derive(Clone)]
pub struct MenuItem {
    pane: Pane,
    path: Rc<RefCell<MenuPath>>,
    /// name displayed in the menu
    selectable: Rc<RefCell<bool>>,
    is_selected: Rc<RefCell<bool>>,
    /// is the item currently selected
    is_folder: Rc<RefCell<bool>>,
    /// is the item a folder
    show_folder_arrow: Rc<RefCell<bool>>,
    /// show the folder arrow, false for primary horizontal bar
    #[allow(clippy::type_complexity)]
    click_fn: Rc<RefCell<Option<MenuItemFn>>>,
}

pub type MenuItemFn = Box<dyn FnMut(Context) -> EventResponses>;

impl MenuItem {
    pub const KIND: &'static str = "menu_item";

    pub fn new(ctx: &Context, path: MenuPath) -> Self {
        let pane = Pane::new(ctx, MenuItem::KIND).with_z(MenuBar::Z_INDEX);
        MenuItem {
            pane,
            path: Rc::new(RefCell::new(path)),
            selectable: Rc::new(RefCell::new(true)),
            is_selected: Rc::new(RefCell::new(false)),
            is_folder: Rc::new(RefCell::new(false)),
            show_folder_arrow: Rc::new(RefCell::new(true)),
            click_fn: Rc::new(RefCell::new(None)),
        }
    }

    pub fn new_folder(ctx: &Context, path: MenuPath) -> Self {
        let item = MenuItem::new(ctx, path);
        *item.is_folder.borrow_mut() = true;
        item
    }

    pub fn set_fn(&self, f: Option<MenuItemFn>) {
        *self.click_fn.borrow_mut() = f;
    }

    pub fn with_fn(self, f: Option<MenuItemFn>) -> Self {
        self.set_fn(f);
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

    /// if the primary menu items are folders, they have an arrow if primary_show_arrow=true
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

    /// draw_padding draws DrawChPos's for padding and return updated DrawChPos array
    /// along with updated x position
    pub fn draw_padding(
        padding: usize, mut x: usize, style: Style, dcps: Vec<DrawChPos>,
    ) -> (usize, Vec<DrawChPos>) {
        let mut dcps = dcps;
        for _ in 0..padding {
            let dc = DrawCh::new(' ', style.clone());
            dcps.push(DrawChPos::new(dc, x as u16, 0));
            x += 1;
        }
        (x, dcps)
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for MenuBar {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::Mouse(me) => self.receive_mouse_event(ctx, me),
            Event::ExternalMouse(me) => self.receive_external_mouse_event(ctx, me),
            Event::KeyCombo(keys) if !keys.is_empty() => self.receive_key_event(ctx, keys[0]),
            _ => self.pane.receive_event(ctx, ev.clone()),
        }
    }

    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        if !self.get_visible() {
            return Vec::with_capacity(0);
        }

        let menu_style_bz = match serde_json::to_vec(&*self.menu_style.borrow()) {
            Ok(v) => v,
            Err(e) => {
                log_err!("error serializing menu style: {}", e);
                return Vec::with_capacity(0);
            }
        };

        let mut out = self.pane.drawing(ctx, dr, force_update);
        //return out;

        // draw each menu item
        for el_details in self.pane.eo.els.borrow().values() {
            // offset pos to location
            let child_ctx = ctx
                .clone()
                .with_metadata(Self::MENU_STYLE_MD_KEY.to_string(), menu_style_bz.clone());
            let child_dr = dr.child_region(&el_details.loc.borrow().l);

            // a bit annoying as this generates ClearUpdates for every menu item every call draw
            // cycle
            let mut upds = el_details.el.drawing(&child_ctx, &child_dr, force_update);

            for upd in &mut upds {
                upd.prepend_id(el_details.el.id(), el_details.loc.borrow().z);
                match upd.action {
                    DrawAction::Update(ref mut dcps) | DrawAction::Extend(ref mut dcps) => {
                        let l = el_details.loc.borrow().l.clone();
                        let s = dr.size;
                        let child_s = child_dr.size;

                        // NOTE this is a computational bottleneck
                        // currently using rayon for parallelization
                        let mut start_x = l.get_start_x_from_size(s);
                        let mut start_y = l.get_start_y_from_size(s);
                        // check for overflow
                        if start_x < 0 {
                            start_x = 0;
                        }
                        if start_y < 0 {
                            start_y = 0;
                        }
                        dcps.par_iter_mut().for_each(|dcp| {
                            dcp.set_draw_size_offset_colors(child_s, start_x, start_y);
                            dcp.x += start_x as u16;
                            dcp.y += start_y as u16;
                        });
                    }
                    DrawAction::Remove => {}
                    DrawAction::ClearAll => {}
                }
            }

            out.extend(upds);
        }
        out
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for MenuItem {
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

    // TODO refactor to cache instead of just returning updates every drawing
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, _force_update: bool) -> Vec<DrawUpdate> {
        if !self.get_visible() {
            return Vec::with_capacity(0);
        }

        let Some(ref md) = ctx.get_metadata(MenuBar::MENU_STYLE_MD_KEY) else {
            return Vec::with_capacity(0);
        };

        let m_sty: MenuStyle = match serde_json::from_slice(md) {
            Ok(v) => v,
            Err(e) => {
                log_err!("error deserializing menu style: {}", e);
                return Vec::with_capacity(0);
            }
        };

        let sty = match (*self.is_selected.borrow(), *self.selectable.borrow()) {
            (true, _) => m_sty.selected_style,
            (false, true) => m_sty.unselected_style,
            (false, false) => m_sty.disabled_style,
        };

        let (mut x, mut out) = MenuItem::draw_padding(m_sty.left_padding, 0, sty.clone(), vec![]);

        // draw name
        let name = self.path.borrow().name().to_string();
        let name_chs = DrawChPos::new_from_string(name, x as u16, 0, sty.clone());
        x += name_chs.len();
        out.extend(name_chs);

        let arrow_text = if *self.is_folder.borrow()
            && (!self.is_primary() || (self.is_primary() && *self.show_folder_arrow.borrow()))
        {
            m_sty.folder_arrow
        } else {
            String::new()
        };

        // add filler space
        while x
            < (dr.size.width as usize)
                .saturating_sub(m_sty.right_padding + arrow_text.chars().count())
        {
            let dc = DrawCh::new(' ', sty.clone());
            let dcp = DrawChPos::new(dc, x as u16, 0);
            out.push(dcp);
            x += 1;
        }

        // draw folder arrow
        let arrow_chs = DrawChPos::new_from_string(arrow_text, x as u16, 0, sty.clone());
        x += arrow_chs.len();
        out.extend(arrow_chs);

        // add right padding
        let (_, out) = MenuItem::draw_padding(m_sty.right_padding, x, sty.clone(), out);
        DrawUpdate::update(out).into()
    }
}

// -----------------------------------------------------------------------
/// MenuPath is a path of menu items within a menu tree.
/// For example:
///
/// "nwmod/cool_stuff/blaze"
///
/// represents a menu item which lives in the top-level menu "nwmod" within the
/// sub-menu "cool_stuff" with the name "blaze".
#[derive(Clone, PartialEq, Debug)]
pub struct MenuPath(pub String);

impl MenuPath {
    /// Name returns the name of the menu item at the end of the menu path
    pub fn name(&self) -> &str {
        let split: Vec<&str> = self.0.split('/').collect();
        if !split.is_empty() { split[split.len() - 1] } else { "" }
    }

    /// Folders returns the folders of the menu path
    pub fn folders(&self) -> Vec<&str> {
        let split: Vec<&str> = self.0.split('/').collect();
        if !split.is_empty() { split[..split.len() - 1].to_vec() } else { vec![] }
    }

    pub fn is_root(&self) -> bool {
        !self.0.contains('/')
    }

    /// check if other is the the same as the parent
    /// but then includes one more item
    pub fn is_immediate_parent_of(&self, other: &MenuPath) -> bool {
        let parent_path1 = self.0.clone();
        let parent_path2 = other.0.clone();
        let split1: Vec<&str> = self.0.split('/').collect();
        let split2: Vec<&str> = other.0.split('/').collect();
        parent_path2.starts_with(&parent_path1) && split2.len() == split1.len() + 1
    }
}
