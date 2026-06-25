use syn::{Ident, Type, Visibility};

use crate::attr::PtrAttr;

pub struct Ptr {
    pub vis: Visibility,
    pub name: Ident,
    pub ty: Type,
    pub attr: PtrAttr,
}
