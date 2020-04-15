use crate::{util::response::ApiResponse, oauth::{twitch_authenticate, twitch_exchange_code}};
use serde::{Serialize, Deserialize};
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use rocket_contrib::json;
use rocket::http::{Status, Cookie, Cookies};
use oauth2::TokenResponse;
use oauth2::prelude::SecretNewType;

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
pub fn twitch_token(twitch_grant: Json<TwitchGrant>, mut cookies: Cookies) -> ApiResponse {

    match twitch_exchange_code(String::from(twitch_grant.code)) {
        Ok(v) => {
            let access_token = v.access_token();
            let refresh_token = v.refresh_token();
            let expires_in = v.expires_in();
            cookies.add_private(Cookie::new("access_token", access_token.secret().to_owned()));
            cookies.add_private(Cookie::new("refresh_token", refresh_token.unwrap().secret().to_owned()));
            cookies.add(Cookie::new("expires_in", expires_in.unwrap().as_millis().to_string()));
            ApiResponse::new(json!({ "status": "OK" }), Status::Ok)
        },
        Err(e) => {
            match e {
                oauth2::RequestTokenError::ServerResponse(response) => {
                    ApiResponse::new(json!({ "status": "NOT OK", "message": response.error_description().unwrap() }), Status::InternalServerError)
                }
                _ => ApiResponse::new(json!({ "status": "NOT OK" }), Status::InternalServerError)
            }
            
        }
    }
}
