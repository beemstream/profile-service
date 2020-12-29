use crate::{database::DbConn, jwt::jwt_validation, models::user::Claims, repository::user::find, util::globals::SECRET_KEY};
use jsonwebtoken::{decode, DecodingKey};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
};

pub struct AccessToken(String);

#[derive(Debug)]
pub enum AccessTokenError {
    Missing,
    Invalid,
}

pub async fn is_token_valid(conn: &DbConn, token: &str) -> bool {
    let decode_key = DecodingKey::from_secret(&SECRET_KEY.as_ref());
    match decode::<Claims>(token, &decode_key, &jwt_validation()) {
        Ok(t) => find(conn, t.claims.sub().to_owned()).await.is_ok(),
        Err(_) => false,
    }
}

#[async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for AccessToken {
    type Error = AccessTokenError;

    async fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let db_conn = request.guard::<DbConn>().await.unwrap();
        let keys: Vec<&str> = request.headers().get("token").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::Unauthorized, AccessTokenError::Missing)),
            1 if is_token_valid(&db_conn, keys[0]).await => Outcome::Success(AccessToken(keys[0].to_string())),
            1 => Outcome::Failure((Status::Unauthorized, AccessTokenError::Invalid)),
            _ => Outcome::Failure((Status::Unauthorized, AccessTokenError::Invalid)),
        }
    }
}
