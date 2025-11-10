use proc_macro::TokenStream;
use quote::quote;

mod macro_impl;

#[proc_macro_derive(Paramters, attributes(no_param, flag, vec))]
pub fn derive_parameters(input: TokenStream) -> TokenStream {
    let parse_result = syn::parse::<syn::ItemStruct>(input);

    match parse_result {
        Ok(item) => macro_impl::derive_parameters(&item).into(),
        Err(_) => quote! { compile_error!("Item needs to be a struct") }.into(),
    } 
}