use proc_macro::TokenStream;
use quote::quote;

mod macro_impl;

#[proc_macro_derive(VkHandle, attributes(handle))]
pub fn derive_vk_handle(input: TokenStream) -> TokenStream {
    let parse_result = syn::parse::<syn::ItemStruct>(input);

    match parse_result {
        Ok(item) => macro_impl::derive_vk_handle(item).into(),
        Err(_) => quote! { compile_error!("Item needs to be a struct") }.into(),
    } 
}