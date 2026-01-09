use crate::controller::{Controller, Error};
use crate::http::{Handler, Request, Response};
use crate::prelude::OpenApi;
use async_trait::async_trait;
use once_cell::sync::{Lazy, OnceCell};
use std::collections::BTreeMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::RwLock;
use utoipa::Modify;

#[derive(Clone, OpenApi, Default)]
#[openapi(
    info(
        title="rwf",
        version="0.2.1",
        contact(name="levkk", url="https://github.com/levkk?tab=packages", email="none@cf.org"),
        license(name="MIT", url="https://github.com/levkk/rwf/blob/main/LICENSE"),
        description = "OpenAPI definitions and informations about the RWF crate. Also provides API/Type descriptions for Models / ModelController"
    ),
    external_docs(url = "https://rustwebframework.org/", description="Getting started Dcumentation"),
    tags(
        (name = "OpenAPI", description = "Related to OpenAPI itself"),
        (name = "Model", description = "MOdel/ModdelController related Enndpoints"),
        (name = "RWF", description = "The OpenAPI Specs of RWF. Metainformations about RWF and the guys working on. Informations about anything what have to do whith an impplementation has a different Tag", external_docs(url="https://github.com/levkk/rwf/", description="Link to the git, as any Question could find a answer there")),
    ),
    paths()
)]
pub struct OpenApiController {
    mount: OnceCell<String>,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Hash)]
enum OpenApiTargets {
    Info,
    Yaml,
    Json,
    Redoc,
    Rapidoc,
}
#[allow(unused)]
#[utoipa::path(
    get,
    path="/json",
    tag="OpenAPI",
    responses((status = 200, content_type="application/json", description="OpenAPI JSON")),
)]
fn openapi_json(_request: &Request) -> Result<Response, Error> {
    Ok(Response::not_implemented())
}
#[allow(unused)]
#[utoipa::path(
    get,
    path="/yaml",
    tag="OpenAPI",
    responses((status = 200, content_type="application/yank", description="OpenAPI YAML"))
)]
fn openapi_yaml(_request: &Request) -> Result<Response, Error> {
    Ok(Response::not_implemented())
}
#[allow(unused)]
#[utoipa::path(
    get,
    path="/redoc",
    tag="OpenAPI",
    responses((status = 200, content_type="text/html", description="Redoc API Browser"))
)]
fn openapi_redoc(_request: &Request) -> Result<Response, Error> {
    Ok(Response::not_implemented())
}
#[allow(unused)]
#[utoipa::path(
    get,
    path="/rapidoc",
    tag="OpenAPI",
    responses((status = 200, content_type="application/html", description="Rapidoc API Browser"))
)]
fn openapi_rapidoc(_request: &Request) -> Result<Response, Error> {
    Ok(Response::not_implemented())
}

impl FromStr for OpenApiTargets {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "yaml" => Ok(OpenApiTargets::Yaml),
            "json" => Ok(OpenApiTargets::Json),
            "redoc" => Ok(OpenApiTargets::Redoc),
            "rapidoc" => Ok(OpenApiTargets::Rapidoc),
            _ => Ok(OpenApiTargets::Info),
        }
    }
}
#[allow(unused)]
#[derive(OpenApi)]
#[openapi(paths(openapi_json, openapi_yaml, openapi_redoc, openapi_rapidoc))]
struct OpenapiOpenapi;

#[derive(Clone)]
pub enum OpenApiNesterOptions {
    Fn(fn() -> utoipa::openapi::OpenApi),
    Value(utoipa::openapi::OpenApi),
}

impl OpenApiNesterOptions {
    pub fn get_openapi(&self) -> utoipa::openapi::OpenApi {
        match self {
            OpenApiNesterOptions::Fn(func) => func(),
            OpenApiNesterOptions::Value(value) => value.clone(),
        }
    }
}

pub trait IntoOpenApiNesterOption {
    fn into_nester_option(self) -> OpenApiNesterOptions;
}
impl IntoOpenApiNesterOption for utoipa::openapi::OpenApi {
    fn into_nester_option(self) -> OpenApiNesterOptions {
        OpenApiNesterOptions::Value(self)
    }
}
impl IntoOpenApiNesterOption for fn() -> utoipa::openapi::OpenApi {
    fn into_nester_option(self) -> OpenApiNesterOptions {
        OpenApiNesterOptions::Fn(self)
    }
}

#[derive(Default)]
struct OpenapiNester {
    map: RwLock<BTreeMap<String, OpenApiNesterOptions>>,
}

static RWF_OPENAPIS: Lazy<OpenapiNester> = Lazy::new(|| OpenapiNester::default());

pub fn registrer_controller(path: impl ToString, openapi: impl IntoOpenApiNesterOption) {
    RWF_OPENAPIS
        .map
        .write()
        .unwrap()
        .insert(path.to_string(), openapi.into_nester_option());
}

impl std::fmt::Display for OpenApiTargets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenApiTargets::Info => write!(f, "info"),
            OpenApiTargets::Yaml => write!(f, "yaml"),
            OpenApiTargets::Json => write!(f, "json"),
            OpenApiTargets::Redoc => write!(f, "redoc"),
            OpenApiTargets::Rapidoc => write!(f, "rapidoc"),
        }
    }
}

#[async_trait]
impl Controller for OpenApiController {
    fn route(self, path: &str) -> Handler
    where
        Self: Sized + 'static,
    {
        let mut openapi = OpenapiOpenapi::openapi();
        <dyn Controller>::modify(&self, &mut openapi);

        self.mount.set(path.to_string()).unwrap();
        registrer_controller(path.to_string(), openapi);

        Handler::wildcard(path, self)
    }

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        lazy_static::lazy_static! {
            static ref RWFAPI: utoipa::openapi::OpenApi = OpenApiController::rwfapi();
            static ref REDOC: utoipa_redoc::Redoc<utoipa::openapi::OpenApi> = utoipa_redoc::Redoc::new(OpenApiController::rwfapi());
        };

        Ok(match self.match_url(request) {
            OpenApiTargets::Info => {
                Response::new().redirect(format!("{}/json", self.mount.get().unwrap()))
            }
            OpenApiTargets::Yaml => {
                Response::new().text(RWFAPI.to_yaml().map_err(|e| Error::Error(Box::new(e)))?)
            }
            OpenApiTargets::Json => Response::new().json(RWFAPI.deref())?,
            OpenApiTargets::Redoc => Response::new().html(REDOC.to_html()),
            OpenApiTargets::Rapidoc => Response::new().html(
                utoipa_rapidoc::RapiDoc::new(format!("{}/json", self.mount.get().unwrap()))
                    .to_html(),
            ),
        })
    }
}

impl OpenApiController {
    fn match_url(&self, request: &Request) -> OpenApiTargets {
        let path = request
            .path()
            .path()
            .replace(self.mount.get().unwrap(), "")
            .replace("/", "");
        OpenApiTargets::from_str(path.as_str()).unwrap()
    }

    fn rwfapi() -> utoipa::openapi::OpenApi {
        let mut rwfapi = Self::openapi();
        for (k, v) in RWF_OPENAPIS.map.read().unwrap().iter() {
            rwfapi = rwfapi.nest(k, v.get_openapi())
        }
        rwfapi
    }
}
