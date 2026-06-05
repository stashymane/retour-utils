use syn::{Item, LitStr};

use crate::{crate_refs, parse::PtrItem};

impl PtrItem {
    /// Generate the accessor struct + static declaration.
    pub fn get_accessor_item(&self) -> Item {
        let vis = &self.vis;
        let name = &self.name;
        let ty = &self.ty;
        let type_name = syn::Ident::new(&format!("{}_PtrAccessor", name), name.span());
        Item::Verbatim(quote::quote! {
            #[allow(non_camel_case_types, dead_code)]
            #vis struct #type_name {
                addr: ::std::sync::atomic::AtomicUsize,
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

            #[allow(non_upper_case_globals)]
            #vis static #name: #type_name = #type_name::new();
        })
    }

    /// Generate the init call that resolves the address and stores it.
    pub fn generate_ptr_init(&self, module_name: Option<&LitStr>) -> Item {
        let parent_krate = crate_refs::parent_crate();
        let lookup_new_fn = self.attr.hook_info.get_lookup_data_new_fn(module_name);
        let name = &self.name;
        Item::Verbatim(quote::quote! {
            ::#parent_krate::init_detour(
                #lookup_new_fn,
                |addr| {
                    #name.addr.store(addr as usize, ::std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
            )?
        })
    }
}
