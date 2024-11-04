use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ImplItemFn, ItemImpl};

#[proc_macro_attribute]
pub fn impl_element_from(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut impl_block = parse_macro_input!(item as ItemImpl);

    // Define the names of the functions we want to check/add

    let mut funcs_found = [
        ("kind", false),
        ("id", false),
        ("receivable", false),
        ("receive_event_inner", false),
        ("change_priority", false),
        ("drawing", false),
        ("get_attribute", false),
        ("set_attribute", false),
        ("set_hook", false),
        ("remove_hook", false),
        ("clear_hooks_by_id", false),
        ("call_hooks_of_kind", false),
        ("set_parent", false),
        ("get_dyn_location_set", false),
        ("get_visible", false),
    ];

    // Check if each function already exists in the `impl` block
    for item in &impl_block.items {
        if let ImplItem::Fn(ImplItemFn { sig, .. }) = item {
            for item in funcs_found.iter_mut() {
                if sig.ident == item.0 {
                    item.1 = true;
                }
            }
        }
    }
    let field_name: syn::Ident = parse_macro_input!(attr as syn::Ident);

    //fn kind(&self) -> &'static str {
    //fn id(&self) -> ElementID;
    //fn receivable(&self) -> SelfReceivableEvents;
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

    if !funcs_found[0].1 {
        let new_fn = quote! {
            fn kind(&self) -> &'static str {
                self.#field_name.kind()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse kind"));
    }
    if !funcs_found[1].1 {
        let new_fn = quote! {
            fn id(&self) -> ElementID {
                self.#field_name.id()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse id"));
    }
    if !funcs_found[2].1 {
        let new_fn = quote! {
            fn receivable(&self) -> SelfReceivableEvents {
                self.#field_name.receivable()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse receivable"));
    }
    if !funcs_found[3].1 {
        let new_fn = quote! {
            fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
                self.#field_name.receive_event_inner(ctx, ev)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse receive_event_inner"));
    }
    if !funcs_found[4].1 {
        let new_fn = quote! {
            fn change_priority(&self, p: Priority) -> ReceivableEventChanges {
                self.#field_name.change_priority(p)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse change_priority"));
    }
    if !funcs_found[5].1 {
        let new_fn = quote! {
            fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
                self.#field_name.drawing(ctx)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse drawing"));
    }
    if !funcs_found[6].1 {
        let new_fn = quote! {
            fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
                self.#field_name.get_attribute(key)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse get_attribute"));
    }
    if !funcs_found[7].1 {
        let new_fn = quote! {
            fn set_attribute(&self, key: &str, value: Vec<u8>) {
                self.#field_name.set_attribute(key, value)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse set_attribute"));
    }
    if !funcs_found[8].1 {
        let new_fn = quote! {
            fn set_hook(&self, kind: &str, el_id: ElementID,
                hook: Box<dyn FnMut(&str, Box<dyn Element>)>,
            ) {
                self.#field_name.set_hook(kind, el_id, hook)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse set_hook"));
    }
    if !funcs_found[9].1 {
        let new_fn = quote! {
            fn remove_hook(&self, kind: &str, el_id: ElementID) {
                self.#field_name.remove_hook(kind, el_id)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse remove_hook"));
    }
    if !funcs_found[10].1 {
        let new_fn = quote! {
            fn clear_hooks_by_id(&self, el_id: ElementID) {
                self.#field_name.clear_hooks_by_id(el_id)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse clear_hooks_by_id"));
    }
    if !funcs_found[11].1 {
        let new_fn = quote! {
            fn call_hooks_of_kind(&self, kind: &str) {
                self.#field_name.call_hooks_of_kind(kind)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse call_hooks_of_kind"));
    }
    if !funcs_found[12].1 {
        let new_fn = quote! {
            fn set_parent(&self, up: Box<dyn Parent>) {
                self.#field_name.set_parent(up)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse set_parent"));
    }
    if !funcs_found[13].1 {
        let new_fn = quote! {
            fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
                self.#field_name.get_dyn_location_set()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse get_dyn_location_set"));
    }
    if !funcs_found[14].1 {
        let new_fn = quote! {
            fn get_visible(&self) -> Rc<RefCell<bool>> {
                self.#field_name.get_visible()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse get_visible"));
    }

    // Return the modified `impl` block
    TokenStream::from(quote! {
        #impl_block
    })
}
