use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreErrorResponseType,
    CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata,
    CoreRevocableToken, CoreRevocationErrorResponse, CoreTokenResponse,
};
use openidconnect::core::{CoreIdTokenFields, CoreTokenIntrospectionResponse, CoreTokenType};
use openidconnect::{
    reqwest, AccessToken, AccessTokenHash, AuthorizationCode, Client, CsrfToken,
    EmptyAdditionalClaims, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier,
    RefreshToken, StandardErrorResponse, StandardTokenResponse, TokenResponse, UserInfoClaims,
};
use openidconnect::{EndpointMaybeSet, EndpointNotSet, EndpointSet};
use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::prelude::*;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::config::get_config;
use crate::model::get_connection;
use tracing::debug;

type RWFOidcClient = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    CoreTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Debug)]
struct VerifyData {
    verifier: PkceCodeVerifier,
    _csrf: CsrfToken,
    nonce: Nonce,
    target: crate::http::path::Path,
}

impl VerifyData {
    fn new(
        verifier: PkceCodeVerifier,
        _csrf: CsrfToken,
        nonce: Nonce,
        target: crate::http::path::Path,
    ) -> Self {
        Self {
            verifier,
            _csrf,
            nonce,
            target,
        }
    }
}

#[derive(Debug, Default)]
struct VerifyMap {
    map: RwLock<BTreeMap<String, VerifyData>>,
}

static VERIFY_MAP: Lazy<VerifyMap> = Lazy::new(VerifyMap::default);

async fn clients(
    config: &crate::config::OidcConfig,
) -> Result<(reqwest::Client, RWFOidcClient), Error> {
    if !config.everything_set() {
        return Err(Error::Config(crate::config::Error::NoConfig));
    }
    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| Error::Error(Box::new(e)))?;
    debug!("{:?}", http_client);
    let metadata = CoreProviderMetadata::discover_async(
        config.discovery_url.as_ref().unwrap().clone(),
        &http_client,
    )
    .await
    .map_err(|e| Error::Error(Box::new(e)))?;
    debug!("{:?}", metadata);
    let client: RWFOidcClient = CoreClient::from_provider_metadata(
        metadata,
        config.client_id.as_ref().unwrap().clone(),
        config.client_secret.clone(),
    )
    .set_redirect_uri(config.redirect_url.as_ref().unwrap().clone());
    Ok((http_client, client))
}

/// A Trait to enable shared responsibility, the user is free to in the design of his User struct, while
/// the full handling of the OIDC Protocol is handled in the lib.
#[async_trait]
pub trait OidcUser: Model {
    /// Initialization Method to create the User from the Token Response
    /// # Example
    /// ```
    /// use openidconnect::{AccessToken, RefreshToken, OAuth2TokenResponse, StandardTokenResponse, UserInfoClaims, EmptyAdditionalClaims, core::CoreGenderClaim};
    /// use openidconnect::core::{CoreIdTokenFields, CoreTokenType};
    /// use time::OffsetDateTime;
    /// use rwf::model::{Model, FromRow, get_connection, ToValue};
    /// use rwf::prelude::{async_trait, Serialize, Deserialize};
    /// use rwf::controller::oidc::{OidcUser};
    /// #[derive(Serialize, Deserialize, rwf::macros::Model, Clone)]
    /// struct User {
    ///     id: Option<i64>,
    ///     sub: uuid::Uuid,
    ///     name: String,
    ///     email: String,
    ///     access: String,
    ///     refresh: String,
    ///     expire: OffsetDateTime
    /// }
    ///
    /// #[async_trait]
    /// impl OidcUser for User {
    ///     async fn from_token(token: StandardTokenResponse<CoreIdTokenFields, CoreTokenType>, userinfo: UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim>) -> Result<Self, rwf::model::Error> {
    ///         let name = userinfo.standard_claims().preferred_username().unwrap().to_string();
    ///         let sub = userinfo.standard_claims().subject().to_string();
    ///         let email = userinfo.standard_claims().email().unwrap().to_string();
    ///         let access = token.access_token().secret().clone();
    ///         let refresh = token.refresh_token().unwrap().secret().clone();
    ///         let expire  = OffsetDateTime::now_utc().checked_add(time::Duration::nanoseconds(token.expires_in().unwrap().as_nanos() as i64)).unwrap();;
    ///
    ///         let mut conn = get_connection().await?;
    ///         User::find_or_create_by(&[
    ///             ("name", name.to_value()),
    ///             ("sub", sub.to_value()),
    ///             ("email", email.to_value()),
    ///             ("access", access.to_value()),
    ///             ("refresh", refresh.to_value()),
    ///             ("expire", expire.to_value())
    ///         ]).unique_by(&["sub"]).fetch(&mut conn).await
    ///     }
    ///     fn access_token(&self) -> AccessToken {
    ///         AccessToken::new(self.access.clone())
    ///     }
    ///     fn refresh_token(&self) -> RefreshToken {
    ///         RefreshToken::new(self.refresh.clone())
    ///     }
    ///     fn expire(&self) -> &OffsetDateTime {
    ///          &self.expire
    ///     }
    ///     fn update_token(mut self, token: StandardTokenResponse<CoreIdTokenFields, CoreTokenType>) -> Self {
    ///         self.access = token.access_token().secret().clone();
    ///         self.refresh = token.refresh_token().unwrap().secret().clone();
    ///         self.expire = OffsetDateTime::now_utc().checked_add(time::Duration::nanoseconds(token.expires_in().unwrap().as_nanos() as i64)).unwrap();
    ///         self
    ///     }
    /// }
    /// ```
    async fn from_token(
        token: StandardTokenResponse<CoreIdTokenFields, CoreTokenType>,
        userinfo: UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim>,
    ) -> Result<Self, crate::model::Error>;
    /// Getter for the AccessToken, hereby is ensured we have access to it so calls on the user behalf's are possible
    fn access_token(&self) -> AccessToken;
    /// Getter for the refresh token, so the access token can be renewed whenever it expires,
    fn refresh_token(&self) -> RefreshToken;
    /// The Timestamp when the access token will be expired.
    fn expire(&self) -> &OffsetDateTime;
    fn is_expired(&self) -> bool {
        self.expire() < &OffsetDateTime::now_utc()
    }
    /// Exchange the tokens stored with the User for new onces
    fn update_token(self, token: StandardTokenResponse<CoreIdTokenFields, CoreTokenType>) -> Self;
}
#[derive(Debug, Clone)]
pub struct OidcAuthentication<U: OidcUser> {
    _marker: PhantomData<U>,
}

impl<U: OidcUser> Default for OidcAuthentication<U> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<U: OidcUser> Modify for OidcAuthentication<U> {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let config = get_config();
        if !config.oidc.everything_set() {
            return;
        }
        if let Some(ref mut components) = openapi.components {
            components.add_security_scheme(
                "oidc_authentication",
                utoipa::openapi::security::SecurityScheme::OpenIdConnect(
                    utoipa::openapi::security::OpenIdConnect::with_description(
                        config.oidc.discovery_url.as_ref().unwrap().to_string(),
                        "A OpenidConnect Authentication Provider.".to_string(),
                    ),
                ),
            );
        }
        let scopes: Vec<String> = Vec::new();
        let requirement =
            utoipa::openapi::SecurityRequirement::new("oidc_authentication", scopes.clone())
                .add("oauth2_authcode", scopes);
        if let Some(ref mut sec) = openapi.security {
            sec.push(requirement.clone());
        } else {
            openapi.security = Some(vec![requirement.clone()]);
        }
        for path in openapi.paths.paths.values_mut() {
            for ref mut op in [
                path.get.as_mut(),
                path.post.as_mut(),
                path.put.as_mut(),
                path.patch.as_mut(),
                path.head.as_mut(),
            ]
            .into_iter()
            .flatten()
            {
                if let Some(ref mut sec) = op.security {
                    sec.push(requirement.clone());
                } else {
                    op.security = Some(vec![requirement.clone()]);
                }
            }
        }
    }
}

#[async_trait]
impl<U: OidcUser + Send + Sync> Authentication for OidcAuthentication<U> {
    async fn authorize(&self, request: &Request) -> Result<bool, Error> {
        let config = get_config();
        if !request.session().authenticated() {
            return Ok(false);
        }
        let mut conn = get_connection().await?;
        let user = request.user_required::<U>(&mut conn).await?;
        if !user.is_expired() {
            Ok(true)
        } else {
            let (http_client, oidc_client) = clients(&config.oidc).await?;
            let res = oidc_client
                .exchange_refresh_token(&user.refresh_token())
                .map_err(|e| Error::Error(Box::new(e)))?
                .request_async(&http_client)
                .await
                .map_err(|e| Error::Error(Box::new(e)))?;
            let _user = user.update_token(res).save().fetch(&mut conn).await?;
            Ok(true)
        }
    }
    async fn denied(&self, request: &Request) -> Result<Response, Error> {
        let config = get_config();
        let (_http_client, client) = clients(&config.oidc).await?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token, noonc) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(openidconnect::Scope::new("roles".to_string()))
            .add_scope(openidconnect::Scope::new("profile".to_string()))
            .add_scope(openidconnect::Scope::new("email".to_string()))
            .add_scope(openidconnect::Scope::new("offline_access".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        debug!("{:#?}", VERIFY_MAP);
        VERIFY_MAP.map.write().await.insert(
            csrf_token.secret().clone(),
            VerifyData::new(pkce_verifier, csrf_token, noonc, request.path().clone()),
        );
        Ok(Response::new().redirect(auth_url))
    }
}
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[schema(bound = "")]
pub struct OidcController<U: OidcUser> {
    #[serde(skip)]
    _marker: PhantomData<U>,
}

impl<U: OidcUser> Default for OidcController<U> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<U: OidcUser + Sync + Send> Controller for OidcController<U> {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let config = get_config();
        let (http_client, oidc_client) = clients(&config.oidc).await?;
        if let Some(state) = request.query().get::<String>("state") {
            debug!("{:?}", VERIFY_MAP);
            if let Some((_, verify)) = VERIFY_MAP.map.write().await.remove_entry(state.as_str()) {
                let auth_code =
                    AuthorizationCode::new(request.query().get_required::<String>("code")?);
                let token = oidc_client
                    .exchange_code(auth_code)
                    .map_err(|e| Error::Error(Box::new(e)))?
                    .set_pkce_verifier(verify.verifier)
                    .request_async(&http_client)
                    .await
                    .map_err(|e| Error::Error(Box::new(e)))?;
                let id_token = token
                    .id_token()
                    .ok_or(crate::http::Error::MissingParameter)?;
                let id_token_verifier = oidc_client.id_token_verifier();
                let claim = id_token
                    .claims(&id_token_verifier, &verify.nonce)
                    .map_err(|e| Error::Error(Box::new(e)))?;
                if let Some(expected_token_hash) = claim.access_token_hash() {
                    let token_hash = AccessTokenHash::from_token(
                        token.access_token(),
                        id_token.signing_alg().map_err(Error::new)?,
                        id_token
                            .signing_key(&id_token_verifier)
                            .map_err(|e| Error::Error(Box::new(e)))?,
                    )
                    .map_err(|e| Error::Error(Box::new(e)))?;
                    if expected_token_hash != &token_hash {
                        Err(Error::Error(Box::new(
                            openidconnect::SignatureVerificationError::NoSignature,
                        )))?;
                    }
                    let userinfo: UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim> =
                        oidc_client
                            .user_info(token.access_token().to_owned(), None)
                            .map_err(|e| Error::Error(Box::new(e)))?
                            .request_async(&http_client)
                            .await
                            .map_err(|e| Error::Error(Box::new(e)))?;
                    debug!("Fetched Userinfo: {:#?}", userinfo);
                    let user = U::from_token(token, userinfo).await?;
                    Ok(request.login_user(&user)?.redirect(verify.target))
                } else {
                    Err(Error::HttpError(Box::new(
                        crate::http::Error::MalformedRequest("Invalid Clain Hasth"),
                    )))
                }
            } else {
                Err(Error::SessionMissingError)
            }
        } else {
            Err(Error::HttpError(Box::new(
                crate::http::Error::MissingParameter,
            )))
        }
    }
}
