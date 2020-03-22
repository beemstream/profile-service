use serde::{Serialize, Deserialize};
use crate::schema::users;
use bcrypt::{DEFAULT_COST, hash};
use validator::{Validate, ValidationErrors, ValidationError};
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};

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

    pub fn parsed_field_errors(&self) -> Vec<JsonValue> {
        match self.validate() {
            Ok(_v) => vec![],
            Err(e) => {
                let errors = e.field_errors();

                let mut top_v = vec![];

                for key in errors.keys() {
                    let errors = errors.get(key).unwrap();

                    let mut v = vec![];
                    for i in 0..errors.len() {
                        v.push(&errors[i].message);
                    }
                    top_v.push(json!({ "name": key, "message": v }));
                }

                top_v
            }
        }
    }
}
