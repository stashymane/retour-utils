use syn::fold;
use syn::fold::Fold;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{BareFnArg, FnArg, Ident, Pat, PatIdent, Signature, Token, Type, TypeBareFn, TypePtr};

use crate::attr::HookAttr;

/// Replace every occurrence of `Self` in `ty` with the concrete `self_ty` ident.
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

/// Convert a `self` / `&self` / `&mut self` receiver into a typed pointer argument.
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

/// Build the bare `fn(…) -> …` type for a `StaticDetour<…>` declaration.
/// When `self_ty` is provided, `Self` is replaced and receivers are converted to pointers.
pub fn bare_fn_type(fn_sig: &Signature, hook_attr: &HookAttr, self_ty: Option<&Ident>) -> Type {
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
        lifetimes: None,
        unsafety: hook_attr.unsafety,
        abi: hook_attr.abi.clone(),
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
