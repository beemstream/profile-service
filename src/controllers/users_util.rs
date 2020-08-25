use jsonwebtoken::{encode, EncodingKey, DecodingKey, decode, TokenData};
use crate::{models::user::{UserType, Claims, User}, jwt::generate_header};
use crate::{util::{response::{JsonStatus, TokenResponse, AuthResponse, StatusReason, FieldError}, globals::{SECRET_KEY, VALIDATION, COOKIE_REFRESH_TOKEN_NAME, REFRESH_TOKEN_EXPIRY, TOKEN_EXPIRY}}, repository::user::find};
use rocket::http::{Status, Cookie, Cookies};
use diesel::result::{Error, DatabaseError, DatabaseErrorInformation};

pub fn get_new_token(user_type: &UserType, duration: i64) -> (Claims, String) {
    let claims = match user_type {
        UserType::LoginUser(u) => Claims::new(u.identifier, duration),
        UserType::StoredUser(u) => Claims::new(&u.username, duration),
    };
    let encode_key = EncodingKey::from_secret(&SECRET_KEY.as_ref());
    let new_token = encode(&generate_header(), &claims, &encode_key).unwrap();
    (claims, new_token)
}

pub fn get_exp_time(claims: Claims) -> chrono::Duration {
    let exp_datetime = chrono::NaiveDateTime::from_timestamp(claims.exp as i64, 0);
    let exp_utc_datetime = chrono::DateTime::<chrono::Utc>::from_utc(exp_datetime, chrono::Utc);
    exp_utc_datetime.signed_duration_since(chrono::Utc::now())
}

pub fn get_cookie_with_expiry_and_max_age<'a>(exp_time: chrono::Duration, refresh_token: String) -> Cookie<'a> {
    Cookie::build(COOKIE_REFRESH_TOKEN_NAME, refresh_token)
            .max_age(exp_time)
            .expires(time::now_utc() + chrono::Duration::seconds(*REFRESH_TOKEN_EXPIRY))
            .finish()
}

pub fn add_refresh_cookie<'a>(user: UserType<'a>, mut cookie: Cookies) -> Option<UserType<'a>> {
    let (refresh_claims, refresh_token) = get_new_token(&user, *REFRESH_TOKEN_EXPIRY);
    let refresh_exp = get_exp_time(refresh_claims);
    cookie.add_private(get_cookie_with_expiry_and_max_age(refresh_exp, refresh_token));
    Some(user)
}

pub fn add_token_response<'a>(user: UserType<'a>) -> Option<(TokenResponse, Status)> {
    let (claims, token) = get_new_token(&user, *TOKEN_EXPIRY);
    let token_exp = get_exp_time(claims);
    Some((TokenResponse::new(JsonStatus::Ok, Some(token), Some(token_exp.num_seconds()), None), Status::Ok))
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
        false => None
    }
}

pub fn get_success_json_response() -> Option<(AuthResponse, Status)> {
    let auth_response: AuthResponse = AuthResponse::new(JsonStatus::Ok, None, None);
    Some((auth_response, Status::Ok))
}

pub fn get_internal_json_response() -> Option<(AuthResponse, Status)> {
    let auth_response = AuthResponse::new(JsonStatus::Error, Some(StatusReason::ServerError), None);
    let status = Status::InternalServerError;
    Some((auth_response, status))
}

pub fn get_validation_errors(errors: Vec<FieldError>) -> Option<(AuthResponse, Status)> {
    let auth_response = AuthResponse::new(JsonStatus::NotOk, Some(StatusReason::FieldErrors), Some(errors));
    let status = Status::BadRequest;
    Some((auth_response, status))
}

pub fn get_database_error_response(db_error: Box<dyn DatabaseErrorInformation + Send + Sync>) -> Option<(AuthResponse, Status)> {
    let auth_response = AuthResponse::new(JsonStatus::NotOk, Some(StatusReason::Other(String::from(db_error.message()))), None);
    let status = Status::BadRequest;
    Some((auth_response, status))
}

pub fn get_auth_error_response(error: Error) -> Option<(AuthResponse, Status)> {
    match error {
        DatabaseError(_, db_error) => get_database_error_response(db_error),
        _ => get_internal_json_response()
    }
}
