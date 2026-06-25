use crate::attr::LookupTarget;
use syn::parse::{Parse, ParseStream};

pub struct PtrAttr {
    pub target: LookupTarget,
}

impl Parse for PtrAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let target = input.parse()?;
        Ok(Self { target })
    }
}
