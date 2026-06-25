use proc_macro2::TokenStream;
use syn::{Ident, Item};

use crate::model::Ptr;

/// Emit the accessor struct definition (placed outside the macro/module).
pub fn accessor_type(ptr: &Ptr) -> Item {
    let vis = &ptr.vis;
    let ty = &ptr.ty;
    let name = &ptr.name;
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

/// Emit the free static declaration for module context.
pub fn module_static(ptr: &Ptr) -> TokenStream {
    let vis = &ptr.vis;
    let name = &ptr.name;
    let type_name = Ident::new(&format!("{}_PtrAccessor", name), name.span());
    quote::quote! {
        #[allow(non_upper_case_globals)]
        #vis static #name: #type_name = #type_name::new();
    }
}

/// Emit the macro-scoped backing static (placed outside the macro block).
pub fn impl_static(ptr: &Ptr) -> TokenStream {
    let name = &ptr.name;
    let type_name = Ident::new(&format!("{}_PtrAccessor", name), name.span());
    let static_name = Ident::new(&format!("__PTR_{}", name), name.span());
    quote::quote! {
        #[allow(non_upper_case_globals)]
        static #static_name: #type_name = #type_name::new();
    }
}

/// Emit the associated const inside the macro block that references the backing static.
pub fn impl_const(ptr: &Ptr) -> TokenStream {
    let vis = &ptr.vis;
    let name = &ptr.name;
    let type_name = Ident::new(&format!("{}_PtrAccessor", name), name.span());
    let static_name = Ident::new(&format!("__PTR_{}", name), name.span());
    quote::quote! {
        #[allow(non_upper_case_globals)]
        #vis const #name: &#type_name = &#static_name;
    }
}
