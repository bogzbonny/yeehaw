use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ImplItemFn, ItemImpl, ItemTrait, TraitItem, TraitItemFn};

#[proc_macro_attribute]
pub fn impl_element_from(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut impl_block = parse_macro_input!(item as ItemImpl);

    // Define the names of the functions we want to check/add
    let tr_code = r"pub trait Element: DynClone {
    fn kind(&self) -> &'static str;
    fn id(&self) -> ElementID;
    fn can_receive(&self, ev: &Event) -> bool;
    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>>;
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses);
    fn set_focused(&self, focused: bool);
    fn get_focused(&self) -> bool;
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate>;
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
    fn get_ref_cell_overflow(&self) -> std::rc::Rc<std::cell::RefCell<bool>>;
    fn set_content_x_offset(&self, ctx: &Context, x: usize);
    fn set_content_y_offset(&self, ctx: &Context, y: usize);
    fn get_content_x_offset(&self) -> usize;
    fn get_content_y_offset(&self) -> usize;
    fn get_content_width(&self, ctx: &Context) -> usize;
    fn get_content_height(&self, ctx: &Context) -> usize;
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
    TokenStream::from(quote! {
        #impl_block
    })
}

#[proc_macro_attribute]
pub fn impl_pane_basics_from(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut impl_block = parse_macro_input!(item as ItemImpl);
    let tr_code_with = r"pub trait PaneBasicsWith {
        fn with_focused(self, focused: bool); 
        fn with_kind(self, kind: &'static str); 
        fn with_overflow(self);
        fn with_z(self, z: ZIndex);
        fn with_start_x<D: Into<DynVal>>(self, x: D);
        fn with_start_y<D: Into<DynVal>>(self, y: D);
        fn with_end_x<D: Into<DynVal>>(self, x: D);
        fn with_end_y<D: Into<DynVal>>(self, y: D);
        fn with_dyn_height<D: Into<DynVal>>(self, h: D);
        fn with_dyn_width<D: Into<DynVal>>(self, w: D);
        fn with_dyn_location(self, l: DynLocation);
        fn with_content(self, content: DrawChs2D);
        fn with_default_ch(self, ch: DrawCh);
        fn with_transparent(self);
        fn with_style(self, style: Style);
        fn with_bg(self, bg: Color);
        fn with_fg(self, fg: Color);
        fn with_focused_receivable_events(self, evs: ReceivableEvents);
        fn with_always_receivable_events(self, evs: ReceivableEvents);
    }
    ";

    // Define the names of the functions we want to check/add
    let tr_code_non_with = r"pub trait PaneBasicsNonWith {
    fn set_at(&self, x: DynVal, y: DynVal);
    fn set_kind(&self, kind: &'static str);
    fn set_overflow(&self);
    fn set_start_x<D: Into<DynVal>>(&self, x: D);
    fn set_start_y<D: Into<DynVal>>(&self, y: D);
    fn set_end_x<D: Into<DynVal>>(&self, x: D);
    fn set_end_y<D: Into<DynVal>>(&self, y: D);
    fn get_start_x(&self, ctx: &Context) -> i32;
    fn get_start_y(&self, ctx: &Context) -> i32;
    fn get_end_x(&self, ctx: &Context) -> i32;
    fn get_end_y(&self, ctx: &Context) -> i32;
    fn get_dyn_start_x(&self) -> DynVal;
    fn get_dyn_start_y(&self) -> DynVal;
    fn get_dyn_end_x(&self) -> DynVal;
    fn get_dyn_end_y(&self) -> DynVal;
    fn get_height(&self, ctx: &Context) -> usize;
    fn get_width(&self, ctx: &Context) -> usize;
    fn set_dyn_height<D: Into<DynVal>>(&self, h: D);
    fn set_dyn_width<D: Into<DynVal>>(&self, w: D);
    fn get_dyn_height(&self) -> DynVal;
    fn get_dyn_width(&self) -> DynVal;
    fn set_z(&self, z: ZIndex);
    fn get_dyn_location(&self) -> DynLocation;
    fn set_dyn_location(&self, l: DynLocation);
    fn set_content(&self, content: DrawChs2D);
    fn get_content(&self) -> Ref<DrawChs2D>;
    fn get_content_mut(&self) -> RefMut<DrawChs2D>;
    fn set_content_from_string<S: Into<String>>(&self, s: S);
    fn set_content_from_string_with_style(&self, ctx: &Context, s: &str, sty: Style);
    fn set_content_style(&self, sty: Style);
    fn content_width(&self) -> usize;
    fn content_height(&self) -> usize;
    fn content_size(&self) -> Size;
    fn scroll_up(&self, ctx: &Context);
    fn scroll_down(&self, ctx: &Context);
    fn scroll_left(&self, ctx: &Context);
    fn scroll_right(&self, ctx: &Context);
    fn set_style(&self, style: Style);
    fn get_style(&self) -> Style;
    fn set_bg(&self, bg: Color);
    fn set_fg(&self, fg: Color);
    fn set_default_ch(&self, ch: DrawCh);
    fn set_transparent(&self);
    fn set_focused_receivable_events(&self, evs: ReceivableEvents);
    fn set_always_receivable_events(&self, evs: ReceivableEvents);
    fn send_responses_upward(&self, ctx: &Context, resps: EventResponses);
    fn has_parent(&self) -> bool;
    fn set_focused(&self, focused: bool);
    fn correct_offsets_to_view_position(&self, ctx: &Context, x: usize, y: usize);
}";
    let tr_parsed_with = syn::parse_str::<ItemTrait>(tr_code_with).expect("Failed to parse trait");
    let tr_parsed_non_with =
        syn::parse_str::<ItemTrait>(tr_code_non_with).expect("Failed to parse trait2");

    // convert the function signatures to Idents
    let mut fn_found_names_with = tr_parsed_with
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
    let mut fn_found_names_non_with = tr_parsed_non_with
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

    // get the kind being implemented
    let self_type = impl_block.self_ty.clone();

    // Check if each function already exists in the `impl` block
    for item in &impl_block.items {
        if let ImplItem::Fn(ImplItemFn { sig, .. }) = item {
            for (found, tr_fn) in fn_found_names_with.iter_mut() {
                if sig.ident == tr_fn.sig.ident {
                    *found = true;
                }
            }
        }
    }
    for item in &impl_block.items {
        if let ImplItem::Fn(ImplItemFn { sig, .. }) = item {
            for (found, tr_fn) in fn_found_names_non_with.iter_mut() {
                if sig.ident == tr_fn.sig.ident {
                    *found = true;
                }
            }
        }
    }

    // the field in which all the default implementations are relying on
    let field_name: syn::Ident = parse_macro_input!(attr as syn::Ident);

    for (found, tr_fn) in fn_found_names_with.iter() {
        if !found {
            // create a default implementation of the function which calls the function on the field
            let fn_sig = &tr_fn.sig;
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

            // convert the function name from a "with_" to a "set_" function
            //let fn_name = &tr_fn.sig.ident;
            let fn_name = syn::Ident::new(
                &format!(
                    "set_{}",
                    tr_fn
                        .sig
                        .ident
                        .to_string()
                        .strip_prefix("with_")
                        .expect("Failed to strip with_ prefix")
                ),
                tr_fn.sig.ident.span(),
            );

            let new_fn = quote! {
                pub #fn_sig -> #self_type
                {
                    self.#field_name.#fn_name(#fn_args);
                    self
                }
            };
            impl_block
                .items
                .push(syn::parse2(new_fn).expect("Failed to parse kind"));
        }
    }
    for (found, tr_fn) in fn_found_names_non_with.iter() {
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
                pub #fn_sig
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
    TokenStream::from(quote! {
        #impl_block
    })
}
