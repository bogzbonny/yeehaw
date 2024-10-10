use {std::cell::RefCell, std::collections::HashMap, std::rc::Rc};

// The sorting hat is the sole entity which assigns element-ids to elements when they are created.
// the element-id is in the form <kind>_<nonce>. The <nonce> is an incrementing number.
// only one sorting-hat should ever exist in a cui.
#[derive(Clone, Default)]
//                                      < kind       , nonce >
pub struct SortingHat(Rc<RefCell<HashMap<&'static str, u64>>>);

// ElementID is a unique identifier nonce assigned to each active element
// in the cui beginning with 0.
// NOTE: the id is a nonce in the scope of a particular element organizer
pub type ElementID = String;

impl SortingHat {
    pub fn create_element_id(&self, kind: &'static str) -> ElementID {
        let nonce =
            if let Some(old_nonce) = self.0.borrow_mut().get(kind) { old_nonce + 1 } else { 0 };
        self.0.borrow_mut().insert(kind, nonce);
        format!("{}_{}", kind, nonce)
    }
}
