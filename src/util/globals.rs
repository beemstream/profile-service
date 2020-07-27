use jsonwebtoken::Validation;
use crate::jwt::jwt_validation;

lazy_static! {
    pub static ref DATABASE_URL: String = std::env::var("DATABASE_URL").expect("TWITCH_CLIENT_SECRET must be set");
}

lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("ROCKET_SECRET_KEY").expect("ROCKET_SECRET_KEY must be set").to_string();
}

lazy_static! {
    pub static ref ALLOWED_ORIGINS: String = std::env::var("ALLOWED_ORIGINS").expect("ALLOWED_ORIGINS must be set").to_string();
}

lazy_static! {
    pub static ref VALIDATION: Validation = jwt_validation();
}

lazy_static! {
    pub static ref TWITCH_CLIENT_ID: String = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID must be set");
}

lazy_static! {
    pub static ref TWITCH_CLIENT_SECRET: String = std::env::var("TWITCH_CLIENT_SECRET").expect("TWITCH_CLIENT_SECRET must be set");
}

lazy_static! {
    pub static ref TWITCH_CALLBACK_URL: String = std::env::var("TWITCH_CALLBACK_URL").expect("TWITCH_CLIENT_SECRET must be set");
}

pub const COOKIE_REFRESH_TOKEN_NAME: &str = "refresh_token";
pub const TOKEN_EXPIRY: i64 = 120;
pub const REFRESH_TOKEN_EXPIRY: i64 = 60 * 60 * 24 * 3;