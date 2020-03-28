use crate::models::user::{NewUser, LoginUser, Claims};
use crate::repository::user::{insert, find};
use crate::models::validator::Validator;
use rocket::http::{Status, ContentType, Cookies, Cookie};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::json;
use diesel::result::Error::DatabaseError;
use jsonwebtoken::{decode, encode, Header, Validation};

#[derive(Debug)]
pub struct ApiResponse {
    pub json: JsonValue,
    pub status: Status,
}

impl ApiResponse {
    pub fn new(json: JsonValue, status: Status) -> ApiResponse {
        ApiResponse {
            json,
            status
        }
    }
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[post("/register", format="application/json", data="<user>")]
pub fn register_user(user: Json<NewUser>) -> ApiResponse {
    let mut user: NewUser = user.into_inner();
    user.hash_password();

    let validation_errors = user.parsed_field_errors();

    if validation_errors.len() > 0 {
        ApiResponse::new(
            json!({ "status": "NOT OK", "reason": "FIELD_ERRORS", "fields": validation_errors }),
            Status::BadRequest
        )
    } else {
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
        let claims = Claims::new();
        let token = encode(&Header::default(), &claims, key.as_ref()).unwrap();
        let cookie = Cookie::build("token", token)
            .max_age(chrono::Duration::minutes(30))
            .finish();
        cookies.add_private(cookie);

        ApiResponse::new(json!({ "status": "OK" }), Status::Ok)
    } else {
        ApiResponse::new(json!({ "status": "NOT OK" }), Status::Unauthorized)
    }
}

#[get("/authenticate")]
pub fn authenticate(mut cookie: Cookies) -> JsonValue {
    let key = std::env::var("ROCKET_secret_key").expect("secret_key must be set");
    let validation = Validation { iss: Some("beemstream".to_string()), sub: Some("normal_user@beemstream.com".to_string()), leeway: 2, ..Validation::default() };
    let token = cookie.get_private("token");

    match token {
        Some(t) => {
            let token_str = &t.to_string();
            let parsed_token = token_str.split("=").nth(1).unwrap();
            match decode::<Claims>(parsed_token, key.as_ref(), &validation) {
                Ok(_c) => json!({ "status": "OK" }),
                Err(_err) => json!({ "status": "NOT OK" }),
            }
        },
        None => json!({ "status": "NOT OK" })
    }
}
