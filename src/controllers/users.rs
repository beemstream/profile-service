use rocket::http::{Status, Cookies, Cookie};
use rocket_contrib::json;
use diesel::result::Error::DatabaseError;
use jsonwebtoken::{decode, encode};
use crate::models::user::{NewUser, LoginUser, Claims};
use crate::repository::user::{insert, find};
use crate::{util::{validator::Validator, response::{JsonResponse, ApiResponse, AuthResponse, JsonStatus, StatusReason, TokenResponse}, authorization::AccessToken, globals::SECRET_KEY}, jwt::{jwt_validation, generate_header}};
use json::Json;

const COOKIE_REFRESH_TOKEN_NAME: &str = "refresh_token";
const TOKEN_EXPIRY: i64 = 120;
const REFRESH_TOKEN_EXPIRY: i64 = 60 * 60 * 24 * 3;

#[post("/register", format="application/json", data="<user>")]
pub fn register_user(user: Json<NewUser>) -> JsonResponse<AuthResponse> {
    let validation_errors = user.parsed_field_errors();
    let mut auth_response: AuthResponse = AuthResponse::new(JsonStatus::Ok, None, None);
    let mut status: Status = Status::Ok;
    match validation_errors {
        Some(errors) => {
            auth_response = AuthResponse::new(JsonStatus::NotOk, Some(StatusReason::FieldErrors), Some(errors));
            status = Status::BadRequest;
        },
        None => {
            let mut user: NewUser = user.into_inner();
            user.hash_password();
            if let Err(e) = insert(&user) {
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
pub fn login_user(user: Json<LoginUser>, mut cookies: Cookies) -> JsonResponse<TokenResponse> {
    let user: LoginUser = user.into_inner();

    let is_verified = match find(&user.identifier) {
        Ok(v) => v.verify(user.password),
        _ => false
    };

    let response: TokenResponse;
    let mut status = Status::Ok;

    if is_verified {
        let token_claims = Claims::new(&user.identifier, TOKEN_EXPIRY);
        let refresh_claims = Claims::new(&user.identifier, REFRESH_TOKEN_EXPIRY);
        let header = generate_header();

        let token = encode(&header, &token_claims, &*SECRET_KEY.as_ref()).unwrap();
        let refresh_token = encode(&header, &refresh_claims, &*SECRET_KEY.as_ref()).unwrap();

        let exp_datetime = chrono::NaiveDateTime::from_timestamp(refresh_claims.exp as i64, 0);
        let exp_utc_datetime = chrono::DateTime::<chrono::Utc>::from_utc(exp_datetime, chrono::Utc);
        let exp_time = exp_utc_datetime.signed_duration_since(chrono::Utc::now());
        let cookie = cookie_with_expiry_and_max_age(exp_time, refresh_token);
        cookies.add_private(cookie);

        response = TokenResponse::new(JsonStatus::Ok, Some(token), Some(exp_time.num_seconds()), None);
    } else {
        response = TokenResponse::new(JsonStatus::NotOk, None, None, Some("Username/email or password is incorrect.".to_string()));
        status = Status::Unauthorized;
    }

    JsonResponse::new(response, status)
}

fn cookie_with_expiry_and_max_age<'a>(exp_time: chrono::Duration, refresh_token: String) -> Cookie<'a> {
    Cookie::build(COOKIE_REFRESH_TOKEN_NAME, refresh_token)
            .max_age(exp_time)
            .expires(time::now_utc() + chrono::Duration::seconds(REFRESH_TOKEN_EXPIRY))
            .finish()
}

#[get("/authenticate")]
pub fn authenticate(_access_token: AccessToken) -> ApiResponse {
    ApiResponse::new(json!({ "status": "Ok" }), Status::Ok)
}
