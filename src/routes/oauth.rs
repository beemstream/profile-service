use super::oauth_util::{get_oauth_response, get_refresh_token};
use crate::{
    oauth::twitch_authenticate,
    util::{
        globals::TwitchConfig,
        response::{Error, Response, TokenResponse},
    },
};
use rocket::{get, http::CookieJar, info, post, response::Redirect, serde::json::Json};
use rocket::{
    http::{Cookie, Status},
    State,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    RefreshToken,
    Code,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchGrant {
    code: Option<String>,
    grant_type: GrantType,
}

#[get("/oauth/twitch")]
pub fn twitch_auth(twitch_config: &State<TwitchConfig>) -> Redirect {
    info!("redirecting to: {}", twitch_config.twitch_callback_url);

    let (auth_url, _) = twitch_authenticate(
        twitch_config.twitch_client_id.to_owned(),
        twitch_config.twitch_client_secret.to_owned(),
        twitch_config.twitch_callback_url.to_owned(),
    );
    Redirect::to(auth_url.to_string())
}

fn handle_grant<'a>(
    code: &str,
    cookies: &CookieJar<'a>,
    twitch_config: &State<TwitchConfig>,
) -> Result<Response<TokenResponse>, Error> {
    match get_oauth_response(
        code.to_string(),
        twitch_config.twitch_client_id.to_owned(),
        twitch_config.twitch_client_secret.to_owned(),
        twitch_config.twitch_callback_url.to_owned(),
    ) {
        Ok(response) => {
            let (access_token, refresh_token, expires_in) = response;
            cookies.add_private(Cookie::new("refresh_token", refresh_token));
            Ok(Response::success(
                Some(TokenResponse::success(
                    access_token,
                    expires_in.as_secs() as i64,
                )),
                Status::Ok,
            ))
        }
        Err(e) => {
            info!("failed to authenticated {:?}", e);
            Err(Error::Error(Status::Unauthorized))
        }
    }
}

pub fn extract_refresh_token(refresh_token: Option<Cookie>) -> Result<String, Error> {
    match refresh_token {
        Some(r) => {
            let token = r.to_string();
            let parsed_token = token.split('=').nth(1);

            Ok(parsed_token.unwrap().to_string())
        }
        None => Err(Error::Error(Status::Unauthorized)),
    }
}

fn handle_refresh<'a>(
    cookies: &CookieJar<'a>,
    twitch_config: &State<TwitchConfig>,
) -> Result<Response<TokenResponse>, Error> {
    info!("handling refresh token");
    let refresh_cookie = cookies.get_private("refresh_token");
    let refresh_token = extract_refresh_token(refresh_cookie)?;
    match get_refresh_token(
        refresh_token,
        twitch_config.twitch_client_id.to_owned(),
        twitch_config.twitch_client_secret.to_owned(),
        twitch_config.twitch_callback_url.to_owned(),
    ) {
        Ok(response) => {
            info!("got refresh token response");
            let (access_token, refresh_token, expires_in) = response;
            cookies.add_private(Cookie::new("refresh_token", refresh_token));
            Ok(Response::success(
                Some(TokenResponse::success(
                    access_token,
                    expires_in.as_secs() as i64,
                )),
                Status::Ok,
            ))
        }
        Err(_) => Err(Error::Error(Status::Unauthorized)),
    }
}

#[post("/oauth/twitch", format = "application/json", data = "<twitch_grant>")]
pub fn twitch_token<'a>(
    twitch_grant: Json<TwitchGrant>,
    cookies: &CookieJar<'a>,
    twitch_config: &State<TwitchConfig>,
) -> Result<Response<TokenResponse>, Error> {
    let twitch_grant_inner = twitch_grant.into_inner();
    match twitch_grant_inner.grant_type {
        GrantType::Code if twitch_grant_inner.code.is_some() => {
            handle_grant(&twitch_grant_inner.code.unwrap(), cookies, twitch_config)
        }
        GrantType::RefreshToken => handle_refresh(cookies, twitch_config),
        _ => Err(Error::Error(Status::Unauthorized)),
    }
}

#[get("/oauth/twitch/logout")]
pub fn logout_twitch<'a>(
    cookies: &CookieJar<'a>,
) -> Status {
    if cookies.get_private("refresh_token").is_some() {
        cookies.remove_private(Cookie::named("refresh_token"));
    }

    Status::Ok
}
