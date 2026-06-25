use crate::codegen::signature::{bare_fn_type, receiver_to_ptr_arg, replace_self_in_type};
use crate::model::{CratePaths, Detour};
use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;
use std::borrow::Cow;
use syn::spanned::Spanned;
use syn::{FnArg, Item, Pat, Signature};

/// Emit the `static NAME: StaticDetour<…> = { … };` item.
pub fn static_detour(detour: &Detour, paths: &CratePaths) -> Item {
    let vis = &detour.attr.visibility;
    let static_detour_type = paths.static_detour();
    let detour_name = &detour.attr.detour_name;
    let fn_type_sig = bare_fn_type(&detour.fn_sig, &detour.attr, detour.self_type.as_ref());
    let target_fn_decl = ffi_trampoline_decl(detour);

    let arg_names: Vec<_> = match &detour.self_type {
        Some(st) => arg_names_with_self(&detour.fn_sig, st)
            .into_iter()
            .map(|c| c.into_owned())
            .collect(),
        None => arg_names(&detour.fn_sig)
            .unwrap()
            .into_iter()
            .cloned()
            .collect(),
    };

    let ffi_body = if detour.attr.chain {
        let chain_name = syn::Ident::new(&format!("{}__chain", detour_name), detour_name.span());
        quote::quote! { #chain_name.call(#(#arg_names),*) }
    } else {
        quote::quote! {
            #[allow(unused_unsafe)]
            (#detour_name.__detour())(#(#arg_names),*)
        }
    };

    Item::Verbatim(quote_spanned! {detour.attr.detour_name.span()=>
        #[allow(non_upper_case_globals, unused_unsafe)]
        #vis static #detour_name: #static_detour_type<#fn_type_sig> = {
            #[inline(never)]
            #[allow(unused_unsafe, unsafe_op_in_unsafe_fn)]
            #target_fn_decl {
                #ffi_body
            }
            #static_detour_type::__new(__ffi_detour)
        };
    })
}

/// Build the `unsafe extern "ABI" fn __ffi_detour(…) -> …` declaration token stream.
fn ffi_trampoline_decl(detour: &Detour) -> TokenStream {
    let input_args: Vec<_> = match &detour.self_type {
        Some(st) => detour
            .fn_sig
            .inputs
            .iter()
            .map(|arg| match arg {
                FnArg::Typed(pt) => {
                    let ty = replace_self_in_type(*pt.ty.clone(), st);
                    let pat = &pt.pat;
                    quote::quote! { #pat: #ty }
                }
                FnArg::Receiver(recv) => {
                    let typed = receiver_to_ptr_arg(&recv, st);
                    if let FnArg::Typed(pt) = typed {
                        let pat = &pt.pat;
                        let ty = &pt.ty;
                        quote::quote! { #pat: #ty }
                    } else {
                        quote::quote! {}
                    }
                }
            })
            .collect(),
        None => detour
            .fn_sig
            .inputs
            .iter()
            .map(|arg| quote::quote! { #arg })
            .collect(),
    };

    let output_type = &detour.fn_sig.output;
    let abi = &detour.attr.abi;
    let unsafety = &detour.attr.unsafety;

    quote::quote! {
        #unsafety #abi fn __ffi_detour(#(#input_args),*) #output_type
    }
}

fn arg_names(fn_sig: &Signature) -> syn::Result<Vec<&Pat>> {
    let mut args = Vec::new();
    let mut errs: Option<syn::Error> = None;
    for arg in &fn_sig.inputs {
        match arg {
            FnArg::Typed(arg) => args.push(arg.pat.as_ref()),
            FnArg::Receiver(_) => {
                let err = syn::Error::new(
                    arg.span(),
                    "`self` is not currently supported by this macro",
                );
                match &mut errs {
                    Some(errs) => errs.combine(err),
                    None => errs = Some(err),
                }
            }
        }
    }
    if let Some(e) = errs { Err(e) } else { Ok(args) }
}

fn arg_names_with_self<'a>(fn_sig: &'a Signature, self_ty: &Ident) -> Vec<Cow<'a, Pat>> {
    let mut args = Vec::new();
    for arg in &fn_sig.inputs {
        match arg {
            FnArg::Typed(arg) => args.push(Cow::Borrowed(arg.pat.as_ref())),
            FnArg::Receiver(recv) => {
                let typed = receiver_to_ptr_arg(recv, self_ty);
                if let FnArg::Typed(pt) = typed {
                    args.push(Cow::Owned(*pt.pat));
                }
            }
        }
    }
    args
}
