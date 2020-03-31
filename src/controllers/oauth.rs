use crate::oauth::{twitch_authenticate, twitch_exchange_code};
use crate::controllers::response::ApiResponse;
use serde::{Serialize, Deserialize};
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use rocket_contrib::json;
use rocket::http::Status;

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchGrant<'a> {
    code: &'a str
}

#[get("/auth/twitch")]
pub fn twitch_auth() -> Redirect {
    let (auth_url, _) = twitch_authenticate();
    Redirect::to(auth_url.to_string())
}

#[post("/auth/twitch", format="application/json", data="<twitch_grant>")]
pub fn twitch_token(twitch_grant: Json<TwitchGrant>) -> ApiResponse {

    match twitch_exchange_code(String::from(twitch_grant.code)) {
        Ok(_v) => {
            ApiResponse::new(json!({ "status": "OK" }), Status::Ok)
        },
        Err(_e) => {
            ApiResponse::new(json!({ "status": "NOT OK" }), Status::InternalServerError)
        }
    }
}
