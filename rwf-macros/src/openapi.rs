use crate::model::TypeParser;
use proc_macro2;
use quote::quote;
use quote::{format_ident, ToTokens};

use syn::{parse_quote, Fields, Generics, ItemStruct, Token, Visibility};

pub fn generate_controller4(input: ItemStruct, targs: TypeParser) -> proc_macro2::TokenStream {
    let paths = targs.gen_doc_paths();
    let mut output = proc_macro2::TokenStream::new();

    let mut attrs = Vec::new();
    attrs.push(parse_quote!{
        #[derive(Debug, Clone, Default, rwf::macros::ModelController, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    });

    ItemStruct {
        attrs,
        vis: Visibility::Public(Token![pub](proc_macro2::Span::call_site())),
        struct_token: Token![struct](proc_macro2::Span::call_site()),
        ident: targs.ctrl.clone(),
        generics: Generics::default(),
        fields: Fields::Unit,
        semi_token: None,
    }
    .to_tokens(&mut output);

    let model = input.ident.clone();
    let pkey_type = targs.ty.clone();
    let ctrl = targs.ctrl.clone();
    let path_impl = targs.gen_oapi(model.clone());
    let apiname = format_ident!("{}OpenapiDoc", model);

    quote! {
        impl rwf::controller::RestController for #ctrl {
            type Resource = #pkey_type;
        }

        impl rwf::controller::ModelController for #ctrl {
            type Model = #model;

            fn crud(self, path: &str) -> rwf::http::Handler {
                rwf::controller::openapi::registrer_controller(path, #apiname :: openapi);
                rwf::http::Handler::rest(path, self)
            }
        }
    }
    .to_tokens(&mut output);

    quote! {
        #[derive(OpenApi)]
        #[openapi(
            components(
                schemas(#model),
                responses(#model)
            ),
            paths(#paths)
        )]
        pub struct #apiname;

        #(
            #path_impl
        )*
    }
    .to_tokens(&mut output);
    output
}
