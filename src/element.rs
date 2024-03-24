use {super::cui_location::Size, std::any::Any};

// Context is a struct which contains information about the current context of a
// given element.
// The context of an element is passed to the element during key function calls
// where the element may need to know its size and Visibility.
//
// Additionally, Metadata may be addended to the context to pass additional
// arbitrary information.
pub struct Context {
    pub s: Size,
    pub visible: bool,
    pub metadata: Option<Box<dyn Any>>,
}

// RelocationRequest contains info for moving an element within its context
pub struct RelocationRequest {
    pub up: i32,
    pub down: i32,
    pub left: i32,
    pub right: i32,
}
