use rocket::http::{Status, Cookies, Cookie};
use rocket_contrib::json;
use diesel::result::Error::DatabaseError;
use jsonwebtoken::{decode, encode};
use crate::models::user::{NewUser, LoginUser, Claims};
use crate::repository::user::{insert, find};
use crate::{util::{validator::Validator, response::{JsonResponse, ApiResponse, AuthResponse, JsonStatus, StatusReason}}, jwt::{validation, header}};
use json::Json;

lazy_static!{
    static ref COOKIE_TOKEN_NAME: String = "access_token".to_string();
    static ref COOKIE_REFRESH_TOKEN_NAME: String = "refresh_token".to_string();
}

#[post("/register", format="application/json", data="<user>")]
pub fn register_user(user: Json<NewUser>) -> JsonResponse {

    let validation_errors = user.parsed_field_errors();

    match validation_errors {
        Some(errors) => {
            JsonResponse::new(
                AuthResponse::new(JsonStatus::NotOk, Some(StatusReason::FieldErrors), errors),
                Status::BadRequest)
        },
        None => {
            let mut user: NewUser = user.into_inner();
            user.hash_password();
            match insert(&user) {
                Ok(_v) => JsonResponse::new(AuthResponse::new(JsonStatus::Ok, None, vec![]), Status::Ok),
                Err(e) => {
                    match e {
                        DatabaseError(_v, e) => {
                            JsonResponse::new(
                                AuthResponse::new(
                                    JsonStatus::NotOk, Some(StatusReason::Other(String::from(e.message()))), vec![]),
                                Status::BadRequest)
                        }
                        _ => {
                            JsonResponse::new(
                                AuthResponse::new(JsonStatus::Error, Some(StatusReason::ServerError), vec![]),
                                Status::InternalServerError)
                        }
                    }
                }
            }
        }
    }
}

#[post("/login", format="application/json", data="<user>")]
pub fn login_user(user: Json<LoginUser>, mut cookies: Cookies) -> ApiResponse {
    let user: LoginUser = user.into_inner();

    let is_verified = match find(&user.identifier) {
        Ok(v) => v.verify(&user.password),
        _ => false
    };

    if is_verified {
        let key = std::env::var("ROCKET_secret_key").expect("secret_key must be set");
        let claims = Claims::new(&user.identifier);
        let header = header();
        let token = encode(&header, &claims, key.as_ref()).unwrap();
        let cookie = Cookie::build(COOKIE_TOKEN_NAME.as_str(), token)
            .max_age(chrono::Duration::minutes(30))
            .finish();
        cookies.add_private(cookie);

        ApiResponse::new(json!({ "status": "OK" }), Status::Ok)
    } else {
        ApiResponse::new(json!({ "status": "NOT OK", "reason": "Username/email or password is incorrect." }), Status::Unauthorized)
    }
}

#[get("/authenticate")]
pub fn authenticate(mut cookie: Cookies) -> ApiResponse {
    let key = std::env::var("ROCKET_secret_key").expect("secret_key must be set");
    let validation = validation();
    let token = cookie.get_private(COOKIE_TOKEN_NAME.as_str());

    match token {
        Some(t) => {
            let token_str = &t.to_string();
            let parsed_token = token_str.split("=").nth(1).unwrap();
            match decode::<Claims>(parsed_token, key.as_ref(), &validation) {
                Ok(c) => {
                    let sub = &c.claims.sub().to_string();
                    let identifier = find(sub);

                    match identifier {
                        Ok(_v) => ApiResponse::new(json!({ "status": "OK" }), Status::Ok),
                        Err(_e) => ApiResponse::new(json!({ "status": "NOT OK" }), Status::Forbidden)
                    }
                }
                Err(_err) => ApiResponse::new(json!({ "status": "NOT OK" }), Status::Forbidden),
            }
        },
        None => ApiResponse::new(json!({ "status": "NOT OK" }), Status::Forbidden)
    }
}
