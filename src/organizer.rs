use {
    crate::{
        prioritizer::EventPrioritizer, Element, ElementID, InputabilityChanges, Locations,
        UpwardPropagator,
    },
    std::collections::HashMap,
};

// ElementOrganizer prioritizes and organizes all the elements contained
// within it
pub struct ElementOrganizer {
    largest: ElementID, // the largest ID currently registered to the organizer
    elements: HashMap<ElementID, Box<dyn Element>>,
    locations: HashMap<ElementID, Locations>, // Locations of all of the elements contained
    visibility: HashMap<ElementID, bool>,     // whether the element is set to display
    prioritizer: EventPrioritizer,

    // TODO turn to debug assert statement?
    //
    // Panic if two children have registered the same ev/cmd at the same
    // priority. If false the event will be sent to the ev/cmd to which happens
    // to be first in the prioritizer
    panic_on_overload: bool,
}

impl Default for ElementOrganizer {
    fn default() -> Self {
        ElementOrganizer {
            largest: 0,
            elements: HashMap::new(),
            locations: HashMap::new(),
            visibility: HashMap::new(),
            prioritizer: EventPrioritizer::default(),
            panic_on_overload: true,
        }
    }
}

impl ElementOrganizer {
    pub fn add_element(
        &mut self, el: Box<dyn Element>, up: Box<dyn UpwardPropagator>, loc: Locations, vis: bool,
    ) -> ElementID {
        // assign the new element id
        self.largest += 1;
        let el_id = self.largest;

        // put it at the top of the z-dim (pushing everything else down))
        self.update_el_z_index(el_id, 0);

        self.locations.insert(el_id, loc);
        self.elements.insert(el_id, el);
        self.visibility.insert(el_id, vis);

        // add the elements recievable events and commands to the prioritizer
        let receivable_evs = el.receivable();
        self.prioritizer
            .include(el_id, receivable_evs, self.panic_on_overload);

        // give the child element a reference to the parent (the up passed in as an
        // input)
        // NOTE: this is used in upwards propagation of changes to inputability
        // initiated by an element other than the parent (via this element organizer)
        // (ex: a sibling initiating a change to inputability, as opposed to this eo
        // passing an event to the child through ReceiveEventKeys)
        el.set_upward_propagator(up);

        el_id
    }

    pub fn remove_element(&mut self, el_id: ElementID) -> InputabilityChanges {
        self.elements.remove(&el_id);
        self.locations.remove(&el_id);
        self.visibility.remove(&el_id);

        if self.largest == el_id {
            self.largest -= 1;
        }

        let rm_evs = self.prioritizer.remove_entire_element(el_id);

        InputabilityChanges::default().with_remove_evs(rm_evs)
    }
}

/*


// RemoveElement removes the element associated with the given id
func (eo *ElementOrganizer) RemoveElement(elID ElementID) (ic InputabilityChanges) {

    delete(eo.Elements, elID)
    delete(eo.Locations, elID)
    delete(eo.Visibility, elID)

    if eo.Largest == elID {
        eo.Largest = eo.Largest - 1
    }

    rmEvs := eo.Prioritizer.RemoveEntireElement(elID)
    ic.RemoveEvs(rmEvs)
    return ic
}

// RemoveAllElements removes all elements from the element organizer
func (eo *ElementOrganizer) RemoveAllElements() (ic InputabilityChanges) {

    eo.Elements = make(map[ElementID]Element)
    eo.Locations = make(map[ElementID]Locations)
    eo.Visibility = make(map[ElementID]bool)
    eo.Largest = -1
    pes := eo.Receivable()
    ic.RemoveEvs(ToPrioritizableEvs(pes))
    eo.Prioritizer = NewEvPrioritizer()
    return ic
}

// GetLargestElID returns the element w/ the largest ID registered to element organizer
func (eo *ElementOrganizer) GetLargestElID() ElementID {
    return eo.Largest
}

// GetElementByID returns the element registered under the given id in the eo
func (eo *ElementOrganizer) GetElementByID(elID ElementID) Element {
    return eo.Elements[elID]
}

// GetIDFromEl returns the id registered under the given element in the eo
func (eo *ElementOrganizer) GetIDFromEl(el Element) ElementID {
    for id, e := range eo.Elements {
        if reflect.DeepEqual(e, el) {
            return id
        }
    }
    return NoElement
}

// MustGetLocations returns the context for the element registered under the given id
func (eo *ElementOrganizer) MustGetLocations(elID ElementID) Locations {

    locs, found := eo.Locations[elID]
    if !found {
        panic(fmt.Sprintf("Location not found for id %v but should exist", elID))
    }
    return locs
}

// GetElAtPos returns the element at the given position
func (eo *ElementOrganizer) GetElAtPos(x, y int) Element {
    for id, locs := range eo.Locations {
        if locs.Contains(x, y) {
            return eo.Elements[id]
        }
    }
    return nil
}

// GetElIDAtPos returns the element id at the given position
func (eo *ElementOrganizer) GetElIDAtPos(x, y int) ElementID {
    for id, locs := range eo.Locations {
        if locs.Contains(x, y) {
            return id
        }
    }
    return NoElement
}

// UpdateElLocationsByID updates the locations of the element with the given id
// to the given locations
func (eo *ElementOrganizer) UpdateElLocations(elID ElementID, locs Locations) {
    eo.Locations[elID] = locs
}

// UpdateElPrimaryLocation updates the primary location of the element with the given id
func (eo *ElementOrganizer) UpdateElPrimaryLocation(elID ElementID, loc Location) {
    locations := NewLocations(loc, eo.Locations[elID].Extra)
    eo.Locations[elID] = locations
}

// UpdateElZIndex updates the z-index of the element with the given id
// NOTE: if the given index is taken, the element currently filling that index
// will be pushed further back in the z-dimension (i.e. its z-index will be
// incremented)
func (eo *ElementOrganizer) UpdateElZIndex(elID ElementID, z ZIndex) {

    if eo.IsZIndexOccupied(z) {
        id := eo.GetElIDAtZIndex(z)
        eo.IncrementZIndexForElID(id)
    }

    loc := eo.Locations[elID].Location
    newLoc := NewLocation(loc.StartX, loc.EndX, loc.StartY, loc.EndY, z)
    eo.UpdateElPrimaryLocation(elID, newLoc)
}

// GetContextForElID returns the context for the element registered under the given id
func (eo *ElementOrganizer) GetContextForElID(elID ElementID) Context {
    size := eo.Locations[elID].GetSize()
    return NewContext(
        size,
        eo.Visibility[elID],
    )
}

// GetHighestZIndex returns the highest z-index of all elements
// NOTE: the highest z-index is the furthest back visually
func (eo *ElementOrganizer) GetHighestZIndex() ZIndex {
    var highest ZIndex
    for _, locs := range eo.Locations {
        if locs.Location.Z > highest {
            highest = locs.Location.Z
        }
    }
    return highest
}

// GetLowestZIndex returns the lowest z-index of all elements
// NOTE: the lowest z-index is the furthest forward visually
func (eo *ElementOrganizer) GetLowestZIndex() ZIndex {
    var lowest ZIndex
    for _, locs := range eo.Locations {
        if locs.Location.Z < lowest {
            lowest = locs.Location.Z
        }
    }
    return lowest
}

// Receivable returns all of the key combos and commands registered to this
// element organizer, along with their priorities
func (eo *ElementOrganizer) Receivable() []PriorityEv {
    var pe []PriorityEv
    for _, el := range eo.Elements {
        pe1 := el.Receivable()
        pe = append(pe, pe1...)
    }
    return pe
}

// ElIDZIndex holds the z index and id of an element
type ElIDZIndex struct {
    id ElementID
    z  ZIndex
}

// NewElIDZIndex returns a new ElIDZIndex
func NewElIDZIndex(id ElementID, z ZIndex) ElIDZIndex {
    return ElIDZIndex{
        id,
        z,
    }
}

// ElIDZOrder is a slice of ElIDZIndex
// NOTE: it is used to sort the elements by z index
type ElIDZOrder []ElIDZIndex

// XXX TODO determine which way the sort is occurring for z index (high to low
// or low to high??)

// fulfill the sort interface
// NOTE: sorts high to low
func (c ElIDZOrder) Len() int           { return len(c) }
func (c ElIDZOrder) Less(i, j int) bool { return (c)[i].z > (c)[j].z }
func (c ElIDZOrder) Swap(i, j int)      { (c)[i], (c)[j] = (c)[j], (c)[i] }

// AllDrawing executes Drawing functions on all elements in the element
// organizer.
// A DrawChPos slice is returned and passed up the chain to the top of the CUI
// element hierarchy.
// NOTE: the elements are sorted by z-index, from highest to lowest (furthest
// back to furthest forward) and then drawn in that order, such that the element
// with the lowest z-index is drawn last and thus is on top of all others in the
// DrawChPos slice
func (eo *ElementOrganizer) AllDrawing() []DrawChPos {

    out := []DrawChPos{}
    var eoz ElIDZOrder

    for elID := range eo.Elements {
        z := eo.Locations[elID].Z
        eoz = append(eoz, NewElIDZIndex(elID, z))
    }

    sort.Sort(eoz) // sort z index from high to low

    // draw elements in order from highest z-index to lowest
    for _, elIDZ := range eoz {

        ctx := eo.GetContextForElID(elIDZ.id)
        el := eo.GetElementByID(elIDZ.id)
        dcps := el.Drawing(ctx)

        locs := eo.MustGetLocations(elIDZ.id)

        for _, dcp := range dcps {
            dcp.AdjustByLocation(locs.Location)
            out = append(out, dcp)
        }
    }

    return out
}

// write func to remove/add evCombos and commands from EvPrioritizer and
// CommandPrioritizer, using the InputabilityChanges struct
func (eo *ElementOrganizer) ProcessChangesToInputability(elID ElementID, ic InputabilityChanges) {
    eo.Prioritizer.Remove(elID, ic.RmRecEvs)
    eo.Prioritizer.Include(elID, ic.AddRecEvs, eo.PanicOnOverload)
}

// Partially process the event response for whatever is possible to be processed
// in the element organizer. Further processing may be required by the element
// which owns this element organizer.
func (eo *ElementOrganizer) PartiallyProcessEvResp(id ElementID,
    r EventResponse) EventResponse {

    // replace this entire element
    if repl, found := r.GetReplacement(); found {
        ctx := eo.GetContextForElID(id) // get element context
        eo.ReplaceEl(id, repl)
        r.RemoveReplacement()

        // resize replacement
        // TODO may not be neccessary. Explore further w/ fixes to resizing
        el := eo.GetElementByID(id)
        el.ReceiveEvent(ctx, ResizeEvent{})
    }

    if elr, found := r.GetExtraLocations(); found {

        // adjust extra locations to be relative to the given element
        loc := eo.Locations[id] // location of element
        var adjExtraLocs []Location
        for _, l := range elr.extraLocs {
            l.AdjustLocationBy(loc.StartX, loc.StartY)
            adjExtraLocs = append(adjExtraLocs, l)
        }

        // update extra locations
        eo.UpdateExtraLocationsForEl(id, adjExtraLocs)
    }

    if window, found := r.GetWindow(); found && window.HasWindow() {
        eo.ProcessCreateWindow(id, window)
        r.RemoveWindow() // remove from response
    }

    if destruct := r.GetDestruct(); destruct {
        ic := eo.RemoveElement(id)

        r.ConcatInputabilityChanges(ic)
        r.RemoveDestruct()
    }

    return r
}

func (eo *ElementOrganizer) ProcessCreateWindow(id ElementID, cw CreateWindow) {

    // adjust location of window to be relative to the given element
    loc := eo.Locations[id] // location of element
    cw.Loc.AdjustLocationsBy(loc.StartX, loc.StartY)

    eo.AddElement(cw.El, nil, cw.Loc, true)
}

// KeyEventsProcess :
// - determines the appropriate element to send key events to
// - sends the event combo to the element
// - processes changes to the elements receivable events
func (eo *ElementOrganizer) KeyEventsProcess(evs []*tcell.EventKey) (
    ElementID, EventResponse) {

    // determine elementID to send events to
    elID := eo.Prioritizer.GetDestinationEl(evs)
    if elID == NoElement {
        return NoElement, EventResponse{}
    }

    el := eo.GetElementByID(elID) // get element
    if el == nil {
        panic("no ID associated with destination el")
    }
    ctx := eo.GetContextForElID(elID) // get element context

    // send EventKeys to element w/ context
    _, r := el.ReceiveEvent(ctx, KeysEvent(evs))
    r = eo.PartiallyProcessEvResp(elID, r)
    //Debug("\n\nEO.KeyEventsProcess: r.IC: %v\n", r.IC)
    if ic, found := r.GetInputabilityChanges(); found {
        eo.ProcessChangesToInputability(elID, ic)
    }
    return elID, r
}

// Refresh does the following:
// - updates prioritizers
// - triggers a resize event in all children.
// This essentially refreshes the state of the element organizer.
//
// NOTE: the refresh allows for less meticulous construction of the
// main.go file. Elements can be added in whatever order, so long as
// MainEl.Refresh() is called after all elements are added.
func (eo *ElementOrganizer) Refresh(ctx Context) {

    // reset prioritizers
    eo.Prioritizer = NewEvPrioritizer()

    for elID, el := range eo.Elements {

        // refresh all children
        elCtx := eo.GetContextForElID(elID)
        el.ReceiveEvent(elCtx, RefreshEvent{})
        el.ReceiveEvent(elCtx, ResizeEvent{})

        // update prioritizers w/ all receivable events/cmds
        pe := el.Receivable()
        eo.Prioritizer.Include(elID, pe, eo.PanicOnOverload)
    }
}

// Replaces the element at the given ID with a new element
func (eo *ElementOrganizer) ReplaceEl(elID ElementID, newEl Element) (
    ic InputabilityChanges) {

    oldEl := eo.GetElementByID(elID) // get the element being replaced

    // remove all events to old element from the prioritizers

    evs := ToPrioritizableEvs(oldEl.Receivable())
    eo.Prioritizer.Remove(elID, evs)
    ic.RemoveEvs(evs)

    // register new element to organizer under ID of old element
    eo.Elements[elID] = newEl

    // register all events of new element to the prioritizers
    newEvs := newEl.Receivable()
    eo.Prioritizer.Include(elID, newEvs, eo.PanicOnOverload)
    ic.AddEvs(newEvs)

    return ic // pass back upward changes to inputability
}

// ChangePriorityForEl updates a child element (elId) to a new priority. It does
// this by asking the child element to return its registered events w/
// priorities updated to a given priority.
func (eo *ElementOrganizer) ChangePriorityForEl(elID ElementID,
    pr Priority) InputabilityChanges {

    // this ic is the ic for THIS element organizer (not the child
    // element)
    var ic InputabilityChanges

    el := eo.GetElementByID(elID) // get element
    ctx := eo.GetContextForElID(elID)

    ic = el.ChangePriority(ctx, pr) // change priority of element
    eo.ProcessChangesToInputability(elID, ic)

    return ic
}

// MouseEventProcess :
// - determines the appropriate element to send mouse events to
// - sends the event to the element
// - processes changes to the element's receivable events
func (eo *ElementOrganizer) MouseEventProcess(ev *tcell.EventMouse) (
    elID ElementID, evResp EventResponse) {

    ezo := eo.getElIDZOrderUnderMouse(ev)

    if len(ezo) == 0 {
        return NoElement, EventResponse{}
    }

    // get highest z element that falls under mouse event
    // NOTE: a reverse sort is required as the default use of the sort is to
    // draw elements in order of highest to lowest z so that lower z
    // elements sit on top of higher
    sort.Sort(sort.Reverse(ezo))
    e := ezo[0] // moused element (ElIDZIndex) with highest z

    el := eo.GetElementByID(e.id)
    locs := eo.Locations[e.id]
    ctx := eo.GetContextForElID(e.id)

    // adjust event to the relative position of the element
    evAdj := locs.AdjustMouseEvent(ev)

    // send mouse event to element
    _, evResp = el.ReceiveEvent(ctx, MouseEvent(evAdj))

    if ic, found := evResp.GetInputabilityChanges(); found {
        eo.ProcessChangesToInputability(e.id, ic)
    }
    evResp = eo.PartiallyProcessEvResp(e.id, evResp)

    // move element to top of z-dim if primary click
    if ev.Buttons() == tcell.Button1 {
        eo.UpdateElZIndex(e.id, 0)
    }

    // send the mouse event as an external event to all other elements
    for id, el := range eo.Elements {
        if id == e.id {
            continue
        }

        // combine the event responses from the elements that receive the event
        // and all the elements that receive an external event

        _, r := el.ReceiveEvent(ctx, ExternalMouseEvent(ev))

        if ic, found := r.GetInputabilityChanges(); found {
            eo.ProcessChangesToInputability(id, ic)
        }
        rProc := eo.PartiallyProcessEvResp(id, r)

        // combine the inputability changes, all other external responses are
        // ignored
        //evResp.Concat(rProc)
        if ic, found := rProc.GetInputabilityChanges(); found {
            evResp.ConcatInputabilityChanges(ic)
        }
    }

    return e.id, evResp
}

// GetElIDAtZIndex returns the element ID at the given z index, or NoElement if
// no element exists at the given z index
func (eo *ElementOrganizer) GetElIDAtZIndex(z ZIndex) ElementID {
    for id, loc := range eo.Locations {
        if loc.Z == z {
            return id
        }
    }
    return NoElement
}

// IncrementZIndexForElID increments the z-index of the element with the given id,
// pushing it further back in the visual stack.
// NOTE: If an element already occupies the index that the given element is
// attempting to occupy, the element occupying the index will be pushed back as
// well.
// To move an element in the z-dimension, relative to other elements, use
// UpdateZIndexForElID
func (eo *ElementOrganizer) IncrementZIndexForElID(elID ElementID) {

    z := eo.Locations[elID].Z // current z of element

    // check if element exists at next z-index
    if eo.IsZIndexOccupied(z + 1) {

        // recursively increment z-index of element at next z-index
        id := eo.GetElIDAtZIndex(z + 1)
        eo.IncrementZIndexForElID(id)
    }

    // increment z-index of element
    eo.UpdateElZIndex(elID, z+1)
}

// IsZIndexOccupied returns true if an element exists at the given z-index
func (eo *ElementOrganizer) IsZIndexOccupied(z ZIndex) bool {
    for _, locs := range eo.Locations {
        if locs.Z == z {
            return true
        }
    }
    return false
}

// SetVisibilityForEl sets the Visibility of the given element ID
func (eo *ElementOrganizer) SetVisibilityForEl(elID ElementID, visible bool) {
    eo.Visibility[elID] = visible
}

// UpdateExtraLocationsForEl updates the extra locations for the given element
func (eo *ElementOrganizer) UpdateExtraLocationsForEl(id ElementID,
    extraLocations []Location) {

    newLocs := NewLocations(eo.Locations[id].Location, extraLocations)
    eo.Locations[id] = newLocs
}

// ReceiveCommandEvent attempts to execute the given command
func (eo *ElementOrganizer) ReceiveCommandEvent(ev CommandEvent,
) (cmdExecuted bool, resp EventResponse) {

    elID := eo.Prioritizer.GetDestinationEl(ev.Cmd)
    if elID == NoElement {
        return false, EventResponse{}
    }
    el := eo.GetElementByID(elID)
    ctx := eo.GetContextForElID(elID) // get context for element

    cmdExecuted, resp = el.ReceiveEvent(ctx, ev)
    if ic, found := resp.GetInputabilityChanges(); found {
        eo.ProcessChangesToInputability(elID, ic)
    }
    resp = eo.PartiallyProcessEvResp(elID, resp)
    return cmdExecuted, resp
}

// SetZIndexForElement sets the z index of the given element
func (eo *ElementOrganizer) SetZIndexForElement(elID ElementID, z ZIndex) {
    oldLoc := eo.Locations[elID]
    newLoc := NewLocation(oldLoc.StartX, oldLoc.EndX, oldLoc.StartY, oldLoc.EndY, z)
    newLocs := NewLocations(newLoc, oldLoc.Extra)
    eo.Locations[elID] = newLocs
}

// getElIDZOrderUnderMouse returns an ElIDZOrder of all Elements whose locations
// include the position of the mouse event
func (eo *ElementOrganizer) getElIDZOrderUnderMouse(ev *tcell.EventMouse) ElIDZOrder {

    // make array to hold all elements that are under the mouse click
    var ezo ElIDZOrder

    // determine what element is being clicked on
    for id := range eo.Elements {

        ctx := eo.GetContextForElID(id)
        locs := eo.Locations[id]

        // check if element is visible
        if !ctx.Visible {
            continue
        }

        // check if mouse click is within location of element
        if locs.Contains(ev.Position()) {
            ezo = append(ezo, NewElIDZIndex(id, locs.Z))
        }
    }

    return ezo
}

*/
