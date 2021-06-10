use futures::TryFutureExt;
use rocket::{State, http::Status, post, serde::json::Json};

use crate::{
    database::DbConn,
    email_sender::send_email,
    models::user::{NewUser, NewUserRequest},
    repository::user::{insert, is_duplicate_user_or_email},
    util::{
        globals::{EmailConfig, GlobalConfig},
        response::Error,
        validator::Validator,
    },
};

#[post("/register", format = "application/json", data = "<user>")]
pub async fn register_user(
    conn: DbConn,
    user: Json<NewUserRequest>,
    global_config: &State<GlobalConfig>,
    email_config: &State<EmailConfig>,
) -> Result<Status, Error> {
    let user_request = user.into_inner();

    user_request.validate_model()?;

    is_duplicate_user_or_email(&conn, user_request)
        .and_then(|user| insert(&conn, NewUser::from(user, &global_config.auth_secret_key)))
        .await
        .map(|user| {
            if email_config.email_enabled {
                rocket::tokio::spawn(send_email(
                    user.email.clone(),
                    email_config.email_username.clone(),
                    email_config.email_password.clone(),
                ));
            }
        })
        .and_then(|_| Ok(Status::Created))
}
