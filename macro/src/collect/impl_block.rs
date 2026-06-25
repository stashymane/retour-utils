use syn::{Ident, ImplItem, ItemImpl, Type};

use super::common::{ConstInfo, FnInfo};
use crate::model::{Detour, ExpansionContext, Ptr};

pub struct CollectedImpl {
    pub impl_block: ItemImpl,
    pub detours: Vec<Detour>,
    pub ptrs: Vec<Ptr>,
}

pub fn extract_self_type(impl_block: &ItemImpl) -> syn::Result<Ident> {
    match impl_block.self_ty.as_ref() {
        Type::Path(tp) => tp.path.get_ident().cloned().ok_or_else(|| {
            syn::Error::new_spanned(&impl_block.self_ty, "Expected a simple struct name")
        }),
        _ => Err(syn::Error::new_spanned(
            &impl_block.self_ty,
            "Expected a simple struct name",
        )),
    }
}

pub fn collect(mut impl_block: ItemImpl, ctx: &ExpansionContext) -> syn::Result<CollectedImpl> {
    let self_type = ctx.self_type().ok_or_else(|| {
        syn::Error::new_spanned(&impl_block.self_ty, "macro context requires a self type")
    })?;

    let mut detours: Vec<Detour> = Vec::new();
    let mut ptrs: Vec<Ptr> = Vec::new();
    let mut new_items: Vec<ImplItem> = Vec::new();

    for item in impl_block.items.drain(..) {
        match item {
            ImplItem::Fn(mut fn_item) => {
                let detour = FnInfo {
                    attrs: &mut fn_item.attrs,
                    sig: &mut fn_item.sig,
                    vis: &fn_item.vis,
                    self_type: Some(self_type),
                }
                .to_detour()?;
                if let Some(detour) = detour {
                    detours.push(detour);
                }
                new_items.push(ImplItem::Fn(fn_item));
            }
            ImplItem::Const(const_item) => {
                let ptr = ConstInfo {
                    attrs: &const_item.attrs,
                    vis: &const_item.vis,
                    ident: &const_item.ident,
                    ty: &const_item.ty,
                }
                .to_ptr()?;
                if let Some(ptr) = ptr {
                    ptrs.push(ptr);
                } else {
                    new_items.push(ImplItem::Const(const_item));
                }
            }
            other => new_items.push(other),
        }
    }

    impl_block.items = new_items;
    Ok(CollectedImpl {
        impl_block,
        detours,
        ptrs,
    })
}
