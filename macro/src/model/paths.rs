use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use syn::Ident;

pub struct CratePaths {
    pub runtime: Ident,
    pub retour: Ident,
}

impl CratePaths {
    pub fn init() -> Self {
        Self {
            runtime: resolve_crate("retour-utils", "retour_utils"),
            retour: resolve_crate("retour", "retour"),
        }
    }

    pub fn lookup_data(&self) -> TokenStream {
        let p = &self.runtime;
        quote::quote! { ::#p::LookupData }
    }

    pub fn static_detour(&self) -> TokenStream {
        let r = &self.retour;
        quote::quote! { ::#r::StaticDetour }
    }

    pub fn function(&self) -> TokenStream {
        let r = &self.retour;
        quote::quote! { ::#r::Function }
    }

    pub fn error(&self) -> TokenStream {
        let p = &self.runtime;
        quote::quote! { ::#p::Error }
    }

    pub fn init_detour(&self) -> TokenStream {
        let p = &self.runtime;
        quote::quote! { ::#p::init_detour }
    }

    pub fn private_static_detour(&self) -> TokenStream {
        let p = &self.runtime;
        quote::quote! { ::#p::__private::StaticDetour }
    }

    pub fn private_mutex(&self) -> TokenStream {
        let p = &self.runtime;
        quote::quote! { ::#p::__private::Mutex }
    }
}

fn resolve_crate(name: &str, default: &str) -> Ident {
    let found_crate = crate_name(name).expect(&format!("{} is present in `Cargo.toml`", name));

    match found_crate {
        FoundCrate::Itself => Ident::new(default, Span::call_site()),
        FoundCrate::Name(name) => Ident::new(&name.replace('-', "_"), Span::call_site()),
    }
}
