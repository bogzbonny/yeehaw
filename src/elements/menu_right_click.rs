use {
    crate::{
        elements::menu::{MenuItem, MenuStyle},
        Context, DrawChPos, Element, ElementID, Event, EventResponse, EventResponses, MenuBar,
        Point, Priority, ReceivableEventChanges, SclLocation, SclLocationSet, SclVal, SortingHat,
        UpwardPropagator, ZIndex,
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
    pub const Z_INDEX: ZIndex = -100;

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
        let (x, y) = (SclVal::new_fixed(x), SclVal::new_fixed(y));

        self.menu.activate();
        self.menu.deselect_all();
        self.menu.collapse_non_primary();
        self.menu.make_primary_visible();
        let mut extra_locs = self.menu.extra_locations();

        for l in extra_locs.iter_mut() {
            l.adjust_location_by(x.clone(), y.clone());
        }

        // the location size doesn't matter as the top level element will be
        // empty, only the sub-elements will be take up space
        let loc = SclLocationSet::default()
            .with_location(SclLocation::new(x.clone(), x, y.clone(), y)) // placeholder
            .with_extra(extra_locs)
            .with_z(Self::Z_INDEX);

        *self.just_created.borrow_mut() = true;
        *self.get_scl_location_set().borrow_mut() = loc;

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

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
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
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.menu.set_upward_propagator(up)
    }
    fn get_scl_location_set(&self) -> Rc<RefCell<SclLocationSet>> {
        self.menu.get_scl_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.menu.get_visible()
    }
}
