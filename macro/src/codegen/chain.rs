use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;
use syn::{FnArg, Item, Signature};

use crate::codegen::signature::{bare_fn_type, receiver_to_ptr_arg, replace_self_in_type};
use crate::model::{CratePaths, Detour};

/// Emit the `struct NAME__ChainType { … }` + `macro` + `static NAME__chain` items.
pub fn chain_static(detour: &Detour, paths: &CratePaths) -> Item {
    let vis = &detour.fn_visibility;
    let detour_name = &detour.attr.detour_name;
    let chain_name = syn::Ident::new(&format!("{}__chain", detour_name), detour_name.span());
    let chain_type_name =
        syn::Ident::new(&format!("{}__ChainType", detour_name), detour_name.span());
    let private_static_detour = paths.private_static_detour();
    let private_mutex = paths.private_mutex();
    let fn_type_sig = bare_fn_type(&detour.fn_sig, &detour.attr, detour.self_type.as_ref());

    let (typed_args, arg_types, arg_names, ret_ty) =
        chain_call_parts(&detour.fn_sig, detour.self_type.as_ref());
    let maybe_unsafe = detour
        .attr
        .unsafety
        .as_ref()
        .map(|_| quote::quote! { unsafe });

    Item::Verbatim(quote_spanned! {detour.attr.detour_name.span()=>
        #[allow(non_camel_case_types, dead_code)]
        #vis struct #chain_type_name {
            detour: &'static #private_static_detour<#fn_type_sig>,
            wrappers: #private_mutex<
                ::std::vec::Vec<::std::sync::Arc<
                    dyn Fn(&dyn Fn(#(#arg_types),*) #ret_ty, #(#arg_types),*) #ret_ty + Send + Sync
                >>
            >,
        }

        #[allow(dead_code)]
        impl #chain_type_name {
            #[doc(hidden)]
            #vis const fn new(
                detour: &'static #private_static_detour<#fn_type_sig>,
            ) -> Self {
                Self {
                    detour,
                    wrappers: #private_mutex::new(::std::vec::Vec::new()),
                }
            }

            /// Register a wrapper closure. Each wrapper receives a `next` closure as the first
            /// argument that invokes the remaining wrappers and ultimately the original function,
            /// followed by the hooked function's arguments.
            /// If a wrapper does not call `next`, the inner wrappers and original are skipped.
            #vis fn hook<__F>(&self, f: __F)
            where
                __F: Fn(&dyn Fn(#(#arg_types),*) #ret_ty, #(#arg_types),*) #ret_ty + Send + Sync + 'static,
            {
                self.wrappers.lock().unwrap().push(::std::sync::Arc::new(f));
            }

            /// Call the wrapper chain. Each wrapper may call `next` to proceed to the next
            /// wrapper; the last `next` calls the original detoured function.
            #vis fn call(&self, #(#typed_args),*) #ret_ty {
                let snapshot: ::std::vec::Vec<::std::sync::Arc<
                    dyn Fn(&dyn Fn(#(#arg_types),*) #ret_ty, #(#arg_types),*) #ret_ty + Send + Sync
                >> = self.wrappers.lock().unwrap().iter().rev().cloned().collect();
                let detour = self.detour;

                fn call_chain(
                    wrappers: &[::std::sync::Arc<dyn Fn(&dyn Fn(#(#arg_types),*) #ret_ty, #(#arg_types),*) #ret_ty + Send + Sync>],
                    detour: &'static #private_static_detour<#fn_type_sig>,
                    #(#typed_args),*
                ) #ret_ty {
                    if let Some((first, rest)) = wrappers.split_first() {
                        let rest = rest.to_vec();
                        first(&move |#(#arg_names),*| call_chain(&rest, detour, #(#arg_names),*), #(#arg_names),*)
                    } else {
                        #maybe_unsafe { (detour.__detour())(#(#arg_names),*) }
                    }
                }
                call_chain(&snapshot, detour, #(#arg_names),*)
            }
        }

        #[allow(non_upper_case_globals)]
        #vis static #chain_name: #chain_type_name =
            #chain_type_name::new(&#detour_name);
    })
}

/// Emit a module-level `static NAME: &NAME__ChainType = &NAME__chain;` alias.
pub fn module_alias(detour: &Detour) -> Item {
    let vis = &detour.fn_visibility;
    let fn_name = &detour.original_fn_name;
    let detour_name = &detour.attr.detour_name;
    let chain_name = syn::Ident::new(&format!("{}__chain", detour_name), detour_name.span());
    let chain_type_name =
        syn::Ident::new(&format!("{}__ChainType", detour_name), detour_name.span());
    Item::Verbatim(quote::quote! {
        #[allow(non_upper_case_globals)]
        #vis static #fn_name: &#chain_type_name = &#chain_name;
    })
}

/// Emit an macro-level `const NAME: &NAME__ChainType = &NAME__chain;` item.
pub fn impl_const(detour: &Detour) -> TokenStream {
    let vis = &detour.fn_visibility;
    let fn_name = &detour.original_fn_name;
    let detour_name = &detour.attr.detour_name;
    let chain_name = syn::Ident::new(&format!("{}__chain", detour_name), detour_name.span());
    let chain_type_name =
        syn::Ident::new(&format!("{}__ChainType", detour_name), detour_name.span());
    quote::quote! {
        #[allow(non_upper_case_globals)]
        #vis const #fn_name: &#chain_type_name = &#chain_name;
    }
}

fn chain_call_parts(
    fn_sig: &Signature,
    self_ty: Option<&Ident>,
) -> (
    Vec<TokenStream>,
    Vec<TokenStream>,
    Vec<TokenStream>,
    TokenStream,
) {
    use proc_macro2::Span;

    let mut typed_args = Vec::new();
    let mut arg_types = Vec::new();
    let mut arg_names = Vec::new();

    for (i, arg) in fn_sig.inputs.iter().enumerate() {
        match arg {
            FnArg::Typed(pt) => {
                let ty = if let Some(st) = self_ty {
                    replace_self_in_type(*pt.ty.clone(), st)
                } else {
                    *pt.ty.clone()
                };
                let pat = &pt.pat;
                typed_args.push(quote::quote! { #pat: #ty });
                arg_types.push(quote::quote! { #ty });
                arg_names.push(quote::quote! { #pat });
            }
            FnArg::Receiver(recv) => {
                if let Some(st) = self_ty {
                    let typed = receiver_to_ptr_arg(recv, st);
                    if let FnArg::Typed(pt) = typed {
                        let pat = &pt.pat;
                        let ty = &pt.ty;
                        typed_args.push(quote::quote! { #pat: #ty });
                        arg_types.push(quote::quote! { #ty });
                        arg_names.push(quote::quote! { #pat });
                    }
                } else {
                    let name = Ident::new(&format!("__arg{}", i), Span::call_site());
                    typed_args.push(quote::quote! { #name: *mut () });
                    arg_types.push(quote::quote! { *mut () });
                    arg_names.push(quote::quote! { #name });
                }
            }
        }
    }

    let ret_ty = match &fn_sig.output {
        syn::ReturnType::Default => quote::quote! {},
        syn::ReturnType::Type(arrow, ty) => quote::quote! { #arrow #ty },
    };

    (typed_args, arg_types, arg_names, ret_ty)
}
