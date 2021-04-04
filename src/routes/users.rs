use rocket::{get, http::Status};

use crate::util::authorization::AccessToken;

#[get("/authenticate")]
pub fn authenticate(_access_token: AccessToken) -> Status {
    Status::Ok
}
