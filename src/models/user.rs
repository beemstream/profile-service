use serde::{Serialize, Deserialize};
use crate::{util::validator::Validator, schema::users};
use bcrypt::{DEFAULT_COST, hash, verify};
use validator::Validate;

pub enum UserType<'a> {
    LoginUser(&'a LoginUser<'a>),
    StoredUser(&'a User)
}

#[derive(Queryable, AsChangeset, Serialize, Deserialize)]
#[table_name="users"]
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
        verify(&non_hashed, &self.password).unwrap()
    }
}

#[derive(Deserialize, Validate, Serialize)]
pub struct NewUserRequest {
    #[validate(email(message = "Please enter a valid email address."))]
    pub email: String,
    #[validate(length(min = 4, message = "Username must be 4 characters or more."))]
    pub username: String,
    #[ validate(
        length(min = 12, message = "Password must be 12 characters or more."),
        must_match(other = "password_repeat", message = "Password does not match."))
    ]
    pub password: String,
    pub password_repeat: String
}

#[derive(Insertable, Deserialize, Serialize)]
#[table_name="users"]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password: String
}

impl NewUser {
    pub fn hash_password(&mut self) -> &mut Self {
        let hashed = hash(&self.password, DEFAULT_COST).unwrap();
        self.password = hashed;
        self
    }

    pub fn from(new_user_request: NewUserRequest) -> Self {
        Self {
            username: new_user_request.username,
            email: new_user_request.email,
            password: new_user_request.password
        }
    }
}


impl Validator for NewUserRequest { }

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginUser<'a> {
    pub identifier: &'a str,
    pub password: &'a str
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
