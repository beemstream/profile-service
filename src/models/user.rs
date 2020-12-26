use crate::util::globals::SECRET_KEY;
use crate::{schema::users, util::validator::Validator};
use argonautica::{Hasher, Verifier};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub enum UserType<'a> {
    LoginUser(&'a LoginUser<'a>),
    StoredUser(&'a User),
}

#[derive(Queryable, AsChangeset, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub is_deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl User {
    pub fn verify(&self, non_hashed: &str) -> bool {
        let mut verifier = Verifier::default();
        let is_valid = verifier
            .with_hash(&self.password)
            .with_password(non_hashed)
            .with_secret_key(SECRET_KEY.as_str())
            .verify()
            .unwrap();
        is_valid
    }
}

#[derive(Deserialize, Validate, Serialize)]
pub struct NewUserRequest {
    #[validate(email(message = "Please enter a valid email address."))]
    pub email: String,
    #[validate(length(min = 4, message = "Username must be 4 characters or more."))]
    pub username: String,
    #[validate(
        length(min = 12, message = "Password must be 12 characters or more."),
        must_match(other = "password_repeat", message = "Password does not match.")
    )]
    pub password: String,
    pub password_repeat: String,
}

#[derive(Insertable, Deserialize, Serialize)]
#[table_name = "users"]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password: String,
}

impl NewUser {
    pub fn hash_password(&mut self) -> &mut Self {
        let mut hasher = Hasher::default();
        let hash = hasher
            .with_password(&self.password)
            .with_secret_key(SECRET_KEY.as_str())
            .hash()
            .unwrap();
        self.password = hash;
        self
    }

    pub fn from(new_user_request: &NewUserRequest) -> Self {
        Self {
            username: new_user_request.username.to_owned(),
            email: new_user_request.email.to_owned(),
            password: new_user_request.password.to_owned(),
        }
    }
}

impl Validator for NewUserRequest {}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginUser<'a> {
    pub identifier: &'a str,
    pub password: &'a str,
}

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
