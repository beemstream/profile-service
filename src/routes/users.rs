use super::users_util::{
    add_refresh_cookie, add_token_response, update_refresh_token_cache, verify_jwt,
    verify_non_hashed_password, verify_username,
};
use crate::util::{
    authorization::AccessToken,
    globals::COOKIE_REFRESH_TOKEN_NAME,
    response::{Error, Response, TokenResponse},
    validator::Validator,
};
use crate::{
    database::DbConn,
    repository::user::{find, insert},
    util::globals::{GlobalConfig, JWTConfig},
};
use crate::{
    models::user::{LoginUser, NewUser, NewUserRequest, UserType},
    repository::user::is_duplicate_user_or_email,
};
use futures::TryFutureExt;
use json::Json;
use rocket::{
    http::{CookieJar, Status},
    State,
};
use rocket_contrib::json;

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

#[post("/login", format = "application/json", data = "<user>")]
pub async fn login_user<'a>(
    conn: DbConn,
    user: Json<LoginUser>,
    cookies: &CookieJar<'a>,
    global_config: State<'a, GlobalConfig>,
) -> Result<Response<TokenResponse>, Error> {
    let user: LoginUser = user.into_inner();

    user.validate_model()?;

    let response = find(&conn, user.identifier.clone().unwrap().clone())
        .await
        .map_err(|_| Error::Error(Status::Unauthorized))
        .ok()
        .and_then(|found_user| {
            verify_non_hashed_password(
                found_user,
                user.password.clone().unwrap().as_ref(),
                &global_config.auth_secret_key,
            )
        })
        .and_then(|_| {
            add_refresh_cookie(
                UserType::LoginUser(&user),
                cookies,
                global_config.refresh_token_expiry,
                &global_config.auth_secret_key,
            )
        })
        .and_then(|_| {
            add_token_response(
                UserType::LoginUser(&user),
                global_config.token_expiry,
                &global_config.auth_secret_key,
            )
        });

    response
        .and_then(|(j, s)| Some(Ok(Response::success(Some(j), s))))
        .or_else(|| Some(Err(Error::Error(Status::Unauthorized))))
        .unwrap()
}

#[get("/refresh-token")]
pub async fn refresh_token<'a>(
    conn: DbConn,
    cookie: &'a CookieJar<'a>,
    _access_token: AccessToken,
    global_config: State<'a, GlobalConfig>,
    jwt_config: State<'a, JWTConfig>,
) -> Result<Response<TokenResponse>, Error> {
    let refresh_token = cookie.get_private(COOKIE_REFRESH_TOKEN_NAME);

    let token_data = match refresh_token
        .as_ref()
        .and_then(|t| verify_jwt(t, &global_config.auth_secret_key, &jwt_config.validation))
    {
        None => return Err(Error::error(None, Status::Unauthorized)),
        Some(v) => v,
    };

    let token_response = verify_username(&conn, token_data)
        .await
        .as_ref()
        .and_then(|user| update_refresh_token_cache(user))
        .and_then(|user| {
            add_refresh_cookie(
                UserType::StoredUser(&user),
                cookie,
                global_config.refresh_token_expiry,
                &global_config.auth_secret_key,
            )
        })
        .and_then(|user| {
            add_token_response(
                user,
                global_config.token_expiry,
                &global_config.auth_secret_key,
            )
        })
        .and_then(|(response, status)| Some(Response::success(Some(response), status)));

    token_response.ok_or(Error::Error(Status::Unauthorized))
}

#[get("/authenticate")]
pub fn authenticate(_access_token: AccessToken) -> Status {
    Status::Ok
}
