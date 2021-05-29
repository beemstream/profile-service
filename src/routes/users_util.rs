use crate::{
    database::DbConn,
    jwt::{generate_header, Claims},
    models::user::{NewRefreshToken, User, UserType},
    util::response::{ErrorResponse, ErrorType},
};
use crate::{
    repository::user::find,
    util::{globals::COOKIE_REFRESH_TOKEN_NAME, response::TokenResponse},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, TokenData, Validation};
use rocket::{
    http::{Cookie, CookieJar, Status},
    info,
};
use rocket_contrib::databases::diesel::result::{DatabaseErrorInformation, Error};

pub fn get_new_token(user_type: &UserType, duration: i64, secret_key: &str) -> (Claims, String) {
    let claims = match user_type {
        UserType::LoginUser(u) => Claims::new(&u.identifier.clone().unwrap(), duration),
        UserType::StoredUser(u) => Claims::new(&u.username, duration),
    };
    let encode_key = EncodingKey::from_secret(secret_key.as_ref());
    let new_token = encode(&generate_header(), &claims, &encode_key).unwrap();
    (claims, new_token)
}

pub fn get_exp_time(claims: &Claims) -> time::Duration {
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

pub fn verify_non_hashed_password(user: &User, password: &str, secret_key: &str) -> bool {
    user.verify(password, secret_key)
}

pub fn add_refresh_cookie<'a>(
    user: UserType<'a>,
    cookie: &CookieJar,
    claims: &Claims,
    refresh_token: &str,
    exp_time: i64,
) -> UserType<'a> {
    let refresh_exp = get_exp_time(&claims);
    cookie.add_private(get_cookie_with_expiry_and_max_age(
        refresh_exp,
        refresh_token.to_string(),
        exp_time,
    ));
    user
}

pub fn add_token_response(
    user: UserType<'_>,
    token_expiry: i64,
    secret_key: &str,
) -> Option<(TokenResponse, Status)> {
    let (claims, token) = get_new_token(&user, token_expiry, secret_key);
    let token_exp = get_exp_time(&claims);
    Some((
        TokenResponse::success(token, token_exp.whole_seconds()),
        Status::Ok,
    ))
}

pub fn get_jwt_claim<'a>(
    value: &'a str,
    secret_key: &str,
    validation: &Validation,
) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let decode_key = DecodingKey::from_secret(secret_key.as_ref());
    decode::<Claims>(value, &decode_key, validation)
}

pub fn verify_jwt(
    cookie: &Cookie,
    secret_key: &str,
    validation: &Validation,
) -> Option<TokenData<Claims>> {
    get_jwt_claim(cookie.value(), secret_key, validation).ok()
}

pub async fn verify_username(
    conn: &DbConn,
    token_data: TokenData<Claims>,
) -> Result<User, crate::util::response::Error> {
    find(&conn, token_data.claims.sub().to_owned())
        .await
        .map_err(|_| crate::util::response::Error::Error(Status::Unauthorized))
}

pub fn get_internal_error_response() -> crate::util::response::Error {
    crate::util::response::Error::Error(Status::InternalServerError)
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
        Error::DatabaseError(_, db_error) => {
            info!("db error {:?}", db_error);
            get_database_error_response(db_error)
        }
        _ => get_internal_error_response(),
    }
}

pub async fn generate_and_store_refresh_token<'a>(
    user: &User,
    refresh_token_expiry: i64,
    auth_secret_key: &str,
    cookie: &'a CookieJar<'a>,
    conn: &DbConn,
) -> Result<(), crate::util::response::Error> {
    let (refresh_claims, refresh_token) = get_new_token(
        &UserType::StoredUser(user),
        refresh_token_expiry,
        auth_secret_key,
    );
    add_refresh_cookie(
        UserType::StoredUser(user),
        cookie,
        &refresh_claims,
        &refresh_token,
        refresh_token_expiry,
    );

    let user_id = user.id;
    crate::repository::refresh_token::insert(
        conn,
        NewRefreshToken {
            user_id,
            token: refresh_token,
            expiry: chrono::Utc::now().naive_utc()
                + chrono::Duration::seconds(refresh_claims.exp as i64),
        },
    )
    .await?;
    Ok(())
}
