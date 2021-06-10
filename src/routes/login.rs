use rocket::{State, http::{CookieJar, Status}, post, serde::json::Json};

use crate::{
    database::DbConn,
    models::user::{LoginUser, UserType},
    repository::user::find,
    util::{
        globals::GlobalConfig,
        response::{Error, Response, TokenResponse},
        validator::Validator,
    },
};

use super::users_util::{
    add_token_response, generate_and_store_refresh_token, verify_non_hashed_password,
};

#[post("/login", format = "application/json", data = "<user>")]
pub async fn login<'a>(
    conn: DbConn,
    user: Json<LoginUser>,
    cookies: &CookieJar<'a>,
    global_config: &State<GlobalConfig>,
) -> Result<Response<TokenResponse>, Error> {
    let user: LoginUser = user.into_inner();

    user.validate_model()?;

    let user = find(&conn, user.identifier.clone().unwrap())
        .await
        .map_err(|_| Error::Error(Status::Unauthorized))
        .and_then(|found_user| {
            let password_verify = verify_non_hashed_password(
                &found_user,
                user.password.clone().unwrap().as_ref(),
                &global_config.auth_secret_key,
            );

            match password_verify {
                false => Err(Error::unauthorized()),
                true => Ok(found_user),
            }
        })?;

    generate_and_store_refresh_token(
        &user,
        global_config.refresh_token_expiry,
        &global_config.auth_secret_key,
        cookies,
        &conn,
    )
    .await?;

    let response = add_token_response(
        UserType::StoredUser(&user),
        global_config.token_expiry,
        &global_config.auth_secret_key,
    );

    response
        .and_then(|(j, s)| Some(Ok(Response::success(Some(j), s))))
        .or_else(|| Some(Err(Error::Error(Status::Unauthorized))))
        .unwrap()
}
