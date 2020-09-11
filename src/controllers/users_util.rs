use jsonwebtoken::{encode, EncodingKey, DecodingKey, decode, TokenData};
use crate::{models::user::{UserType, Claims, User}, jwt::generate_header};
use crate::{util::{response::{JsonStatus, TokenResponse, AuthResponse, StatusReason, FieldError}, globals::{SECRET_KEY, VALIDATION, COOKIE_REFRESH_TOKEN_NAME, REFRESH_TOKEN_EXPIRY, TOKEN_EXPIRY}}, repository::user::find};
use rocket::http::{Status, Cookie, Cookies};
use diesel::result::{Error, DatabaseErrorInformation};

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

pub fn verify_non_hashed_password(user: User, password: &str) -> Option<bool> {
    bool_as_option(user.verify(password))
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
    Some((TokenResponse::success(JsonStatus::Ok, token, token_exp.num_seconds()), Status::Ok))
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

pub fn get_database_error_response(db_error: Box<dyn DatabaseErrorInformation + Send + Sync>) -> Option<(AuthResponse, Status)> {
    let db_error_message = String::from(db_error.message());
    let auth_response = AuthResponse::validation_error(StatusReason::Other(db_error_message), vec![]);
    Some((auth_response, Status::BadRequest))
}

pub fn get_auth_error_response(error: Error) -> Option<(AuthResponse, Status)> {
    match error {
        Error::DatabaseError(_, db_error) => get_database_error_response(db_error),
        _ => get_internal_error_response()
    }
}

pub fn update_refresh_token_cache(user: &User) -> Option<&User> {
    Some(user)
}
