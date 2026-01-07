use crate::model::TypeParser;
use proc_macro2;
use quote::quote;
use quote::{format_ident, ToTokens};
use syn::{parse_quote, Fields, Generics, ItemStruct, Token, Visibility};

pub fn generate_controller4(input: ItemStruct, targs: TypeParser) -> proc_macro2::TokenStream {
    let model = input.ident.clone();
    let paths = targs.gen_doc_paths();
    let mopd_name = format_ident!("oapi_{}", targs.sufix);

    let mut stor = Vec::new();

    stor.push(quote! {
        use super::#model;
        use rwf::prelude::*;
        use rwf::http::{Request, Response};
        use rwf::controller::ModelListQuery;
    });

    let mut attrs = Vec::new();
    attrs.push(parse_quote!{
        #[derive(Debug, Clone, Default, rwf::macros::ModelController, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    });

    let controller = ItemStruct {
        attrs,
        vis: Visibility::Public(Token![pub](proc_macro2::Span::call_site())),
        struct_token: Token![struct](proc_macro2::Span::call_site()),
        ident: targs.ctrl.clone(),
        generics: Generics::default(),
        fields: Fields::Unit,
        semi_token: None,
    }
    .into_token_stream();
    stor.push(controller);

    let model = input.ident.clone();
    let pkey_type = targs.ty.clone();
    let ctrl = targs.ctrl.clone();

    let path_impl = targs.gen_oapi(model.clone());

    stor.push(
        quote! {

            impl rwf::controller::RestController for #ctrl {
                type Resource = #pkey_type;
            }

            impl rwf::controller::ModelController for #ctrl {
                type Model = #model;
            }
        }
        .into_token_stream(),
    );

    stor.push(
        quote! {
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
        .into_token_stream(),
    );
    let output = quote! {
        pub mod #mopd_name {
            #(
                #stor
            )*
        }
    };
    //eprintln!("{}", output);
    output.into_token_stream()
}
