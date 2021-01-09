use super::oauth_util::get_oauth_response;
use crate::{
    oauth::twitch_authenticate,
    util::{
        globals::TwitchConfig,
        response::{JsonResponse, JsonStatus, TokenResponse},
    },
};
use rocket::{http::CookieJar, response::Redirect};
use rocket::{
    http::{Cookie, Status},
    State,
};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchGrant<'a> {
    code: &'a str,
}

#[get("/oauth/twitch")]
pub fn twitch_auth(twitch_config: State<'_, TwitchConfig>) -> Redirect {
    let (auth_url, _) = twitch_authenticate(
        twitch_config.twitch_client_id.to_owned(),
        twitch_config.twitch_client_secret.to_owned(),
        twitch_config.twitch_callback_url.to_owned(),
    );
    Redirect::to(auth_url.to_string())
}

#[post("/oauth/twitch", format = "application/json", data = "<twitch_grant>")]
pub fn twitch_token<'a>(
    twitch_grant: Json<TwitchGrant>,
    cookies: &CookieJar<'a>,
    twitch_config: State<'a, TwitchConfig>,
) -> JsonResponse<TokenResponse> {
    let response = match get_oauth_response(
        twitch_grant.code,
        twitch_config.twitch_client_id.to_owned(),
        twitch_config.twitch_client_secret.to_owned(),
        twitch_config.twitch_callback_url.to_owned(),
    ) {
        Ok(response) => {
            let (access_token, refresh_token, expires_in) = response;
            cookies.add_private(Cookie::new("refresh_token", refresh_token));
            (
                Status::Ok,
                TokenResponse::success(JsonStatus::Ok, access_token, expires_in.as_secs() as i64),
            )
        }
        Err(e) => match e {
            oauth2::RequestTokenError::ServerResponse(response) => {
                let error = response.error_description().unwrap();
                (
                    Status::BadRequest,
                    TokenResponse::error(JsonStatus::NotOk, error.to_string()),
                )
            }
            _ => (
                Status::BadGateway,
                TokenResponse::error(JsonStatus::NotOk, "InternalServerError".to_string()),
            ),
        },
    };

    let (status, r) = response;
    JsonResponse::new(r, status)
}
