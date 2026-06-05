use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{fold::Fold, spanned::Spanned, ImplItem, ItemImpl, ItemMod, LitStr, Type};

use crate::fold::Detours;

pub fn expand_impl(impl_block: ItemImpl, attribute_meta: Option<LitStr>) -> Result<TokenStream, syn::Error> {
    let struct_name = match impl_block.self_ty.as_ref() {
        Type::Path(tp) => tp
            .path
            .get_ident()
            .cloned()
            .ok_or_else(|| syn::Error::new_spanned(&impl_block.self_ty, "Expected a simple struct name"))?,
        _ => return Err(syn::Error::new_spanned(&impl_block.self_ty, "Expected a simple struct name")),
    };

    let mut detours = Detours::new(attribute_meta);

    let mut new_items: Vec<ImplItem> = impl_block
        .items
        .into_iter()
        .filter_map(|item| match item {
            ImplItem::Fn(fn_item) => Some(ImplItem::Fn(detours.collect_impl_item_fn(fn_item, &struct_name))),
            ImplItem::Const(const_item) => detours.collect_impl_item_const(const_item).map(ImplItem::Const),
            other => Some(other),
        })
        .collect();

    let init_fn = detours.generate_init_detours_for_impl(&struct_name);
    new_items.push(ImplItem::Verbatim(init_fn));

    for chain_const in detours.generate_chain_consts_for_impl() {
        new_items.push(ImplItem::Verbatim(chain_const));
    }

    for ptr_const in detours.generate_ptr_accessor_impl_consts() {
        new_items.push(ImplItem::Verbatim(ptr_const));
    }

    let statics = detours.generate_detour_decls();
    let ptr_accessors = detours.generate_ptr_accessor_decls();
    let ptr_impl_statics = detours.generate_ptr_accessor_impl_statics();

    let rebuilt_impl = ItemImpl {
        items: new_items,
        ..impl_block
    };

    let mut output = TokenStream::new();
    for s in statics {
        s.to_tokens(&mut output);
    }
    for p in ptr_accessors {
        p.to_tokens(&mut output);
    }
    for s in ptr_impl_statics {
        proc_macro2::TokenStream::from(s).to_tokens(&mut output);
    }
    rebuilt_impl.to_tokens(&mut output);

    Ok(output)
}

pub fn expand(mod_block: ItemMod, attribute_meta: Option<LitStr>) -> Result<TokenStream, syn::Error> {
    let mut detours = Detours::new(attribute_meta);
    let mut result = detours.fold_item_mod(mod_block);

    let Some((_, ref mut content)) = result.content.as_mut() else {
        return Err(syn::Error::new(result.span(), "Could not get content inside `mod`"))
    };
    content.push(detours.get_module_name_decl());
    let decls = detours.generate_detour_decls();
    content.extend(decls);
    let ptr_decls = detours.generate_ptr_accessor_decls();
    let ptr_statics: Vec<_> = detours.generate_ptr_accessor_statics()
        .into_iter()
        .map(syn::Item::Verbatim)
        .collect();
    content.extend(ptr_decls);
    content.extend(ptr_statics);
    content.extend(detours.generate_chain_aliases_for_mod());
    content.push(detours.generate_init_detours());

    Ok(result.to_token_stream())
}
