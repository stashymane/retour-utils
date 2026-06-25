use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::ImplItem;

use crate::codegen::{chain, detour, init, ptr};
use crate::collect::impl_block::CollectedImpl;
use crate::model::{CratePaths, ExpansionContext};

pub fn generate(collected: CollectedImpl, ctx: &ExpansionContext) -> syn::Result<TokenStream> {
    let paths = CratePaths::init();
    let CollectedImpl {
        mut impl_block,
        detours,
        ptrs,
    } = collected;

    impl_block
        .items
        .push(ImplItem::Verbatim(init::emit_init_detours_fn(
            &detours, &ptrs, ctx, &paths,
        )));
    for d in detours.iter().filter(|d| d.attr.chain) {
        impl_block
            .items
            .push(ImplItem::Verbatim(chain::impl_const(d)));
    }
    for p in &ptrs {
        impl_block
            .items
            .push(ImplItem::Verbatim(ptr::impl_const(p)));
    }

    let mut output = TokenStream::new();

    for d in &detours {
        detour::static_detour(d, &paths).to_tokens(&mut output);
        if d.attr.chain {
            chain::chain_static(d, &paths).to_tokens(&mut output);
        }
    }
    for p in &ptrs {
        ptr::accessor_type(p).to_tokens(&mut output);
        proc_macro2::TokenStream::from(ptr::impl_static(p)).to_tokens(&mut output);
    }

    impl_block.to_tokens(&mut output);

    Ok(output)
}
