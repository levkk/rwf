use super::*;
use parse::Parse;
use proc_macro2::Span;
use quote::format_ident;
use syn::parse::ParseStream;
use syn::*;

pub fn impl_derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let relationships = handle_relationships(&input, &input.attrs);

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
                quote! {
                    fn id(&self) -> rwf::model::Value {
                        rwf::model::Value::Null
                    }
                }
            };

            let without_id = data
                .fields
                .iter()
                .filter(|field| field.ident.clone().unwrap() != "id");

            let column_names = without_id.clone().map(|field| {
                let ident = &field.ident;

                quote! {
                    stringify!(#ident),
                }
            });

            let values = without_id.clone().map(|field| {
                let ident = &field.ident;

                quote! {
                    self.#ident.to_value(),
                }
            });

            let singular = snake_case(&ident.to_string());
            let foreign_key = format!("{}_id", singular);

            let table_name = pluralizer::pluralize(singular.as_str(), 2, false);

            let table_name = handle_override(
                "table_name",
                quote! {
                    fn table_name() -> &'static str {
                        #table_name
                    }
                },
                &input.attrs,
            );

            let foreign_key = handle_override(
                "foreign_key",
                quote! {
                    fn foreign_key() -> &'static str {
                        #foreign_key
                    }
                },
                &input.attrs,
            );

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
                    #table_name
                    #foreign_key

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

                #relationships
            }
            .into()
        }

        _ => panic!("macro can only be used on structs"),
    }
}

fn handle_override(
    name: &str,
    default_value: proc_macro2::TokenStream,
    attributes: &[Attribute],
) -> proc_macro2::TokenStream {
    let mut overrides = attributes
        .iter()
        .filter(|attr| {
            attr.path()
                .segments
                .first()
                .expect("segment")
                .ident
                .to_string()
                == name
        })
        .map(|attr| match &attr.meta {
            Meta::List(list) => {
                let segment = list.path.segments.first();

                if let Some(_) = segment {
                    let tokens = &list.tokens;
                    match name {
                        "table_name" => {
                            quote! {
                                fn table_name() -> &'static str {
                                    #tokens
                                }
                            }
                        }

                        "foreign_key" => {
                            quote! {
                                fn foreign_key() -> &'static str {
                                    #tokens
                                }
                            }
                        }

                        _ => panic!("unexpected attribute: {}", name),
                    }
                } else {
                    quote! {}
                }
            }

            _ => quote! {},
        })
        .collect::<Vec<_>>();

    if let Some(overrides) = overrides.pop() {
        quote! {
            #overrides
        }
    } else {
        quote! {
            #default_value
        }
    }
}

struct Relationships {
    relationships: Vec<Relationship>,
}

impl Parse for Relationships {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let mut relationships = Vec::new();
        while let Ok(relationship) = input.parse() {
            relationships.push(relationship);
        }

        Ok(Self { relationships })
    }
}

struct Relationship {
    path: Path,
    #[allow(dead_code)]
    comma: Option<Token![,]>,
}

impl Parse for Relationship {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
            comma: input.parse()?,
        })
    }
}

fn handle_relationships(input: &DeriveInput, attributes: &[Attribute]) -> proc_macro2::TokenStream {
    let ident = match &input.data {
        Data::Struct(_data) => input.ident.clone(),

        _ => panic!("macro can only be used on structs"),
    };

    let rels = attributes
        .iter()
        .filter(|attr| {
            ["belongs_to", "has_many"].contains(
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
                    Some(quote! {
                        rwf::model::AssociationType::BelongsTo
                    })
                } else if path.ident == "has_many" {
                    Some(quote! {
                        rwf::model::AssociationType::HasMany
                    })
                } else {
                    None
                };

                if let Some(association) = association {
                    let relationships = syn::parse2::<Relationships>(list.tokens.clone()).unwrap();

                    let associations =
                        relationships.relationships.into_iter().map(|relationship| {
                            let token = relationship.path;
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
                } else {
                    quote! {}
                }
            }

            _ => panic!("associations must be a list"),
        });

    quote! {
        #(#rels)*
    }
}

#[derive(Clone)]
pub struct TypeParser {
    pub ty: Type,
    _comma: Token![,],
    pub ctrl: Ident,
    _comma2: Token![,],
    pub sufix: Ident,
    _comma3: Token![,],
    pub _apipth: LitStr,
}

impl TypeParser {
    pub fn gen_doc_paths(&self) -> proc_macro2::TokenStream {
        let paths = self.path_name_list();
        quote! {
            #(#paths),*
        }
    }
    pub fn gen_oapi(&self, model: Ident) -> Vec<proc_macro2::TokenStream> {
        let mut result = Vec::with_capacity(6);

        let operations = self.operation();
        let paths = self.apipths();
        let responses = self.rewsponses(model.clone());
        let params = self.params();
        let requests = self.request_body(model.clone());
        let funcs = self.path_name_list();

        operations
            .into_iter()
            .zip(paths.into_iter())
            .zip(params.into_iter().zip(requests.into_iter()))
            .map(|((o, p), (par, req))| (o, p, par, req))
            .zip(responses.into_iter().zip(funcs.into_iter()))
            .map(|((o, p, par, req), (res, func))| (o, p, par, req, res, func))
            .map(|(o, p, par, req, res, func)| {
                if o.eq("get") || o.eq("delete") {
                    quote! {
                        #[utoipa::path(
                            #o,
                            path=#p,
                            #par,
                            responses(#res)
                        )]
                        fn #func (_request: &Request) -> Result<Response, rwf::http::Error> {
                            Ok(Response::not_implemented())
                        }
                    }
                } else {
                    quote! {
                        #[utoipa::path(
                            #o,
                            path=#p,
                            #par,
                            #req,
                            responses(#res)
                        )]
                        fn #func (_request: &Request) -> Result<Response, rwf::http::Error> {
                            Ok(Response::not_implemented())
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
    pub fn rewsponses(&self, model: Ident) -> Vec<proc_macro2::TokenStream> {
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
                params(ModelListQuery)
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
    pub fn request_body(&self, model: Ident) -> Vec<proc_macro2::TokenStream> {
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
impl Parse for TypeParser {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = input.parse()?;
        let _comma = input.parse()?;
        let ctrl: Ident = input.parse()?;
        let _comma2 = input.parse()?;
        let sufix: Ident = input.parse()?;
        let _comma3 = input.parse()?;
        let _apipth: LitStr = input.parse()?;
        Ok(Self {
            ty,
            _comma,
            ctrl,
            _comma2,
            sufix,
            _comma3,
            _apipth,
        })
    }
}

/// Relevant again, once we create OpenApi Models from Scratch

/*
pub fn handle_generate_full_model(mut input: ItemStruct) -> proc_macro2::TokenStream {
    let mut data = proc_macro2::TokenStream::new();
    input.attrs.push(
            parse_quote!(
                    #[derive(Clone, rwf_macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::prelude::ToSchema, rwf::prelude::ToResponse)]
            )
    );
    let model_name = input.ident.clone();
    for field in &mut input.fields {
        let fname = field.ident.as_ref().unwrap();
        eprintln!("{}", fname);
        if fname.eq("id") {
            field.attrs.push(parse_quote!(
                #[schema(minimum=1, format="Int64")]
            ))
        }
    }
    //let pkey_type = input.fields.iter().filter(|f| f.ident.is_some()).find(|f|
    //    f.ident.as_ref().unwrap().clone().to_string().eq("id")
    //).map(|f| f.ty.clone()).unwrap();
    eprintln!("{:?}", model_name);
    //let pkey_type = quote!{#pkey_type}.to_string().replace("Option", "").trim().strip_prefix("<").unwrap().strip_suffix(">").unwrap().trim().replace("\"", "");
    input.to_tokens(&mut data);
    /*quote!{
            impl rwf :: controller :: PkeyParamGenerator for #model_name
    {
        fn param(val : impl IntoPkey) -> rwf :: controller :: ModelPkeyParam
        { rwf :: controller :: ModelPkeyParam :: from(val.pkey_type() ) }
    }
        }.to_tokens(&mut data);*/
    data.into_token_stream()
}*/

#[cfg(test)]
mod test {

    #[test]
    fn test_relationsips() {
        macrotest::expand("tests/model/relationship.rs");
    }
}
