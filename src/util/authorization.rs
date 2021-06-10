use crate::{database::DbConn, jwt::Claims, repository::user::find};
use jsonwebtoken::{decode, DecodingKey, Validation};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
};

use super::globals::{GlobalConfig, JWTConfig};
use async_trait::async_trait;

#[derive(Debug)]
pub struct AccessToken(pub String);

#[derive(Debug)]
pub enum AccessTokenError {
    Missing,
    Invalid,
}

pub async fn is_token_valid(
    conn: &DbConn,
    token: &str,
    secret_key: &str,
    validation: &Validation,
) -> bool {
    let decode_key = DecodingKey::from_secret(secret_key.as_ref());
    let request_token: Vec<&str> = token.split(' ').collect();

    match request_token.starts_with(&["Bearer"]) {
        true => match decode::<Claims>(request_token[1], &decode_key, validation) {
            Ok(t) => find(conn, t.claims.sub().to_owned()).await.is_ok(),
            Err(_) => false,
        },
        _ => false,
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for AccessToken {
    type Error = AccessTokenError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
                 let db_conn = request.guard::<DbConn>().await.unwrap();
         let config = request.rocket().state::<GlobalConfig>().unwrap();
         let jwt_config = request.rocket().state::<JWTConfig>().unwrap();
         let keys: Vec<&str> = request.headers().get("token").collect();
         match keys.len() {
             0 => Outcome::Failure((Status::Unauthorized, AccessTokenError::Missing)),
             1 if is_token_valid(
                 &db_conn,
                 keys[0],
                 &config.auth_secret_key,
                 &jwt_config.validation,
             )
             .await =>
             {
                 Outcome::Success(AccessToken(keys[0].to_string()))
             }
             _ => Outcome::Failure((Status::Unauthorized, AccessTokenError::Invalid)),
         }
    }
}
