//! Password authentication controller.
//!
//! Create an account if one doesn't exist. If one exists, attempt to log in.
//!
//! ### Errors
//!
//! The following errors are set in the template:
//!
//! - `error_form`: Any of the required fields are missing.
//! - `error_password`: Account exists and the password is incorrect.
//! - `error_password2`: Passwords do not match on account creation. Only set if `<input name="password2">` is present in the form.
use std::marker::PhantomData;

use rwf::{
    model::{user::Error as UserError, UserModel},
    prelude::*,
};

use crate::models::User;

/// Account creation and login form.
#[derive(macros::Form)]
pub struct PasswordForm {
    identifier: String,
    password: String,
    password2: Option<String>,
}

/// Password errors.
///
/// These are passed to the template in the context.
#[derive(macros::Context, Default)]
pub struct Errors {
    /// Form has missing fields.
    pub error_form: bool,
    /// Password was incorrect or the user didn't exist.
    pub error_password: bool,
    /// Passwords do not match.
    pub error_password2: bool,
}

impl Errors {
    fn form() -> Self {
        let mut ctx = Self::default();
        ctx.error_form = true;
        ctx
    }

    fn wrong_password() -> Self {
        let mut ctx = Self::default();
        ctx.error_password = true;
        ctx
    }

    fn wrong_password_match() -> Self {
        let mut ctx = Self::default();
        ctx.error_password2 = true;
        ctx
    }
}

/// Generic password authentication controller. Can be used with any model
/// which implements the [`rwf::model::UserModel`] trait.
#[derive(Default)]
pub struct Password<T: UserModel> {
    template_path: String,
    redirect_url: String,
    _marker: PhantomData<T>,
}

impl<T: UserModel> Password<T> {
    /// Create controller with the specified template path.
    pub fn template(template_path: &str) -> Self {
        Self {
            template_path: template_path.to_owned(),
            redirect_url: "/".into(),
            _marker: PhantomData,
        }
    }

    /// Redirect to the specified URL on successful authentication.
    pub fn redirect(mut self, redirect_url: &str) -> Self {
        self.redirect_url = redirect_url.to_owned();
        self
    }
}

#[async_trait]
impl<T: UserModel> Controller for Password<T> {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        PageController::handle(self, request).await
    }
}

#[async_trait]
impl<T: UserModel> PageController for Password<T> {
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        render!(request, &self.template_path)
    }

    async fn post(&self, request: &Request) -> Result<Response, Error> {
        let tpl = Template::load(&self.template_path)?;

        let form = if let Ok(form) = request.form::<PasswordForm>() {
            form
        } else {
            return Ok(Response::new().html(tpl.render(Errors::form())?).code(400));
        };

        // If second password passed in, make sure they match.
        if let Some(ref password2) = form.password2 {
            if password2 != &form.password {
                return Ok(Response::new()
                    .html(tpl.render(Errors::wrong_password_match())?)
                    .code(400));
            }
        }

        let user = match T::create_user(&form.identifier, &form.password).await {
            Ok(user) => user,
            Err(UserError::UserExists) => {
                match T::login_user(&form.identifier, &form.password).await {
                    Ok(user) => user,
                    Err(UserError::WrongPassword) => {
                        return Ok(Response::new()
                            .html(tpl.render(Errors::wrong_password())?)
                            .code(400))
                    }
                    Err(err) => return Err(err.into()),
                }
            }

            Err(err) => return Err(err.into()),
        };

        Ok(request.login_user(&user)?.redirect(&self.redirect_url))
    }
}

/// Password controller implemented for the [`rwf_auth::models::User`] model.
pub type PasswordController = Password<User>;
