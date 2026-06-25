use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::Item;

use crate::codegen::{chain, detour, init, ptr};
use crate::collect::module::CollectedModule;
use crate::model::{CratePaths, ExpansionContext};

pub fn generate(collected: CollectedModule, ctx: &ExpansionContext) -> syn::Result<TokenStream> {
    let paths = CratePaths::init();
    let CollectedModule {
        mut module,
        detours,
        ptrs,
    } = collected;

    let Some((_, ref mut content)) = module.content else {
        return Err(syn::Error::new_spanned(
            &module,
            "Could not get content inside `mod`",
        ));
    };

    content.push(module_name_decl(ctx));

    for d in &detours {
        content.push(detour::static_detour(d, &paths));
        if d.attr.chain {
            content.push(chain::chain_static(d, &paths));
        }
    }

    for p in &ptrs {
        content.push(ptr::accessor_type(p));
        content.push(Item::Verbatim(ptr::module_static(p)));
    }

    for d in detours.iter().filter(|d| d.attr.chain) {
        content.push(chain::module_alias(d));
    }

    content.push(Item::Verbatim(init::emit_init_detours_fn(
        &detours, &ptrs, ctx, &paths,
    )));

    Ok(quote::quote! { #module })
}

fn module_name_decl(ctx: &ExpansionContext) -> Item {
    let module_name = ctx.module_name();
    let span = module_name
        .map(|m| m.span())
        .unwrap_or_else(Span::call_site);
    let value = module_name
        .map(|m| quote::quote! { #m })
        .unwrap_or_else(|| quote::quote! { "" });

    Item::Verbatim(quote_spanned! {span=>
        #[allow(unused)]
        pub const MODULE_NAME: &str = #value;
    })
}
