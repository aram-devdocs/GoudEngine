use proc_macro2::TokenStream;
use quote::quote;
use syn::FnArg;

/// Receiver type for a method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReceiverKind {
    Ref,
    Mut,
    None,
}

/// Determines the receiver kind from a method's parameters.
pub fn determine_receiver(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> ReceiverKind {
    for input in inputs {
        if let FnArg::Receiver(recv) = input {
            return if recv.mutability.is_some() {
                ReceiverKind::Mut
            } else {
                ReceiverKind::Ref
            };
        }
    }
    ReceiverKind::None
}

/// Generates the registry access expression for the given receiver kind.
pub fn registry_access_for(recv: ReceiverKind) -> TokenStream {
    match recv {
        ReceiverKind::Ref => quote! {
            let context = match registry.get(context_id) { Some(c) => c, None => return Default::default() };
        },
        ReceiverKind::Mut => quote! {
            let context = match registry.get_mut(context_id) { Some(c) => c, None => return Default::default() };
        },
        ReceiverKind::None => quote! {},
    }
}
