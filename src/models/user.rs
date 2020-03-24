use serde::{Serialize, Deserialize};
use crate::schema::users;
use bcrypt::{DEFAULT_COST, hash, verify};
use validator::Validate;
use crate::models::validator::Validator;

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
    pub fn verify(&self, non_hashed: &String) -> bool {
        verify(&non_hashed, &self.password).unwrap()
    }
}

#[derive(Insertable, Validate, Deserialize, Serialize)]
#[table_name="users"]
pub struct NewUser {
    #[validate(email(message = "Please enter a valid email address."))]
    pub email: String,
    #[validate(length(min = 4, message = "Username must be 4 characters or more."))]
    pub username: String,
    #[validate(length(min = 12, message = "Password must be 12 characters or more."))]
    pub password: String
}

impl NewUser {
    pub fn hash_password(&mut self) {
        let hashed = hash(&self.password, DEFAULT_COST).unwrap();
        self.password = hashed;
    }
}


impl Validator for NewUser { }

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginUser {
    pub identifier: String,
    pub password: String
}

