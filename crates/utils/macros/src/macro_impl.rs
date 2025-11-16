use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;

pub fn derive_parameters(item: &syn::ItemStruct) -> TokenStream {
    let item_ident = &item.ident;

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut field_functions: Vec<TokenStream> = vec![];

    'outer: for field in &item.fields {
        let field_type = field.ty.clone();
        let field_ident = field.ident.clone().unwrap();
        let mut flag_add_ident = None;

        let mut vec_push_ident = None;

        for field_attr in &field.attrs {
            if field_attr.path().is_ident("no_param") {
                continue 'outer;
            } else if field_attr.path().is_ident("flag") {
                flag_add_ident = match field_attr.parse_args::<syn::Ident>() {
                    Ok(ident) => Some(ident.to_token_stream()),
                    Err(_) => TokenStream::from_str(format!("add_{}", field_ident).as_str()).ok(),
                }
            } else if field_attr.path().is_ident("vec") {
                vec_push_ident = if let syn::Type::Path(syn::TypePath {
                    path: syn::Path { ref segments, .. },
                    ..
                }) = field_type
                {
                    if let Some(syn::PathSegment {
                        ident,
                        arguments:
                            syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                                args,
                                ..
                            }),
                    }) = segments.last()
                    {
                        if ident.to_string() != "Vec" {
                            return quote_spanned! { field_attr.meta.span() => compile_error!("Attribute 'vec' on a non-'Vec' field"); };
                        }

                        let element_type = match args.first() {
                            Some(syn::GenericArgument::Type(ty)) => ty.clone(),
                            _ => {
                                return quote_spanned! { field_attr.meta.span() => compile_error!("Could not identify element type"); };
                            }
                        };

                        match field_attr.parse_args::<syn::Ident>() {
                            Ok(ident) => Some((element_type, ident.to_token_stream())),
                            Err(_) => {
                                TokenStream::from_str(format!("push_{}", field_ident).as_str())
                                    .ok()
                                    .map(|id| (element_type, id))
                            }
                        }
                    } else {
                        return quote_spanned! { field_attr.meta.span() => compile_error!("Attribute 'vec' on a non-'Vec' field"); };
                    }
                } else {
                    return quote_spanned! { field_attr.meta.span() => compile_error!("Attribute 'vec' on a non-'Vec' field"); };
                }
            }
        }

        field_functions.push(quote! {
            pub fn #field_ident(mut self, val: impl Into<#field_type>) -> Self {
                self.#field_ident = val.into();
                self
            }
        });

        if let Some(flag_add_ident) = flag_add_ident {
            field_functions.push(quote! {
                pub fn #flag_add_ident(mut self, val: impl Into<#field_type>) -> Self {
                    self.#field_ident |= val.into();
                    self
                }
            });
        } else if let Some((ty, id)) = vec_push_ident {
            field_functions.push(quote! {
                pub fn #id(mut self, val: impl Into<#ty>) -> Self {
                    self.#field_ident.push(val.into());
                    self
                }
            });
        }
    }
    quote! {
        impl #impl_generics #item_ident #ty_generics #where_clause {
            #(#field_functions)*
        }
    }
}

pub fn derive_share(item: &syn::Item) -> TokenStream {
    let item_ident;
    let item_generics;

    match item {
        syn::Item::Enum(item) => {
            item_ident = &item.ident;
            item_generics = &item.generics;
        }
        syn::Item::Struct(item) => {
            item_ident = &item.ident;
            item_generics = &item.generics;
        }
        _ => return quote! { compile_error!("Item needs to be a struct or enum") },
    }

    let (impl_generics, ty_generics, where_clause) = item_generics.split_for_impl();

    quote! {
        impl #impl_generics ::utils::Share for #item_ident #ty_generics #where_clause {
            type Internal = #item_ident #ty_generics;

            #[inline]
            fn share(self) -> ::utils::Shared<Self::Internal> {
                ::utils::Shared::new(self)
            }
        }

        impl #impl_generics #item_ident #ty_generics #where_clause {
            #[inline]
            pub fn share(self) -> ::utils::Shared<#item_ident #ty_generics> {
                ::utils::Shared::new(self)
            }
        }
    }
}
