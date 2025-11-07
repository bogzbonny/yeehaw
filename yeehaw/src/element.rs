use {
    crate::{
        Context, DrawChPos, DrawRegion, DynLocation, DynLocationSet, ElementID, Event,
        EventResponses, Label, ReceivableEvents, ZIndex,
    },
    dyn_clone::DynClone,
    std::{
        cell::{Ref, RefCell},
        rc::Rc,
    },
};

//   ELEMENT FARMER       ✲
//                          /|\      *
//   ⌂  ⌂  ⌂         ✲      \|/   /  *  \
//                  ✲            * time  *
//   water      ~  _|_  ~         \  *  /      ⌃
//   light        /   \              *       \   /
//   nutrience  ./ 6 6 \.  hi,             discovery
//   eneergy        ~      dont u d4re       /   \
//   darkness        \       munch my crops    ⌄
//                    -<<<-
//      |    |    |    |    |    |    |    |     f
//     \|/  \|/  \|/  \|/  \|/  \|/  \|/  \|/  \ o /
//     \|/  \|/  \:)  \|/  \|\  \|/  \|/  \|/  \ c /
//     \|/  \|/  \|/  \|/  \|/  \|/  \|/  \|/  \ u /
//      |    |    |    | oo |    |    |    |     s

dyn_clone::clone_trait_object!(Element);

/// Element is the base interface which all viewable elements are
/// expected to fulfill
pub trait Element: DynClone {
    /// TODO consider removing kind
    /// a name for the kind of the element
    fn kind(&self) -> &'static str;

    /// the element id as assigned by the SortingHat
    fn id(&self) -> ElementID;

    /// can the element receive the event provided
    fn can_receive(&self, ev: &Event) -> bool;

    /// get the receivable events for the element
    /// TODO it'd be nicer to return an iterator here, bit of a pain to make Element clonable then though.
    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>>;

    /// Receive an event from a parent. The receiving element may consume the event and/or pass it
    /// to a child. The element is expected to return a response to the event, along with any
    /// changes receivable events. When the event is captured, the element is expected to returns
    /// captured=true.
    //                                                     (captured, response      )
    #[must_use]
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses);

    fn set_focused(&self, focused: bool);
    fn get_focused(&self) -> bool;

    /// Get the element's full drawing for the provided context.
    /// if force update is set to true then an DrawUpdate should be provided regardless of
    /// if the element has changed since the last draw
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate>;

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
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: HookFn);

    fn remove_hook(&self, kind: &str, el_id: ElementID);

    /// remove all hooks for the element with the given id
    fn clear_hooks_by_id(&self, el_id: ElementID);

    /// calls all the hooks of the provided kind
    fn call_hooks_of_kind(&self, kind: &str);

    /// Assign a reference to the element's parent through the Parent trait. This is used
    /// to pass ReceivableEventChanges to the parent. (see UpwardPropogator for more context)
    fn set_parent(&self, up: Box<dyn Parent>);

    /// get/set the scalable location of the widget
    fn get_dyn_location_set(&self) -> Ref<'_, DynLocationSet>;
    fn get_visible(&self) -> bool;

    fn get_z(&self) -> ZIndex {
        self.get_dyn_location_set().z
    }

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
    fn get_ref_cell_overflow(&self) -> Rc<RefCell<bool>>;

    // -------------------------------------------------------
    /// used by scrollbars
    fn set_content_x_offset(&self, dr: Option<&DrawRegion>, x: usize);
    fn set_content_y_offset(&self, dr: Option<&DrawRegion>, y: usize);
    fn get_content_x_offset(&self) -> usize;
    fn get_content_y_offset(&self) -> usize;
    fn get_content_width(&self, dr: Option<&DrawRegion>) -> usize;
    fn get_content_height(&self, dr: Option<&DrawRegion>) -> usize;

    // -------------------------------------------------------
    // Freebies

    fn with_description(self, desc: String) -> Self
    where
        Self: Sized,
    {
        self.set_description(desc);
        self
    }

    fn get_description(&self) -> Option<String> {
        let bz = self.get_attribute(ATTR_DESCRIPTION)?;
        serde_json::from_slice(&bz).unwrap_or_default()
    }

    fn set_description(&self, desc: String) {
        let bz = match serde_json::to_vec(&desc) {
            Ok(v) => v,
            Err(e) => {
                log_err!("failed to serialize description: {}", e);
                return; // TODO log error
            }
        };
        self.set_attribute(ATTR_DESCRIPTION, bz)
    }

    /// create a label for this element
    fn label(&self, ctx: &Context, label: &str) -> Label {
        Label::new_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_above_right(&self, ctx: &Context, label: &str) -> Label {
        Label::new_above_right_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_above(&self, ctx: &Context, label: &str) -> Label {
        Label::new_above_left_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_above_left(&self, ctx: &Context, label: &str) -> Label {
        Label::new_above_left_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_below_right(&self, ctx: &Context, label: &str) -> Label {
        Label::new_below_right_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_below_left(&self, ctx: &Context, label: &str) -> Label {
        Label::new_below_left_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_below(&self, ctx: &Context, label: &str) -> Label {
        Label::new_below_left_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_left(&self, ctx: &Context, label: &str) -> Label {
        Label::new_left_top_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_left_top(&self, ctx: &Context, label: &str) -> Label {
        Label::new_left_top_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_left_bottom(&self, ctx: &Context, label: &str) -> Label {
        Label::new_left_bottom_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_right_top(&self, ctx: &Context, label: &str) -> Label {
        Label::new_right_top_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_right(&self, ctx: &Context, label: &str) -> Label {
        Label::new_right_top_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
    fn label_right_bottom(&self, ctx: &Context, label: &str) -> Label {
        Label::new_right_bottom_for_el(ctx, self.get_dyn_location_set().l.clone(), label)
    }
}

pub type HookFn = Box<dyn FnMut(&str, Box<dyn Element>)>;

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

pub trait Parent: DynClone {
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
    fn get_parent_focused(&self) -> bool;

    fn get_id(&self) -> ElementID;
}

// -----------------------------------------------------

#[derive(Clone, Debug)]
/// The DrawUpdate is a primitive type used to convey draw updates of an element.
/// A sub-id is provided to allow for an element to sub-divide its draw updates into
/// sub-sections. This is useful for container elements which contain sub-elements which
/// may only be updated individually.
pub struct DrawUpdate {
    /// sub element-id attributed to these changes too. This is useful for container elements which
    /// contain sub-elements which may only be updated individually.
    /// For non-container elements, this should just be an empty vector.
    pub sub_id: ElementIDPath,

    /// cooresponding z-index for each layer of element-id
    /// NOTE we need to use the entire z-index path (instead of just the upper most z index)
    ///      to be able to resolve the upper drawing order of elements where the upper most
    ///      z-indicies are the same. This will occur for container elements.
    pub z_indicies: ZIndexPath,

    /// The draw update action to take
    pub action: DrawAction,
}

/// A single combined id (for one el) which has all the sub-ids concatenated together
pub type ElementIDPath = Vec<ElementID>;

/// a single combined z-index (for one el) which has all the z-indices concatenated together
pub type ZIndexPath = Vec<ZIndex>;

impl From<DrawUpdate> for Vec<DrawUpdate> {
    fn from(d: DrawUpdate) -> Self {
        vec![d]
    }
}

#[derive(Clone, Debug)]
pub enum DrawAction {
    /// delete everything at or prefixed with this sub_id
    ClearAll,

    /// deletes everything at this sub_id, does not effect
    /// other items with this sub_id prefix
    Remove,

    /// remove-all then add DrawChPos's
    Update(Vec<DrawChPos>),

    /// extend to the DrawChPos's at the sub_id.
    /// no old draw items are removed.
    Extend(Vec<DrawChPos>),
}

impl DrawUpdate {
    pub fn clear_all() -> Self {
        Self {
            sub_id: Vec::new(),
            z_indicies: Vec::new(),
            action: DrawAction::ClearAll,
        }
    }

    pub fn remove() -> Self {
        Self {
            sub_id: Vec::new(),
            z_indicies: Vec::new(),
            action: DrawAction::Remove,
        }
    }

    pub fn update(updates: Vec<DrawChPos>) -> Self {
        Self {
            sub_id: Vec::new(),
            z_indicies: Vec::new(),
            action: DrawAction::Update(updates),
        }
    }

    pub fn extend(updates: Vec<DrawChPos>) -> Self {
        Self {
            sub_id: Vec::new(),
            z_indicies: Vec::new(),
            action: DrawAction::Extend(updates),
        }
    }

    pub fn clear_all_at_sub_id(sub_id: Vec<ElementID>) -> Self {
        Self {
            sub_id,
            z_indicies: Vec::new(),
            action: DrawAction::ClearAll,
        }
    }

    pub fn remove_at_sub_id(sub_id: Vec<ElementID>) -> Self {
        Self {
            sub_id,
            z_indicies: Vec::new(),
            action: DrawAction::Remove,
        }
    }

    pub fn prepend_id(&mut self, id: ElementID, z: ZIndex) {
        self.sub_id.insert(0, id);
        self.z_indicies.insert(0, z);
    }
}
