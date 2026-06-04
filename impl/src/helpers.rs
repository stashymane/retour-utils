use std::borrow::Cow;

use syn::{
    fold::{self, Fold},
    punctuated::Punctuated,
    spanned::Spanned,
    BareFnArg, FnArg, Ident, Pat, PatIdent, Signature, Token, Type, TypeBareFn, TypePtr,
};

use crate::parse::HookAttributeArgs;

pub fn replace_self_in_type(ty: Type, self_ty: &Ident) -> Type {
    struct SelfReplacer<'a>(&'a Ident);
    impl Fold for SelfReplacer<'_> {
        fn fold_type(&mut self, ty: Type) -> Type {
            match &ty {
                Type::Path(tp) if tp.qself.is_none() && tp.path.is_ident("Self") => {
                    Type::Path(syn::TypePath {
                        qself: None,
                        path: self.0.clone().into(),
                    })
                }
                _ => fold::fold_type(self, ty),
            }
        }
    }
    SelfReplacer(self_ty).fold_type(ty)
}

pub fn receiver_to_ptr_arg(receiver: &syn::Receiver, self_ty: &Ident) -> FnArg {
    let self_path: Type = Type::Path(syn::TypePath {
        qself: None,
        path: self_ty.clone().into(),
    });
    let ptr_ty: Type = if let Some((and, _lifetime)) = &receiver.reference {
        let mutability = receiver.mutability;
        Type::Ptr(TypePtr {
            star_token: Token![*](and.span()),
            const_token: if mutability.is_none() {
                Some(Token![const](and.span()))
            } else {
                None
            },
            mutability,
            elem: Box::new(self_path),
        })
    } else {
        Type::Ptr(TypePtr {
            star_token: Token![*](receiver.self_token.span),
            const_token: None,
            mutability: Some(Token![mut](receiver.self_token.span)),
            elem: Box::new(self_path),
        })
    };
    FnArg::Typed(syn::PatType {
        attrs: receiver.attrs.clone(),
        pat: Box::new(Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: Ident::new("self_ptr", receiver.self_token.span),
            subpat: None,
        })),
        colon_token: Token![:](receiver.self_token.span),
        ty: Box::new(ptr_ty),
    })
}

pub fn fn_type(fn_sig: &Signature, hook_info: &HookAttributeArgs) -> Type {
    fn_type_impl(fn_sig, hook_info, None)
}

pub fn fn_type_with_self(fn_sig: &Signature, hook_info: &HookAttributeArgs, self_ty: &Ident) -> Type {
    fn_type_impl(fn_sig, hook_info, Some(self_ty))
}

fn fn_type_impl(fn_sig: &Signature, hook_info: &HookAttributeArgs, self_ty: Option<&Ident>) -> Type {
    let mut args = Punctuated::new();
    for arg in &fn_sig.inputs {
        match arg {
            FnArg::Typed(arg) => {
                let ty = if let Some(st) = self_ty {
                    replace_self_in_type(*arg.ty.clone(), st)
                } else {
                    *arg.ty.clone()
                };
                args.push(BareFnArg {
                    attrs: arg.attrs.clone(),
                    name: None,
                    ty,
                });
            }
            FnArg::Receiver(recv) => {
                if let Some(st) = self_ty {
                    let typed = receiver_to_ptr_arg(recv, st);
                    if let FnArg::Typed(pt) = typed {
                        args.push(BareFnArg {
                            attrs: pt.attrs,
                            name: None,
                            ty: *pt.ty,
                        });
                    }
                }
            }
        }
    }

    Type::BareFn(TypeBareFn {
        lifetimes: None, // TODO: maybe support lifetimes
        unsafety: hook_info.unsafety,
        abi: hook_info.abi.clone(),
        fn_token: fn_sig.fn_token,
        paren_token: fn_sig.paren_token,
        inputs: args,
        variadic: fn_sig.variadic.clone().map(|var| syn::BareVariadic {
            attrs: var.attrs,
            name: None,
            dots: var.dots,
            comma: var.comma,
        }),
        output: fn_sig.output.clone(),
    })
}

pub fn fn_arg_names(fn_sig: &Signature) -> Result<Vec<&Pat>, syn::Error> {
    let mut args = Vec::new();
    let mut errs: Option<syn::Error> = None;
    for arg in &fn_sig.inputs {
        match arg {
            FnArg::Typed(arg) => args.push(arg.pat.as_ref()),
            FnArg::Receiver(_) => {
                let err = syn::Error::new(
                    arg.span(),
                    "`self` is not currently supported by this macro",
                );
                match &mut errs {
                    Some(errs) => errs.combine(err),
                    None => errs = Some(err),
                }
            }
        }
    }
    if let Some(e) = errs {
        Err(e)
    } else {
        Ok(args)
    }
}

pub fn fn_arg_names_with_self<'a>(
    fn_sig: &'a Signature,
    self_ty: &Ident,
) -> Vec<Cow<'a, Pat>> {
    let mut args = Vec::new();
    for arg in &fn_sig.inputs {
        match arg {
            FnArg::Typed(arg) => args.push(Cow::Borrowed(arg.pat.as_ref())),
            FnArg::Receiver(recv) => {
                let typed = receiver_to_ptr_arg(recv, self_ty);
                if let FnArg::Typed(pt) = typed {
                    args.push(Cow::Owned(*pt.pat));
                }
            }
        }
    }
    args
}

pub fn fn_types(fn_sig: &Signature) -> Result<Vec<&Type>, syn::Error> {
    let mut types = Vec::new();
    let mut errs: Option<syn::Error> = None;
    for arg in &fn_sig.inputs {
        match arg {
            FnArg::Typed(arg) => types.push(arg.ty.as_ref()),
            FnArg::Receiver(_) => {
                let err = syn::Error::new(
                    arg.span(),
                    "`self` is not currently supported by this macro",
                );
                match &mut errs {
                    Some(errs) => errs.combine(err),
                    None => errs = Some(err),
                }
            }
        }
    }
    if let Some(e) = errs {
        Err(e)
    } else {
        Ok(types)
    }
}
