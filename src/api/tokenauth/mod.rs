use std::sync::Arc;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use super::AuthKeyProvider;

#[cfg(feature = "keytar")]
const KEYTAR_PACKAGE_NAME: &str = "io.mareel.vpn.vpnd";
#[cfg(feature = "keytar")]
const KEYTAR_ACC_NAME: &str = "apikey";

#[cfg(feature = "keytar")]
lazy_static! {
    static ref APITOKEN: Arc<Option<String>> = {
        // Load API key from keytar
        let keyresult = match keytar::get_password(KEYTAR_PACKAGE_NAME, KEYTAR_ACC_NAME) {
            Ok(x) => x,
            Err(_) => return Arc::new(None),
        };

        // Ugly API but I cannot do anything about it for now...
        // Track issue https://github.com/stoically/keytar-rs/issues/2
        Arc::new(match keyresult.success {
            true => Some(keyresult.password),
            false => None,
        })
    };
}

pub struct ApiKey;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let key = match req.rocket().state::<AuthKeyProvider>() {
            Some(x) => &x.auth_key,
            None => return Outcome::Failure((Status::InternalServerError, ())),
        };

        let keys: Vec<&str> = req.headers().get("Authorization").collect();
        match keys.len() {
            1 => match key == keys[0] {
                true => Outcome::Success(Self),
                false => Outcome::Failure((Status::Unauthorized, ())),
            },
            _ => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}
