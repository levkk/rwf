use crate::models::*;
use rwf::prelude::*;

#[controller]
pub async fn profile(request: &Request) -> Result<Response, Error> {
    let user = {
        let mut conn = Pool::connection().await?;
        request.user_required::<User>(&mut conn).await?
    };

    render!(request, "templates/profile.html", "user" => user)
}
