use jsonwebtoken::{Header, Validation, Algorithm};

pub fn header() -> Header {
    Header::new(Algorithm::HS512)
}

pub fn validation() -> Validation {
    let mut validation = Validation::new(Algorithm::HS512);
    validation.leeway = 2;
    validation.iss = Some("beemstream".to_string());
    validation
}
