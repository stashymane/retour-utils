use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, token::Unsafe, Abi, Ident, LitInt, LitStr, Token, Visibility};

use crate::crate_refs::parent_crate;

pub mod kw {
    syn::custom_keyword!(hook);
    syn::custom_keyword!(offset);
    syn::custom_keyword!(symbol);
    syn::custom_keyword!(chain);
}

pub struct HookAttributeArgs {
    pub vis: Visibility,
    pub unsafety: Option<Unsafe>,
    pub abi: Option<Abi>,
    pub detour_name: Ident,
    pub comma: Token![,],
    pub hook_info: HookArg,
    /// If `true`, a `HookChain` static will be emitted alongside the `StaticDetour`.
    pub chain: bool,
}

impl Parse for HookAttributeArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let unsafety = input.parse()?;
        let abi = input.parse()?;
        let detour_name = input.parse()?;
        let comma = input.parse()?;
        let hook_info = input.parse()?;
        // Optional trailing `, chain`
        let chain = if input.peek(Token![,]) && input.peek2(kw::chain) {
            let _: Token![,] = input.parse()?;
            let _: kw::chain = input.parse()?;
            true
        } else {
            false
        };
        Ok(Self { vis, unsafety, abi, detour_name, comma, hook_info, chain })
    }
}

impl ToTokens for HookAttributeArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.detour_name.to_tokens(tokens);
        self.comma.to_tokens(tokens);
        self.hook_info.to_tokens(tokens);
    }
}

pub enum HookArg {
    Offset {
        offset_token: kw::offset,
        eq: Token![=],
        value: LitInt,
    },
    Symbol {
        symbol_token: kw::symbol,
        eq: Token![=],
        value: LitStr,
    },
}

impl HookArg {
    pub fn get_lookup_data_new_fn(&self, module_name: Option<&LitStr>) -> TokenStream {
        let krate_name = parent_crate();
        match (self, module_name) {
            (Self::Offset { value, .. }, Some(m)) => quote::quote! {
                ::#krate_name::LookupData::from_offset(#m, #value)
            },
            (Self::Offset { value, .. }, None) => quote::quote! {
                ::#krate_name::LookupData::from_self_offset(#value)
            },
            (Self::Symbol { value, .. }, Some(m)) => quote::quote! {
                ::#krate_name::LookupData::from_symbol(#m, #value)
            },
            (Self::Symbol { value, .. }, None) => quote::quote! {
                ::#krate_name::LookupData::from_self_symbol(#value)
            },
        }
    }
}

impl Parse for HookArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::offset) {
            Ok(Self::Offset {
                offset_token: input.parse::<kw::offset>()?,
                eq: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::symbol) {
            Ok(Self::Symbol {
                symbol_token: input.parse::<kw::symbol>()?,
                eq: input.parse()?,
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for HookArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            HookArg::Offset {
                offset_token,
                eq,
                value,
            } => {
                offset_token.to_tokens(tokens);
                eq.to_tokens(tokens);
                value.to_tokens(tokens);
            }
            HookArg::Symbol {
                symbol_token,
                eq,
                value,
            } => {
                symbol_token.to_tokens(tokens);
                eq.to_tokens(tokens);
                value.to_tokens(tokens);
            }
        }
    }
}
