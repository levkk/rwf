use proc_macro2;
use quote::{format_ident, ToTokens};
use syn::{parse_quote, Fields, Generics, ItemStruct, Token, Visibility};
use quote::quote;
use crate::model::TypeParser;


pub fn generate_controller4(input: ItemStruct, targs: TypeParser) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();

    let mut attrs = Vec::new();
    attrs.push(parse_quote!{
        #[derive(Debug, Clone, Default, rwf::macros::ModelController, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    });
    ItemStruct {
        attrs: attrs,
        vis: Visibility::Public(Token![pub](proc_macro2::Span::call_site())),
        struct_token: Token![struct](proc_macro2::Span::call_site()),
        ident: targs.ctrl.clone(),
        generics: Generics::default(),
        fields: Fields::Unit,
        semi_token: None,
    }.to_tokens(&mut output);

    let model = input.ident.clone();
    let pkey_type = targs.ty.clone();
    let ctrl = targs.ctrl.clone();
    let suffix = targs.sufix.clone();
    let apipth = targs.apipth.clone();

    let path_impl = targs.gen_oapi(model.clone());

    quote!{
        impl rwf::controller::RestController for #ctrl {
            type Resource = #pkey_type;
        }

        impl rwf::controller::ModelController for #ctrl {
            type Model = #model;
        }
    }.to_tokens(&mut output);
    let module_name = format_ident!("oapi_{}", suffix);
    let paths = targs.gen_doc_paths();
    quote!{

        impl #ctrl {
            pub fn ocrud(self, path: &str) -> (Handler, Into<OpenApi>) {
                (self.crud(path.clone()), #module_name::ApiDoc::openapi)
            }
        }

        pub(crate) mod #module_name {
            use rwf::prelude::*;

            #[derive(OpenApi)]
            #[openapi(
                components(
                    schemas(#model),
                    responses(#model)
                ),
                paths(#paths)
            )]
            pub struct ApiDoc;

            #(
                #path_impl
            )*
        }
    }.to_tokens(&mut output);


    output.into_token_stream()
}