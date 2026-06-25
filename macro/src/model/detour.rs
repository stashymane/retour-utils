use syn::{Ident, Signature, Visibility};

use crate::attr::HookAttr;

pub struct Detour {
    pub attr: HookAttr,
    pub original_fn_name: Ident,
    pub fn_sig: Signature,
    pub self_type: Option<Ident>,
    pub fn_visibility: Visibility,
}
