extern crate proc_macro;

use proc_macro::TokenStream;

use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, punctuated::Punctuated, Attribute, Data, DeriveInput, Expr, LitStr, Meta,
    Result, Token, Type,
};

use quote::quote;

mod model;

/// The `#[derive(Model)]` macro.
#[proc_macro_derive(Model, attributes(belongs_to, has_many, table_name, foreign_key))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    model::impl_derive_model(input)
}

#[proc_macro_derive(WebsocketController, attributes(auth, middleware, skip_csrf))]
pub fn derive_websocket_controller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let overrides = handle_overrides(&input.attrs);

    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    quote! {
       #[rwf::async_trait]
        impl rwf::controller::Controller for #ident {
            #overrides

            async fn handle(&self, request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::controller::Error> {
                rwf::controller::WebsocketController::handle(self, request).await
            }

            async fn handle_stream(&self, request: &rwf::http::Request, stream: rwf::http::Stream<'_>) -> Result<bool, rwf::controller::Error> {
                rwf::controller::WebsocketController::handle_stream(self, request, stream).await
            }
        }
    }.into()
}

#[proc_macro_derive(ModelController, attributes(auth, middleware, skip_csrf))]
pub fn derive_model_controller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let overrides = handle_overrides(&input.attrs);

    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    quote! {
       #[rwf::async_trait]
        impl rwf::controller::Controller for #ident {
            #overrides

            async fn handle(&self, request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::controller::Error> {
                rwf::controller::ModelController::handle(self, request).await
            }
        }
    }.into()
}

fn handle_overrides(attributes: &[Attribute]) -> proc_macro2::TokenStream {
    let overrides = attributes
        .iter()
        .map(|attr| {
            let name = &attr
                .meta
                .path()
                .segments
                .first()
                .expect("segment")
                .ident
                .to_string();

            match name.as_str() {
                "auth" => match &attr.meta {
                    Meta::List(list) => {
                        let path = list.path.segments.first();
                        let tokens = &list.tokens;

                        if let Some(_) = path {
                            quote! {
                                fn auth(&self) -> &rwf::controller::AuthHandler {
                                    &self.#tokens
                                }
                            }
                        } else {
                            quote! {}
                        }
                    }

                    _ => quote! {},
                },

                "middleware" => match &attr.meta {
                    Meta::List(list) => {
                        let path = list.path.segments.first();
                        let tokens = &list.tokens;

                        if let Some(_) = path {
                            quote! {
                                fn middleware(&self) -> &rwf::controller::MiddlewareSet {
                                    &self.#tokens
                                }
                            }
                        } else {
                            quote! {}
                        }
                    }

                    _ => quote! {},
                },

                "skip_csrf" => quote! {
                    fn skip_csrf(&self) -> bool {
                        true
                    }
                },

                _ => quote! {},
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#overrides)*
    }
}

#[proc_macro_derive(PageController, attributes(auth, middleware, skip_csrf))]
pub fn derive_page_controller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let overrides = handle_overrides(&input.attrs);

    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    quote! {
       #[rwf::async_trait]
        impl rwf::controller::Controller for #ident {
            #overrides

            async fn handle(&self, request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::controller::Error> {
                rwf::controller::PageController::handle(self, request).await
            }
        }
    }.into()
}

#[proc_macro_derive(RestController, attributes(auth, middleware, skip_csrf))]
pub fn derive_rest_controller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let overrides = handle_overrides(&input.attrs);

    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    quote! {
       #[rwf::async_trait]
        impl rwf::controller::Controller for #ident {
            #overrides

            async fn handle(&self, request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::controller::Error> {
                rwf::controller::RestController::handle(self, request).await
            }
        }
    }.into()
}

// #[proc_macro_derive(Template, attributes(path))]
// pub fn derive_rest_controller(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let overrides = handle_overrides(&input.attrs);

//     let ident = match &input.data {
//         Data::Struct(_data) => input.ident.clone(),

//         _ => panic!("macro can only be used on structs"),
//     };

//     quote! {
//         #[derive(rwf::macros::Context)]

//     }.into()
// }

#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(ref data) => {
            let ident = input.ident;

            let from_row_fields = data.fields.iter().map(|field| {
                let ident = &field.ident;
                quote! {
                    #ident: row.try_get(stringify!(#ident))?,
                }
            });

            quote! {
                #[automatically_derived]
                impl rwf::model::FromRow for #ident {
                    fn from_row(row: rwf::tokio_postgres::Row) -> Result<Self, rwf::model::Error> {
                        Ok(Self {
                            #(#from_row_fields)*
                        })
                    }
                }
            }
            .into()
        }

        _ => panic!("macro can only be used on structs"),
    }
}

#[proc_macro_derive(TemplateValue)]
pub fn derive_template_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(ref data) => {
            let ident = input.ident;

            let fields = data.fields.iter().map(|field| {
                let ident = &field.ident;
                quote! {
                    hash.insert(stringify!(#ident).to_string(), self.#ident.to_template_value()?);
                }
            });

            quote! {
                #[automatically_derived]
                impl rwf::view::ToTemplateValue for #ident {
                    fn to_template_value(&self) -> Result<rwf::view::Value, rwf::view::Error> {
                        let mut hash = std::collections::HashMap::new();

                        #(#fields)*

                        Ok(rwf::view::Value::Hash(hash))
                    }
                }
            }
            .into()
        }

        _ => panic!("macro can only be used on structs"),
    }
}

#[proc_macro_derive(Form)]
pub fn derive_form(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(ref data) => {
            let ident = input.ident;

            let from_row_fields = data.fields.iter().map(|field| {
                let ident = &field.ident;

                let optional = match &field.ty {
                    Type::Path(path) => {
                        let optional = &path
                            .path
                            .segments
                            .iter()
                            .next()
                            .map(|segment| segment.ident == "Option");
                        optional.unwrap_or(false)
                    }

                    _ => false,
                };

                if optional {
                    quote! {
                        #ident: form_data.get(stringify!(#ident)),
                    }
                } else {
                    quote! {
                        #ident: form_data.get_required(stringify!(#ident))?,
                    }
                }
            });

            quote! {
                #[automatically_derived]
                impl rwf::http::FromFormData for #ident {
                    fn from_form_data(form_data: &rwf::http::FormData) -> Result<Self, rwf::http::Error> {
                        Ok(Self {
                            #(#from_row_fields)*
                        })
                    }
                }
            }
            .into()
        }

        _ => panic!("macro can only be used on structs"),
    }
}

#[proc_macro_derive(Context)]
pub fn drive_context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(ref data) => {
            let ident = input.ident;
            let fields = data.fields.iter().map(|field| {
                let ident = &field.ident;

                quote! {
                    result[stringify!(#ident)] = rwf::view::template::ToTemplateValue::to_template_value(&context.#ident)?;
                }
            });
            let fields_ref = fields.clone();

            quote! {
                #[automatically_derived]
                impl TryFrom<#ident> for rwf::view::Context {
                    type Error = rwf::view::Error;

                    fn try_from(context: #ident) -> Result<Self, Self::Error> {
                        let mut result = rwf::view::Context::new();

                        #(#fields)*

                        Ok(result)
                    }
                }

                impl TryFrom<&#ident> for rwf::view::Context {
                    type Error = rwf::view::Error;

                    fn try_from(context: &#ident) -> Result<Self, Self::Error> {
                        let mut result = rwf::view::Context::new();

                        #(#fields_ref)*

                        Ok(result)
                    }
                }
            }
            .into()
        }

        _ => panic!("macro can only be used on structs"),
    }
}

#[proc_macro]
pub fn error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    quote! {
        return Err(rwf::controller::Error::new(#input));
    }
    .into()
}

#[proc_macro]
pub fn route(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated<Expr, Token![=>]>::parse_terminated);
    let mut iter = input.into_iter();

    let route = iter.next().unwrap();
    let controller = iter.next().unwrap();

    quote! {
        #controller::default().route(#route)
    }
    .into()
}

#[proc_macro]
pub fn crud(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated<Expr, Token![=>]>::parse_terminated);
    let mut iter = input.into_iter();

    let route = iter.next().unwrap();
    let controller = iter.next().unwrap();

    quote! {
        #controller::default().crud(#route)
    }
    .into()
}

#[proc_macro]
pub fn rest(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated<Expr, Token![=>]>::parse_terminated);
    let mut iter = input.into_iter();

    let route = iter.next().unwrap();
    let controller = iter.next().unwrap();

    quote! {
        #controller::default().rest(#route)
    }
    .into()
}

#[proc_macro]
pub fn engine(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated<Expr, Token![=>]>::parse_terminated);
    let mut iter = input.into_iter();

    let route = iter.next().unwrap();
    let engine = iter.next().unwrap();

    quote! {
        #engine.remount(&rwf::http::Path::parse(#route).unwrap()).wildcard(#route)
    }
    .into()
}

struct RenderInput {
    template_name: LitStr,
    _comma: Option<Token![,]>,
    context: Vec<ContextInput>,
}

struct ContextInput {
    name: LitStr,
    _separator: Token![=>],
    value: Expr,
    _comma: Option<Token![,]>,
}

struct Context {
    values: Vec<ContextInput>,
}

impl Parse for Context {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut values = vec![];
        loop {
            let context: Result<ContextInput> = input.parse();

            if let Ok(context) = context {
                values.push(context);
            } else {
                break;
            }
        }

        Ok(Context { values })
    }
}

impl Parse for ContextInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ContextInput {
            name: input.parse()?,
            _separator: input.parse()?,
            value: input.parse()?,
            _comma: input.parse()?,
        })
    }
}

impl Parse for RenderInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let template_name: LitStr = input.parse()?;
        let _comma: Option<Token![,]> = input.parse()?;

        let context = if _comma.is_some() {
            let mut result = vec![];
            loop {
                let context: Result<ContextInput> = input.parse();

                if let Ok(context) = context {
                    result.push(context);
                } else {
                    break;
                }
            }

            result
        } else {
            vec![]
        };

        Ok(RenderInput {
            template_name,
            _comma,
            context,
        })
    }
}

#[proc_macro]
pub fn context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Context);
    let mut expansion = vec![quote! {
        let mut context = rwf::view::template::Context::new();
    }];

    for value in input.values {
        let name = value.name;
        let value = value.value;
        expansion.push(quote! {
            context.set(#name, #value)?;
        });
    }

    quote! {
        {
            #(#expansion)*
            context
        }
    }
    .into()
}

#[proc_macro]
pub fn render(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as RenderInput);
    let template_name = input.template_name;

    let render_call = if input.context.is_empty() {
        vec![quote! {
            let html = template.render_default()?;
        }]
    } else {
        let mut values = vec![quote! {
            let mut context = rwf::view::template::Context::new();
        }];

        for value in input.context {
            let name = value.name;
            let val = value.value;
            values.push(quote! {
                context.set(#name, #val)?;
            })
        }

        values.push(quote! {
            let html = template.render(&context)?;
        });

        values
    };

    quote! {
        {
            let template = rwf::view::template::Template::load(#template_name)?;
            #(#render_call)*
            return Ok(rwf::http::Response::new().html(html))
        }
    }
    .into()
}

fn snake_case(string: &str) -> String {
    let mut result = "".to_string();

    for (i, c) in string.chars().enumerate() {
        if c.is_ascii_uppercase() && i != 0 {
            result.push('_');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    result
}
