//! Hide unique identifiers (e.g. primary keys) in your URLs.
//!
//! This allows to use database identifiers, like primary keys, in URLs, but doesn't
//! reveal potentially private information, e.g. how many users you have, by leaking those
//! identifiers.
//!
//! This middleware automatically converts the hidden version of the ID to the actual number,
//! allowing it to be used in database queries.

use crate::controller::middleware::prelude::*;
use crate::crypto::{decrypt_number, encrypt_number};
use crate::http::{Body, Path};
use utoipa::openapi::OpenApi;

/// Hide unique identifiers.
pub struct SecureId {
    /// Block requests that use plain text identifiers.
    pub block_unencrypted: bool,
    pub transform_response: bool,
}

impl Default for SecureId {
    fn default() -> Self {
        Self {
            block_unencrypted: true,
            transform_response: true,
        }
    }
}

#[async_trait::async_trait]
impl Middleware for SecureId {
    async fn handle_request(&self, mut request: Request) -> Result<Outcome, Error> {
        let id = request.parameter::<String>("id");
        if self.transform_response {
            if let Ok(mut data) = request.json_raw() {
                if let Some(arr) = data.as_array_mut() {
                    for obj in arr.iter_mut() {
                        if let Some(id) = obj.get_mut("id") {
                            if let Some(enc_id) = id.as_str() {
                                *id = serde_json::json!(decrypt_number(enc_id)?);
                            }
                        }
                    }
                    request.replace_body(serde_json::to_vec(&data)?);
                } else if let Some(obj) = data.as_object_mut() {
                    if let Some(id) = obj.get_mut("id") {
                        if let Some(enc_id) = id.as_str() {
                            *id = serde_json::json!(decrypt_number(enc_id)?);
                        }
                    }
                }
                request.replace_body(serde_json::to_vec(&data)?);
            }
        }

        if let Ok(Some(id)) = id {
            // Block requests to a numeric ID.
            if self.block_unencrypted && id.chars().all(|c| c.is_numeric()) {
                return Ok(Outcome::Stop(request, Response::not_found()));
            }

            let path = request.path().clone();

            if let Ok(decrypted) = decrypt_number(&id) {
                let base = path.base().replace(&id, &decrypted.to_string());

                let head = request.head_mut();
                head.replace_path(Path::from_parts(&base, path.query()));

                return Ok(Outcome::Forward(request));
            } else {
                return Ok(Outcome::Stop(request, Response::not_found()));
            }
        }

        Ok(Outcome::Forward(request))
    }
    async fn handle_response(
        &self,
        _request: &Request,
        response: Response,
    ) -> Result<Response, Error> {
        Ok(if self.transform_response {
            if let Body::Json(ref data) = response.get_body() {
                let mut data: serde_json::Value = serde_json::from_slice(data)?;
                if let Some(arr) = data.as_array_mut() {
                    for obj in arr.iter_mut() {
                        if let Some(id) = obj.get_mut("id") {
                            if let Some(num_id) = id.as_i64() {
                                *id = serde_json::json!(encrypt_number(num_id)?);
                            }
                        }
                    }
                } else if let Some(obj) = data.as_object_mut() {
                    if let Some(id) = obj.get_mut("id") {
                        if let Some(num_id) = id.as_i64() {
                            *id = serde_json::json!(encrypt_number(num_id)?);
                        }
                    }
                }
                response.body(Body::Json(serde_json::to_vec(&data)?))
            } else {
                response
            }
        } else {
            response
        })
    }
}

impl utoipa::Modify for SecureId {
    fn modify(&self, openapi: &mut OpenApi) {
        let encrypted_id = utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::Object::builder()
                .description(Some("A encrypted Databse primary key."))
                .schema_type(utoipa::openapi::Type::String)
                .examples(vec![
                    serde_json::json!(crate::crypto::encrypt_number(21).unwrap()),
                    serde_json::json!(crate::crypto::encrypt_number(42).unwrap()),
                    serde_json::json!(crate::crypto::encrypt_number(4321).unwrap()),
                ])
                .title(Some("DatabaseId"))
                .build(),
        ));
        if let Some(ref mut components) = openapi.components {
            components
                .schemas
                .insert("encrypted_id".to_string(), encrypted_id);
            let encrypted_id =
                utoipa::openapi::RefOr::Ref(utoipa::openapi::Ref::from_schema_name("encrypted_id"));
            if self.transform_response {
                for schema in components.schemas.values_mut() {
                    if let utoipa::openapi::RefOr::T(schema) = schema {
                        if let utoipa::openapi::schema::Schema::Object(obj) = schema {
                            if obj.schema_type
                                == utoipa::openapi::schema::SchemaType::Type(
                                    utoipa::openapi::schema::Type::Object,
                                )
                            {
                                if let Some(id) = obj.properties.get_mut("id") {
                                    *id = encrypted_id.clone();
                                }
                            }
                        }
                    }
                }
                for response in components.responses.values_mut() {
                    if let utoipa::openapi::RefOr::T(res) = response {
                        for content in res.content.values_mut() {
                            if let Some(example) = content.example.as_mut() {
                                if let Some(id) = example.get_mut("id") {
                                    if let Some(num_id) = id.as_i64() {
                                        *id = serde_json::json!(encrypt_number(num_id).unwrap());
                                    }
                                }
                            }
                            for example in content.examples.values_mut() {
                                if let utoipa::openapi::RefOr::T(example) = example {
                                    if let Some(val) = example.value.as_mut() {
                                        if let Some(id) = val.get_mut("id") {
                                            if let Some(num_id) = id.as_i64() {
                                                *id = serde_json::json!(
                                                    encrypt_number(num_id).unwrap()
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(schema) = content.schema.as_mut() {
                                if let utoipa::openapi::RefOr::T(schema) = schema {
                                    if let utoipa::openapi::schema::Schema::Object(obj) = schema {
                                        if obj.schema_type
                                            == utoipa::openapi::schema::SchemaType::Type(
                                                utoipa::openapi::schema::Type::Object,
                                            )
                                        {
                                            if let Some(id) = obj.properties.get_mut("id") {
                                                *id = encrypted_id.clone();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for path in openapi.paths.paths.values_mut() {
            for ref mut op in [
                &mut path.get,
                &mut path.post,
                &mut path.put,
                &mut path.patch,
                &mut path.delete,
                &mut path.head,
            ]
            .into_iter()
            .flatten()
            {
                if let Some(ref mut params) = op.parameters {
                    for param in params.iter_mut() {
                        if param.name.eq("id")
                            && param
                                .parameter_in
                                .eq(&utoipa::openapi::path::ParameterIn::Path)
                        {
                            param.description = Some("The encrypted id of the Database object. Prevents leaking internal data.".to_string());
                            param.schema = Some(utoipa::openapi::RefOr::Ref(
                                utoipa::openapi::Ref::from_schema_name("encrypted_id"),
                            ));
                            param.style = Some(utoipa::openapi::path::ParameterStyle::Simple);
                            param.required = utoipa::openapi::Required::True;
                        }
                    }
                }
            }
        }
    }
}
