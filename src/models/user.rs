use serde::{Serialize, Deserialize};
use crate::schema::users;

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

#[derive(Insertable, Deserialize, Serialize)]
#[table_name="users"]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password: String
}

impl NewUser {
    pub fn from(user: User) -> NewUser {
        NewUser {
            email: user.email,
            username: user.username,
            password: user.password
        }
    }
}
