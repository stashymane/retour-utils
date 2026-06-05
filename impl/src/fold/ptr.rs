use proc_macro2::TokenStream;
use syn::{Ident, Item, LitStr};

use crate::{crate_refs, parse::PtrItem};

impl PtrItem {
    /// Generate the accessor struct definition (placed outside the impl).
    pub fn get_accessor_item(&self) -> Item {
        let vis = &self.vis;
        let ty = &self.ty;
        let name = &self.name;
        let type_name = Ident::new(&format!("{}_PtrAccessor", name), name.span());
        Item::Verbatim(quote::quote! {
            #[allow(non_camel_case_types, dead_code)]
            #vis struct #type_name {
                pub addr: ::std::sync::atomic::AtomicUsize,
            }

            #[allow(dead_code)]
            impl #type_name {
                #[doc(hidden)]
                pub const fn new() -> Self {
                    Self { addr: ::std::sync::atomic::AtomicUsize::new(0) }
                }

                /// Returns a raw pointer to the value at the resolved address.
                pub fn as_ptr(&self) -> *mut #ty {
                    self.addr.load(::std::sync::atomic::Ordering::Relaxed) as *mut #ty
                }

                /// Reads the value at the resolved address.
                ///
                /// # Safety
                /// The address must have been initialised and point to a valid `#ty`.
                pub unsafe fn read(&self) -> #ty {
                    unsafe { self.as_ptr().read() }
                }

                /// Writes a value to the resolved address.
                ///
                /// # Safety
                /// The address must have been initialised and point to a valid `#ty`.
                pub unsafe fn write(&self, val: #ty) {
                    unsafe { self.as_ptr().write(val) }
                }
            }
        })
    }

    /// Generate the free static declaration (for module context).
    pub fn get_accessor_static(&self) -> TokenStream {
        let vis = &self.vis;
        let name = &self.name;
        let type_name = Ident::new(&format!("{}_PtrAccessor", name), name.span());
        quote::quote! {
            #[allow(non_upper_case_globals)]
            #vis static #name: #type_name = #type_name::new();
        }
    }

    /// Generate the impl-scoped static (placed outside the impl, referenced by the impl const).
    pub fn get_accessor_impl_static(&self) -> TokenStream {
        let name = &self.name;
        let type_name = Ident::new(&format!("{}_PtrAccessor", name), name.span());
        let static_name = Ident::new(&format!("__PTR_{}", name), name.span());
        quote::quote! {
            #[allow(non_upper_case_globals)]
            static #static_name: #type_name = #type_name::new();
        }
    }

    /// Generate the associated const inside the impl block that references the external static.
    pub fn get_accessor_impl_const(&self) -> TokenStream {
        let vis = &self.vis;
        let name = &self.name;
        let type_name = Ident::new(&format!("{}_PtrAccessor", name), name.span());
        let static_name = Ident::new(&format!("__PTR_{}", name), name.span());
        quote::quote! {
            #[allow(non_upper_case_globals)]
            #vis const #name: &#type_name = &#static_name;
        }
    }

    /// Generate the init call that resolves the address and stores it.
    /// `use_self` should be true when inside an impl block (uses `Self::NAME`),
    /// false when in a module context (uses plain `NAME`).
    pub fn generate_ptr_init(&self, module_name: Option<&LitStr>, use_self: bool) -> Item {
        let parent_krate = crate_refs::parent_crate();
        let lookup_new_fn = self.attr.hook_info.get_lookup_data_new_fn(module_name);
        let name = &self.name;
        let accessor = if use_self {
            let static_name = Ident::new(&format!("__PTR_{}", name), name.span());
            quote::quote! { #static_name }
        } else {
            quote::quote! { #name }
        };
        Item::Verbatim(quote::quote! {
            ::#parent_krate::init_detour(
                #lookup_new_fn,
                |addr| {
                    #accessor.addr.store(addr as usize, ::std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
            )?
        })
    }
}
