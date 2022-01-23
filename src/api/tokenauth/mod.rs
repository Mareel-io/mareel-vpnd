use argon2::password_hash::{PasswordHash, PasswordVerifier};
use argon2::Argon2;
use regex::Regex;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use super::AuthKeyProvider;

lazy_static! {
    static ref AUTH_REGEX: Regex = Regex::new("^(Bearer |)(.*)$").unwrap();
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
            1 => match AUTH_REGEX.captures(keys[0]) {
                Some(x) => {
                    let pass = x.get(2).unwrap().as_str().as_bytes();
                    let hash = PasswordHash::new(key).unwrap();
                    match Argon2::default().verify_password(pass, &hash) {
                        Ok(_) => Outcome::Success(Self),
                        Err(_) => Outcome::Failure((Status::Unauthorized, ())),
                    }
                }
                None => Outcome::Failure((Status::Unauthorized, ())),
            },
            _ => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}
