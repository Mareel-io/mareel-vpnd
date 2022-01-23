use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use super::AuthKeyProvider;

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
            1 => match (key == keys[0]) || (format!("Bearer {}", key) == keys[0]) {
                true => Outcome::Success(Self),
                false => Outcome::Failure((Status::Unauthorized, ())),
            },
            _ => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}
