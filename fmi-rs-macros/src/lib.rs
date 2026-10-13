use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, Pat, parse_macro_input};

#[proc_macro_attribute]
pub fn log_call(attribute: TokenStream, item: TokenStream) -> TokenStream {
    if !attribute.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "log_call does not accept arguments",
        )
        .to_compile_error()
        .into();
    }

    let mut function = parse_macro_input!(item as ItemFn);
    let name = function.sig.ident.to_string();
    let mut characters = name.chars();
    let capitalized_name = if let Some(first) = characters.next() {
        first.to_uppercase().chain(characters).collect::<String>()
    } else {
        name
    };
    let function_name = syn::LitStr::new(
        &format!("fmi3{capitalized_name}"),
        proc_macro2::Span::call_site(),
    );
    let mut argument_names = Vec::new();

    for argument in &function.sig.inputs {
        if let FnArg::Typed(argument) = argument {
            if let Pat::Ident(pattern) = argument.pat.as_ref() {
                argument_names.push(pattern.ident.clone());
            } else {
                return syn::Error::new_spanned(
                    &argument.pat,
                    "log_call requires named function arguments",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    let argument_format = argument_names
        .iter()
        .map(|name| format!("{}={{:?}}", name))
        .collect::<Vec<_>>()
        .join(", ");
    let body = function.block;

    function.block = Box::new(syn::parse_quote!({
        let __fmi_log_status = #body;

        if self.logCalls {
            let __fmi_log_arguments = format!(#argument_format, #(#argument_names),*);
            let __fmi_log_message = format!(
                "{}({}) -> {:?}",
                #function_name,
                __fmi_log_arguments,
                __fmi_log_status,
            );
            self.log_call(__fmi_log_status, &__fmi_log_message);
        }

        __fmi_log_status
    }));

    quote!(#function).into()
}
