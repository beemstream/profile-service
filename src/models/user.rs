use crate::{
    schema::{refresh_tokens, users},
    util::validator::Validator,
};
use argon2::{self, hash_encoded, verify_encoded_ext, Config};
use rand::Rng;
use serde::{Deserialize, Serialize};
use validator::Validate;

pub enum UserType<'a> {
    LoginUser(&'a LoginUser),
    StoredUser(&'a User),
}

#[derive(Identifiable, Queryable, AsChangeset, Serialize, Deserialize, Debug, PartialEq, Clone)]
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
    pub fn verify(&self, non_hashed: &str, secret_key: &str) -> bool {
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
    fn hash_password(password: String, secret_key: &str) -> String {
        let config = Config {
            secret: secret_key.as_bytes(),
            ..Config::default()
        };
        let salt = rand::thread_rng().gen::<[u8; 32]>();
        let hash = hash_encoded(password.as_bytes(), &salt, &config).unwrap();

        hash
    }

    pub fn from(new_user_request: NewUserRequest, secret_key: &str) -> Self {
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

#[derive(Identifiable, Queryable, Serialize, Deserialize, Associations, Debug, PartialEq)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "refresh_tokens"]
pub struct RefreshToken {
    pub id: i32,
    pub token: String,
    pub expiry: chrono::NaiveDateTime,
    pub user_id: i32,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "refresh_tokens"]
pub struct NewRefreshToken {
    pub token: String,
    pub expiry: chrono::NaiveDateTime,
    pub user_id: i32,
}
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LoginUser {
    #[validate(required)]
    pub identifier: Option<String>,
    #[validate(required, length(min = 12))]
    pub password: Option<String>,
}

impl Validator for LoginUser {}
