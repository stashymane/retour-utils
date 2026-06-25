use syn::{Ident, Visibility};

use crate::attr::{HookAttr, PtrAttr};
use crate::model::{Detour, Ptr};

pub struct FnInfo<'a> {
    pub attrs: &'a mut Vec<syn::Attribute>,
    pub sig: &'a mut syn::Signature,
    pub vis: &'a Visibility,
    pub self_type: Option<&'a Ident>,
}

impl<'a> FnInfo<'a> {
    pub fn to_detour(self) -> syn::Result<Option<Detour>> {
        let mut kept_attrs = Vec::new();
        let mut detour: Option<Detour> = None;
        let mut renamed_sig: Option<syn::Signature> = None;

        for attr in self.attrs.drain(..) {
            if !attr.path().is_ident("hook") {
                kept_attrs.push(attr);
                continue;
            }

            let hook_attr = attr.parse_args::<HookAttr>()?;
            let original_fn_name = self.sig.ident.clone();
            let fn_sig = if hook_attr.chain {
                let new_ident = syn::Ident::new(
                    &format!("__{}_detour", original_fn_name),
                    original_fn_name.span(),
                );
                let mut sig = self.sig.clone();
                sig.ident = new_ident;
                renamed_sig = Some(sig.clone());
                sig
            } else {
                self.sig.clone()
            };
            detour = Some(Detour {
                attr: hook_attr,
                original_fn_name,
                fn_sig,
                self_type: self.self_type.cloned(),
                fn_visibility: self.vis.clone(),
            });
        }

        *self.attrs = kept_attrs;
        if let Some(sig) = renamed_sig {
            *self.sig = sig;
        }
        Ok(detour)
    }
}

pub struct ConstInfo<'a> {
    pub attrs: &'a [syn::Attribute],
    pub vis: &'a Visibility,
    pub ident: &'a Ident,
    pub ty: &'a syn::Type,
}

impl<'a> ConstInfo<'a> {
    pub fn to_ptr(self) -> syn::Result<Option<Ptr>> {
        let mut ptr: Option<Ptr> = None;

        for attr in self.attrs {
            if !attr.path().is_ident("ptr") {
                continue;
            }

            let ptr_attr = attr.parse_args::<PtrAttr>()?;
            ptr = Some(Ptr {
                vis: self.vis.clone(),
                name: self.ident.clone(),
                ty: self.ty.clone(),
                attr: ptr_attr,
            });
        }

        Ok(ptr)
    }
}
