use crate::models::*;
use rwf::prelude::*;

#[derive(macros::Form)]
struct SignupForm {
    email: String,
    password: String,
}

#[derive(Default, macros::PageController)]
pub struct Signup;

#[async_trait]
impl PageController for Signup {
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        let user = request.user::<User>(Pool::pool()).await?;

        if user.is_some() {
            return Ok(Response::new().redirect("/profile"));
        }

        render!(request, "templates/signup.html")
    }

    async fn post(&self, request: &Request) -> Result<Response, Error> {
        let form = request.form::<SignupForm>()?;
        let user = User::signup(&form.email, &form.password).await?;

        match user {
            UserLogin::Ok(user) => Ok(request.login_user(&user)?.redirect("/profile")),
            _ => render!(request, "templates/signup.html", "error" => true, 400),
        }
    }
}

#[controller]
pub async fn login(request: &Request) -> Result<Response, Error> {
    let form = request.form::<SignupForm>()?;

    let user = User::login(&form.email, &form.password).await?;

    if let UserLogin::Ok(_) = user {
        Ok(Response::new().redirect("/profile"))
    } else {
        render!(
            request,
            "templates/signup.html",
            "login" => true,
            "error" => true,
            400
        )
    }
}

#[controller]
pub async fn profile(request: &Request) -> Result<Response, Error> {
    let user = {
        let mut conn = Pool::connection().await?;
        request.user_required::<User>(&mut conn).await?
    };

    render!(request, "templates/profile.html", "user" => user)
}
