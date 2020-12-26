use crate::{
    jwt::generate_header,
    models::user::{Claims, User, UserType},
};
use crate::{
    repository::user::find,
    util::{
        globals::{
            COOKIE_REFRESH_TOKEN_NAME, REFRESH_TOKEN_EXPIRY, SECRET_KEY, TOKEN_EXPIRY, VALIDATION,
        },
        response::{AuthResponse, FieldError, JsonStatus, StatusReason, TokenResponse},
    },
};
use diesel::result::{DatabaseErrorInformation, Error};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, TokenData};
use rocket::http::{Cookie, CookieJar, Status};

pub fn get_new_token(user_type: &UserType, duration: i64) -> (Claims, String) {
    let claims = match user_type {
        UserType::LoginUser(u) => Claims::new(u.identifier, duration),
        UserType::StoredUser(u) => Claims::new(&u.username, duration),
    };
    let encode_key = EncodingKey::from_secret(&SECRET_KEY.as_ref());
    let new_token = encode(&generate_header(), &claims, &encode_key).unwrap();
    (claims, new_token)
}

pub fn get_exp_time(claims: Claims) -> time::Duration {
    let exp_datetime = time::OffsetDateTime::from_unix_timestamp(claims.exp as i64);
    let time_now = time::OffsetDateTime::now_utc();

    exp_datetime - time_now
}

pub fn get_cookie_with_expiry_and_max_age<'a>(
    exp_time: time::Duration,
    refresh_token: String,
) -> Cookie<'a> {
    Cookie::build(COOKIE_REFRESH_TOKEN_NAME, refresh_token)
        .max_age(exp_time)
        .expires(time::OffsetDateTime::now_utc() + time::Duration::seconds(*REFRESH_TOKEN_EXPIRY))
        .secure(true)
        .http_only(true)
        .finish()
}

pub fn verify_non_hashed_password(user: User, password: &str) -> Option<bool> {
    bool_as_option(user.verify(password))
}

pub fn add_refresh_cookie<'a>(user: UserType<'a>, cookie: &CookieJar) -> Option<UserType<'a>> {
    let (refresh_claims, refresh_token) = get_new_token(&user, *REFRESH_TOKEN_EXPIRY);
    let refresh_exp = get_exp_time(refresh_claims);
    cookie.add_private(get_cookie_with_expiry_and_max_age(
        refresh_exp,
        refresh_token,
    ));
    Some(user)
}

pub fn add_token_response<'a>(user: UserType<'a>) -> Option<(TokenResponse, Status)> {
    let (claims, token) = get_new_token(&user, *TOKEN_EXPIRY);
    let token_exp = get_exp_time(claims);
    Some((
        TokenResponse::success(JsonStatus::Ok, token, token_exp.whole_seconds()),
        Status::Ok,
    ))
}

pub fn verify_jwt(cookie: &Cookie) -> Option<TokenData<Claims>> {
    let decode_key = DecodingKey::from_secret(&SECRET_KEY.as_ref());
    decode::<Claims>(cookie.value(), &decode_key, &VALIDATION).ok()
}

pub fn verify_username(token_data: TokenData<Claims>) -> Option<User> {
    find(token_data.claims.sub()).ok()
}

pub fn bool_as_option(is_verified: bool) -> Option<bool> {
    match is_verified {
        true => Some(true),
        false => None,
    }
}

pub fn get_success_json_response() -> Option<(AuthResponse, Status)> {
    let auth_response: AuthResponse = AuthResponse::success();
    Some((auth_response, Status::Ok))
}

pub fn get_internal_error_response() -> Option<(AuthResponse, Status)> {
    let auth_response = AuthResponse::internal_error(StatusReason::ServerError);
    Some((auth_response, Status::InternalServerError))
}

pub fn get_validation_errors_response(errors: Vec<FieldError>) -> Option<(AuthResponse, Status)> {
    let auth_response = AuthResponse::validation_error(StatusReason::FieldErrors, errors);
    Some((auth_response, Status::BadRequest))
}

pub fn get_database_error_response(
    db_error: Box<dyn DatabaseErrorInformation + Send + Sync>,
) -> Option<(AuthResponse, Status)> {
    let db_error_message = String::from(db_error.message());
    let auth_response =
        AuthResponse::validation_error(StatusReason::Other(db_error_message), vec![]);
    Some((auth_response, Status::BadRequest))
}

pub fn get_auth_error_response(error: Error) -> Option<(AuthResponse, Status)> {
    match error {
        Error::DatabaseError(_, db_error) => get_database_error_response(db_error),
        _ => get_internal_error_response(),
    }
}

pub fn update_refresh_token_cache(user: &User) -> Option<&User> {
    Some(user)
}
