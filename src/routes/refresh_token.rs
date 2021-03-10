use rocket::{
    http::{CookieJar, Status},
    State,
};

use crate::{
    database::DbConn,
    models::user::UserType,
    util::{
        authorization::AccessToken,
        globals::{GlobalConfig, JWTConfig, COOKIE_REFRESH_TOKEN_NAME},
        response::{Error, Response, TokenResponse},
    },
};

use super::users_util::{
    add_token_response, generate_and_store_refresh_token, verify_jwt, verify_username,
};

#[get("/refresh-token")]
pub async fn refresh_token<'a>(
    conn: DbConn,
    cookie: &'a CookieJar<'a>,
    _access_token: AccessToken,
    global_config: State<'a, GlobalConfig>,
    jwt_config: State<'a, JWTConfig>,
) -> Result<Response<TokenResponse>, Error> {
    let refresh_token = cookie.get_private(COOKIE_REFRESH_TOKEN_NAME);

    let cookie_data = refresh_token.as_ref();

    let token_data = match cookie_data {
        None => return Err(Error::error(None, Status::Unauthorized)),
        Some(v) => v,
    };

    info!("found token in cookie");

    let found_token =
        crate::repository::refresh_token::find_by_token(&conn, token_data.value().to_owned())
            .await?;

    info!("found token {:?}", found_token);

    let deleted_token = crate::repository::refresh_token::delete(&conn, found_token.id).await?;

    info!("deleted token {:?}", deleted_token);

    let verified_token = match verify_jwt(
        token_data,
        &global_config.auth_secret_key,
        &jwt_config.validation,
    ) {
        Some(v) => v,
        None => return Err(Error::Error(Status::Unauthorized)),
    };

    let user = verify_username(&conn, verified_token).await?;

    generate_and_store_refresh_token(
        &user,
        global_config.refresh_token_expiry,
        &global_config.auth_secret_key,
        cookie,
        &conn,
    )
    .await?;

    let token_response = add_token_response(
        UserType::StoredUser(&user),
        global_config.token_expiry,
        &global_config.auth_secret_key,
    )
    .and_then(|(response, status)| Some(Response::success(Some(response), status)));

    token_response.ok_or(Error::Error(Status::Unauthorized))
}
