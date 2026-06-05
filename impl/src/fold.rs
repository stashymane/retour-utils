mod detour;
mod ptr;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{fold::Fold, Ident, ImplItemFn, Item, ItemFn, LitStr, Signature};

use crate::{
    crate_refs,
    parse::{HookAttributeArgs, PtrItem},
};

pub use detour::DetourInfo;

pub struct Detours {
    module_name: Option<LitStr>,
    detours: Vec<DetourInfo>,
    ptrs: Vec<PtrItem>,
}

impl Detours {
    pub fn new(module_name: Option<LitStr>) -> Self {
        Self {
            module_name,
            detours: Vec::new(),
            ptrs: Vec::new(),
        }
    }

    pub fn generate_ptr_accessor_decls(&self) -> Vec<Item> {
        self.ptrs.iter().map(|p| p.get_accessor_item()).collect()
    }

    pub fn generate_ptr_accessor_impl_consts(&self) -> Vec<TokenStream> {
        self.ptrs.iter().map(|p| p.get_accessor_impl_const()).collect()
    }

    pub fn generate_ptr_accessor_impl_statics(&self) -> Vec<TokenStream> {
        self.ptrs.iter().map(|p| p.get_accessor_impl_static()).collect()
    }

    pub fn generate_ptr_accessor_statics(&self) -> Vec<TokenStream> {
        self.ptrs.iter().map(|p| p.get_accessor_static()).collect()
    }

    pub fn generate_ptr_inits(&self, use_self: bool) -> Vec<Item> {
        self.ptrs
            .iter()
            .map(|p| p.generate_ptr_init(self.module_name.as_ref(), use_self))
            .collect()
    }

    pub fn generate_detour_decls(&self) -> Vec<Item> {
        let mut items = Vec::new();
        for info in &self.detours {
            items.push(info.get_static_detour());
            if info.hook_attr.chain {
                items.push(info.get_chain_static());
            }
        }
        items
    }

    pub fn generate_chain_aliases_for_mod(&self) -> Vec<Item> {
        self.detours
            .iter()
            .filter(|info| info.hook_attr.chain)
            .map(|info| info.get_chain_mod_alias())
            .collect()
    }

    pub fn generate_chain_consts_for_impl(&self) -> Vec<TokenStream> {
        self.detours
            .iter()
            .filter(|info| info.hook_attr.chain)
            .map(|info| info.get_chain_impl_const())
            .collect()
    }

    /// Returns the const expression containing the module name
    /// ```
    /// pub const MODULE_NAME: &str = "lua52.dll";
    /// ```
    pub fn get_module_name_decl(&self) -> Item {
        let module_name = &self.module_name;
        let span = module_name
            .as_ref()
            .map(|m| m.span())
            .unwrap_or_else(Span::call_site);
        let value = module_name
            .as_ref()
            .map(|m| quote::quote! { #m })
            .unwrap_or_else(|| quote::quote! { "" });

        Item::Verbatim(quote_spanned! {span=>
            #[allow(unused)]
            pub const MODULE_NAME: &str = #value;
        })
    }

    pub fn generate_init_detours_for_impl(&self, struct_name: &syn::Ident) -> TokenStream {
        let krate_name = crate_refs::parent_crate();
        let init_funcs: Vec<Item> = self
            .detours
            .iter()
            .map(|func| func.generate_detour_init_with_prefix(self.module_name.as_ref(), struct_name))
            .collect();
        let ptr_inits = self.generate_ptr_inits(true);
        quote::quote! {
            pub fn init_detours() -> Result<(), #krate_name::Error> {
                #(#init_funcs;)*
                #(#ptr_inits;)*
                Ok(())
            }
        }
    }

    pub fn generate_init_detours(&self) -> Item {
        let krate_name = crate_refs::parent_crate();
        let init_funcs: Vec<Item> = self
            .detours
            .iter()
            .map(|func| func.generate_detour_init(self.module_name.as_ref()))
            .collect();
        let ptr_inits = self.generate_ptr_inits(false);
        Item::Verbatim(quote::quote! {
            pub fn init_detours() -> Result<(), #krate_name::Error> {
                #(#init_funcs;)*
                #(#ptr_inits;)*
                Ok(())
            }
        })
    }

    pub fn collect_impl_item_fn(&mut self, item_fn: ImplItemFn, self_ty: &Ident) -> ImplItemFn {
        let mut attrs = Vec::new();
        let mut renamed_sig: Option<Signature> = None;

        for attr in item_fn.attrs {
            if !attr.path().is_ident("hook") {
                attrs.push(attr);
                continue;
            }
            let Ok(hook_attrs) = attr.parse_args::<HookAttributeArgs>() else {
                continue;
            };
            let original_fn_name = item_fn.sig.ident.clone();
            let fn_sig = if hook_attrs.chain {
                let new_ident = syn::Ident::new(
                    &format!("__{}_detour", original_fn_name),
                    original_fn_name.span(),
                );
                let mut sig = item_fn.sig.clone();
                sig.ident = new_ident;
                renamed_sig = Some(sig.clone());
                sig
            } else {
                item_fn.sig.clone()
            };
            self.detours.push(DetourInfo {
                hook_attr: hook_attrs,
                fn_sig,
                self_ty: Some(self_ty.clone()),
                original_fn_name,
                fn_vis: item_fn.vis.clone(),
            });
        }
        let sig = renamed_sig.unwrap_or(item_fn.sig);
        ImplItemFn { attrs, sig, block: item_fn.block, vis: item_fn.vis, defaultness: item_fn.defaultness }
    }

    /// Collect a `const NAME: TYPE` impl item that has a `#[ptr(...)]` attribute.
    /// Returns `None` if the item should be removed (i.e. it was a ptr declaration).
    pub fn collect_impl_item_const(&mut self, item: syn::ImplItemConst) -> Option<syn::ImplItemConst> {
        use crate::parse::PtrAttributeArgs;
        let mut attrs = Vec::new();
        let mut found = false;
        for attr in &item.attrs {
            if attr.path().is_ident("ptr") {
                let Ok(ptr_attr) = attr.parse_args::<PtrAttributeArgs>() else {
                    attrs.push(attr.clone());
                    continue;
                };
                self.ptrs.push(PtrItem {
                    vis: item.vis.clone(),
                    name: item.ident.clone(),
                    ty: item.ty.clone(),
                    attr: ptr_attr,
                });
                found = true;
            } else {
                attrs.push(attr.clone());
            }
        }
        if found { None } else { Some(item) }
    }
}

impl Fold for Detours {
    fn fold_item(&mut self, item: Item) -> Item {
        use crate::parse::PtrAttributeArgs;
        if let Item::Const(ref c) = item {
            let has_ptr = c.attrs.iter().any(|a| a.path().is_ident("ptr"));
            if has_ptr {
                for attr in &c.attrs {
                    if attr.path().is_ident("ptr") {
                        let Ok(ptr_attr) = attr.parse_args::<PtrAttributeArgs>() else {
                            continue;
                        };
                        self.ptrs.push(PtrItem {
                            vis: c.vis.clone(),
                            name: c.ident.clone(),
                            ty: *c.ty.clone(),
                            attr: ptr_attr,
                        });
                    }
                }
                // Remove the original const item entirely
                return Item::Verbatim(quote::quote! {});
            }
        }
        syn::fold::fold_item(self, item)
    }

    fn fold_item_fn(&mut self, item_fn: ItemFn) -> ItemFn {
        let mut attrs = Vec::new();
        let mut renamed_sig: Option<Signature> = None;

        for attr in item_fn.attrs {
            if !attr.path().is_ident("hook") {
                attrs.push(attr);
                continue;
            }
            let Ok(hook_attrs) = attr.parse_args::<HookAttributeArgs>() else {
                continue;
            };
            let original_fn_name = item_fn.sig.ident.clone();
            let fn_sig = if hook_attrs.chain {
                let new_ident = syn::Ident::new(
                    &format!("__{}_detour", original_fn_name),
                    original_fn_name.span(),
                );
                let mut sig = item_fn.sig.clone();
                sig.ident = new_ident;
                renamed_sig = Some(sig.clone());
                sig
            } else {
                item_fn.sig.clone()
            };
            self.detours.push(DetourInfo {
                hook_attr: hook_attrs,
                original_fn_name,
                fn_sig,
                self_ty: None,
                fn_vis: item_fn.vis.clone(),
            })
        }
        let sig = renamed_sig.unwrap_or(item_fn.sig);
        ItemFn { attrs, sig, ..item_fn }
    }
}
