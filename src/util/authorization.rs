use jsonwebtoken::{DecodingKey, decode};
use crate::{jwt::jwt_validation, models::user::Claims, repository::user::find, util::globals::SECRET_KEY};
use rocket::{http::Status, request::{FromRequest, Outcome}};

pub struct AccessToken(String);

#[derive(Debug)]
pub enum AccessTokenError {
    Missing,
    Invalid,
}

pub fn is_token_valid(token: &str) -> bool {
    let decode_key = DecodingKey::from_secret(&SECRET_KEY.as_ref());
    match decode::<Claims>(token, &decode_key, &jwt_validation()) {
        Ok(t) => find(t.claims.sub()).is_ok(),
        Err(_) => false
    }
}

#[async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for AccessToken {
    type Error = AccessTokenError;

    async fn from_request(request: &'a rocket::Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
        let keys: Vec<&str> = request.headers().get("token").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::Unauthorized, AccessTokenError::Missing)),
            1 if is_token_valid(keys[0]) => Outcome::Success(AccessToken(keys[0].to_string())),
            1 => Outcome::Failure((Status::Unauthorized, AccessTokenError::Invalid)),
            _ => Outcome::Failure((Status::Unauthorized, AccessTokenError::Invalid)),
        }
    }
}
