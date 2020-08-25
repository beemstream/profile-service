use rocket::http::{Status, Cookies};
use rocket_contrib::json;
use crate::models::user::{NewUser, LoginUser, NewUserRequest, UserType};
use crate::repository::user::{insert, find};
use crate::util::{validator::Validator, response::{JsonResponse, AuthResponse, JsonStatus, TokenResponse}, authorization::AccessToken, globals::COOKIE_REFRESH_TOKEN_NAME};
use json::Json;
use super::users_util::{add_refresh_cookie, bool_as_option, add_token_response, verify_jwt, verify_username, get_validation_errors, get_success_json_response, get_auth_error_response};

#[post("/register", format="application/json", data="<user>")]
pub fn register_user(user: Json<NewUserRequest>) -> JsonResponse<AuthResponse> {
    let user_request = user.into_inner();
    let validation_errors = user_request.parsed_field_errors();

    let respond_with_success = || get_success_json_response();

    let respond_with_validation_errors = validation_errors
        .and_then(|errors| get_validation_errors(errors));

    let try_register_user = || Some(NewUser::from(user_request))
        .as_mut()
        .and_then(|user| Some(user.hash_password()))
        .and_then(|user| insert(&user).err())
        .and_then(|err| get_auth_error_response(err));

    let (auth_response, status) = respond_with_validation_errors
        .or_else(try_register_user)
        .or_else(respond_with_success).unwrap();

    JsonResponse::new(auth_response, status)
}

#[post("/login", format="application/json", data="<user>")]
pub fn login_user(user: Json<LoginUser>, cookies: Cookies) -> JsonResponse<TokenResponse> {
    let user: LoginUser = user.into_inner();

    let error_response = || Some((
            TokenResponse::new(JsonStatus::NotOk, None, None, Some("Username/email or password is incorrect.".to_string())),
            Status::Unauthorized
    ));

    let token_response = find(&user.identifier).ok()
        .and_then(|u| bool_as_option(u.verify(user.password)))
        .and_then(|_| add_refresh_cookie(UserType::LoginUser(&user), cookies))
        .and_then(|_| add_token_response(UserType::LoginUser(&user)));

    let (response, status) = token_response.or_else(error_response).unwrap();
    JsonResponse::new(response, status)
}

#[get("/authenticate")]
pub fn authenticate(_access_token: AccessToken) -> Status {
    Status::Ok
}

#[get("/refresh-token")]
pub fn refresh_token(mut cookie: Cookies, _access_token: AccessToken) -> JsonResponse<TokenResponse> {
    let refresh_token = cookie.get_private(COOKIE_REFRESH_TOKEN_NAME);

    let error_response = Some((
            TokenResponse::new(JsonStatus::NotOk, None, None, None),
            Status::Unauthorized
    ));
    let token_response = refresh_token.as_ref()
        .and_then(|t| verify_jwt(t))
        .and_then(|token_data| verify_username(token_data))
        .as_ref()
        .and_then(|user| add_refresh_cookie(UserType::StoredUser(&user), cookie))
        .and_then(|user| add_token_response(user));

    let (response, status) = token_response.or_else(|| error_response).unwrap();
    JsonResponse::new(response, status)
}
