use crate::attr::LookupTarget;
use crate::model::{CratePaths, Detour, ExpansionContext, Ptr};
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{Item, LitStr};

enum InitEntry<'a> {
    Detour(&'a Detour),
    Ptr(&'a Ptr),
}

impl<'a> InitEntry<'a> {
    fn to_init_item(&self, ctx: &ExpansionContext, paths: &CratePaths) -> Item {
        match self {
            InitEntry::Detour(detour) => {
                let lookup = emit_lookup_data(&detour.attr.target, ctx.module_name(), paths);
                let detour_name = &detour.attr.detour_name;
                let orig_func_name = &detour.fn_sig.ident;
                let init_detour = paths.init_detour();
                let function = paths.function();
                match ctx {
                    ExpansionContext::Module { .. } => {
                        Item::Verbatim(quote_spanned! { detour.attr.detour_name.span() =>
                            #init_detour(
                                #lookup,
                                |addr| unsafe {
                                    #detour_name
                                        .initialize(#function::from_ptr(addr), #orig_func_name)?
                                        .enable()?;
                                    Ok(())
                                }
                            )?
                        })
                    }
                    ExpansionContext::Impl { self_type, .. } => {
                        Item::Verbatim(quote_spanned! { detour.attr.detour_name.span() =>
                            #init_detour(
                                #lookup,
                                |addr| unsafe {
                                    #detour_name
                                        .initialize(#function::from_ptr(addr), #self_type::#orig_func_name)?
                                        .enable()?;
                                    Ok(())
                                }
                            )?
                        })
                    }
                }
            }
            InitEntry::Ptr(ptr) => {
                let lookup = emit_lookup_data(&ptr.attr.target, ctx.module_name(), paths);
                let name = &ptr.name;
                let init_detour = paths.init_detour();
                match ctx {
                    ExpansionContext::Module { .. } => Item::Verbatim(quote::quote! {
                        #init_detour(
                            #lookup,
                            |addr| {
                                #name.addr.store(addr as usize, ::std::sync::atomic::Ordering::Relaxed);
                                Ok(())
                            }
                        )?
                    }),
                    ExpansionContext::Impl { .. } => {
                        let static_name = syn::Ident::new(&format!("__PTR_{}", name), name.span());
                        Item::Verbatim(quote::quote! {
                            #init_detour(
                                #lookup,
                                |addr| {
                                    #static_name.addr.store(addr as usize, ::std::sync::atomic::Ordering::Relaxed);
                                    Ok(())
                                }
                            )?
                        })
                    }
                }
            }
        }
    }
}

pub fn emit_lookup_data(
    target: &LookupTarget,
    module_name: Option<&LitStr>,
    paths: &CratePaths,
) -> TokenStream {
    let lookup_data = paths.lookup_data();
    match (target, module_name) {
        (LookupTarget::Offset(value), Some(m)) => quote::quote! {
            #lookup_data::from_offset(#m, #value)
        },
        (LookupTarget::Offset(value), None) => quote::quote! {
            #lookup_data::from_self_offset(#value)
        },
        (LookupTarget::Symbol(value), Some(m)) => quote::quote! {
            #lookup_data::from_symbol(#m, #value)
        },
        (LookupTarget::Symbol(value), None) => quote::quote! {
            #lookup_data::from_self_symbol(#value)
        },
    }
}

pub fn emit_init_detours_fn(
    detours: &[Detour],
    ptrs: &[Ptr],
    ctx: &ExpansionContext,
    paths: &CratePaths,
) -> TokenStream {
    let error_ty = paths.error();
    let inits: Vec<Item> = detours
        .iter()
        .map(InitEntry::Detour)
        .chain(ptrs.iter().map(InitEntry::Ptr))
        .map(|e| e.to_init_item(ctx, paths))
        .collect();
    quote::quote! {
        pub fn init_detours() -> Result<(), #error_ty> {
            #(#inits;)*
            Ok(())
        }
    }
}
