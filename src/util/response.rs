use rocket::{http::{ContentType, Status}, serde::json::Json};
use rocket::request::Request;
use rocket::response::Responder;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AuthResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    error_codes: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    RequestInvalid,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<ErrorType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_codes: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum Response<T>
where
    T: Serialize,
{
    Success(Status),
    SuccessWithBody(JsonResponse<T>),
}

#[derive(Debug)]
pub enum Error {
    ErrorWithBody(JsonResponse<ErrorResponse>),
    Error(Status),
}

impl Error {
    pub fn error(error: Option<(Vec<String>, ErrorType)>, status: Status) -> Self {
        match error {
            Some(e) => {
                let body = ErrorResponse {
                    error_codes: Some(e.0),
                    error_type: Some(e.1),
                };
                Self::ErrorWithBody(JsonResponse::new(body, status))
            }
            None => Self::Error(status),
        }
    }

    pub fn error_with_body(json: ErrorResponse, status: Status) -> Self {
        Self::ErrorWithBody(JsonResponse::new(json, status))
    }

    pub fn unauthorized() -> Self {
        Error::Error(Status::Unauthorized)
    }
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Error::Error(e) => e.respond_to(&request),
            Error::ErrorWithBody(e) => {
                rocket::Response::build_from(e.json.respond_to(&request).unwrap())
                    .status(e.status)
                    .header(ContentType::JSON)
                    .ok()
            }
        }
    }
}

impl<'r, T: serde::Serialize> Responder<'r, 'static> for Response<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Response::Success(s) => s.respond_to(&request),
            Response::SuccessWithBody(s) => {
                rocket::Response::build_from(s.json.respond_to(&request).unwrap())
                    .status(s.status)
                    .header(ContentType::JSON)
                    .ok()
            }
        }
    }
}

impl<T> Response<T>
where
    T: Serialize,
{
    pub fn success(json: Option<T>, status: Status) -> Self {
        match json {
            Some(j) => Self::SuccessWithBody(JsonResponse::new(j, status)),
            None => Self::Success(status),
        }
    }
}

#[derive(Debug)]
pub struct JsonResponse<T> {
    pub json: Json<T>,
    pub status: Status,
}

impl<T> JsonResponse<T> {
    pub fn new(json: T, status: Status) -> JsonResponse<T> {
        JsonResponse {
            json: Json(json),
            status,
        }
    }
}

impl<'r, T: serde::Serialize> Responder<'r, 'static> for JsonResponse<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        rocket::Response::build_from(self.json.respond_to(&request).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[derive(Serialize)]
pub struct TokenResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<i64>,
}

impl TokenResponse {
    pub fn success(access_token: String, expires_in: i64) -> Self {
        Self {
            access_token: Some(access_token),
            expires_in: Some(expires_in),
        }
    }
}

#[derive(Deserialize)]
pub struct TokenRequest<'a> {
    #[allow(dead_code)]
    access_token: &'a str,
}
