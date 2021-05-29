use jsonwebtoken::Validation;
use rocket::config::SecretKey;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GlobalConfig {
    pub token_expiry: i64,
    pub refresh_token_expiry: i64,
    pub secret_key: SecretKey,
    pub auth_secret_key: String,
    pub allowed_origins: Vec<String>,
}

pub struct JWTConfig {
    pub validation: Validation,
}

#[derive(Deserialize)]
pub struct TwitchConfig {
    pub twitch_client_id: String,
    pub twitch_client_secret: String,
    pub twitch_callback_url: String,
}

#[derive(Deserialize)]
pub struct EmailConfig {
    pub email_username: String,
    pub email_password: String,
    pub email_enabled: bool,
}

pub const COOKIE_REFRESH_TOKEN_NAME: &str = "refresh_token";
