use rocket_contrib::json::{Json, JsonValue};
use rocket::response::{self, Responder, Response};
use rocket::http::{Status, ContentType};
use rocket::request::Request;
use serde::Serialize;

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

#[derive(Serialize)]
pub enum JsonStatus {
    #[serde(rename(serialize = "ok"))]
    Ok,
    #[serde(rename(serialize = "not ok"))]
    NotOk,
    #[serde(rename(serialize = "error"))]
    Error
}

#[derive(Serialize)]
pub enum StatusReason {
    #[serde(rename(serialize = "FIELD_ERRORS"))]
    FieldErrors,
    #[serde(rename(serialize = "SERVER_ERROR"))]
    ServerError,
    Other(String)
}

#[derive(Serialize)]
pub struct FieldError {
    name: String,
    message: Vec<String>
}

impl FieldError {
    pub fn new(name: String, message: Vec<String>) -> Self {
        Self {
            name,
            message
        }
    }
}

#[derive(Serialize)]
pub struct AuthResponse {
    status: JsonStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<StatusReason>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<FieldError>
}

impl AuthResponse {
    pub fn new(status: JsonStatus, reason: Option<StatusReason>, fields: Vec<FieldError>) -> Self {
        Self {
            status,
            reason,
            fields
        }
    }
}

pub struct JsonResponse {
    pub json: Json<AuthResponse>,
    pub status: Status,
}

impl JsonResponse {
    pub fn new(json: AuthResponse, status: Status) -> JsonResponse {
        JsonResponse {
            json: Json(json),
            status
        }
    }
}

impl<'r> Responder<'r> for JsonResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

