use crate::attr::kw;
use syn::parse::{Parse, ParseStream};
use syn::{LitInt, LitStr, Token};

pub enum LookupTarget {
    Offset(LitInt),
    Symbol(LitStr),
}

impl Parse for LookupTarget {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::offset) {
            let _: kw::offset = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self::Offset(input.parse()?))
        } else if lookahead.peek(kw::symbol) {
            let _: kw::symbol = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self::Symbol(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}
