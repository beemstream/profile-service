use crate::database::DbConn;
use crate::models::user::{NewUser, NewUserRequest, User};
use crate::schema::users;
use rocket_contrib::databases::diesel::result::DatabaseErrorKind::UniqueViolation;
use rocket_contrib::databases::diesel::result::Error::DatabaseError;
use rocket_contrib::databases::diesel::{self, prelude::*};

struct RegisterError<'a> {
    column_name: &'a str,
    error_description: &'a str,
}

impl<'a> RegisterError<'a> {
    fn new(column_name: &'a str, error_description: &'a str) -> Box<RegisterError<'a>> {
        Box::new(RegisterError {
            column_name,
            error_description,
        })
    }
}

impl<'a> diesel::result::DatabaseErrorInformation for RegisterError<'a> {
    fn message(&self) -> &str {
        self.error_description
    }
    fn hint(&self) -> Option<&str> {
        Some("Try a different value.")
    }
    fn details(&self) -> Option<&str> {
        None
    }
    fn column_name(&self) -> Option<&str> {
        Some(&self.column_name)
    }
    fn constraint_name(&self) -> Option<&str> {
        None
    }
    fn table_name(&self) -> Option<&str> {
        Some("users")
    }
}

pub fn get_by_username(username: &String, conn: &PgConnection) -> QueryResult<i32> {
    users::table
        .select(users::id)
        .filter(users::username.eq(&username))
        .get_result::<i32>(conn)
}

pub fn get_by_email(email: &String, conn: &PgConnection) -> QueryResult<i32> {
    users::table
        .select(users::id)
        .filter(users::email.eq(&email))
        .get_result::<i32>(conn)
}

pub async fn is_duplicate_user_or_email(
    conn: &DbConn,
    user: NewUserRequest,
) -> Result<NewUserRequest, diesel::result::Error> {
    conn.run(|c| {
        let found_username = get_by_username(&user.username.to_owned().unwrap(), c);
        let found_email = get_by_email(&user.email.to_owned().unwrap(), c);

        let is_found_by_username = has_found_user(found_username);
        let is_found_by_email = has_found_user(found_email);

        if is_found_by_username {
            Err(DatabaseError(
                UniqueViolation,
                RegisterError::new("username", "username_exists"),
            ))
        } else if is_found_by_email {
            Err(DatabaseError(
                UniqueViolation,
                RegisterError::new("email", "email_exists"),
            ))
        } else {
            Ok(user)
        }
    })
    .await
}

pub async fn insert(conn: &DbConn, user: NewUser) -> Result<usize, diesel::result::Error> {
    conn.run(|c| diesel::insert_into(users::table).values(user).execute(c))
        .await
}

pub async fn find(conn: &DbConn, identifier: String) -> Result<User, diesel::result::Error> {
    conn.run(move |c| {
        users::table
            .filter(users::email.eq(&identifier))
            .or_filter(users::username.eq(&identifier))
            .get_result::<User>(c)
    })
    .await
}

pub async fn update(conn: &DbConn, id: i32, user: User) -> QueryResult<User> {
    conn.run(move |c| {
        diesel::update(users::table.find(id))
            .set(&user)
            .get_result(c)
    })
    .await
}

pub async fn delete(conn: &DbConn, id: i32, mut user: User) -> QueryResult<User> {
    conn.run(move |c| {
        user.is_deleted = true;

        diesel::update(users::table.find(id))
            .set(user)
            .get_result(c)
    })
    .await
}

pub fn has_found_user(user: QueryResult<i32>) -> bool {
    match user {
        Ok(_v) => true,
        Err(_e) => false,
    }
}
