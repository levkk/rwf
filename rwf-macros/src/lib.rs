extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{parse_macro_input, Attribute, Data, DeriveInput, Meta, Type};

use quote::quote;

#[proc_macro_derive(Model, attributes(belongs_to, has_many))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let attrs = handle_model_attrs(&input, &input.attrs);

    match input.data {
        Data::Struct(ref data) => {
            let ident = input.ident.clone();
            let from_row_fields = data.fields.iter().map(|field| {
                let ident = field.ident.clone();
                quote! {
                    #ident: row.try_get(stringify!(#ident))?,
                }
            });
            let has_id = data
                .fields
                .iter()
                .any(|field| field.ident.clone().unwrap() == "id");

            let id = if has_id {
                quote! {
                    fn id(&self) -> rwf::model::Value {
                        use rwf::model::ToValue;
                        self.id.to_value()
                    }
                }
            } else {
                quote! {}
            };

            let column_names = data
                .fields
                .iter()
                .filter(|field| field.ident.clone().unwrap() != "id")
                .map(|field| {
                    let ident = field.ident.clone();

                    quote! {
                        stringify!(#ident),
                    }
                });

            let values = data
                .fields
                .iter()
                .filter(|field| field.ident.clone().unwrap() != "id")
                .map(|field| {
                    let ident = field.ident.clone();

                    quote! {
                        self.#ident.to_value(),
                    }
                });

            let singular = snake_case(&ident.to_string());
            let foreign_key = format!("{}_id", singular);

            let table_name = pluralizer::pluralize(singular.as_str(), 2, false);

            quote! {
                #[automatically_derived]
                impl rwf::model::FromRow for #ident {
                    fn from_row(row: rwf::tokio_postgres::Row) -> Result<Self, rwf::model::Error> {
                        Ok(Self {
                            #(#from_row_fields)*
                        })
                    }
                }

                #[automatically_derived]
                impl rwf::model::Model for #ident {
                    fn table_name() -> &'static str {
                        #table_name
                    }

                    fn foreign_key() -> &'static str {
                        #foreign_key
                    }

                    fn column_names() -> &'static[&'static str] {
                        &[
                            #(#column_names)*
                        ]
                    }

                    fn values(&self) -> Vec<rwf::model::Value> {
                        use rwf::model::ToValue;
                        vec![
                            #(#values)*
                        ]
                    }

                    #id
                }

                #attrs
            }
            .into()
        }

        _ => panic!("macro can only be used on structs"),
    }
}

fn handle_model_attrs(input: &DeriveInput, attributes: &[Attribute]) -> proc_macro2::TokenStream {
    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    let rels = attributes
        .iter()
        .filter(|attr| {
            ["belongs_to", "has_many", "table_name"].contains(
                &attr
                    .meta
                    .path()
                    .segments
                    .first()
                    .expect("segment")
                    .ident
                    .to_string()
                    .as_str(),
            )
        })
        .map(|attr| match &attr.meta {
            Meta::List(list) => {
                let path = list.path.segments.first().expect("segment");

                let association = if path.ident == "belongs_to" {
                    quote! {
                        rwf::model::AssociationType::BelongsTo
                    }
                } else if path.ident == "has_many" {
                    quote! {
                        rwf::model::AssociationType::HasMany
                    }
                } else {
                    panic!("unsupported association: {}", path.ident);
                };

                let associations = list.tokens.clone().into_iter().map(|token| {
                    quote! {
                        #[automatically_derived]
                        impl rwf::model::Association<#token> for #ident {
                            fn association_type() -> rwf::model::AssociationType {
                                #association
                            }
                        }
                    }
                });

                quote! {
                    #(#associations)*
                }
            }

            _ => panic!("macro can only be used on structs"),
        });

    quote! {
        #(#rels)*
    }
}

#[proc_macro_derive(WebsocketController)]
pub fn derive_websocket_controller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    quote! {
       #[rwf::async_trait]
        impl rwf::controller::Controller for #ident {
            async fn handle(&self, request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::controller::Error> {
                rwf::controller::WebsocketController::handle(self, request).await
            }

            async fn handle_stream(&self, request: &rwf::http::Request, stream: rwf::http::Stream<'_>) -> Result<bool, rwf::controller::Error> {
                rwf::controller::WebsocketController::handle_stream(self, request, stream).await
            }
        }
    }.into()
}

#[proc_macro_derive(ModelController)]
pub fn derive_model_controller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    quote! {
       #[rwf::async_trait]
        impl rwf::controller::Controller for #ident {
            async fn handle(&self, request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::controller::Error> {
                rwf::controller::ModelController::handle(self, request).await
            }
        }
    }.into()
}

#[proc_macro_derive(PageController, attributes(auth))]
pub fn derive_page_controller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let overrides = input
        .attrs
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

                        if let Some(path) = path {
                            quote! {
                                fn auth(&self) -> &rwf::controller::AuthHandler {
                                    &self.#path
                                }
                            }
                        } else {
                            quote! {}
                        }
                    }

                    _ => quote! {},
                },

                _ => quote! {},
            }
        })
        .collect::<Vec<_>>();

    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    quote! {
       #[rwf::async_trait]
        impl rwf::controller::Controller for #ident {
            #(#overrides)*

            async fn handle(&self, request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::controller::Error> {
                rwf::controller::PageController::handle(self, request).await
            }
        }
    }.into()
}

#[proc_macro_derive(RestController)]
pub fn derive_rest_controller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    quote! {
       #[rwf::async_trait]
        impl rwf::controller::Controller for #ident {
            async fn handle(&self, request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::controller::Error> {
                rwf::controller::RestController::handle(self, request).await
            }
        }
    }.into()
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
