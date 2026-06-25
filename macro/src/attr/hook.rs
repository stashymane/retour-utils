use super::{kw, LookupTarget};
use syn::parse::{Parse, ParseStream};
use syn::token::Unsafe;
use syn::{Abi, Ident, Token, Visibility};

pub struct HookAttr {
    pub visibility: Visibility,
    pub unsafety: Option<Unsafe>,
    pub abi: Option<Abi>,
    pub detour_name: Ident,
    pub target: LookupTarget,
    pub chain: bool,
}

impl Parse for HookAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let visibility = input.parse()?;
        let unsafety = input.parse()?;
        let abi = input.parse()?;
        let detour_name = input.parse()?;
        let _: Token![,] = input.parse()?;
        let target = input.parse()?;

        let chain = if input.peek(Token![,]) && input.peek2(kw::chain) {
            let _: Token![,] = input.parse()?;
            let _: kw::chain = input.parse()?;
            true
        } else {
            false
        };

        Ok(Self {
            visibility,
            unsafety,
            abi,
            detour_name,
            target,
            chain,
        })
    }
}
