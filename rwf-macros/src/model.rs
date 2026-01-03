use super::*;
use parse::Parse;
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

#[cfg(test)]
mod test {

    #[test]
    fn test_relationsips() {
        macrotest::expand("tests/model/relationship.rs");
    }
}
