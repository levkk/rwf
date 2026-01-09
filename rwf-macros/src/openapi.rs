use proc_macro2;
use proc_macro2::{Ident, Span};
use quote::quote;
use quote::{format_ident, ToTokens};

use syn::{ItemStruct, LitStr, Token, Type};
use syn::parse::{Parse, ParseStream};
use crate::snake_case;

pub fn generate_controller4(input: ItemStruct, targs: TypeParserInput) -> proc_macro2::TokenStream {
    let targs = targs.into_type_parser(&input);

    let paths = targs.gen_doc_paths();
    let mut output = proc_macro2::TokenStream::new();


    let model = targs.model.clone();
    let pkey_type = targs.ty.clone();
    let ctrl = targs.ctrl.clone();
    let path_impl = targs.gen_oapi();
    let apiname = format_ident!("{}OpenapiDoc", model);

    quote! {
        impl rwf::controller::RestController for #ctrl {
            type Resource = #pkey_type;
        }

        impl rwf::controller::ModelController for #ctrl {
            type Model = #model;

            fn crud(self, path: &str) -> rwf::http::Handler {
                let auth = rwf::controller::Controller::auth(&self);

                let mut spec = #apiname :: openapi() ;
                <dyn rwf::controller::Controller>::modify(&self, &mut spec);
                rwf::controller::openapi::registrer_controller(path, spec);
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
            paths(#paths),
        )]
        pub struct #apiname;

        #(
            #path_impl
        )*
    }
    .to_tokens(&mut output);
    output
}


#[derive(Clone)]
pub struct TypeParser {
    pub ty: Type,
    pub ctrl: Ident,
    pub sufix: Ident,
    pub model: Ident,
}

impl TypeParser {
    pub fn gen_doc_paths(&self) -> proc_macro2::TokenStream {
        let paths = self.path_name_list();
        quote! {
            #(#paths),*
        }
    }
    pub fn gen_oapi(&self) -> Vec<proc_macro2::TokenStream> {
        let mut result = Vec::with_capacity(6);

        let operations = self.operation();
        let paths = self.apipths();
        let responses = self.rewsponses();
        let params = self.params();
        let requests = self.request_body();
        let funcs = self.path_name_list();

        let model_tag = LitStr::new(self.model.to_string().as_str(), Span::call_site());

        operations
            .into_iter()
            .zip(paths.into_iter())
            .zip(params.into_iter().zip(requests.into_iter()))
            .map(|((o, p), (par, req))| (o, p, par, req))
            .zip(responses.into_iter().zip(funcs.into_iter()))
            .map(|((o, p, par, req), (res, func))| (o, p, par, req, res, func))
            .map(|(o, p, par, req, res, func)| {
                let endpoint_tag = LitStr::new({
                                                   if p.token().to_string().as_str().contains("{id}") {"InstanceUrl"} else {"CollectionUrl"}
                                               }, Span::call_site());
                if o.eq("get") || o.eq("delete") {
                    quote! {
                        #[utoipa::path(
                            #o,
                            path=#p,
                            #par,
                            responses(#res),
                            tags=["ModelController", #model_tag, #endpoint_tag],
                        )]
                        fn #func (_request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::http::Error> {
                            Ok(rwf::http::Response::not_implemented())
                        }
                    }
                } else {
                    quote! {
                        #[utoipa::path(
                            #o,
                            path=#p,
                            #par,
                            #req,
                            responses(#res),
                            tags=["ModelController", #model_tag, #endpoint_tag],
                        )]
                        fn #func (_request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::http::Error> {
                            Ok(rwf::http::Response::not_implemented())
                        }
                    }
                }
            })
            .for_each(|ts| result.push(ts));

        result
    }
    pub fn path_name_list(&self) -> Vec<Ident> {
        vec![
            format_ident!("list_{}", self.sufix),
            format_ident!("create_{}", self.sufix),
            format_ident!("get_{}", self.sufix),
            format_ident!("update_{}", self.sufix),
            format_ident!("patch_{}", self.sufix),
            format_ident!("delete_{}", self.sufix),
        ]
    }
    pub fn operation(&self) -> Vec<syn::Ident> {
        vec![
            format_ident!("get"),
            format_ident!("post"),
            format_ident!("get"),
            format_ident!("put"),
            format_ident!("patch"),
            format_ident!("delete"),
        ]
    }
    pub fn apipths(&self) -> Vec<LitStr> {
        vec![
            LitStr::new("/", Span::call_site()),
            LitStr::new("/", Span::call_site()),
            LitStr::new("/{id}", Span::call_site()),
            LitStr::new("/{id}", Span::call_site()),
            LitStr::new("/{id}", Span::call_site()),
            LitStr::new("/{id}", Span::call_site()),
        ]
    }
    pub fn rewsponses(&self) -> Vec<proc_macro2::TokenStream> {
        let model = self.model.clone();
        vec![
            quote! {
                (status = 200, body=Vec<#model>),
                (status = 500, description="Server Error")
            },
            quote! {
                (status = 200, body=#model),
                (status = 400, description = "Invalid User Input"),
                (status = 500, description="Server Error")
            },
            quote! {
                (status = 200, body=#model),
                (status = 404, description = "No such model found"),
                (status = 500, description = "Server Error")
            },
            quote! {
                (status = 200, body=#model),
                (status = 400, description = "Invalid User Input"),
                (status = 404, description = "No such model found"),
                (status = 500, description = "Server Error")
            },
            quote! {
                (status = 200, body=#model),
                (status = 400, description= "Invalid User Input"),
                (status = 404, description = "No such model found"),
                (status = 500, description = "Server Error")
            },
            quote! {
                (status = 200, body=#model),
                (status = 404, description = "No such model found"),
                (status = 500, description = "Server Error")
            },
        ]
    }
    pub fn params(&self) -> Vec<proc_macro2::TokenStream> {
        let pkey_type = self.ty.clone();
        vec![
            quote! {
                params(rwf::controller::ModelListQuery)
            },
            quote! {
                params(
                    ("x-csrf-token" = String, Header, description = "X-CSRF-Token for protection purposes")
                )
            },
            quote! {
                params(("id" = #pkey_type , Path, description = "Database ID of the Model"))
            },
            quote! {
                params(
                    ("id" = #pkey_type, Path, description = "Database ID of the Model"),
                    ("x-csrf-token" = String, Header, description = "X-CSRF-Token for protection purposes")
                )
            },
            quote! {
                params(
                    ("id" = #pkey_type , Path, description = "Database ID of the Model"),
                    ("x-csrf-token" = String, Header, description = "X-CSRF-Token for protection purposes")
                )
            },
            quote! {
                params(("id" = #pkey_type, Path, description = "Database ID of the Model"))
            },
        ]
    }
    pub fn request_body(&self) -> Vec<proc_macro2::TokenStream> {
        let model = self.model.clone();
        vec![
            quote! {},
            quote! {
                request_body(content=#model, description = "The new Model to create")
            },
            quote! {},
            quote! {
                request_body(content=#model, description="Full Model for full update")
            },
            quote! {
                request_body(content_type="application/json", description="Partial Model for partial update")
            },
            quote! {},
        ]
    }
}

pub struct TypeParserInput {
    ty: syn::Type,
    _comma: Token![,],
    model: Ident,
}
impl TypeParserInput {
    pub fn into_type_parser(self, input: &ItemStruct) -> TypeParser {

        let sufix = Ident::new( pluralizer::pluralize(snake_case(self.model.to_string().as_str()).as_str(), 2, false).as_str(), Span::call_site());
        TypeParser {ty: self.ty, ctrl: input.ident.clone(), model: self.model, sufix}
    }
}
impl Parse for TypeParserInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;
        let _comma = input.parse()?;
        let model = input.parse()?;
        Ok(TypeParserInput { ty, _comma, model })
    }
}