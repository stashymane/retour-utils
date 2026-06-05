use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{spanned::Spanned, Ident, Item, LitStr, Signature};

use crate::{
    crate_refs,
    helpers::{fn_arg_names, fn_arg_names_with_self, fn_type, fn_type_with_self},
    parse::HookAttributeArgs,
};

pub struct DetourInfo {
    pub hook_attr: HookAttributeArgs,
    pub fn_sig: Signature,
    pub self_ty: Option<Ident>,
    pub original_fn_name: Ident,
    pub fn_vis: syn::Visibility,
}

impl DetourInfo {
    pub fn get_static_detour(&self) -> Item {
        let vis = self.hook_attr.vis.clone();

        let detour_krate = crate_refs::retour_crate();
        let detour_name: &Ident = &self.hook_attr.detour_name;
        let fn_type_sig = match &self.self_ty {
            Some(st) => fn_type_with_self(&self.fn_sig, &self.hook_attr, st),
            None => fn_type(&self.fn_sig, &self.hook_attr),
        };
        let target_fn_decl = self.target_fn_decl();
        let arg_names: Vec<_> = match &self.self_ty {
            Some(st) => fn_arg_names_with_self(&self.fn_sig, st)
                .into_iter()
                .map(|c| c.into_owned())
                .collect(),
            None => fn_arg_names(&self.fn_sig)
                .unwrap()
                .into_iter()
                .cloned()
                .collect(),
        };

        let ffi_body = if self.hook_attr.chain {
            let chain_name = syn::Ident::new(
                &format!("{}__chain", detour_name),
                detour_name.span(),
            );
            quote::quote! { #chain_name.call(#(#arg_names),*) }
        } else {
            quote::quote! {
                #[allow(unused_unsafe)]
                (#detour_name.__detour())(#(#arg_names),*)
            }
        };

        Item::Verbatim(quote_spanned! {self.hook_attr.span()=>
            #[allow(non_upper_case_globals, unused_unsafe)]
            #vis static #detour_name: ::#detour_krate::StaticDetour<#fn_type_sig> = {
                #[inline(never)]
                #[allow(unused_unsafe, unsafe_op_in_unsafe_fn)]
                #target_fn_decl {
                    #ffi_body
                }
                ::#detour_krate::StaticDetour::__new(__ffi_detour)
            };
        })
    }

    fn target_fn_decl(&self) -> TokenStream {
        let input_args: Vec<_> = match &self.self_ty {
            Some(st) => {
                use crate::helpers::{receiver_to_ptr_arg, replace_self_in_type};
                use syn::FnArg;
                self.fn_sig
                    .inputs
                    .iter()
                    .map(|arg| match arg {
                        FnArg::Typed(pt) => {
                            let ty = replace_self_in_type(*pt.ty.clone(), st);
                            let pat = &pt.pat;
                            quote::quote! { #pat: #ty }
                        }
                        FnArg::Receiver(recv) => {
                            let typed = receiver_to_ptr_arg(recv, st);
                            if let FnArg::Typed(pt) = typed {
                                let pat = &pt.pat;
                                let ty = &pt.ty;
                                quote::quote! { #pat: #ty }
                            } else {
                                quote::quote! {}
                            }
                        }
                    })
                    .collect()
            }
            None => self
                .fn_sig
                .inputs
                .iter()
                .map(|arg| quote::quote! { #arg })
                .collect(),
        };

        let output_type = &self.fn_sig.output;
        let abi = &self.hook_attr.abi;
        let unsafety = &self.hook_attr.unsafety;

        quote::quote! {
            #unsafety #abi fn __ffi_detour(#(#input_args),*) #output_type
        }
    }

    pub fn get_chain_mod_alias(&self) -> Item {
        let vis = self.fn_vis.clone();
        let fn_name = &self.original_fn_name;
        let detour_name = &self.hook_attr.detour_name;
        let chain_name = syn::Ident::new(&format!("{}__chain", detour_name), detour_name.span());
        let chain_type_name = syn::Ident::new(&format!("{}__ChainType", detour_name), detour_name.span());
        Item::Verbatim(quote::quote! {
            #[allow(non_upper_case_globals)]
            #vis static #fn_name: &#chain_type_name = &#chain_name;
        })
    }

    pub fn get_chain_impl_const(&self) -> TokenStream {
        let vis = self.fn_vis.clone();
        let fn_name = &self.original_fn_name;
        let detour_name = &self.hook_attr.detour_name;
        let chain_name = syn::Ident::new(&format!("{}__chain", detour_name), detour_name.span());
        let chain_type_name = syn::Ident::new(&format!("{}__ChainType", detour_name), detour_name.span());
        quote::quote! {
            #[allow(non_upper_case_globals)]
            #vis const #fn_name: &#chain_type_name = &#chain_name;
        }
    }

    pub fn get_chain_static(&self) -> Item {
        let vis = self.fn_vis.clone();
        let detour_name = &self.hook_attr.detour_name;
        let chain_name = syn::Ident::new(
            &format!("{}__chain", detour_name),
            detour_name.span(),
        );
        let chain_type_name = syn::Ident::new(
            &format!("{}__ChainType", detour_name),
            detour_name.span(),
        );
        let parent_krate = crate_refs::parent_crate();
        let fn_type_sig = match &self.self_ty {
            Some(st) => fn_type_with_self(&self.fn_sig, &self.hook_attr, st),
            None => fn_type(&self.fn_sig, &self.hook_attr),
        };

        let (typed_args, arg_types, arg_names, ret_ty) = self.chain_call_parts();
        let maybe_unsafe = self.hook_attr.unsafety.as_ref().map(|_| quote::quote! { unsafe });

        Item::Verbatim(quote_spanned! {self.hook_attr.span()=>
            #[allow(non_camel_case_types, dead_code)]
            #vis struct #chain_type_name {
                detour: &'static ::#parent_krate::__private::StaticDetour<#fn_type_sig>,
                wrappers: ::#parent_krate::__private::Mutex<
                    ::std::vec::Vec<::std::sync::Arc<
                        dyn Fn(#(#arg_types,)* &dyn Fn(#(#arg_types),*) #ret_ty) #ret_ty + Send + Sync
                    >>
                >,
            }

            #[allow(dead_code)]
            impl #chain_type_name {
                #[doc(hidden)]
                #vis const fn new(
                    detour: &'static ::#parent_krate::__private::StaticDetour<#fn_type_sig>,
                ) -> Self {
                    Self {
                        detour,
                        wrappers: ::#parent_krate::__private::Mutex::new(::std::vec::Vec::new()),
                    }
                }

                /// Register a wrapper closure. Each wrapper receives the arguments plus a `next`
                /// closure that invokes the remaining wrappers and ultimately the original function.
                /// If a wrapper does not call `next`, the inner wrappers and original are skipped.
                #vis fn hook<__F>(&self, f: __F)
                where
                    __F: Fn(#(#arg_types,)* &dyn Fn(#(#arg_types),*) #ret_ty) #ret_ty + Send + Sync + 'static,
                {
                    self.wrappers.lock().unwrap().push(::std::sync::Arc::new(f));
                }

                /// Call the wrapper chain. Each wrapper may call `next` to proceed to the next
                /// wrapper; the last `next` calls the original detoured function.
                #vis fn call(&self, #(#typed_args),*) #ret_ty {
                    let snapshot: ::std::vec::Vec<::std::sync::Arc<
                        dyn Fn(#(#arg_types,)* &dyn Fn(#(#arg_types),*) #ret_ty) #ret_ty + Send + Sync
                    >> = self.wrappers.lock().unwrap().iter().rev().cloned().collect();
                    let detour = self.detour;

                    fn call_chain(
                        wrappers: &[::std::sync::Arc<dyn Fn(#(#arg_types,)* &dyn Fn(#(#arg_types),*) #ret_ty) #ret_ty + Send + Sync>],
                        detour: &'static ::#parent_krate::__private::StaticDetour<#fn_type_sig>,
                        #(#typed_args),*
                    ) #ret_ty {
                        if let Some((first, rest)) = wrappers.split_first() {
                            let rest = rest.to_vec();
                            first(#(#arg_names,)* &move |#(#arg_names),*| call_chain(&rest, detour, #(#arg_names),*))
                        } else {
                            #maybe_unsafe { (detour.__detour())(#(#arg_names),*) }
                        }
                    }
                    call_chain(&snapshot, detour, #(#arg_names),*)
                }
            }

            #[allow(non_upper_case_globals)]
            #vis static #chain_name: #chain_type_name =
                #chain_type_name::new(&#detour_name);
        })
    }

    /// Returns `(typed_args, arg_types, arg_names, return_type)` tokens for chain call/hook methods.
    fn chain_call_parts(
        &self,
    ) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>, TokenStream) {
        use crate::helpers::{receiver_to_ptr_arg, replace_self_in_type};
        use syn::FnArg;

        let mut typed_args = Vec::new();
        let mut arg_types = Vec::new();
        let mut arg_names = Vec::new();

        for (i, arg) in self.fn_sig.inputs.iter().enumerate() {
            match arg {
                FnArg::Typed(pt) => {
                    let ty = if let Some(st) = &self.self_ty {
                        replace_self_in_type(*pt.ty.clone(), st)
                    } else {
                        *pt.ty.clone()
                    };
                    let pat = &pt.pat;
                    typed_args.push(quote::quote! { #pat: #ty });
                    arg_types.push(quote::quote! { #ty });
                    arg_names.push(quote::quote! { #pat });
                }
                FnArg::Receiver(recv) => {
                    if let Some(st) = &self.self_ty {
                        let typed = receiver_to_ptr_arg(recv, st);
                        if let FnArg::Typed(pt) = typed {
                            let pat = &pt.pat;
                            let ty = &pt.ty;
                            typed_args.push(quote::quote! { #pat: #ty });
                            arg_types.push(quote::quote! { #ty });
                            arg_names.push(quote::quote! { #pat });
                        }
                    } else {
                        let name = syn::Ident::new(&format!("__arg{}", i), Span::call_site());
                        typed_args.push(quote::quote! { #name: *mut () });
                        arg_types.push(quote::quote! { *mut () });
                        arg_names.push(quote::quote! { #name });
                    }
                }
            }
        }

        let ret_ty = match &self.fn_sig.output {
            syn::ReturnType::Default => quote::quote! {},
            syn::ReturnType::Type(arrow, ty) => quote::quote! { #arrow #ty },
        };

        (typed_args, arg_types, arg_names, ret_ty)
    }

    pub fn generate_detour_init_with_prefix(&self, module_name: Option<&LitStr>, struct_name: &syn::Ident) -> Item {
        let lookup_new_fn = self.hook_attr.hook_info.get_lookup_data_new_fn(module_name);
        let detour_name = &self.hook_attr.detour_name;
        let orig_func_name = &self.fn_sig.ident;
        let parent_krate = crate_refs::parent_crate();
        let detour_krate = crate_refs::retour_crate();
        Item::Verbatim(quote_spanned! {self.hook_attr.span()=>
            ::#parent_krate::init_detour(
                #lookup_new_fn,
                |addr| unsafe {
                    #detour_name
                        .initialize(::#detour_krate::Function::from_ptr(addr), #struct_name::#orig_func_name)?
                        .enable()?;
                    Ok(())
                }
            )?
        })
    }

    pub fn generate_detour_init(&self, module_name: Option<&LitStr>) -> Item {
        let lookup_new_fn = self.hook_attr.hook_info.get_lookup_data_new_fn(module_name);
        let detour_name = &self.hook_attr.detour_name;
        let orig_func_name = &self.fn_sig.ident;
        let parent_krate = crate_refs::parent_crate();
        let detour_krate = crate_refs::retour_crate();
        Item::Verbatim(quote_spanned! {self.hook_attr.span()=>
            ::#parent_krate::init_detour(
                #lookup_new_fn,
                |addr| unsafe {
                    #detour_name
                        .initialize(::#detour_krate::Function::from_ptr(addr), #orig_func_name)?
                        .enable()?;
                    Ok(())
                }
            )?
        })
    }
}
