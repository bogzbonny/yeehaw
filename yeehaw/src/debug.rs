/// The debugger is a utility intended to be used to write TUI debug information
/// to a seperate file such that debug output may be read while the a TUI is
/// running.
///
/// DebugFilepath is the path to the debug file. If this is empty, no debug
/// information will be written.
/// The debug filepath is specified at the top of the main file of the package
/// being debugged
use {once_cell::sync::Lazy, parking_lot::RwLock, std::fs::OpenOptions, std::io::prelude::*};

//let s = std::fmt::format(std::alloc::__export::format_args!($($arg)*));
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::debug::log(s);
    }}
}

/*
#[macro_export]
macro_rules! impl_element {
    (
        // Name of the struct to implement the trait for, and the field to use
        for $struct_name:ident.$field_name:ident

        // Optional overrides for trait methods
        $(fn $method_name:ident(&$self:ident$(,)* $($arg_name:ident : $arg_ty:ty),*) $(-> $ret:ty)? $body:block)*
    ) => {
        impl Element for $struct_name {
            $(
                // Use custom implementation if provided
                //fn $method_name(, $($arg_name : $arg_ty),*) $(-> $ret)? $body
                fn $method_name($self: &Self, $($arg_name : $arg_ty),*) $(-> $ret)? $body
                //fn $method_name$body
            )*

            // For methods not overridden, default to first field's implementation
            //$(
            //    fn $method_name(&self, $($arg_name : $arg_ty),*) $(-> $ret)? {
            //        self.$field_name.$method_name($($arg_name),*)
            //    }
            //)*

            fn id(&self) -> ElementID {
                self.$field_name.id()
            }

            //fn kind(&self) -> &'static str {
            //    self.$field_name.kind()
            //}

            fn receivable(&self) -> SelfReceivableEvents {
                self.$field_name.receivable()
            }

            fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
                self.$field_name.receive_event_inner(ctx, ev)
            }

            fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
                self.$field_name.change_priority(p)
            }

            //fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
            //    self.$field_name.drawing(ctx)
            //}

            fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
                self.$field_name.get_attribute(key)
            }

            fn set_attribute(&self, key: &str, value: Vec<u8>) {
                self.$field_name.set_attribute(key, value)
            }

            fn set_hook(&self, kind: &str, el_id: ElementID,
                hook: Box<dyn FnMut(&str, Box<dyn Element>)>,
            ) {
                self.$field_name.set_hook(kind, el_id, hook)
            }

            fn remove_hook(&self, kind: &str, el_id: ElementID) {
                self.$field_name.remove_hook(kind, el_id)
            }

            fn clear_hooks_by_id(&self, el_id: ElementID) {
                self.$field_name.clear_hooks_by_id(el_id)
            }

            fn call_hooks_of_kind(&self, kind: &str) {
                self.$field_name.call_hooks_of_kind(kind)
            }

            fn set_parent(&self, up: Box<dyn Parent>) {
                self.$field_name.set_parent(up)
            }

            fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
                self.$field_name.get_dyn_location_set()
            }

            fn get_visible(&self) -> Rc<RefCell<bool>> {
                self.$field_name.get_visible()
            }

            //fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses);
            //fn change_priority(&self, p: Priority) -> ReceivableEventChanges;
            //fn drawing(&self, ctx: &Context) -> Vec<DrawChPos>;
            //fn get_attribute(&self, key: &str) -> Option<Vec<u8>>;
            //fn set_attribute(&self, key: &str, value: Vec<u8>);
            //fn set_hook(&self, kind: &str, el_id: ElementID,
            //    hook: Box<dyn FnMut(&str, Box<dyn Element>)>,
            //);
            //fn remove_hook(&self, kind: &str, el_id: ElementID);
            //fn clear_hooks_by_id(&self, el_id: ElementID);
            //fn call_hooks_of_kind(&self, kind: &str);
            //fn set_parent(&self, up: Box<dyn Parent>);
            //fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>>;
            //fn get_visible(&self) -> Rc<RefCell<bool>>;

            //impl_element!(@default_impl $field_name, $($method_name)* );
            //impl_element!(@default_impl $field_name, [ $($method_name)* ]);
            //impl_element!(@default_impl $field_name, [ $($method_name)* ]);
            impl_element!(@impl_if_missing $field_name, $($method_name)* );

            // Auto-implement default function if not provided
            //if ![$(stringify!($method_name)),*].contains(&"kind") {
            $(
            //    fn kind(&self) -> &'static str {
            //        self.$field_name.kind()
            //    }
            //}
        }
    };

    // Default implementations for specific functions if they weren't provided
    //(@default_impl $name:ident, $field_name:ident, [ $(fn $method_name:ident)* ]) => {
    //(@default_impl $field_name:ident,  $(fn $method_name:ident)* ) => {
    //    $(
    //        impl_element!(@impl_if_missing $field_name, $method_name);
    //    )*
    //};

    (@impl_if_missing $field_name:ident, kind) => {
        fn kind(&self) -> &'static str {
            self.$field_name.kind()
        }
    };

    (@impl_if_missing $field_name:ident, drawing) => {
        fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
            self.$field_name.drawing(ctx)
        }
    };

    // Define no-op for provided functions
    (@impl_if_missing $field_name:ident, $method_name:ident) => {};
}
*/

#[derive(Clone)]
pub struct Debugger {
    pub log_file: Option<String>, // if some then output debug to file
    pub enabled: bool,
    pub max_lines: usize,
    pub lines: Vec<String>,
}

static GLOBAL_DEBUGGER: Lazy<RwLock<Debugger>> = Lazy::new(|| {
    RwLock::new(Debugger {
        log_file: None,
        enabled: true,
        max_lines: 300,
        lines: Vec::new(),
    })
});

pub fn get_content() -> String {
    GLOBAL_DEBUGGER.read().lines.join("\n")
}
pub fn get_max_lines() -> usize {
    (GLOBAL_DEBUGGER.read()).max_lines
}
pub fn is_enabled() -> bool {
    (GLOBAL_DEBUGGER.write()).enabled
}

pub fn enable() {
    (GLOBAL_DEBUGGER.write()).enabled = true;
}

pub fn disable() {
    (GLOBAL_DEBUGGER.write()).enabled = false;
}

pub fn set_log_file(file: String) {
    (GLOBAL_DEBUGGER.write()).log_file = Some(file);
}

pub fn log(content: String) {
    if !GLOBAL_DEBUGGER.read().enabled {
        return;
    }

    let mut lines = GLOBAL_DEBUGGER.read().lines.clone();
    let max_lines = GLOBAL_DEBUGGER.read().max_lines;
    lines.push(content.clone());
    if lines.len() > max_lines {
        lines.remove(0);
    }
    (GLOBAL_DEBUGGER.write()).lines = lines;

    // push to file
    if let Some(file) = GLOBAL_DEBUGGER.read().log_file.clone() {
        // append content to new line at end of file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
            .unwrap();
        writeln!(file, "{content}").unwrap();
    }
}

pub fn clear() {
    (GLOBAL_DEBUGGER.write()).lines.clear();

    // clear file
    if let Some(file) = &(GLOBAL_DEBUGGER.write()).log_file {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file)
            .unwrap();
        file.set_len(0).unwrap();
    }
}
