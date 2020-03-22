use crate::models::user::NewUser;
use crate::repository::user::insert;
use rocket::http::{Status, ContentType};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::json;
use diesel::result::Error::DatabaseError;

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
        println!("there are validation errors");
        ApiResponse::new(
            json!({ "status": "not ok", "reason": "FIELD_ERRORS", "fields": validation_errors }),
            Status::BadRequest
        )
    } else {
        match insert(user) {
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
                            json!({ "status": "error", "reason": "SERVER_ERROR" }),
                            Status::InternalServerError
                        )
                    }
                }
            }
        }
    }

}

// #[post("/authenticate", format="json", data="<user>")]
// pub fn authenticate(user: Json<User>) -> Json<User> {
//     let user: User = user.into_inner();

//     Json(get(2).unwrap())
//     match verify(user.password) {
//         Ok => Json(insert(user)),
//         _ => Json()
//     }
// }
