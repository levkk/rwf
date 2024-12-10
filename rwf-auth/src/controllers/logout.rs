//! Logout controller.
use rwf::prelude::*;

/// Log the user out.
pub struct LogoutController {
    redirect: String,
}

impl LogoutController {
    /// Redirect user to this URL after logging out.
    pub fn redirect(redirect: impl ToString) -> Self {
        Self {
            redirect: redirect.to_string(),
        }
    }
}

impl Default for LogoutController {
    fn default() -> Self {
        Self {
            redirect: "/".to_string(),
        }
    }
}

#[async_trait]
impl Controller for LogoutController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(request.logout().redirect(self.redirect.clone()))
    }
}
