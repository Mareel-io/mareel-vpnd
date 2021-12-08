use std::sync::Arc;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

const KEYTAR_PACKAGE_NAME: &str = "io.mareel.vpn.vpnd";
const KEYTAR_ACC_NAME: &str = "apikey";

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
        let keys: Vec<&str> = req.headers().get("Authorization").collect();
        match keys.len() {
            1 => match Option::as_ref(&APITOKEN) {
                Some(x) => match x.as_str() == keys[0] {
                    true => Outcome::Success(Self),
                    false => Outcome::Failure((Status::Unauthorized, ())),
                },
                None => Outcome::Failure((Status::InternalServerError, ())),
            },
            _ => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}
