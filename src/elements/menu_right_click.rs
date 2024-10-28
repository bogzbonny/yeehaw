use {
    crate::{
        elements::menu::{MenuItem, MenuStyle},
        Context, DrawChPos, DynLocation, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponse, EventResponses, MenuBar, Parent, Point, Priority, ReceivableEventChanges,
        SortingHat, ZIndex,
    },
    crossterm::event::{MouseButton, MouseEvent, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

//pub struct RightClickMenuTemplate(pub MenuBar);

// ---------------------------------------------------
// The menu destroys itself when it is closed.
#[derive(Clone)]
pub struct RightClickMenu {
    pub menu: MenuBar,

    // the position of the inital right click which
    // opened the menu. This information is passed
    // into the menuItem function events through the
    // context
    pub pos: Rc<RefCell<Point>>,

    // used to prevent the first external mouse event
    // from closing the right click menu
    pub just_created: Rc<RefCell<bool>>,
}

impl RightClickMenu {
    pub const MENU_POSITION_MD_KEY: &'static str = "menu_position";
    pub const Z_INDEX: ZIndex = 100;

    pub fn new(hat: &SortingHat, sty: MenuStyle) -> Self {
        let menu = MenuBar::right_click_menu(hat).with_menu_style(sty);
        Self {
            menu,
            pos: Rc::new(RefCell::new(Point::default())),
            just_created: Rc::new(RefCell::new(true)),
        }
    }

    pub fn with_menu_items(self, hat: &SortingHat, ctx: &Context, items: Vec<MenuItem>) -> Self {
        self.menu.set_items(hat, ctx, items);
        self
    }

    pub fn set_menu_pos(&mut self, pos: Point) {
        *self.pos.borrow_mut() = pos;
    }

    // Output Some if a right click menu should be created
    pub fn create_menu_if_right_click(&self, me: MouseEvent) -> Option<EventResponses> {
        if me.kind != MouseEventKind::Up(MouseButton::Right) {
            return None;
        }

        // adjust locations
        let (ev_x, ev_y) = (me.column, me.row);
        *self.pos.borrow_mut() = Point::new(ev_x.into(), ev_y.into());

        let (x, y): (i32, i32) = (ev_x.into(), (ev_y + 1).into()); // offset the menu 1 below the cursor
        let (x, y) = (DynVal::new_fixed(x), DynVal::new_fixed(y));

        // the location size doesn't matter as the top level element will be
        // empty, only the sub-elements will be take up space
        let loc = DynLocationSet::new(
            DynLocation::new(x.clone(), x, y.clone(), y),
            vec![],
            Self::Z_INDEX,
        );
        *self.get_dyn_location_set().borrow_mut() = loc;

        self.menu.activate();
        self.menu.deselect_all();
        self.menu.collapse_non_primary();
        self.menu.make_primary_visible();
        self.menu.set_visible(true);
        *self.just_created.borrow_mut() = true;

        let loc = self.get_dyn_location_set().borrow().clone();
        debug!(
            "RightClickMenu::create_menu_if_right_click: loc: {:#?}",
            loc
        );

        Some(EventResponse::NewElement(Rc::new(RefCell::new(self.clone()))).into())
    }
}

impl Element for RightClickMenu {
    fn kind(&self) -> &'static str {
        self.menu.kind()
    }
    fn id(&self) -> ElementID {
        self.menu.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.menu.receivable()
    }

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::Mouse(_) => {
                // addend the mouse position to the context metadata
                // so that the menu items may know the overall position
                // of the right click.
                let pos_bz = match serde_json::to_vec(&*self.pos.borrow()) {
                    Ok(v) => v,
                    Err(_e) => {
                        // TODO log error
                        return (true, EventResponses::default());
                    }
                };
                let ctx = ctx
                    .clone()
                    .with_metadata(Self::MENU_POSITION_MD_KEY.to_string(), pos_bz);

                self.menu.receive_event(&ctx, ev)
            }

            Event::ExternalMouse(_) => {
                if *self.just_created.borrow() {
                    *self.just_created.borrow_mut() = false;
                    return (true, EventResponses::default());
                }
                self.menu.receive_event(ctx, ev)
            }
            _ => (true, EventResponses::default()),
        }
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.menu.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.menu.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.menu.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.menu.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn Parent>) {
        self.menu.set_upward_propagator(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.menu.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.menu.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.menu.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.menu.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.menu.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.menu.get_visible()
    }
}
