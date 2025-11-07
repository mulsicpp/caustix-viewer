use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub fn derive_vk_handle(item: syn::ItemStruct) -> TokenStream {
    let item_ident = item.ident;

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let fields = item.fields.iter().collect::<Vec<_>>();

    let mut attr_field = None;
    let mut handle_field = None;
    let first_field = fields.first().map(|&f| (0, f));

    for (i, &field) in fields.iter().enumerate() {
        for field_attr in &field.attrs {
            if attr_field.is_none() && field_attr.path().is_ident("handle") {
                attr_field = Some((i, field));
            }
        }

        if handle_field.is_none() && field.ident.as_ref().and_then(|ident| Some(ident.to_string() == "handle")).unwrap_or(false) {
            handle_field = Some((i, field));
        }
    }

    let field = attr_field.or(handle_field.or(first_field));

    if let Some((i, field)) = field {
        let ref field_type = field.ty;
        let field_ident = if let Some(ident) = field.ident.as_ref() {
            ident.to_token_stream()
        } else {
            syn::Index::from(i).to_token_stream()
        };

        quote! {
                impl #impl_generics crate::handle::VkHandle for #item_ident #ty_generics #where_clause {
                    type HandleType = #field_type;

                    fn handle(&self) -> Self::HandleType {
                        self.#field_ident
                    }
                }
            }
    } else {
        quote! {
            compile_error!("Failed to find a suitable field to be used as the handle")
        }
    }
}
