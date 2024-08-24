use {
    crate::{
        element::ReceivableEventChanges, Context, DrawCh, DrawChPos, DrawChs2D, Element, ElementID,
        Event, EventResponses, Priority, SortingHat, UpwardPropagator,
    },
    std::{
        collections::HashMap,
        ops::{Deref, DerefMut},
        {cell::RefCell, rc::Rc},
    },
};

// ParentPane is a pane element which other objects can embed and build off
// of. It is a pane which can have children panes.
#[derive(Clone)]
pub struct ParentPane {
    pub pane: StandardPane,
    pub eo: Rc<RefCell<ElementOrganizer>>,
}

impl ParentPane {
    pub fn new(hat: &SortingHat, kind: &'static str) -> StandardPane {
        let pane = StandardPane::new(hat, kind);
        ParentPane {
            pane,
            eo: Rc::new(RefCell::new(ElementOrganizer::default())),
        }
    }
}

/*

func (pp *ParentPane) ReceiveEvent(ctx yh.Context, ev interface{}) (
    captured bool, resp yh.EventResponse) {
    switch ev := ev.(type) {
    case yh.RefreshEvent:
        pp.EO.Refresh(ctx)
        return false, resp // resize events aren't captured
    default:
        return pp.StandardPane.ReceiveEvent(ctx, ev)
    }
}

// This function is primarily so that the parent pane fulfills the element
// interface.
//
// NOTE this function should NOT be used if the parent pane is used as a base for a
// more complex element. As the developer you should be fulfilling the
// PropagateUpwardChangesToInputability function directly in your element and
// utilizing the PropagateUpwardChangesToInputabilityHelper function (below).
// For example the standard horizontal pane element does this:
//
// func (shp *StandardHorizontalPanes) PropagateUpwardChangesToInputability(
//
//	   childEl Element, ic InputabilityChanges, updateThisElementsPrioritizers bool) {
//
//	   shp.PropagateUpwardChangesToInputabilityHelper(
//	       childEl, shp, ic, updateThisElementsPrioritizers)
//	}
func (pp *ParentPane) PropagateUpwardChangesToInputability(childEl yh.Element,
    ic yh.InputabilityChanges, updateThisElementsPrioritizers bool) {

    // process changes in pp's element organizer and pass the changes to
    // pp's parent element
    pp.PropagateUpwardChangesToInputabilityHelper(childEl, pp, ic,
        updateThisElementsPrioritizers)
}

// Passes changes to inputability to this element's parent element.
// If updateThisElementsPrioritizers is TRUE then this element's prioritizers
// should be updated using the given IC. This should be set to false when an
// upwards propagation is being initiated as all of the changes to the
// prioritzers should have already been handled. The boolean should be set to
// true on all further calls as the changes are propagated upstream so as to
// update the ancestors' prioritizers.
//
// childEl is the element which is invoking the propagation from BELOW this
// parent pane. This is used by the parent to determine which events/cmds to
// update the prioritizers for.
//
// The propagateEl is the element to send further upward propagation to.
// Typically this means the Element which is inheriting THIS parent pane.
// NOTE: propagateEl is necessary as the parent pane will usually have
// registered an element that extends ParentPane. If this ParentPane sent
// itself, it would not match the child registered in the parent's EO.
func (pp *ParentPane) PropagateUpwardChangesToInputabilityHelper(childEl,
    propagateEl yh.Element, ic yh.InputabilityChanges,
    updateThisElementsPrioritizers bool) {

    if updateThisElementsPrioritizers {
        // process changes in element organizer
        childElID := pp.EO.GetIDFromEl(childEl)

        pp.EO.ProcessChangesToInputability(childElID, ic)
    }

    pp.UP.PropagateUpwardChangesToInputability(propagateEl, ic, true)
}

// ReceivablePriorityKeyCombosAndCommands returns the event keys and commands
// that can be received by the element, along with their priorities
func (pp *ParentPane) Receivable() []yh.PriorityEv {
    pes := pp.PerceivedPrioritiesOfEO()  // Registered Receivable Events
    pes2 := pp.StandardPane.Receivable() // Self Receivable Events
    return append(pes, pes2...)
}

// TODO consolidate switch statement logic with logic from ChangePriority.
func (pp *ParentPane) PerceivedPrioritiesOfEO() []yh.PriorityEv {
    pr := pp.ElementPriority
    pes := pp.EO.Receivable() // Registered Receivable Events
    return GeneratePerceivedPriorities(pr, pes)
}

// ChangePriority returns a priority change (InputabilityChanges) to its
// parent organizer so as to update the priority of all events registered to
// this element.
//
// NOTE: The priority changes (ic) that this parent pane sends up is the
// combination of:
//   - this element's priority changes (the SelfEvs, aka the
//     Self Receivable Events)
//   - the "perceived priorities" of the childens' Receivable Events
//     (aka the results of the child's Receivable() function) The "perceived
//     priorities" are the effective priority FROM the perspective of the
//     element ABOVE this element in the tree.
func (pp *ParentPane) ChangePriority(ctx yh.Context, pr yh.Priority) yh.InputabilityChanges {

    // first change the priority of the self evs. These are "this elements
    // priority changes". NO changes should be made to the childen,
    // the perceived priorities of the children should be interpreted.
    ic := pp.StandardPane.ChangePriority(ctx, pr)

    // update the perceived priorities of the children
    for _, el := range pp.EO.Elements {
        pes := el.Receivable() // self evs (and eo's evs)

        perceivedPr := GeneratePerceivedPriorities(pr, pes)
        for _, pe := range perceivedPr {
            ic.UpdatePriorityForEv(pe.Ev, pe.Pr)
        }
    }

    return ic
}

// GeneratePerceivedPriorities generates the "perceived priorities" of the
// provided events. It receives a function which can then use each perceived
// priority however it needs to.
//
// **IMPORTANT NOTE**
//
// The "perceived priorities" are the effective priorities of an element FROM
// the perspective of an element two or more levels ABOVE the element in the tree.
//
// Relative priorities between the children elements of a parent element
// should be perserved. To ensure this, the priorities of children should
// never be modified but instead interpreted as "perceived priorities".
//
// EXAMPLE:
//
//				  	 Element 0 (AboveFocused)
//				  	  evA (AboveFocused)     ┐
//				      evB (AboveFocused)     ├─[Perceived Priorities]
//				      evC (AboveFocused)     │
//		              evD (HighestFocus)     ┘
//				     	     │
//			                 │
//				  	 Element 1
//				  	  evA (AboveFocused)
//				  	  evB (Focused)
//				      evC (Unfocused)
//				      evD (HighestFocus)
//	                         │
//	            ┌────────────┴───────────┐
//	            │                        │
//		   	Element 2                Element 3
//		   	 evA (AboveFocused)       evC (Unfocused)
//		   	 evB (Focused)            evD (HighestFocus)
//
// This function does not modify the priorities of any child element, but
// instead generates the "perceived priorities" in the following way:
//  1. If the input priority (pr) is Unfocused:
//     - simply interpret all the childrens' priorities as unfocused.
//     (everything set in the ic will be unfocused).
//  2. if the input priority (pr) is Focused or greater:
//     - individually interpret each child's Receivable Event priority as
//     the greatest of either the input priority to this function (pr),
//     or the child event's current priority.
//
// INPUTS
//   - The realPES is the real priority events of the child element.
//   - The parentPr is the priority that the parent element is being changed to
//   - The perceivedPES is the perceived priority events of a child element for
//     this element for this element's parent (the grandparent of the child).
func GeneratePerceivedPriorities(parentPr yh.Priority, realPES []yh.PriorityEv) (perceivedPES []yh.PriorityEv) {

    perceivedPES = make([]yh.PriorityEv, len(realPES)) // make new slice length of pes
    switch {
    case parentPr == yh.Unfocused:
        for i, childPE := range realPES {
            perceivedPES[i] = yh.NewPriorityEv(yh.Unfocused, childPE.Ev)
        }
        // leave the children alone! they're fine

    case parentPr < yh.Unfocused: // "Focused or greater"
        for i, childPE := range realPES {
            switch {
            case childPE.Pr == yh.Unfocused:
                perceivedPES[i] = yh.NewPriorityEv(yh.Unfocused, childPE.Ev)
            case childPE.Pr < parentPr: // if child event PR higher than the parent Pr
                perceivedPES[i] = yh.NewPriorityEv(childPE.Pr, childPE.Ev)
            default: // the parent pr must be higher than the child event pr
                perceivedPES[i] = yh.NewPriorityEv(parentPr, childPE.Ev)
            }
        }
    }
    return perceivedPES
}

// ChangePriorityForEl changes the priority of all evCombos/commands registered to the
// element with the given ID
func (pp *ParentPane) ChangePriorityForEl(elID yh.ElementID,
    p yh.Priority) yh.InputabilityChanges {

    ic := pp.EO.ChangePriorityForEl(elID, p)

    // Check if any of the ic.RmRecEvs match pp.SelfEvs. If so, add those events to
    // the ic.AddRecEvs.
    // NOTE: this is necessary because:
    // 1. An event passed in the ic.RmRecEvs will remove ALL instances of an
    // event registered to the ElementOrganizer (EO) of the parent of this
    // element. This is true because all events in the parent of this element
    // are registered with the ID of THIS element.
    //    e.g. if EventX is being passed in the ic.RmRecEvs and EventX occurs
    //    twice in the prioritizer of the EO of the parent of this element, BOTH
    //    instances of EventX will be removed when the EO processes the
    //    InputabilityChanges.
    // 2. If this element has registered EventX as a SelfEv and EventX is also
    // passed up in the ic.RmRecEvs, then EventX will be removed from the parent
    // organizer and this element will no longer be able to recieve EventX even
    // though it still wants to.

    // NOTE: Leaving the remove event in the ic.RmRecEvs removes artifacts further
    // up the tree. I.e, if we simply remove the event from the ic.RmRecEvs, then
    // the parent of this element will have an artifact registration for an
    // event that serves no purpose.

    // NOTE: If there are duplicate events in the ic.RmRecEvs, then the
    // following code will add duplicate events to the ic.AddRecEvs. This will
    // result in duplicate events registered with the same priority and ID in
    // this element's parent. This seems harmless and is probably more efficient
    // than checking for duplicates.

    for _, rmRecEv := range ic.RmRecEvs {
        for _, selfEv := range pp.SelfEvs {
            if reflect.DeepEqual(rmRecEv, selfEv.Ev) {
                ic.AddEv(selfEv.Ev, selfEv.Pr)
            }
        }
    }

    return ic
    //return pp.EO.ChangePriorityForEl(elID, p)
}

func (pp *ParentPane) GetElementByID(id yh.ElementID) yh.Element {
    return pp.EO.GetElementByID(id)
}

// SetZIndexForElement sets the z-index of the element with the given ID
func (pp *ParentPane) SetZIndexForElement(elID yh.ElementID, z yh.ZIndex) {
    pp.EO.SetZIndexForElement(elID, z)
}
*/
