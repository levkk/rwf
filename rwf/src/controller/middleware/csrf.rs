use super::prelude::*;
use crate::{crypto::csrf_token_validate, http::Method};

pub static CSRF_HEADER: &str = "X-CSRF-Token";
pub static CSRF_INPUT: &str = "rwf_csrf_token";

pub struct Csrf;

impl Csrf {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Middleware for Csrf {
    async fn handle_request(&self, request: Request) -> Result<Outcome, Error> {
        if request.skip_csrf() {
            return Ok(Outcome::Forward(request));
        }

        if ![Method::Put, Method::Post, Method::Patch].contains(request.method()) {
            return Ok(Outcome::Forward(request));
        }

        let header = request.header(CSRF_HEADER);

        if let Some(header) = header {
            if csrf_token_validate(header) {
                return Ok(Outcome::Forward(request));
            }
        }

        match request.form_data() {
            Ok(form_data) => {
                if let Some(token) = form_data.get::<String>(CSRF_INPUT) {
                    if csrf_token_validate(&token) {
                        return Ok(Outcome::Forward(request));
                    }
                }
            }

            Err(_) => (),
        }

        Ok(Outcome::Stop(request, Response::csrf_error()))
    }
}
