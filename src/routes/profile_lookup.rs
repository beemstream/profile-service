use rocket::{http::Status, State};
use rocket_contrib::json::Json;

use crate::{
    database::DbConn,
    models::user::User,
    repository::user::find,
    util::{
        authorization::AccessToken,
        globals::{GlobalConfig, JWTConfig},
    },
};

use super::users_util::get_jwt_claim;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UserLookUpResponse {
    id: i32,
    username: String,
    email: String,
}

impl UserLookUpResponse {
    pub fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    }
}

#[get("/profile")]
pub async fn profile_lookup<'a>(
    db_conn: DbConn,
    access_token: AccessToken,
    global_config: State<'a, GlobalConfig>,
    jwt_config: State<'a, JWTConfig>,
) -> Result<Json<UserLookUpResponse>, Status> {
    let AccessToken(token) = access_token;
    let request_token: Vec<&str> = token.split(" ").collect();

    let token_claim = get_jwt_claim(
        request_token[1],
        &global_config.auth_secret_key,
        &jwt_config.validation,
    )
    .map_err(|_| Status::Unauthorized)?;

    let identifier = token_claim.claims.sub();

    let user = find(&db_conn, identifier.to_string()).await?;

    Ok(Json(UserLookUpResponse::from(user)))
}
