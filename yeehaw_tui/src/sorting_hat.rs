use {std::cell::RefCell, std::collections::HashMap, std::rc::Rc};

/// ElementID is a unique identifier nonce assigned to each active element
/// in the tui beginning with 0.
/// NOTE: the id is a nonce in the scope of a particular element organizer
pub type ElementID = String;

/// The sorting hat is the sole entity which assigns element-ids to elements when they are created.
/// The element-id is in the form `<kind>_<nonce>`. The `<nonce>` is an incrementing number.
/// Only one sorting-hat should ever exist in a tui.
/// Displaying the element kind in the id was a design choice to make debugging more clear -
/// the element-id acts as a human readable assigned name for each element.
#[derive(Clone, Debug, Default)]
//                                      < kind       , nonce >
pub struct SortingHat(Rc<RefCell<HashMap<&'static str, u64>>>);

impl SortingHat {
    pub fn create_element_id(&self, kind: &'static str) -> ElementID {
        let nonce =
            if let Some(old_nonce) = self.0.borrow_mut().get(kind) { old_nonce + 1 } else { 0 };
        self.0.borrow_mut().insert(kind, nonce);
        format!("{}_{}", kind, nonce)
    }
}
