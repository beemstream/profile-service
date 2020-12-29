use crate::jwt::jwt_validation;
use jsonwebtoken::Validation;

lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("ROCKET_SECRET_KEY")
        .expect("ROCKET_SECRET_KEY must be set")
        .to_string();
}

lazy_static! {
    pub static ref ALLOWED_ORIGINS: String = std::env::var("ALLOWED_ORIGINS")
        .expect("ALLOWED_ORIGINS must be set")
        .to_string();
}

lazy_static! {
    pub static ref VALIDATION: Validation = jwt_validation();
}

lazy_static! {
    pub static ref TWITCH_CLIENT_ID: String =
        std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID must be set");
}

lazy_static! {
    pub static ref TWITCH_CLIENT_SECRET: String =
        std::env::var("TWITCH_CLIENT_SECRET").expect("TWITCH_CLIENT_SECRET must be set");
}

lazy_static! {
    pub static ref TWITCH_CALLBACK_URL: String =
        std::env::var("TWITCH_CALLBACK_URL").expect("TWITCH_CLIENT_SECRET must be set");
}

pub const COOKIE_REFRESH_TOKEN_NAME: &str = "refresh_token";

lazy_static! {
    pub static ref TOKEN_EXPIRY: i64 = {
        std::env::var("TOKEN_EXPIRY").map_or_else(
            |_| 120,
            |v| {
                v.parse::<i64>()
                    .expect("TOKEN_EXPIRY must be number in seconds.")
            },
        )
    };
}

lazy_static! {
    pub static ref REFRESH_TOKEN_EXPIRY: i64 = {
        std::env::var("REFRESH_TOKEN_EXPIRY").map_or_else(
            |_| 60 * 60 * 24 * 3,
            |v| {
                v.parse::<i64>()
                    .expect("REFRESH_TOKEN_EXPIRY must be number in seconds.")
            },
        )
    };
}
