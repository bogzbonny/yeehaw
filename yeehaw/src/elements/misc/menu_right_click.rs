use {
    crate::{
        Context, DrawRegion, DrawUpdate, DynLocation, DynLocationSet, DynVal, Element, ElementID,
        Event, EventResponse, EventResponses, MenuBar, MouseEvent, Parent, Point, ReceivableEvents,
        ZIndex,
        elements::menu::{MenuItem, MenuStyle},
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

// ---------------------------------------------------

/// This menu destroys itself when it is closed.
#[derive(Clone)]
pub struct RightClickMenu {
    pub menu: MenuBar,

    /// the position of the inital right click which
    /// opened the menu. This information is passed
    /// into the menuItem function events through the
    /// context
    pub pos: Rc<RefCell<Point>>,

    /// used to prevent the first external mouse event
    /// from closing the right click menu
    pub just_created: Rc<RefCell<bool>>,
}

impl RightClickMenu {
    pub const MENU_POSITION_MD_KEY: &'static str = "menu_position";
    pub const Z_INDEX: ZIndex = 200;

    pub fn new(ctx: &Context, sty: MenuStyle) -> Self {
        let menu = MenuBar::right_click_menu(ctx).with_menu_style(sty);
        Self {
            menu,
            pos: Rc::new(RefCell::new(Point::default())),
            just_created: Rc::new(RefCell::new(true)),
        }
    }

    pub fn with_menu_items(self, ctx: &Context, items: Vec<MenuItem>) -> Self {
        self.menu.set_items(ctx, items);
        self
    }

    pub fn set_menu_pos(&mut self, pos: Point) {
        *self.pos.borrow_mut() = pos;
    }

    /// Output Some if a right click menu should be created
    pub fn create_menu_if_right_click(&self, me: &MouseEvent) -> Option<EventResponses> {
        if me.kind != MouseEventKind::Up(MouseButton::Right) {
            return None;
        }

        // adjust locations
        let (ev_x, ev_y) = (me.column, me.row);
        *self.pos.borrow_mut() = Point::new(ev_x, ev_y);

        let (x, y): (i32, i32) = (ev_x, (ev_y + 1)); // offset the menu 1 below the cursor
        let (x, y) = (DynVal::new_fixed(x), DynVal::new_fixed(y));

        // the location size doesn't matter as the top level element will be
        // empty, only the sub-elements will be take up space
        let loc = DynLocationSet::new(
            DynLocation::new(x.clone(), x, y.clone(), y),
            vec![],
            Self::Z_INDEX,
        );
        self.set_dyn_location_set(loc);

        self.menu.activate();
        self.menu.deselect_all();
        self.menu.collapse_non_primary();
        self.menu.make_primary_visible();
        self.menu.set_visible(true);
        *self.just_created.borrow_mut() = true;

        let resps = EventResponse::BringToFront.into();

        Some(EventResponse::NewElement(Box::new(self.clone()), Some(resps)).into())
    }
}

#[yeehaw_derive::impl_element_from(menu)]
impl Element for RightClickMenu {
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        match ev {
            Event::Mouse(_) => {
                // addend the mouse position to the context metadata
                // so that the menu items may know the overall position
                // of the right click.
                let pos_bz = match serde_json::to_vec(&*self.pos.borrow()) {
                    Ok(v) => v,
                    Err(e) => {
                        log_err!("failed to serialize right click menu position: {}", e);
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
}
