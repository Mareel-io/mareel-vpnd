/*
 * SPDX-FileCopyrightText: 2022 Empo Inc.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

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
