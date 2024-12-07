use std::marker::PhantomData;

use rwf::{
    model::{user::Error as UserError, UserModel},
    prelude::*,
};

#[derive(macros::Form)]
struct PasswordForm {
    identifier: String,
    password: String,
}

/// Errors passed to the template.
#[derive(macros::Context, Default)]
pub struct Errors {
    /// Something was wrong with the identifier.
    pub error_identifier: bool,
    /// Password was incorrect or the user didn't exist.
    pub error_password: bool,
}

impl Errors {
    fn form() -> Self {
        let mut ctx = Self::default();
        ctx.error_identifier = true;
        ctx.error_password = true;
        ctx
    }

    fn wrong_password() -> Self {
        let mut ctx = Self::default();
        ctx.error_password = true;
        ctx
    }
}

#[derive(Default)]
pub struct Password<T: UserModel> {
    template_path: String,
    redirect_url: String,
    _marker: PhantomData<T>,
}

impl<T: UserModel> Password<T> {
    pub fn template(template_path: &str) -> Self {
        Self {
            template_path: template_path.to_owned(),
            redirect_url: "/".into(),
            _marker: PhantomData,
        }
    }

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
