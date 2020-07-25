lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("ROCKET_secret_key").expect("secret_key must be set").to_string();
}

