use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, parse_macro_input, Pat, PatIdent, PatType, Signature};

#[proc_macro_attribute]
pub fn two_f_a(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut inner = parse_macro_input!(item as ItemFn);
    let wrapper_sig = inner.sig.clone();
    let inner_method_name = format_ident!("{}_log_error", inner.sig.ident);
    inner.sig.ident = inner_method_name.clone();

    let is_async = inner.sig.asyncness.is_some();
    let arg_names = get_arg_names(&inner.sig);

    let function_call = if is_async {
        quote! { #inner_method_name ( #(#arg_names),* ) .await }
    } else {
        quote! { #inner_method_name ( #(#arg_names),* ) }
    };

    let expanded = quote!(
        #[allow(unused_mut)]
        #wrapper_sig {
            secure_2fa();
            #function_call
        }
        #inner
    );
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn admin(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut inner = parse_macro_input!(item as ItemFn);
    let wrapper_sig = inner.sig.clone();
    let inner_method_name = format_ident!("{}_admin", inner.sig.ident);
    inner.sig.ident = inner_method_name.clone();

    let is_async = inner.sig.asyncness.is_some();
    let arg_names = get_arg_names(&inner.sig);

    let function_call = if is_async {
        quote! { #inner_method_name ( #(#arg_names),* ) .await }
    } else {
        quote! { #inner_method_name ( #(#arg_names),* ) }
    };

    let expanded = quote!(
        #[allow(unused_mut)]
        #wrapper_sig {
            let caller = get_caller();
            if AdminRepo::get().eq(&caller)
                || ControllersRepo::contains(&caller) {
                #function_call
            } else {
                trap("Unauthorized")
            }
        }
        #inner
    );
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn lambda(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut inner = parse_macro_input!(item as ItemFn);
    let wrapper_sig = inner.sig.clone();
    let inner_method_name = format_ident!("{}_admin", inner.sig.ident);
    inner.sig.ident = inner_method_name.clone();

    let is_async = inner.sig.asyncness.is_some();
    let arg_names = get_arg_names(&inner.sig);

    let function_call = if is_async {
        quote! { #inner_method_name ( #(#arg_names),* ) .await }
    } else {
        quote! { #inner_method_name ( #(#arg_names),* ) }
    };

    let expanded = quote!(
        #[allow(unused_mut)]
        #wrapper_sig {
            let caller = get_caller();
                    if AdminRepo::get().eq(&caller)
                    || ConfigurationRepo::get().lambda.eq(&caller) {
                #function_call
            } else {
                trap("Unauthorized")
            }
        }
        #inner
    );
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn operator(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut inner = parse_macro_input!(item as ItemFn);
    let wrapper_sig = inner.sig.clone();
    let inner_method_name = format_ident!("{}_admin", inner.sig.ident);
    inner.sig.ident = inner_method_name.clone();

    let is_async = inner.sig.asyncness.is_some();
    let arg_names = get_arg_names(&inner.sig);

    let function_call = if is_async {
        quote! { #inner_method_name ( #(#arg_names),* ) .await }
    } else {
        quote! { #inner_method_name ( #(#arg_names),* ) }
    };

    let expanded = quote!(
        #[allow(unused_mut)]
        #wrapper_sig {
            let caller = get_caller();
                    if ConfigurationRepo::get().operator.eq(&caller) {
                #function_call
            } else {
                trap("Unauthorized")
            }
        }
        #inner
    );
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn paused(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut inner = parse_macro_input!(item as ItemFn);
    let wrapper_sig = inner.sig.clone();
    let inner_method_name = format_ident!("{}_admin", inner.sig.ident);
    inner.sig.ident = inner_method_name.clone();

    let is_async = inner.sig.asyncness.is_some();
    let arg_names = get_arg_names(&inner.sig);

    let function_call = if is_async {
        quote! { #inner_method_name ( #(#arg_names),* ) .await }
    } else {
        quote! { #inner_method_name ( #(#arg_names),* ) }
    };

    let expanded = quote!(
        #[allow(unused_mut)]
        #wrapper_sig {
            if ConfigurationRepo::get().account_creation_paused {
                trap("Account creation is paused due to high demand. Please try again later.")
            } else {
                #function_call
            }
        }
        #inner
    );
    TokenStream::from(expanded)
}

fn get_arg_names(signature: &Signature) -> Vec<Ident> {
    signature
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(r) => r.self_token.into(),
            FnArg::Typed(PatType { pat, .. }) => {
                if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
                    ident.clone()
                } else {
                    panic!("Unable to determine arg name");
                }
            }
        })
        .collect()
}
