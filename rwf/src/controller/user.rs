use std::marker::PhantomData;

use super::{Controller, Error, PageController};
use crate::view::{Context, Template};
use crate::{
    http::{Request, Response},
    model::{user::Error as UserError, UserModel},
};
use async_trait::async_trait;

pub struct LoginController<T> {
    redirect: Option<String>,
    template: &'static str,
    _marker: PhantomData<T>,
}

impl<T: UserModel> LoginController<T> {
    pub fn new(template: &'static str) -> Self {
        Self {
            redirect: None,
            template,
            _marker: PhantomData,
        }
    }

    pub fn redirect(mut self, redirect: impl ToString) -> Self {
        self.redirect = Some(redirect.to_string());
        self
    }

    fn error(&self, request: &Request, error: &str) -> Result<Response, Error> {
        let template = Template::load(self.template)?;
        let mut ctx = Context::new();
        ctx.set(error, true)?;
        ctx.set("request", request.clone())?;
        Ok(Response::new().html(template.render(&ctx)?).code(400))
    }
}

#[async_trait]
impl<T: UserModel> PageController for LoginController<T> {
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        let mut ctx = Context::new();
        ctx.set("request", request.clone())?;
        let template = Template::load(self.template)?;
        Ok(Response::new().html(template.render(&ctx)?))
    }

    async fn post(&self, request: &Request) -> Result<Response, Error> {
        let form = request.form_data()?;
        let identifier: String = match form.get_required("identifier") {
            Ok(field) => field,
            Err(_) => return self.error(request, "error_identifier"),
        };
        let identifier = identifier.trim().to_string();
        let password: String = match form.get_required("password") {
            Ok(field) => field,
            Err(_) => return self.error(request, "error_password"),
        };

        match T::create_user(identifier, password).await {
            Ok(user) => {
                let id = user.id().integer()?;
                let response = request.login(id);

                if let Some(ref redirect) = self.redirect {
                    Ok(response.redirect(redirect))
                } else {
                    Ok(response)
                }
            }
            Err(err) => match err {
                UserError::UserExists => return self.error(request, "error_user_exists"),
                err => return Err(err.into()),
            },
        }
    }
}

#[async_trait]
impl<T: UserModel> Controller for LoginController<T> {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        PageController::handle(self, request).await
    }
}
