use {
    super::{Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Color, Context, DrawChPos, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Keyboard as KB, Parent, Priority, ReceivableEvent, ReceivableEventChanges,
        SelfReceivableEvents, Style,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct Toggle {
    pub base: WidgetBase,
    pub left: Rc<RefCell<String>>,
    pub right: Rc<RefCell<String>>,
    pub left_selected: Rc<RefCell<bool>>, // otherwise right is selected
    pub clicked_down: Rc<RefCell<bool>>,  // activated when mouse is clicked down while over button
    pub selected_sty: Rc<RefCell<Style>>,
    //                                   selected
    pub toggled_fn: Rc<RefCell<dyn FnMut(Context, String) -> EventResponses>>,
}

impl Toggle {
    const KIND: &'static str = "widget_button";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new(Some(Color::BLACK), Some(Color::LIGHT_YELLOW2), None),
        ready_style: Style::new(Some(Color::BLACK), Some(Color::WHITE), None),
        unselectable_style: Style::new(Some(Color::BLACK), Some(Color::GREY13), None),
    };

    // for the selected toggle
    const DEFAULT_SELECTED_STY: Style =
        Style::new(Some(Color::BLACK), Some(Color::LIGHT_BLUE), None);

    pub fn default_receivable_events() -> Vec<ReceivableEvent> {
        vec![
            KB::KEY_ENTER.into(),
            KB::KEY_LEFT.into(),
            KB::KEY_RIGHT.into(),
            KB::KEY_H.into(),
            KB::KEY_L.into(),
        ]
    }

    pub fn new(
        ctx: &Context, left: String, right: String,
        toggeld_fn: Box<dyn FnMut(Context, String) -> EventResponses>,
    ) -> Self {
        let wb = WidgetBase::new(
            ctx,
            Self::KIND,
            DynVal::new_fixed(left.chars().count() as i32 + right.chars().count() as i32),
            DynVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        wb.set_content_from_string(ctx, &(left.clone() + &right));
        Toggle {
            base: wb,
            left: Rc::new(RefCell::new(left)),
            right: Rc::new(RefCell::new(right)),
            left_selected: Rc::new(RefCell::new(true)),
            clicked_down: Rc::new(RefCell::new(false)),
            selected_sty: Rc::new(RefCell::new(Self::DEFAULT_SELECTED_STY)),
            toggled_fn: Rc::new(RefCell::new(toggeld_fn)),
        }
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }

    // ----------------------------------------------

    pub fn selected(&self) -> String {
        if *self.left_selected.borrow() {
            return self.left.borrow().clone();
        }
        self.right.borrow().clone()
    }

    pub fn perform_toggle(&self, ctx: &Context) -> EventResponses {
        let l_sel = *self.left_selected.borrow();
        *self.left_selected.borrow_mut() = !l_sel;
        self.toggled_fn.borrow_mut()(ctx.clone(), self.selected())
    }
}

impl Widget for Toggle {}

#[yeehaw_derive::impl_element_from(base)]
impl Element for Toggle {
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponses::default());
                }
                match true {
                    _ if ke[0] == KB::KEY_ENTER => {
                        return (true, self.perform_toggle(ctx));
                    }
                    _ if ke[0] == KB::KEY_LEFT || ke[0] == KB::KEY_H => {
                        if !*self.left_selected.borrow() {
                            return (true, self.perform_toggle(ctx));
                        }
                        return (true, EventResponses::default());
                    }
                    _ if ke[0] == KB::KEY_RIGHT || ke[0] == KB::KEY_L => {
                        if *self.left_selected.borrow() {
                            return (true, self.perform_toggle(ctx));
                        }
                        return (true, EventResponses::default());
                    }
                    _ => {}
                }
                return (false, EventResponses::default());
            }
            Event::Mouse(me) => {
                let clicked_down = *self.clicked_down.borrow();
                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        *self.clicked_down.borrow_mut() = true;
                        return (true, EventResponses::default());
                    }
                    MouseEventKind::Drag(MouseButton::Left) if clicked_down => {}
                    MouseEventKind::Up(MouseButton::Left) if clicked_down => {
                        *self.clicked_down.borrow_mut() = false;
                        let x = me.column as usize;
                        let left_sel = *self.left_selected.borrow();
                        if (!left_sel && x < self.left.borrow().chars().count())
                            || (left_sel && x >= self.left.borrow().chars().count())
                        {
                            return (true, self.perform_toggle(ctx));
                        }
                    }
                    _ => {
                        *self.clicked_down.borrow_mut() = false;
                    }
                }
                return (false, EventResponses::default());
            }
            _ => {}
        }
        (false, EventResponses::default())
    }

    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        // need to re set the content in order to reflect active style
        let left = self.left.borrow();
        let right = self.right.borrow();
        let left_len = left.chars().count();
        let right_len = right.chars().count();
        self.base
            .set_content_from_string(ctx, &(left.clone() + &right));
        if *self.left_selected.borrow() {
            for i in 0..left_len {
                self.base.pane.content.borrow_mut()[0][i].style =
                    self.selected_sty.borrow().clone();
            }
        } else {
            for i in left_len..left_len + right_len {
                self.base.pane.content.borrow_mut()[0][i].style =
                    self.selected_sty.borrow().clone();
            }
        }
        self.base.drawing(ctx)
    }
}
