extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields, Meta};

use quote::quote;

#[proc_macro_derive(Model, attributes(belongs_to))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // let attrs = input.attrs;
    // let ident = input.ident.clone();

    // println!("{:#?}", attrs);

    // let belongs_to = attrs.iter().filter(|attr| {
    //     if let Some(path_segment) = attr.meta.path().segments.first() {
    //         path_segment.ident == "belongs_to"
    //     } else {
    //         false
    //     }
    // })
    // .map(|attr| {
    //     match &attr.meta {
    //         Meta::List(meta) => {
    //             meta.tokens.clone().iter().map(|token| {
    //                 quote! {
    //                     impl rum::model::Association<#token> for #ident {}
    //                 }
    //             })
    //         }

    //         _ => panic!("incorrect"),
    //     }
    // });

    match input.data {
        Data::Struct(ref data) => {
            let ident = input.ident.clone();
            let from_row_fields = data.fields.iter().map(|field| {
                let ident = field.ident.clone();
                quote! {
                    #ident: row.get(stringify!(#ident)),
                }
            });

            quote!  {
                #[automatically_derived]
                impl rum::model::FromRow  for #ident {
                    fn from_row(row: rum::tokio_postgres::Row) -> Self {
                        Self {
                            #(#from_row_fields)*
                        }
                    }
                }

                #[automatically_derived]
                impl rum::model::Model for #ident {}
            }.into()
        }

        _ => panic!("macro can only be used on structs"),
    }
}

#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(ref data) => {
            let ident = input.ident;

            let from_row_fields = data.fields.iter().map(|field| {
                let ident = &field.ident;
                quote! {
                    #ident: row.get(stringify!(#ident)),
                }
            });

            quote!{
                #[automatically_derived]
                impl rum::model::FromRow for #ident {
                    fn from_row(row: rum::tokio_postgres::Row) -> Self {
                        Self {
                            #(#from_row_fields)*
                        }
                    }
                }
            }.into()
        }

        _ => panic!("macro can only be used on structs"),
    }
}
