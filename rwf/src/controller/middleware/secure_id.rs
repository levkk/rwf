use crate::controller::middleware::prelude::*;
use crate::crypto::decrypt_number;
use crate::http::Path;

pub struct SecureId {
    /// Block requests that use plain text identifiers.
    pub block_unencrypted: bool,
}

impl Default for SecureId {
    fn default() -> Self {
        Self {
            block_unencrypted: true,
        }
    }
}

#[async_trait::async_trait]
impl Middleware for SecureId {
    async fn handle_request(&self, mut request: Request) -> Result<Outcome, Error> {
        let id = request.parameter::<String>("id");

        if let Ok(Some(id)) = id {
            // Block requests to a numeric ID.
            if self.block_unencrypted && id.chars().all(|c| c.is_numeric()) {
                return Ok(Outcome::Stop(request, Response::not_found()));
            }

            let path = request.path().clone();

            if let Ok(decrypted) = decrypt_number(&id) {
                let base = path.base().replace(&id, &decrypted.to_string());

                let head = request.head_mut();
                head.replace_path(Path::from_parts(&base, path.query()));

                return Ok(Outcome::Forward(request));
            } else {
                return Ok(Outcome::Stop(request, Response::not_found()));
            }
        }

        Ok(Outcome::Forward(request))
    }
}
