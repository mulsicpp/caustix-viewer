use proc_macro2::TokenStream;
use quote::quote;

pub fn derive_parameters(item: syn::ItemStruct) -> TokenStream {
    let item_ident = item.ident;

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut field_functions: Vec<TokenStream> = vec![];
    
    'outer: for field in item.fields {
        let field_type = field.ty;
        let field_ident = field.ident.unwrap();

        for field_attr in field.attrs {
            if field_attr.path().is_ident("no_param") { continue 'outer; }
        }

        field_functions.push(quote! {
            pub fn #field_ident(mut self, val: impl Into<#field_type>) -> Self {
                self.#field_ident = val.into();
                self
            }
        });
    }
    quote! { 
        impl #impl_generics #item_ident #ty_generics #where_clause {
            #(#field_functions)*
        }
    }
}
