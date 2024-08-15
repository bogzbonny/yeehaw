use {
    super::{SclVal, Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        AttributesMirror, Context, DrawChPos, Element, ElementID, Event, EventResponse,
        Keyboard as KB, Priority, ReceivableEventChanges, RgbColour, SortingHat, Style,
        UpwardPropagator,
    },
    crossterm::event::{MouseButton, MouseEventKind},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct Checkbox {
    pub base: WidgetBase,
    pub checked: Rc<RefCell<bool>>, // whether the checkbox is checked or not

    // rune to use for the checkmark
    // recommended:  √ X x ✖
    pub checkmark: Rc<RefCell<char>>,

    // function which executes when checkbox is checked or unchecked,
    // bool is the new state of the checkbox (true = checked)
    pub clicked_fn: Rc<RefCell<dyn FnMut(bool) -> EventResponse>>,
}

impl Checkbox {
    const KIND: &'static str = "widget_checkbox";

    const STYLE: WBStyles = WBStyles {
        selected_style: Style::new()
            .with_bg(RgbColour::YELLOW)
            .with_fg(RgbColour::BLACK)
            .with_attr(AttributesMirror::new().with_bold()),
        ready_style: Style::new()
            .with_bg(RgbColour::WHITE)
            .with_fg(RgbColour::BLACK)
            .with_attr(AttributesMirror::new().with_bold()),
        unselectable_style: Style::new()
            .with_bg(RgbColour::GREY13)
            .with_fg(RgbColour::BLACK)
            .with_attr(AttributesMirror::new().with_bold()),
    };

    pub fn default_receivable_events() -> Vec<Event> {
        vec![KB::KEY_ENTER.into()] // when "active" hitting enter will click the button
    }

    pub fn new(
        hat: &SortingHat, ctx: &Context, text: String,
        clicked_fn: Box<dyn FnMut() -> EventResponse>,
    ) -> Self {
        let wb = WidgetBase::new(
            hat,
            Self::KIND,
            ctx.clone(),
            SclVal::new_fixed(text.len() + 2),
            SclVal::new_fixed(1),
            Self::STYLE,
            Self::default_receivable_events(),
        );
        _ = wb.set_selectability(Selectability::Unselectable);
        let b = Checkbox {
            base: wb,
            text: Rc::new(RefCell::new(text)),
            sides: Rc::new(RefCell::new((']', '['))),
            clicked_fn: Rc::new(RefCell::new(clicked_fn)),
        };
        b.base.set_content_from_string(&b.button_text());
        b
    }

    // ----------------------------------------------
    // decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base.set_styles(styles);
        self
    }

    pub fn with_sides(self, sides: (char, char)) -> Self {
        *self.sides.borrow_mut() = sides;
        self
    }

    pub fn at(mut self, loc_x: SclVal, loc_y: SclVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }

    // ----------------------------------------------
    pub fn click(&self) -> EventResponse {
        let resp = (self.clicked_fn.borrow_mut())();
        resp.with_deactivate()
    }
}

impl Widget for Checkbox {}

impl Element for Checkbox {
    fn kind(&self) -> &'static str {
        self.base.kind()
    }
    fn id(&self) -> ElementID {
        self.base.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.base.receivable()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponse) {
        let _ = self.base.receive_event(ctx, ev.clone());
        match ev {
            Event::KeyCombo(ke) => {
                if self.base.get_selectability() != Selectability::Selected || ke.is_empty() {
                    return (false, EventResponse::default());
                }
                if ke[0].matches(&KB::KEY_ENTER) {
                    return (true, self.click());
                }
            }
            Event::Mouse(me) => {
                if let MouseEventKind::Up(MouseButton::Left) = me.kind {
                    return (true, self.click());
                }
            }
            _ => {}
        }
        (false, EventResponse::default())
    }

    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.base.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.base.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.base.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.base.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Rc<RefCell<dyn UpwardPropagator>>) {
        self.base.set_upward_propagator(up)
    }
}

/*


func NewCheckbox(pCtx yh.Context) *Checkbox {
    wb := NewWidgetBase(pCtx, NewStatic(1), NewStatic(1), CheckboxStyle, CheckboxEvCombos)

    cb := &Checkbox{
        WidgetBase: wb,
        Checked:    false,
        Checkmark:  '√',
        ClickedFn:  nil,
    }
    wb.SetContentFromString(cb.Text())
    return cb
}

func (cb *Checkbox) At(locX, locY SclVal) *Checkbox {
    cb.WidgetBase.At(locX, locY)
    return cb
}

func (cb *Checkbox) ToWidgets() Widgets {
    return Widgets{cb}
}

// ---------------------------------------------------------

func (cb *Checkbox) Text() string {
    if cb.Checked {
        return string(cb.Checkmark)
    }
    return " "
}

func (cb *Checkbox) SetClickedFn(fn func(checked bool) yh.EventResponse) {
    cb.ClickedFn = fn
}

func (cb *Checkbox) Click() yh.EventResponse {
    cb.Checked = !cb.Checked
    resp := yh.NewEventResponse()
    if cb.ClickedFn != nil {
        resp = cb.ClickedFn(cb.Checked)
    }
    cb.SetContentFromString(cb.Text())
    return resp
}

// need to re set the content in order to reflect active style
func (cb *Checkbox) Drawing() (chs []yh.DrawChPos) {
    cb.SetContentFromString(cb.Text())
    return cb.WidgetBase.Drawing()
}

func (cb *Checkbox) ReceiveKeyEventCombo(
    evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {

    if cb.Selectedness != Selected {
        return false, yh.NewEventResponse()
    }

    switch {
    case yh.EnterEKC.Matches(evs):
        return true, cb.Click()
    }
    return true, yh.NewEventResponse()
}

func (cb *Checkbox) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {
    if ev.Buttons() == tcell.Button1 {
        return true, cb.Click()
    }
    return false, yh.NewEventResponse()
}
*/
