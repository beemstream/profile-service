use crate::{
    database::DbConn,
    jwt::generate_header,
    models::user::{Claims, User, UserType},
    util::response::{ErrorResponse, ErrorType},
};
use crate::{
    repository::user::find,
    util::{globals::COOKIE_REFRESH_TOKEN_NAME, response::TokenResponse},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, TokenData, Validation};
use rocket::http::{Cookie, CookieJar, Status};
use rocket_contrib::databases::diesel::result::{DatabaseErrorInformation, Error};

pub fn get_new_token(user_type: &UserType, duration: i64, secret_key: &String) -> (Claims, String) {
    let claims = match user_type {
        UserType::LoginUser(u) => Claims::new(u.identifier.clone().unwrap().as_ref(), duration),
        UserType::StoredUser(u) => Claims::new(&u.username, duration),
    };
    let encode_key = EncodingKey::from_secret(secret_key.as_ref());
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
    refresh_token_expiry: i64,
) -> Cookie<'a> {
    Cookie::build(COOKIE_REFRESH_TOKEN_NAME, refresh_token)
        .max_age(exp_time)
        .expires(time::OffsetDateTime::now_utc() + time::Duration::seconds(refresh_token_expiry))
        .secure(true)
        .http_only(true)
        .finish()
}

pub fn verify_non_hashed_password(user: User, password: &str, secret_key: &String) -> Option<bool> {
    bool_as_option(user.verify(password, secret_key))
}

pub fn add_refresh_cookie<'a>(
    user: UserType<'a>,
    cookie: &CookieJar,
    refresh_token_expiry: i64,
    secret_key: &String,
) -> Option<UserType<'a>> {
    let (refresh_claims, refresh_token) = get_new_token(&user, refresh_token_expiry, secret_key);
    let refresh_exp = get_exp_time(refresh_claims);
    cookie.add_private(get_cookie_with_expiry_and_max_age(
        refresh_exp,
        refresh_token,
        refresh_token_expiry,
    ));
    Some(user)
}

pub fn add_token_response<'a>(
    user: UserType<'a>,
    token_expiry: i64,
    secret_key: &String,
) -> Option<(TokenResponse, Status)> {
    let (claims, token) = get_new_token(&user, token_expiry, secret_key);
    let token_exp = get_exp_time(claims);
    Some((
        TokenResponse::success(token, token_exp.whole_seconds()),
        Status::Ok,
    ))
}

pub fn verify_jwt(
    cookie: &Cookie,
    secret_key: &String,
    validation: &Validation,
) -> Option<TokenData<Claims>> {
    let decode_key = DecodingKey::from_secret(secret_key.as_ref());
    decode::<Claims>(cookie.value(), &decode_key, validation).ok()
}

pub async fn verify_username(conn: &DbConn, token_data: TokenData<Claims>) -> Option<User> {
    find(&conn, token_data.claims.sub().to_owned()).await.ok()
}

pub fn bool_as_option(is_verified: bool) -> Option<bool> {
    match is_verified {
        true => Some(true),
        false => None,
    }
}

pub fn get_internal_error_response() -> crate::util::response::Error {
    crate::util::response::Error::Error(Status::InternalServerError)
}

pub fn get_unprocessable_entity_error(error_codes: Vec<String>) -> Option<(ErrorResponse, Status)> {
    let auth_response = ErrorResponse {
        error_type: Some(ErrorType::RequestInvalid),
        error_codes: Some(error_codes),
    };
    Some((auth_response, Status::UnprocessableEntity))
}

pub fn get_database_error_response(
    db_error: Box<dyn DatabaseErrorInformation + Send + Sync>,
) -> crate::util::response::Error {
    let db_error_message = String::from(db_error.message());
    let auth_response = ErrorResponse {
        error_type: Some(ErrorType::RequestInvalid),
        error_codes: Some(vec![db_error_message]),
    };
    crate::util::response::Error::error_with_body(auth_response, Status::Conflict)
}

pub fn get_auth_error_response(error: Error) -> crate::util::response::Error {
    match error {
        Error::DatabaseError(_, db_error) => get_database_error_response(db_error),
        _ => get_internal_error_response(),
    }
}

pub fn update_refresh_token_cache(user: &User) -> Option<&User> {
    Some(user)
}
