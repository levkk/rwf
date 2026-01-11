use crate::snake_case;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use quote::{format_ident, ToTokens};
use std::fmt::Debug;
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_quote, Expr, ExprMethodCall, ImplItem, ItemStruct, Lit, LitStr, Stmt, Token, Type,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResponseTypes {
    HTML,
    JSON,
    TEXT,
    TURBO,
    NotImplemented,
    NotFound,
    Forbidden,
    BadRequest,
    Redirect,
}

impl ResponseTypes {
    fn default_code(&self) -> u16 {
        match self {
            ResponseTypes::HTML => 200,
            ResponseTypes::JSON => 200,
            ResponseTypes::TEXT => 200,
            ResponseTypes::TURBO => 200,
            ResponseTypes::NotImplemented => 501,
            ResponseTypes::NotFound => 404,
            ResponseTypes::Forbidden => 403,
            ResponseTypes::BadRequest => 400,
            ResponseTypes::Redirect => 302,
        }
    }
}
#[derive(Default)]
struct ResponseBuilder {
    ty: Option<ResponseTypes>,
    code: Option<Lit>,
}
struct Response {
    ty: ResponseTypes,
    code: Lit,
    json: Option<Type>,
}
impl From<ResponseTypes> for Response {
    fn from(value: ResponseTypes) -> Self {
        let code = Lit::new(Literal::from_str(value.default_code().to_string().as_str()).unwrap());
        Self {
            ty: value,
            code,
            json: None,
        }
    }
}
impl ResponseBuilder {
    fn build(self) -> Response {
        if self.ty.is_none() {
            panic!("Failed to Infer Response type")
        }
        let ty = self.ty.unwrap();
        if self.code.is_some() {
            Response {
                ty,
                code: self.code.unwrap(),
                json: None,
            }
        } else {
            Response::from(ty)
        }
    }
}

impl ToTokens for ResponseTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ResponseTypes::HTML => quote! {content_type="text/html", description="A HTML Response"},
            ResponseTypes::JSON => {
                quote! {content_type="application/json", description="A JSON Response"}
            }
            ResponseTypes::TEXT => {
                quote! {content_type="text/plain", description="A Text Response"}
            }
            ResponseTypes::TURBO => {
                quote! {content_type="text/vnd.turbo-stream.html", description="A Turbo Response"}
            }
            ResponseTypes::NotFound => quote! {description="Requested Content was not found"},
            ResponseTypes::Forbidden => quote! {description="Request is forbidden"},
            ResponseTypes::BadRequest => quote! {description="Bad Request"},
            ResponseTypes::Redirect => quote! {description="Redirect to another location"},
            ResponseTypes::NotImplemented => {
                quote! {description="The endpoint is not implemented yet"}
            }
        }
        .to_tokens(tokens);
    }
}

impl ToTokens for Response {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let code = self.code.clone();
        let ty = self.ty.clone();
        let json = if let Some(json) = self.json.as_ref() {
            quote! {, body=#json}
        } else {
            TokenStream::new()
        };
        quote! {
            (status = #code, #ty #json)
        }
        .to_tokens(tokens);
    }
}

fn parse_method_chain(call: &ExprMethodCall, builder: &mut ResponseBuilder) {
    // eprintln!("METHOD CALL: {} - {}", call.receiver.to_token_stream(), call.method.to_token_stream());
    let receiver = call.receiver.to_token_stream().to_string();
    let method = call.method.to_token_stream().to_string();
    if receiver.starts_with("Response") {
        if method.eq("html") {
            builder.ty = Some(ResponseTypes::HTML);
        } else if method.eq("text") {
            builder.ty = Some(ResponseTypes::TEXT);
        } else if method.eq("turbo_stream") {
            builder.ty = Some(ResponseTypes::TURBO);
        } else if method.eq("json") {
            builder.ty = Some(ResponseTypes::JSON);
        } else if method.eq("code") {
            if let Some(Expr::Lit(code)) = call.args.iter().next() {
                builder.code = Some(code.lit.clone())
            }
        } else if method.eq("redirect") {
            builder.ty = Some(ResponseTypes::Redirect)
        }
        if let Expr::MethodCall(expr) = call.receiver.as_ref() {
            parse_method_chain(expr, builder);
        }
    }
}
fn parse_call(expr: &Expr, acc: &mut Vec<Response>) {
    if let Expr::Call(call) = expr {
        //eprintln!("CALL: {} - {}", call.func.to_token_stream(), call.args.to_token_stream());
        let func = call.func.to_token_stream().to_string();
        if func == "Ok" {
            if call.args.len() == 1 {
                return parse_call(call.args.iter().next().unwrap(), acc);
            }
        } else if func.starts_with("Response") {
            if func.contains("not_implemented") {
                acc.push(Response::from(ResponseTypes::NotImplemented))
            } else if func.contains("not_found") {
                acc.push(Response::from(ResponseTypes::NotFound))
            } else if func.contains("bad_request") {
                acc.push(Response::from(ResponseTypes::BadRequest))
            } else if func.contains("forbidden") {
                acc.push(Response::from(ResponseTypes::Forbidden))
            }
        }
    } else if let Expr::MethodCall(call) = expr {
        let mut builder = ResponseBuilder::default();
        parse_method_chain(call, &mut builder);
        if !builder.ty.is_none() {
            acc.push(builder.build());
        }
    } else if let Expr::If(expr_if) = expr {
        parse_block(expr_if.then_branch.stmts.clone(), acc);
        if let Some(ref else_expr) = expr_if.else_branch {
            return parse_call(else_expr.1.as_ref(), acc);
        }
    } else if let Expr::Block(block) = expr {
        parse_block(block.block.stmts.clone(), acc)
    } else if let Expr::Return(retexpr) = expr {
        if let Some(expr) = retexpr.expr.as_ref() {
            parse_call(expr, acc)
        }
    } else if let Expr::Try(expr_try) = expr {
        parse_call(expr_try.expr.as_ref(), acc)
    } else if let Expr::Macro(expr_macro) = expr {
        if expr_macro.mac.path.is_ident("render") {
            acc.push(Response::from(ResponseTypes::HTML))
        } else if expr_macro.mac.path.is_ident("turbo_stream") {
            acc.push(Response::from(ResponseTypes::TURBO))
        }
    } else if let Expr::Match(expr_match) = expr {
        for arm in expr_match.arms.iter() {
            parse_call(arm.body.as_ref(), acc)
        }
    }
}

fn parse_block(stmts: Vec<Stmt>, acc: &mut Vec<Response>) {
    for stmt in stmts {
        // eprintln!("{}", stmt.to_token_stream());
        if let Stmt::Expr(expr, ..) = stmt {
            parse_call(&expr, acc);
        }
        // eprintln!();
    }
}

pub fn generate_api_specs_controller(
    mut input: syn::ItemImpl,
    json: Option<Type>,
) -> proc_macro2::TokenStream {
    let mut outputt = proc_macro2::TokenStream::new();
    let mut fnames = Vec::new();

    let mut visited = false;

    let apiname = format_ident!("{}ApiDoc", input.self_ty.to_token_stream().to_string());

    for item in input.items.clone() {
        if let ImplItem::Fn(fnimpl) = item {
            if ["handle", "list", "get", "post", "delete", "put", "patch"]
                .contains(&fnimpl.sig.ident.to_string().as_str())
            {
                let (method, path) = if let Some((_, pth, _)) = input.trait_.clone() {
                    if pth.is_ident("Controller") {
                        if !visited {
                            input.items.push(parse_quote!{
                                fn route(self, path: &str) -> rwf::http::Handler
                                where Self: Sized + 'static,
                                {
                                    let mut spec = #apiname :: openapi() ;
                                    <dyn rwf::controller::Controller>::modify(&self, &mut spec);
                                    rwf::controller::openapi::registrer_controller(path.to_string(), spec);
                                    rwf::http::Handler::route(path, self)
                                }
                            });
                            input.to_tokens(&mut outputt);
                            visited = true;
                        }
                        (format_ident!("get"), LitStr::new("/", Span::call_site()))
                    } else if pth.is_ident("PageController") {
                        if fnimpl.sig.ident.ne("handle") {
                            if !visited {
                                input.items.push(parse_quote!{
                                fn route(self, path: &str) -> rwf::http::Handler
                                where Self: Sized + 'static,
                                {
                                    let mut spec = #apiname :: openapi() ;
                                    <dyn rwf::controller::Controller>::modify(&self, &mut spec);
                                    rwf::controller::openapi::registrer_controller(path.to_string(), spec);
                                    rwf::http::Handler::route(path, self)
                                }
                            });
                                input.to_tokens(&mut outputt);
                                visited = true;
                            }
                            (
                                fnimpl.sig.ident.clone(),
                                LitStr::new("/", Span::call_site()),
                            )
                        } else {
                            continue;
                        }
                    } else if pth.is_ident("RestController") {
                        if !visited {
                            input.items.push(parse_quote!{
                                fn rest(self, path: &str) -> rwf::http::Handler
                                where Self: Sized + 'static,
                                {
                                    let mut spec = #apiname :: openapi() ;
                                    <dyn rwf::controller::Controller>::modify(&self, &mut spec);
                                    rwf::controller::openapi::registrer_controller(path.to_string(), spec);
                                    rwf::http::Handler::rest(path, self)
                                }
                            });
                            input.to_tokens(&mut outputt);
                            visited = true;
                        }
                        if fnimpl.sig.ident.eq("handle") {
                            continue;
                        } else if fnimpl.sig.ident.eq("list") {
                            (format_ident!("get"), LitStr::new("/", Span::call_site()))
                        } else if fnimpl.sig.ident.eq("post") {
                            (format_ident!("post"), LitStr::new("/", Span::call_site()))
                        } else {
                            (
                                fnimpl.sig.ident.clone(),
                                LitStr::new("/{id}", Span::call_site()),
                            )
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
                let mut acc = Vec::new();
                parse_block(fnimpl.block.stmts, &mut acc);
                if !acc.is_empty() {
                    let controller = input.self_ty.clone();
                    let ident = fnimpl.sig.ident.clone();
                    let fnname = format_ident!(
                        "{}_{}",
                        snake_case(controller.to_token_stream().to_string().as_str()),
                        ident
                    );
                    for res in acc.iter_mut() {
                        if ResponseTypes::JSON == res.ty {
                            res.json = json.clone()
                        }
                    }
                    let generate = quote! {
                        #[rwf::prelude::utoipa::path(
                            #method,
                            path=#path,
                            responses(
                                #(
                                    #acc
                                ),*
                                , (status = 500, description = "A InternalServerError occoured")
                            )
                        )]
                        fn #fnname(request: &Request) -> Result<Response, rwf::controller::Error> {
                            Ok(Response::not_implemented())
                        }
                    };
                    generate.to_tokens(&mut outputt);
                    fnames.push(fnname);
                }
            }
        }
    }
    let components = if let Some(json) = json {
        quote! {
            components(
                schemas(#json),
                responses(#json)
            ),
        }
    } else {
        proc_macro2::TokenStream::new()
    };
    quote! {
        #[derive(rwf::prelude::OpenApi)]
        #[openapi(
            #components
            paths(
                #(
                    #fnames
                ),*
            )
        )]
        struct #apiname;
    }
    .to_tokens(&mut outputt);
    outputt
}
pub fn generate_controller4(
    mut input: ItemStruct,
    targs: TypeParserInput,
) -> proc_macro2::TokenStream {
    let targs = targs.into_type_parser(&input);

    let paths = targs.gen_doc_paths();
    let mut output = proc_macro2::TokenStream::new();

    let model = targs.model.clone();
    let ctrl = targs.ctrl.clone();
    let path_impl = targs.gen_oapi();
    let apiname = format_ident!("{}OpenapiDoc", model);
    let resource = targs.ty.clone();
    if !input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("resource"))
    {
        input.attrs.push(parse_quote!(#[resource(#resource)]))
    }
    input.to_tokens(&mut output);
    quote! {
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
            .zip(paths)
            .zip(params.into_iter().zip(requests))
            .map(|((o, p), (par, req))| (o, p, par, req))
            .zip(responses.into_iter().zip(funcs))
            .map(|((o, p, par, req), (res, func))| (o, p, par, req, res, func))
            .map(|(o, p, par, req, res, func)| {
                let endpoint_tag = LitStr::new({
                                                   if p.token().to_string().as_str().contains("{id}") {"InstanceUrl"} else {"CollectionUrl"}
                                               }, Span::call_site());
                if o.eq("get") || o.eq("delete") {
                    quote! {
                        #[rwf::prelude::utoipa::path(
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
                } else if o.eq("post") {
                    quote! {
                        #[rwf::prelude::utoipa::path(
                            #o,
                            path=#p,
                            #req,
                            responses(#res),
                            tags=["ModelController", #model_tag, #endpoint_tag],
                        )]
                        fn #func (_request: &rwf::http::Request) -> Result<rwf::http::Response, rwf::http::Error> {
                            Ok(rwf::http::Response::not_implemented())
                        }
                    }
                } else {
                    quote! {
                        #[rwf::prelude::utoipa::path(
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
                )
            },
            quote! {
                params(("id" = #pkey_type , Path, description = "Database ID of the Model"))
            },
            quote! {
                params(
                    ("id" = #pkey_type, Path, description = "Database ID of the Model"),
                )
            },
            quote! {
                params(
                    ("id" = #pkey_type , Path, description = "Database ID of the Model"),
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
        let sufix = Ident::new(
            pluralizer::pluralize(
                snake_case(self.model.to_string().as_str()).as_str(),
                2,
                false,
            )
            .as_str(),
            Span::call_site(),
        );
        TypeParser {
            ty: self.ty,
            ctrl: input.ident.clone(),
            model: self.model,
            sufix,
        }
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
