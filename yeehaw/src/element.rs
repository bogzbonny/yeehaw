use {
    crate::{
        prioritizer::Priority, Context, DrawChPos, DynLocation, DynLocationSet, ElementID, Event,
        EventResponses, ReceivableEventChanges, SelfReceivableEvents,
    },
    dyn_clone::DynClone,
    std::{
        cell::{Ref, RefCell},
        rc::Rc,
    },
};

dyn_clone::clone_trait_object!(Element);

/// Element is the base interface which all viewable elements are
/// expected to fulfill
pub trait Element: DynClone {
    /// TODO consider removing kind
    fn kind(&self) -> &'static str;
    /// a name for the kind of the element

    fn id(&self) -> ElementID;
    /// the element id as assigned by the SortingHat

    /// Returns all event kinds (key events and commands, etc.) which are receivable by the element.
    /// This includes all events that are registered to the element itself, as well as its children,
    /// via its ElementOrganizer (if it has one).
    ///
    /// NOTE in this current design, elements are always routed mouse events independently of
    /// whether or not they are receivable according to this function.
    fn receivable(&self) -> SelfReceivableEvents;

    /// Receive an event from a parent. The receiving element may consume the event and/or pass it
    /// to a child. The element is expected to return a response to the event, along with any
    /// changes receivable events. When the event is captured, the element is expected to returns
    /// captured=true.
    ///                                                      (captured, response     )
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses);

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.call_hooks_of_kind(PRE_EVENT_HOOK_NAME);
        let (captured, resp) = self.receive_event_inner(ctx, ev);
        self.call_hooks_of_kind(POST_EVENT_HOOK_NAME);
        (captured, resp)
    }

    /// change_priority is expected to change the priority of an element relative to its parents.
    /// All receivable-events registered directly by the element should have their local priority
    /// changed to 'p' while everything registered in this element's prioritizers will remain
    /// unchanged (if this element is a parent element). The element will then respond with the
    /// appropriate changes to its receivable events through the function return variable.
    ///
    /// If the priority is changing from Focused to Unfocused, then the element should respond with
    /// ReceivableEventChanges where all events (local, and of its children) are set to Unfocused.
    ///
    /// If the priority is changed from Unfocused to Focused, then the element should respond with
    /// ReceivableEventChanges with the element's local receivable events set to Focused, and all of
    /// the elements children's receivable events set to whatever their local priority is.
    ///
    /// In all cases the reponse of this function is intended to be passed to the element's
    /// parent's event prioritizer.
    fn change_priority(&self, p: Priority) -> ReceivableEventChanges;

    /// Get the element's full drawing for the provided context.
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos>;

    /// Element attributes can be used to store arbitrary values (as encoded bytes) within the
    /// element. Typically if you are developing a new Element, you can simply store local values
    /// within your struct, only in the situation where you a creating a new sub-class of Elements
    /// (such as Widgets) does it make sense to be utilizing the get/set attribute functions.
    ///
    /// Current attributes used within this library:
    ///  - "description": a string description of the element used everywhere
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>>;

    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        let pre_hook_name = format!("{PRE_ATTR_CHANGE_HOOK_NAME_PREFIX}{key}");
        let post_hook_name = format!("{POST_ATTR_CHANGE_HOOK_NAME_PREFIX}{key}");
        self.call_hooks_of_kind(&pre_hook_name);
        self.set_attribute_inner(key, value);
        self.call_hooks_of_kind(&post_hook_name);
    }

    fn set_attribute_inner(&self, key: &str, value: Vec<u8>);

    /// sets the hook for the element, the hook is a function that is called when the element is
    /// although a developer may implement any custom hook kind, the default hooks are:
    ///  - "pre-visible-change": called before the element visibility changes
    ///  - "post-visible-change": called before the element visibility changes
    ///  - "pre-event": called before the element receives an event
    ///  - "post-event": called after the element receives an event
    ///  - "pre-location-change": called before the element location changes
    ///  - "post-location-change": called after the element location changes
    ///
    /// NOTE use caution when setting hooks, they can be used to create circular references between elements
    /// el_id is the element-id of the element registering the hook to THIS element
    /// the hook is a function that takes the kind of the hook and the hooked element

    #[allow(clippy::type_complexity)]
    fn set_hook(
        &self,
        kind: &str,
        el_id: ElementID,
        //                  kind, hooked element
        hook: Box<dyn FnMut(&str, Box<dyn Element>)>,
    );

    fn remove_hook(&self, kind: &str, el_id: ElementID);

    /// remove all hooks for the element with the given id
    fn clear_hooks_by_id(&self, el_id: ElementID);

    /// calls all the hooks of the provided kind
    fn call_hooks_of_kind(&self, kind: &str);

    /// Assign a reference to the element's parent through the Parent trait. This is used
    /// to pass ReceivableEventChanges to the parent. (see UpwardPropogator for more context)
    fn set_parent(&self, up: Box<dyn Parent>);

    /// get/set the scalable location of the widget
    fn get_dyn_location_set(&self) -> Ref<DynLocationSet>;
    fn get_visible(&self) -> bool;

    fn set_dyn_location_set(&self, l: DynLocationSet) {
        self.call_hooks_of_kind(PRE_LOCATION_CHANGE_HOOK_NAME);
        *self.get_ref_cell_dyn_location_set().borrow_mut() = l;
        self.call_hooks_of_kind(POST_LOCATION_CHANGE_HOOK_NAME);
    }

    fn set_dyn_location(&self, l: DynLocation) {
        self.call_hooks_of_kind(PRE_LOCATION_CHANGE_HOOK_NAME);
        self.get_ref_cell_dyn_location_set().borrow_mut().l = l;
        self.call_hooks_of_kind(POST_LOCATION_CHANGE_HOOK_NAME);
    }

    fn set_dyn_location_extra(&self, extra: Vec<DynLocation>) {
        self.call_hooks_of_kind(PRE_LOCATION_CHANGE_HOOK_NAME);
        self.get_ref_cell_dyn_location_set().borrow_mut().extra = extra;
        self.call_hooks_of_kind(POST_LOCATION_CHANGE_HOOK_NAME);
    }

    fn set_visible(&self, v: bool) {
        self.call_hooks_of_kind(PRE_VISIBLE_CHANGE_HOOK_NAME);
        *self.get_ref_cell_visible().borrow_mut() = v;
        self.call_hooks_of_kind(POST_VISIBLE_CHANGE_HOOK_NAME);
    }

    /// gets the reference to the location set and visible.
    /// The intention is so that these can be read without requiring a mutable reference to the element
    /// however, the element should not be modified through these references, if this is done then
    /// the relevant hooks will not be called when values are set.
    /// NOTE these functions should NOT be used to set values, use the set functions below to ensure
    /// that hooks are called.
    fn get_ref_cell_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>>;
    fn get_ref_cell_visible(&self) -> Rc<RefCell<bool>>;

    // -------------------------------------------------------
    /// used by scrollbars

    fn set_content_x_offset(&self, ctx: &Context, x: usize);
    fn set_content_y_offset(&self, ctx: &Context, y: usize);
    fn get_content_x_offset(&self) -> usize;
    fn get_content_y_offset(&self) -> usize;
    fn get_content_width(&self) -> usize;
    fn get_content_height(&self) -> usize;

    // -------------------------------------------------------
    /// Freebies

    fn with_description(self, desc: String) -> Self
    where
        Self: Sized,
    {
        self.set_description(desc);
        self
    }

    fn get_description(&self) -> Option<String> {
        let bz = self.get_attribute(ATTR_DESCRIPTION)?;
        match serde_json::from_slice(&bz) {
            Ok(v) => v,
            Err(_e) => None,
        }
    }

    fn set_description(&self, desc: String) {
        let bz = match serde_json::to_vec(&desc) {
            Ok(v) => v,
            Err(_e) => {
                return; // TODO log error
            }
        };
        self.set_attribute(ATTR_DESCRIPTION, bz)
    }
}

pub type HookFn = Box<dyn FnMut(&str, Rc<RefCell<dyn Element>>)>;

pub const ATTR_DESCRIPTION: &str = "standard_pane";

pub const PRE_VISIBLE_CHANGE_HOOK_NAME: &str = "pre-visible-change";
pub const POST_VISIBLE_CHANGE_HOOK_NAME: &str = "post-visible-change";
pub const PRE_EVENT_HOOK_NAME: &str = "pre-event";
pub const POST_EVENT_HOOK_NAME: &str = "post-event";
pub const PRE_LOCATION_CHANGE_HOOK_NAME: &str = "pre-location-change";
pub const POST_LOCATION_CHANGE_HOOK_NAME: &str = "post-location-change";
/// prefixes because the actual attribute key is appended
pub const PRE_ATTR_CHANGE_HOOK_NAME_PREFIX: &str = "pre-attr-change-";
/// prefixes because the actual attribute key is appended
pub const POST_ATTR_CHANGE_HOOK_NAME_PREFIX: &str = "post-attr-change-";

// ----------------------------------------

dyn_clone::clone_trait_object!(Parent);

pub trait Parent: dyn_clone::DynClone {
    /// DO NOT CALL THIS FUNCTION DIRECTLY
    /// This function is intended for internal propogation ONLY if you need to propogate changes
    /// use the function: send_responses_upward found in Pane and ParentPane
    ///
    /// The Parent is a trait that a parent element should fulfill which can then be
    /// provided to child elements as a means for those child elements to propagate changes upward
    /// to their parent (and grand-parents etc.).
    ///
    /// In most cases, receivable event changes are passed to the parent in the return values of a
    /// function invoked on the element by the parent (ex. ReceiveEvent). However, when changes are
    /// initiated through hooks of non-parent elements, the parent must be notified of the changes
    /// from the child directly. By providing this trait to a child element it enables it to propagate
    /// receivable event changes when it hasn't been modified directly by its parent.
    ///
    /// For instance, a file-navigator may with to initiate a content change in an adjacent
    /// content-pane in this case it could activate the content-pane and deactivate itself, this
    /// newly activated content-pane would need a way to inform its parent pane of its receivable
    /// event changes.
    ///
    /// child_el_id is the child element-id which is invoking the propagation from BELOW the element
    /// fulfilling the Parent trait. This is used by the parent element (this one) to
    /// determine which events/cmds to update the prioritizers for.
    fn propagate_responses_upward(
        &self, parent_ctx: &Context, child_el_id: &ElementID, resps: EventResponses,
    );

    /// The parent's store is a cool way for child elements to store common information among it's parent.
    /// This is a useful mechanism for child elements to communicate with their parent without having to
    /// pass information directly between each other. Additionally the store can survive the destruction
    /// of a child element so... well that's a feature to consider.
    fn get_store_item(&self, key: &str) -> Option<Vec<u8>>;
    fn set_store_item(&self, key: &str, value: Vec<u8>);

    /// Get the priority of the parent element, useful for processing in the organizer.
    fn get_priority(&self) -> Priority;
}
