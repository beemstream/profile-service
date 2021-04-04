use futures::TryFutureExt;
use rocket::{http::Status, post, State};
use rocket_contrib::json::Json;

use crate::{
    database::DbConn,
    models::user::{NewUser, NewUserRequest},
    repository::user::{insert, is_duplicate_user_or_email},
    util::{globals::GlobalConfig, response::Error, validator::Validator},
};

#[post("/register", format = "application/json", data = "<user>")]
pub async fn register_user(
    conn: DbConn,
    user: Json<NewUserRequest>,
    global_config: State<'_, GlobalConfig>,
) -> Result<Status, Error> {
    let user_request = user.into_inner();

    user_request.validate_model()?;

    is_duplicate_user_or_email(&conn, user_request)
        .and_then(|user| insert(&conn, NewUser::from(user, &global_config.auth_secret_key)))
        .await
        .and_then(|_| Ok(Status::Created))
}
