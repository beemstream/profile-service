use rocket::http::{Status, Cookies, Cookie};
use rocket_contrib::json::Json;
use rocket_contrib::json;
use diesel::result::Error::DatabaseError;
use jsonwebtoken::{decode, encode};
use crate::models::user::{NewUser, LoginUser, Claims};
use crate::repository::user::{insert, find, get_by_username, get_by_email, has_found_user};
use crate::models::validator::Validator;
use crate::controllers::response::ApiResponse;
use crate::database::get_pooled_connection;
use crate::jwt::{validation, header};

#[post("/register", format="application/json", data="<user>")]
pub fn register_user(user: Json<NewUser>) -> ApiResponse {

    let validation_errors = user.parsed_field_errors();
    if validation_errors.len() > 0 {
        ApiResponse::new(
            json!({ "status": "NOT OK", "reason": "FIELD_ERRORS", "fields": validation_errors }),
            Status::BadRequest
        )
    } else {
        let mut user: NewUser = user.into_inner();
        user.hash_password();
        match insert(&user) {
            Ok(_v) => ApiResponse::new(json!({ "status": "OK" }), Status::Ok),
            Err(e) => {
                match e {
                    DatabaseError(_v, e) => {
                        ApiResponse::new(
                            json!({ "status": "NOT OK", "reason": String::from(e.message()) }),
                            Status::BadRequest
                            )
                    }
                    _ => {
                        ApiResponse::new(
                            json!({ "status": "ERROR", "reason": "SERVER_ERROR" }),
                            Status::InternalServerError
                            )
                    }
                }
            }
        }
    }

}

#[post("/login", format="application/json", data="<user>")]
pub fn login_user(user: Json<LoginUser>, mut cookies: Cookies) -> ApiResponse {
    let user: LoginUser = user.into_inner();

    let is_verified = match find(&user) {
        Ok(v) => v.verify(&user.password),
        _ => false
    };

    if is_verified {
        let key = std::env::var("ROCKET_secret_key").expect("secret_key must be set");
        let claims = Claims::new(&user.identifier);
        let header = header();
        let token = encode(&header, &claims, key.as_ref()).unwrap();
        let cookie = Cookie::build("token", token)
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
    let token = cookie.get_private("token");

    match token {
        Some(t) => {
            let token_str = &t.to_string();
            let parsed_token = token_str.split("=").nth(1).unwrap();
            match decode::<Claims>(parsed_token, key.as_ref(), &validation) {
                Ok(c) => {
                    let conn = &*get_pooled_connection();
                    let sub = &c.claims.sub().to_string();
                    let email = get_by_email(sub, conn);
                    let username = get_by_username(sub, conn);

                    if has_found_user(email) || has_found_user(username) {
                        ApiResponse::new(json!({ "status": "OK" }), Status::Ok)
                    } else {
                        ApiResponse::new(json!({ "status": "NOT OK" }), Status::Forbidden)
                    }
                }
                Err(_err) => ApiResponse::new(json!({ "status": "NOT OK" }), Status::Forbidden),
            }
        },
        None => ApiResponse::new(json!({ "status": "NOT OK" }), Status::Forbidden)
    }
}
