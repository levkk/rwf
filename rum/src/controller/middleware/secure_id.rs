use crate::controller::middleware::prelude::*;
use crate::crypto::decrypt_number;
use crate::http::Path;

pub struct SecureId;

#[middleware]
impl Middleware for SecureId {
    async fn handle_request(&self, mut request: Request) -> Result<Outcome, Error> {
        let id = request.parameter::<String>("id");

        if let Ok(Some(id)) = id {
            // Already a number, no need to decrypt.
            if id.chars().all(|c| c.is_numeric()) {
                return Ok(Outcome::Forward(request));
            }

            let path = request.path().clone();

            if let Ok(decrypted) = decrypt_number(&id) {
                let base = path.base().replace(&id, &decrypted.to_string());

                let head = request.head_mut();
                head.replace_path(Path::from_parts(&base, path.query()));

                return Ok(Outcome::Forward(request));
            }
        }

        Ok(Outcome::Forward(request))
    }
}
