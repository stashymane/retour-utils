mod crate_refs;
mod expand;
mod fold;
mod helpers;
mod parse;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl, ItemMod, LitStr};

#[proc_macro_attribute]
pub fn hook_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemMod);
    let args = parse_macro_input!(args as LitStr);

    let stream = expand::expand(ast, args).unwrap_or_else(syn::Error::into_compile_error);
    stream.into()
}

#[proc_macro_attribute]
pub fn hook_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemImpl);
    let args = parse_macro_input!(args as LitStr);

    let stream = expand::expand_impl(ast, args).unwrap_or_else(syn::Error::into_compile_error);
    stream.into()
}
