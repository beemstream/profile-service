use rocket_contrib::json::JsonValue;
use rocket::response::{self, Responder, Response};
use rocket::http::{Status, ContentType};
use rocket::request::Request;

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


