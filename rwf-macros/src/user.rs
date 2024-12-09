use super::prelude::*;

struct UserModel {
    identifier: Ident,
    password: Ident,
}

impl Parse for UserModel {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let identifier: Ident = input.parse()?;
        let _: Token![,] = input.parse()?;
        let password = input.parse()?;

        Ok(UserModel {
            identifier,
            password,
        })
    }
}

pub(crate) fn impl_derive_user_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    if let Some(attr) = input.attrs.first() {
        match attr.meta {
            Meta::List(ref attrs) => {
                if let Ok(attrs) = syn::parse2::<UserModel>(attrs.tokens.clone()) {
                    let identifier = attrs.identifier.to_string();
                    let password = attrs.password.to_string();

                    return quote! {
                        impl rwf::model::UserModel for #ident {
                            fn identifier_column() -> &'static str {
                                #identifier
                            }

                            fn password_column() -> &'static str {
                                #password
                            }
                        }
                    }
                    .into();
                }
            }

            _ => (),
        }
    }

    quote! {}.into()
}
