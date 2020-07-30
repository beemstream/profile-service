use rocket::http::{Status, Cookies, Cookie};
use rocket_contrib::json;
use diesel::result::Error::DatabaseError;
use jsonwebtoken::{decode, encode, TokenData, DecodingKey, EncodingKey};
use crate::models::user::{NewUser, LoginUser, Claims, User, NewUserRequest};
use crate::repository::user::{insert, find};
use crate::{util::{validator::Validator, response::{JsonResponse, AuthResponse, JsonStatus, StatusReason, TokenResponse}, authorization::AccessToken, globals::{SECRET_KEY, VALIDATION, COOKIE_REFRESH_TOKEN_NAME, REFRESH_TOKEN_EXPIRY, TOKEN_EXPIRY}}, jwt::generate_header};
use json::Json;


#[post("/register", format="application/json", data="<user>")]
pub fn register_user(user: Json<NewUserRequest>) -> JsonResponse<AuthResponse> {
    let user_request = user.into_inner();
    let validation_errors = user_request.parsed_field_errors();
    let mut auth_response: AuthResponse = AuthResponse::new(JsonStatus::Ok, None, None);
    let mut status: Status = Status::Ok;

    match validation_errors {
        Some(errors) => {
            auth_response = AuthResponse::new(JsonStatus::NotOk, Some(StatusReason::FieldErrors), Some(errors));
            status = Status::BadRequest;
        },
        None => {
            let mut new_user = NewUser::from(user_request);
            new_user.hash_password();
            if let Err(e) = insert(&new_user) {
                if let DatabaseError(_v, db_error) = e {
                    auth_response = AuthResponse::new(JsonStatus::NotOk, Some(StatusReason::Other(String::from(db_error.message()))), None);
                    status = Status::BadRequest;
                } else {
                    auth_response = AuthResponse::new(JsonStatus::Error, Some(StatusReason::ServerError), None);
                    status = Status::InternalServerError;
                }
            }
        }
    }
    JsonResponse::new(auth_response, status)
}

#[post("/login", format="application/json", data="<user>")]
pub fn login_user(user: Json<LoginUser>, cookies: Cookies) -> JsonResponse<TokenResponse> {
    let user: LoginUser = user.into_inner();

    let error_response = Some((
            TokenResponse::new(JsonStatus::NotOk, None, None, Some("Username/email or password is incorrect.".to_string())),
            Status::Unauthorized
    ));

    let token_response = find(&user.identifier).ok()
        .and_then(|u| bool_as_option(u.verify(user.password)))
        .and_then(|_| add_refresh_cookie(UserType::LoginUser(&user), cookies))
        .and_then(|_| add_token_response(UserType::LoginUser(&user)));

    let (response, status) = token_response.or_else(|| error_response).unwrap();
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

enum UserType<'a> {
    LoginUser(&'a LoginUser<'a>),
    StoredUser(&'a User)
}

fn get_new_token(user_type: &UserType, duration: i64) -> (Claims, String) {
    let claims = match user_type {
        UserType::LoginUser(u) => Claims::new(u.identifier, duration),
        UserType::StoredUser(u) => Claims::new(&u.username, duration),
    };
    let encode_key = EncodingKey::from_secret(&SECRET_KEY.as_ref());
    let new_token = encode(&generate_header(), &claims, &encode_key).unwrap();
    (claims, new_token)
}

fn get_exp_time(claims: Claims) -> chrono::Duration {
    let exp_datetime = chrono::NaiveDateTime::from_timestamp(claims.exp as i64, 0);
    let exp_utc_datetime = chrono::DateTime::<chrono::Utc>::from_utc(exp_datetime, chrono::Utc);
    exp_utc_datetime.signed_duration_since(chrono::Utc::now())
}

fn get_cookie_with_expiry_and_max_age<'a>(exp_time: chrono::Duration, refresh_token: String) -> Cookie<'a> {
    Cookie::build(COOKIE_REFRESH_TOKEN_NAME, refresh_token)
            .max_age(exp_time)
            .expires(time::now_utc() + chrono::Duration::seconds(*REFRESH_TOKEN_EXPIRY))
            .finish()
}

fn add_refresh_cookie<'a>(user: UserType<'a>, mut cookie: Cookies) -> Option<UserType<'a>> {
    let (refresh_claims, refresh_token) = get_new_token(&user, *REFRESH_TOKEN_EXPIRY);
    let refresh_exp = get_exp_time(refresh_claims);
    cookie.add_private(get_cookie_with_expiry_and_max_age(refresh_exp, refresh_token));
    Some(user)
}

fn add_token_response<'a>(user: UserType<'a>) -> Option<(TokenResponse, Status)> {
    println!("TOKEN EXPIRY {}", *TOKEN_EXPIRY);
    let (claims, token) = get_new_token(&user, *TOKEN_EXPIRY);
    let token_exp = get_exp_time(claims);
    Some((TokenResponse::new(JsonStatus::Ok, Some(token), Some(token_exp.num_seconds()), None), Status::Ok))
}

fn verify_jwt(cookie: &Cookie) -> Option<TokenData<Claims>> {
    let decode_key = DecodingKey::from_secret(&SECRET_KEY.as_ref());
    decode::<Claims>(cookie.value(), &decode_key, &VALIDATION).ok()
}

fn verify_username(token_data: TokenData<Claims>) -> Option<User> {
    find(token_data.claims.sub()).ok()
}

fn bool_as_option(is_verified: bool) -> Option<bool> {
    match is_verified {
        true => Some(true),
        false => None
    }
}

