use {
    crate::{element::ReceivableEventChanges, DrawCh, Element, Event, Priority, UpwardPropagator},
    std::{cell::RefCell, rc::Rc},
};

// StandardPane is a pane element which other objects can embed and build off
// of. It defines the basic draw functionality of a pane.
pub struct StandardPane {
    // The SelfEvs are NOT handled by the standard pane. The element inheriting the
    // standard pane is expected to handle all SelfReceivableEvents in the
    // ReceiveEvent function. The standard pane is only responsible for
    // listing the receivable events when Receivable() is called
    self_evs: SelfReceivableEvents,

    // This elements "overall" reference priority
    //
    // NOTE this is only used currently by the ParentPane,
    // consider moving this field into the ParentPane, if nothing
    // else ever uses it.
    element_priority: Priority,

    up: Option<Rc<RefCell<dyn UpwardPropagator>>>,

    view_height: usize,
    view_width: usize,

    // The pane's Content need not be the dimensions provided within
    // the Location, however the Content will simply be cut off if it exceeds
    // any dimension of the Location. If the Content is less than the dimensions
    // of the Location all extra characters will be filled with the DefaultCh.
    // The location of where to begin drawing from within the Content can be
    // offset using content_view_offset_x and content_view_offset_y
    content: Vec<Vec<DrawCh>>,
    default_ch: DrawCh,
    default_line: Vec<DrawCh>,
    content_view_offset_x: usize,
    content_view_offset_y: usize,
}

impl Default for StandardPane {
    fn default() -> Self {
        StandardPane::new(vec![], DrawCh::default(), vec![], 0, 0)
    }
}

impl StandardPane {
    pub fn new(
        content: Vec<Vec<DrawCh>>, default_ch: DrawCh, default_line: Vec<DrawCh>,
        content_view_offset_x: usize, content_view_offset_y: usize,
    ) -> StandardPane {
        StandardPane {
            self_evs: SelfReceivableEvents::new(),
            element_priority: Priority::UNFOCUSED,
            up: None,
            view_height: 0,
            view_width: 0,
            content,
            default_ch,
            default_line,
            content_view_offset_x,
            content_view_offset_y,
        }
    }
}

impl UpwardPropagator for StandardPane {
    // NOTE this function should NOT be used if the standard pane is used as a base for a
    // more complex element. As the developer you should be fulfilling the
    // PropagateUpwardChangesToInputability function directly in your element.
    fn propagate_receivable_event_changes_upward(
        &mut self, child_el: Rc<RefCell<dyn Element>>, rec: ReceivableEventChanges,
        update_this_elements_prioritizers: bool,
    ) {
        if let Some(up) = self.up.as_ref() {
            up.borrow_mut().propagate_receivable_event_changes_upward(
                child_el,
                rec,
                update_this_elements_prioritizers,
            );
        }
    }
}

impl Element for StandardPane {
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.self_evs.clone()
    }
    //fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponse);
    //fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges;
    //fn drawing(&self, ctx: &Context) -> Vec<DrawChPos>;
    //fn set_upward_propagator(&self, up: Rc<RefCell<dyn UpwardPropagator>>);
}

/*



// Receivable returns the event keys and commands that can
// be received by this element along with their priorities
func (sp *StandardPane) Receivable() []yh.PriorityEv {
    return sp.SelfEvs
}

// DeregisterEvCombo removes an event combo from the element's
// list of receivable event combos
func (sp *StandardPane) DeregisterEv(ec yh.PrioritizableEv) {
    for i, pef := range sp.SelfEvs {
        if pef.Ev.Key() != ec.Key() {
            continue
        }
        sp.SelfEvs = append(sp.SelfEvs[:i], sp.SelfEvs[i+1:]...)
        return
    }
}

func (sp *StandardPane) ReceiveEvent(ctx yh.Context, ev interface{}) (
    captured bool, resp yh.EventResponse) {

    sp.ViewHeight, sp.ViewWidth = ctx.GetHeight(), ctx.GetWidth()

    return false, yh.EventResponse{}
}

// Drawing compiles all of the DrawChPos necessary to draw this element
func (sp *StandardPane) Drawing(ctx yh.Context) (chs []yh.DrawChPos) {

    // convert the Content to DrawChPos
    // NOTE: width/height values must subtract 1 to get final cell locations
    for y := 0; y <= ctx.S.Height-1; y++ {
        for x := 0; x <= ctx.S.Width-1; x++ {

            // default ch being added next is the DefaultCh
            chOut := sp.DefaultCh

            // TODO XXX allow for negative offsets! currently crashes

            offsetY := y + sp.contentViewOffsetY
            offsetX := x + sp.contentViewOffsetX

            // if the offset isn't pushing all the content out of view,
            // assign the next ch to be the one at the offset in the Content
            // matrix
            if offsetY < len(sp.Content) && offsetX < len(sp.Content[offsetY]) {
                chOut = sp.Content[offsetY][offsetX]
            }

            // if y is greater than the height of the visible content,
            // trigger a default line.
            // NOTE: height of the visible content is the height of the
            // content minus the offset
            if y > len(sp.Content) {
                if x < len(sp.DefaultLine) {
                    chOut = sp.DefaultLine[x]
                } else {
                    chOut = sp.DefaultCh
                }
            }

            chs = append(chs, yh.NewDrawChPos(chOut, x, y))
        }
    }
    return chs
}

// Update the priority for an event on the standard pane.
// This function is intended to be used by higher level elements
// embedding the StandardPane.
func (sp *StandardPane) UpdatePriorityForEv(ev yh.PrioritizableEv, pr yh.Priority) yh.InputabilityChanges {
    var ic yh.InputabilityChanges
    for i, pef := range sp.SelfEvs {
        if pef.Ev.Key() != ev.Key() {
            continue
        }
        sp.SelfEvs[i].Pr = pr
        ic.UpdatePriorityForEv(ev, pr)
        break
    }
    return ic
}

func (sp *StandardPane) UpdatePriorityForEvs(evs []yh.PrioritizableEv, pr yh.Priority) yh.InputabilityChanges {
    var ic yh.InputabilityChanges
    for _, ev := range evs {
        ic.Concat(sp.UpdatePriorityForEv(ev, pr))
    }
    return ic
}

// ChangePriority returns a priority change request to its parent organizer so
// as to update the priority of all commands registered to this element.
// The element iterates through its registered cmds/evCombos, and returns a
// priority change request for each one.
func (sp *StandardPane) ChangePriority(_ yh.Context, pr yh.Priority) yh.InputabilityChanges {

    var ic yh.InputabilityChanges

    // update the priority of all registered events
    for _, pef := range sp.SelfEvs {
        sp.SelfEvs.UpdatePriorityForEv(pef.Ev, pr)
        ic.UpdatePriorityForEv(pef.Ev, pr)
    }

    sp.ElementPriority = pr

    return ic
}

func (sp *StandardPane) SetUpwardPropagator(up yh.UpwardPropagator) {
    sp.UP = up
}

// ---------------------------------------------------------------------------

func (sp *StandardPane) GetContentViewOffsetX() int {
    return sp.contentViewOffsetX
}

func (sp *StandardPane) GetContentViewOffsetY() int {
    return sp.contentViewOffsetY
}

func (sp *StandardPane) SetContentViewOffsetX(offsetX int) {
    sp.contentViewOffsetX = offsetX
}

func (sp *StandardPane) SetContentViewOffsetY(offsetY int) {
    sp.contentViewOffsetY = offsetY
}

// ---------------------------------------------------------------------------
*/

// The SelfReceivableEvents are used to manage events and associated functions
// registered directly to an element (AND NOT to that elements children!). They
// are similar to the EvPrioritizer, but they are used to manage the events and
// commands that are registered locally to this specific element.
// (The EvPrioritizer and CmdPrioritizer are used to manage the events and
// commands that are registered to an element's
// children in the ElementOrganizer).
// NOTE: these fulfill a similar function to the prioritizers
// in that they manage inclusion/removal more cleanly and can be sorted
#[derive(Clone)]
pub struct SelfReceivableEvents(Vec<(Event, Priority)>);

impl SelfReceivableEvents {
    pub fn new() -> SelfReceivableEvents {
        SelfReceivableEvents(Vec::new())
    }

    // TRANSLATION NewSelfReceivableEventsFromPrioritizableEv(
    pub fn new_from_priority_events(p: Priority, evs: Vec<Event>) -> SelfReceivableEvents {
        SelfReceivableEvents(evs.into_iter().map(|ev| (ev, p)).collect())
    }

    // Include
    pub fn push(&mut self, ev: Event, p: Priority) {
        self.0.push((ev, p))
    }

    // IncludeMany
    pub fn extend(&mut self, evs: Vec<(Event, Priority)>) {
        self.0.extend(evs)
    }
}

/*

func (sre *SelfReceivableEvents) Remove(ec yh.EvKeyCombo) {
    for i, pef := range *sre {
        if pef.Ev.Key() != ec.Key() {
            continue
        }
        *sre = append((*sre)[:i], (*sre)[i+1:]...)
    }
}

func (sre *SelfReceivableEvents) RemoveMany(ecs []yh.EvKeyCombo) {
    for _, ec := range ecs {
        sre.Remove(ec)
    }
}

// UpdatePriorityForEvCombo updates the priority of the given event combo
// registered directly to this element
func (sre *SelfReceivableEvents) UpdatePriorityForEv(ec yh.PrioritizableEv, pr yh.Priority) {
    for i, pef := range *sre {
        if pef.Ev.Key() != ec.Key() {
            continue
        }
        (*sre)[i].Pr = pr
    }
}

// UpdatePrioritiesForEvCombos updates the priorities of the given event combos
// registered directly to this element
func (sre *SelfReceivableEvents) UpdatePrioritiesForEvCombos(ecs []yh.PrioritizableEv, pr yh.Priority) {
    for _, ec := range ecs {
        sre.UpdatePriorityForEv(ec, pr)
    }
}
*/
