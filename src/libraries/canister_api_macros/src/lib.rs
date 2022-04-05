use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatIdent, PatType, Signature};

#[proc_macro_attribute]
pub fn log_error(_: TokenStream, item: TokenStream) -> TokenStream {
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
            let result = #function_call;
            if(result.error.is_some()) {
                 let l = Log {
                 level: LogLevel::ERROR,
                 log: result.error.clone().unwrap().to_string(),
                 timestamp: ic_service::get_time()
            };
               LogRepo::save(l);
            }
            result
        }
        #inner
    );
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn replicate_account(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut inner = parse_macro_input!(item as ItemFn);
    let wrapper_sig = inner.sig.clone();
    let inner_method_name = format_ident!("{}_replicate", inner.sig.ident);
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
            let princ = caller().to_text();
            let result = #function_call;
            storage::get_mut::<AccountsToReplicate>().insert(princ);
            result
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
                    || (ConfigurationRepo::get().whitelisted_canisters.is_some() &&
                ConfigurationRepo::get().whitelisted_canisters.as_ref().unwrap().contains(&caller)) {
                #function_call
            } else {
                trap("Unauthorized")
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
