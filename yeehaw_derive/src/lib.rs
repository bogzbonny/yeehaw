use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ImplItemFn, ItemImpl, ItemTrait, TraitItem, TraitItemFn};

#[proc_macro_attribute]
pub fn impl_element_from(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut output = TokenStream::new();
    let mut impl_block = parse_macro_input!(item as ItemImpl);

    // add some use statements to the output token stream
    //"use std::{cell::{Ref, RefCell}, rc::Rc};"
    //let use_stmts = quote! {
    //    use std::{cell::{Ref, RefCell}, rc::Rc};
    //};
    //output.extend(TokenStream::from(use_stmts));

    // Define the names of the functions we want to check/add
    let tr_code = r"pub trait Element: DynClone {
    fn kind(&self) -> &'static str;
    fn id(&self) -> ElementID;
    fn receivable(&self) -> SelfReceivableEvents;
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses);
    fn change_priority(&self, p: Priority) -> ReceivableEventChanges;
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos>;
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>>;
    fn set_attribute_inner(&self, key: &str, value: Vec<u8>);
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>);
    fn remove_hook(&self, kind: &str, el_id: ElementID);
    fn clear_hooks_by_id(&self, el_id: ElementID);
    fn call_hooks_of_kind(&self, kind: &str);
    fn set_parent(&self, up: Box<dyn Parent>);
    fn get_dyn_location_set(&self) -> std::cell::Ref<DynLocationSet>;
    fn get_visible(&self) -> bool;
    fn get_ref_cell_dyn_location_set(&self) -> std::rc::Rc<std::cell::RefCell<DynLocationSet>>;
    fn get_ref_cell_visible(&self) -> std::rc::Rc<std::cell::RefCell<bool>>;
    fn set_content_x_offset(&self, ctx: &Context, x: usize);
    fn set_content_y_offset(&self, ctx: &Context, y: usize);
    fn get_content_x_offset(&self) -> usize;
    fn get_content_y_offset(&self) -> usize;
    fn get_content_width(&self) -> usize;
    fn get_content_height(&self) -> usize;
}";
    let tr_parsed = syn::parse_str::<ItemTrait>(tr_code).expect("Failed to parse trait");

    // convert the function signatures to Idents
    let mut fn_found_names = tr_parsed
        .items
        .iter()
        .map(|item| {
            if let TraitItem::Fn(tr_fn) = item {
                (false, tr_fn.clone())
            } else {
                panic!("Unexpected item in trait");
            }
        })
        .collect::<Vec<(bool, TraitItemFn)>>();

    // Check if each function already exists in the `impl` block
    for item in &impl_block.items {
        if let ImplItem::Fn(ImplItemFn { sig, .. }) = item {
            for (found, tr_fn) in fn_found_names.iter_mut() {
                if sig.ident == tr_fn.sig.ident {
                    *found = true;
                }
            }
        }
    }

    // the field in which all the default implementations are relying on
    let field_name: syn::Ident = parse_macro_input!(attr as syn::Ident);

    for (found, tr_fn) in fn_found_names.iter() {
        if !found {
            // create a default implementation of the function which calls the function on the field
            let fn_sig = &tr_fn.sig;
            let fn_name = &tr_fn.sig.ident;
            let fn_args = &tr_fn.sig.inputs;
            // get the arg variable names for non-self args
            let fn_args = fn_args
                .iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat) = arg {
                        if let syn::Pat::Ident(ident) = &*pat.pat {
                            Some(ident.ident.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<syn::Ident>>();
            let fn_args = quote! {#(#fn_args),*};
            let new_fn = quote! {
                #fn_sig
                {
                    self.#field_name.#fn_name(#fn_args)
                }
            };
            impl_block
                .items
                .push(syn::parse2(new_fn).expect("Failed to parse kind"));
        }
    }

    // Return the modified `impl` block
    output.extend(TokenStream::from(quote! {
        #impl_block
    }));
    output
}

/*
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
        ("get_ref_cell_dyn_location_set", false),
        ("get_ref_cell_visible", false),
        ("set_content_x_offset", false),
        ("set_content_y_offset", false),
        ("get_content_x_offset", false),
        ("get_content_y_offset", false),
        ("get_content_width", false),
        ("get_content_height", false),
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
            fn get_ref_cell_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
                self.#field_name.get_dyn_location_set()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse get_dyn_location_set"));
    }
    if !funcs_found[14].1 {
        let new_fn = quote! {
            fn get_ref_cell_visible(&self) -> Rc<RefCell<bool>> {
                self.#field_name.get_visible()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse get_visible"));
    }
    if !funcs_found[15].1 {
        let new_fn = quote! {
            fn set_content_x_offset(&self, ctx: &Context, x: usize) {
                self.#field_name.set_content_x_offset(ctx, x)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse set_content_x_offset"));
    }
    if !funcs_found[16].1 {
        let new_fn = quote! {
            fn set_content_y_offset(&self, ctx: &Context, y: usize) {
                self.#field_name.set_content_y_offset(ctx, y)
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse set_content_y_offset"));
    }
    if !funcs_found[17].1 {
        let new_fn = quote! {
            fn get_content_x_offset(&self) -> usize {
                self.#field_name.get_content_x_offset()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse get_content_y_offset"));
    }
    if !funcs_found[18].1 {
        let new_fn = quote! {
            fn get_content_y_offset(&self) -> usize {
                self.#field_name.get_content_y_offset()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse get_content_y_offset"));
    }
    if !funcs_found[19].1 {
        let new_fn = quote! {
            fn get_content_width(&self) -> usize {
                self.#field_name.get_content_width()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse get_content_width"));
    }
    if !funcs_found[20].1 {
        let new_fn = quote! {
            fn get_content_height(&self) -> usize {
                self.#field_name.get_content_height()
            }
        };
        impl_block
            .items
            .push(syn::parse2(new_fn).expect("Failed to parse get_content_height"));
    }

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
    //fn set_content_x_offset(&self, ctx: &Context, x: usize) {
    //fn set_content_y_offset(&self, ctx: &Context, y: usize) {
    //fn get_content_x_offset(&self) -> usize;
    //fn get_content_y_offset(&self) -> usize;
    //fn get_content_width(&self) -> usize;
    //fn get_content_height(&self) -> usize;

    // Return the modified `impl` block
    TokenStream::from(quote! {
        #impl_block
    })
}
*/
