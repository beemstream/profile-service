use rocket_contrib::json::Json;
use rocket::response::{self, Responder, Response};
use rocket::http::{Status, ContentType};
use rocket::request::Request;
use serde::{Serialize, Deserialize};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<FieldError>>
}

impl AuthResponse {
    pub fn success() -> Self {
        Self {
            status: JsonStatus::Ok,
            reason: None,
            fields: None
        }
    }

    pub fn validation_error(reason: StatusReason, fields: Vec<FieldError>) -> Self {
        Self {
            status: JsonStatus::NotOk,
            reason: Some(reason),
            fields: Some(fields)
        }
    }

    pub fn internal_error(reason: StatusReason) -> Self {
        Self {
            status: JsonStatus::Error,
            reason: Some(reason),
            fields: None
        }
    }
}

pub struct JsonResponse<T> {
    pub json: Json<T>,
    pub status: Status,
}

impl<T> JsonResponse<T> {
    pub fn new(json: T, status: Status) -> JsonResponse<T> {
        JsonResponse {
            json: Json(json),
            status
        }
    }
}

impl<'r, T: serde::Serialize> Responder<'r, 'static> for JsonResponse<T> {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.json.respond_to(&request).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()

    }
}

#[derive(Serialize)]
pub struct TokenResponse {
    status: JsonStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>
}

impl TokenResponse {
    pub fn success(status: JsonStatus, access_token: String, expires_in: i64) -> Self {
        Self {
            status,
            access_token: Some(access_token),
            expires_in: Some(expires_in),
            reason: None
        }
    }

    pub fn error(status: JsonStatus, reason: String) -> Self {
        Self {
            status,
            reason: Some(reason),
            expires_in: None,
            access_token: None
        }
    }
}

#[derive(Deserialize)]
pub struct TokenRequest<'a> {
    #[allow(dead_code)]
    access_token: &'a str
}
