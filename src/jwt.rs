use jsonwebtoken::{Algorithm, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    iss: String,
    pub exp: usize,
    iat: usize,
    nbf: usize,
}

const ISSUER: &'static str = "beemstream";

impl Claims {
    pub fn new(identifier: &str, refresh_interval: i64) -> Claims {
        let time_now = chrono::Utc::now();
        let exp = time_now + chrono::Duration::seconds(refresh_interval);
        let nbf = time_now + chrono::Duration::seconds(2);

        Claims {
            sub: String::from(identifier),
            iss: String::from(ISSUER),
            exp: exp.timestamp() as usize,
            iat: time_now.timestamp() as usize,
            nbf: nbf.timestamp() as usize,
        }
    }

    pub fn sub(&self) -> &str {
        &self.sub
    }
}

pub fn generate_header() -> Header {
    Header::new(Algorithm::HS512)
}

pub fn jwt_validation() -> Validation {
    let mut validation = Validation::new(Algorithm::HS512);
    validation.leeway = 2;
    validation.iss = Some(ISSUER.to_owned());
    validation
}
