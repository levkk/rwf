use std::marker::PhantomData;

use super::{Controller, Error, PageController};
use crate::view::{Context, Template};
use crate::{
    http::{Request, Response},
    model::{user::Error as UserError, UserModel},
};
use async_trait::async_trait;

/// Controller to log a user in.
pub struct LoginController<T> {
    redirect: Option<String>,
    template: &'static str,
    signup: bool,
    _marker: PhantomData<T>,
}

/// Controller to create user accounts.
pub struct SignupController<T>(LoginController<T>);

impl<T: UserModel> SignupController<T> {
    /// Create new signup controller with the given template.
    pub fn new(template: &'static str) -> Self {
        Self(LoginController::<T>::new(template).signup())
    }

    /// Redirect to this URL if account creation was successful.
    pub fn redirect(mut self, redirect: impl ToString) -> Self {
        self.0 = self.0.redirect(redirect);
        self
    }
}

#[async_trait]
impl<T: UserModel> Controller for SignupController<T> {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        PageController::handle(&self.0, request).await
    }
}

impl<T: UserModel> LoginController<T> {
    /// Create new login controller with the provided template.
    pub fn new(template: &'static str) -> Self {
        Self {
            redirect: None,
            template,
            signup: false,
            _marker: PhantomData,
        }
    }

    /// Redirect on successful account creation to this URL.
    pub fn redirect(mut self, redirect: impl ToString) -> Self {
        self.redirect = Some(redirect.to_string());
        self
    }

    fn signup(mut self) -> Self {
        self.signup = true;
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

        if self.signup {
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
        } else {
            match T::login_user(identifier, password).await {
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
                    UserError::UserDoesNotExist => {
                        return self.error(request, "error_user_does_not_exist")
                    }
                    UserError::WrongPassword => return self.error(request, "error_password"),
                    err => return Err(err.into()),
                },
            }
        }
    }
}

#[async_trait]
impl<T: UserModel> Controller for LoginController<T> {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        PageController::handle(self, request).await
    }
}

/// Controller to log the user out.
#[derive(Default)]
pub struct LogoutController {
    redirect: Option<String>,
}

impl LogoutController {
    /// Create new logout controller.
    pub fn new() -> Self {
        Self { redirect: None }
    }

    /// Redirect to this URL after logging the user out.
    pub fn redirect(mut self, redirect: impl ToString) -> Self {
        self.redirect = Some(redirect.to_string());
        self
    }
}

#[async_trait]
impl Controller for LogoutController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let response = request.logout();
        if let Some(ref redirect) = self.redirect {
            Ok(response.redirect(redirect))
        } else {
            Ok(response)
        }
    }
}
