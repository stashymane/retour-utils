use syn::{Ident, LitStr};

pub enum ExpansionContext {
    Module {
        module_name: Option<LitStr>,
    },
    Impl {
        module_name: Option<LitStr>,
        self_type: Ident,
    },
}

impl ExpansionContext {
    pub fn module_name(&self) -> Option<&LitStr> {
        match self {
            Self::Module { module_name } => module_name.as_ref(),
            Self::Impl { module_name, .. } => module_name.as_ref(),
        }
    }

    pub fn self_type(&self) -> Option<&Ident> {
        match self {
            Self::Module { .. } => None,
            Self::Impl { self_type, .. } => Some(self_type),
        }
    }
}
