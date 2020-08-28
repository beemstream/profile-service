use crate::models::user::{NewUser, User};
use crate::database::get_pooled_connection;
use diesel::result::Error::DatabaseError;
use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::prelude::*;
use crate::schema::users;

struct RegisterError<'a> {
    column_name: &'a str,
    error_description: &'a str
}

impl<'a> RegisterError<'a> {
    fn new(column_name: &'a str, error_description: &'a str) -> Box<RegisterError<'a>> {
        Box::new(RegisterError {
            column_name,
            error_description
        })
    }
}

impl<'a> diesel::result::DatabaseErrorInformation for RegisterError<'a> {
    fn message(&self) -> &str { self.error_description }
    fn hint(&self) -> Option<&str> { Some("Try a different value.") }
    fn details(&self) -> Option<&str> { None }
    fn column_name(&self) -> Option<&str> { Some(&self.column_name) }
    fn constraint_name(&self) -> Option<&str> { None }
    fn table_name(&self) -> Option<&str> { Some("users") }
}

pub fn get_by_username(username: &String, conn: &PgConnection) -> QueryResult<i32> {
    users::table.select(users::id).filter(users::username.eq(&username)).get_result::<i32>(conn)
}

pub fn get_by_email(email: &String, conn: &PgConnection) -> QueryResult<i32> {
    users::table.select(users::id).filter(users::email.eq(&email)).get_result::<i32>(conn)
}

pub fn insert(user: &NewUser) -> Result<usize, diesel::result::Error> {
    let conn = &*get_pooled_connection();
    let found_username = get_by_username(&user.username, conn);
    let found_email = get_by_email(&user.email, conn);

    let is_found_by_username = has_found_user(found_username);
    let is_found_by_email = has_found_user(found_email);

    if is_found_by_username {
        Err(DatabaseError(UniqueViolation, RegisterError::new("username", "Username already exists.")))
    } else if is_found_by_email {
        Err(DatabaseError(UniqueViolation, RegisterError::new("email", "Email already exists.")))
    } else {
        diesel::insert_into(users::table)
            .values(user)
            .execute(conn)
    }
}

pub fn find(identifier: &str) -> Result<User, diesel::result::Error> {
    let conn = &*get_pooled_connection();

    users::table
        .filter(users::email.eq(identifier))
        .or_filter(users::username.eq(identifier))
        .get_result::<User>(conn)
}

pub fn update(id: i32, user: User) -> QueryResult<User> {
    let conn = &*get_pooled_connection();
    diesel::update(users::table.find(id))
        .set(&user)
        .get_result(conn)
}

pub fn delete(id: i32, mut user: User) -> QueryResult<User> {
    let conn = &*get_pooled_connection();
    user.is_deleted = true;

    diesel::update(users::table.find(id))
        .set(user)
        .get_result(conn)
}

pub fn has_found_user(user: QueryResult<i32>) -> bool {
    match user {
        Ok(_v) => true,
        Err(_e) => false
    }
}
