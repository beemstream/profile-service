use super::users_util::{
    add_refresh_cookie, add_token_response, get_auth_error_response, get_success_json_response,
    get_validation_errors_response, update_refresh_token_cache, verify_jwt,
    verify_non_hashed_password, verify_username,
};
use crate::util::{
    authorization::AccessToken,
    globals::COOKIE_REFRESH_TOKEN_NAME,
    response::{AuthResponse, JsonResponse, JsonStatus, TokenResponse},
    validator::Validator,
};
use crate::{
    database::DbConn,
    repository::user::{find, insert},
};
use crate::{
    models::user::{LoginUser, NewUser, NewUserRequest, UserType},
    repository::user::is_duplicate_user_or_email,
};
use futures::TryFutureExt;
use json::Json;
use rocket::{
    http::{CookieJar, Status},
    tokio::time::Instant,
};
use rocket_contrib::json;

#[post("/register", format = "application/json", data = "<user>")]
pub async fn register_user(conn: DbConn, user: Json<NewUserRequest>) -> JsonResponse<AuthResponse> {
    let user_request = user.into_inner();
    let validation_errors = user_request.parsed_field_errors();

    let respond_with_validation_errors =
        validation_errors.and_then(|errors| get_validation_errors_response(errors));

    let saved_user = is_duplicate_user_or_email(&conn, user_request)
        .and_then(|user| {
            let mut new_user = NewUser::from(user);
            let before = Instant::now();
            new_user.hash_password();
            let after = Instant::now() - before;
            println!("hashing took {:?}", after);
            insert(&conn, new_user)
        })
        .await
        .or_else(|e| Err(get_auth_error_response(e).unwrap()));

    let (auth_response, status) = respond_with_validation_errors
        .or_else(|| saved_user.err())
        .or_else(|| get_success_json_response())
        .unwrap();

    JsonResponse::new(auth_response, status)
}

#[post("/login", format = "application/json", data = "<user>")]
pub async fn login_user<'a>(
    conn: DbConn,
    user: Json<LoginUser>,
    cookies: &CookieJar<'a>,
) -> JsonResponse<TokenResponse> {
    let user: LoginUser = user.into_inner();

    let error_response = || {
        Some((
            TokenResponse::error(
                JsonStatus::NotOk,
                "Username/email or password is incorrect.".to_string(),
            ),
            Status::Unauthorized,
        ))
    };

    let token_response = find(&conn, user.identifier.clone())
        .await
        .ok()
        .and_then(|found_user| {
            let before = Instant::now();
            let v = verify_non_hashed_password(found_user, user.password.as_ref());
            let after = Instant::now() - before;
            println!("verify took {:?}", after);
            v
        })
        .and_then(|_| add_refresh_cookie(UserType::LoginUser(&user), cookies))
        .and_then(|_| add_token_response(UserType::LoginUser(&user)));

    let (response, status) = token_response.or_else(error_response).unwrap();
    JsonResponse::new(response, status)
}

#[get("/refresh-token")]
pub async fn refresh_token<'a>(
    conn: DbConn,
    cookie: &'a CookieJar<'a>,
    _access_token: AccessToken,
) -> JsonResponse<TokenResponse> {
    let refresh_token = cookie.get_private(COOKIE_REFRESH_TOKEN_NAME);

    let error_response = Some((
        TokenResponse::error(JsonStatus::NotOk, "Unauthorized".to_string()),
        Status::Unauthorized,
    ));

    let token_data = refresh_token.as_ref().and_then(|t| verify_jwt(t)).unwrap();

    let token_response = verify_username(&conn, token_data)
        .await
        .as_ref()
        .and_then(|user| update_refresh_token_cache(user))
        .and_then(|user| add_refresh_cookie(UserType::StoredUser(&user), cookie))
        .and_then(|user| add_token_response(user));

    let (response, status) = token_response.or_else(|| error_response).unwrap();
    JsonResponse::new(response, status)
}

#[get("/authenticate")]
pub fn authenticate(_access_token: AccessToken) -> Status {
    Status::Ok
}
