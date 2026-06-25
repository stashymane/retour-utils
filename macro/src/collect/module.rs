use super::common::{ConstInfo, FnInfo};
use crate::model::{Detour, ExpansionContext, Ptr};
use syn::{Item, ItemMod};

pub struct CollectedModule {
    pub module: ItemMod,
    pub detours: Vec<Detour>,
    pub ptrs: Vec<Ptr>,
}

pub fn collect(mut module: ItemMod, _ctx: &ExpansionContext) -> syn::Result<CollectedModule> {
    let mut detours: Vec<Detour> = Vec::new();
    let mut ptrs: Vec<Ptr> = Vec::new();

    let Some((_, ref mut content)) = module.content else {
        return Err(syn::Error::new_spanned(
            &module,
            "Could not get content inside `mod`",
        ));
    };

    let mut new_items: Vec<Item> = Vec::new();
    for item in content.drain(..) {
        match item {
            Item::Fn(mut fn_item) => {
                let detour = FnInfo {
                    attrs: &mut fn_item.attrs,
                    sig: &mut fn_item.sig,
                    vis: &fn_item.vis,
                    self_type: None,
                }
                .to_detour()?;
                if let Some(detour) = detour {
                    detours.push(detour);
                }
                new_items.push(Item::Fn(fn_item));
            }
            Item::Const(ref c) => {
                let ptr = ConstInfo {
                    attrs: &c.attrs,
                    vis: &c.vis,
                    ident: &c.ident,
                    ty: &c.ty,
                }
                .to_ptr()?;
                if let Some(ptr) = ptr {
                    ptrs.push(ptr);
                } else {
                    new_items.push(item);
                }
            }
            other => new_items.push(other),
        }
    }

    *content = new_items;

    Ok(CollectedModule {
        module,
        detours,
        ptrs,
    })
}
