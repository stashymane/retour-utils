mod attr;
mod codegen;
mod collect;
mod model;

use crate::collect::impl_block::extract_self_type;
use crate::model::ExpansionContext;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl, ItemMod, LitStr};

#[proc_macro_attribute]
pub fn hook_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemMod);
    let args = if args.is_empty() {
        None
    } else {
        Some(parse_macro_input!(args as LitStr))
    };

    let stream = expand_module(ast, args).unwrap_or_else(syn::Error::into_compile_error);
    stream.into()
}

fn expand_module(
    module: ItemMod,
    module_name: Option<LitStr>,
) -> syn::Result<proc_macro2::TokenStream> {
    let ctx = ExpansionContext::Module { module_name };
    let collected = collect::module::collect(module, &ctx)?;
    codegen::module::generate(collected, &ctx)
}

#[proc_macro_attribute]
pub fn hook_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemImpl);
    let args = if args.is_empty() {
        None
    } else {
        Some(parse_macro_input!(args as LitStr))
    };

    let stream = expand_impl(ast, args).unwrap_or_else(syn::Error::into_compile_error);
    stream.into()
}

fn expand_impl(
    item_impl: ItemImpl,
    module_name: Option<LitStr>,
) -> syn::Result<proc_macro2::TokenStream> {
    let self_type = extract_self_type(&item_impl)?;
    let ctx = ExpansionContext::Impl {
        module_name,
        self_type,
    };
    let collected = collect::impl_block::collect(item_impl, &ctx)?;
    codegen::impl_block::generate(collected, &ctx)
}
