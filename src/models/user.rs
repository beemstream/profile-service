use crate::{schema::users, util::validator::Validator};
use argon2::{self, hash_encoded, verify_encoded_ext, Config};
use rand::Rng;
use serde::{Deserialize, Serialize};
use validator::Validate;

pub enum UserType<'a> {
    LoginUser(&'a LoginUser),
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
    pub fn verify(&self, non_hashed: &str, secret_key: &String) -> bool {
        verify_encoded_ext(
            &self.password,
            non_hashed.as_bytes(),
            secret_key.as_bytes(),
            &[],
        )
        .unwrap()
    }
}

#[derive(Deserialize, Validate, Serialize)]
pub struct NewUserRequest {
    #[validate(required, email(message = "email_invalid"))]
    pub email: Option<String>,
    #[validate(required, length(min = 4, message = "username_length_invalid"))]
    pub username: Option<String>,
    #[validate(
        required,
        length(min = 12, message = "password_length_invalid"),
        must_match(other = "password_repeat", message = "password_not_matching")
    )]
    pub password: Option<String>,
    #[validate(required)]
    pub password_repeat: Option<String>,
}

#[derive(Insertable, Deserialize, Serialize)]
#[table_name = "users"]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password: String,
}

impl NewUser {
    pub fn hash_password(password: String, secret_key: &String) -> String {
        let config = Config {
            secret: secret_key.as_bytes(),
            ..Config::default()
        };
        let salt = rand::thread_rng().gen::<[u8; 32]>();
        let hash = hash_encoded(password.as_bytes(), &salt, &config).unwrap();

        hash
    }

    pub fn from(new_user_request: NewUserRequest, secret_key: &String) -> Self {
        Self {
            username: new_user_request.username.to_owned().unwrap(),
            email: new_user_request.email.to_owned().unwrap(),
            password: NewUser::hash_password(
                new_user_request.password.to_owned().unwrap(),
                secret_key,
            ),
        }
    }
}

impl Validator for NewUserRequest {}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LoginUser {
    #[validate(required)]
    pub identifier: Option<String>,
    #[validate(required, length(min = 12))]
    pub password: Option<String>,
}

impl Validator for LoginUser {}

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
