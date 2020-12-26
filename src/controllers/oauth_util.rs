use crate::oauth::{twitch_exchange_code, ExchangeError};
use oauth2::{prelude::SecretNewType, TokenResponse};
use std::time::Duration;

pub type RefreshToken = String;
pub type OAuthSuccessResponse = (String, RefreshToken, Duration);

pub fn get_oauth_response<'a>(code_grant: &'a str) -> Result<OAuthSuccessResponse, ExchangeError> {
    match twitch_exchange_code(code_grant) {
        Ok(exchange_response) => {
            let access_token = exchange_response.access_token().secret().to_owned();
            let refresh_token = exchange_response
                .refresh_token()
                .unwrap()
                .secret()
                .to_owned();
            let expires_in = exchange_response.expires_in();

            Ok((access_token, refresh_token, expires_in.unwrap()))
        }
        Err(err) => Err(err),
    }
}
