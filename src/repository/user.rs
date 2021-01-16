use crate::{database::DbConn, routes::users_util::get_auth_error_response, util::response::{Error, ErrorType}};
use crate::models::user::{NewUser, NewUserRequest, User};
use crate::schema::users;
use rocket::http::Status;
use rocket_contrib::databases::diesel::{self, prelude::*};

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
) -> Result<NewUserRequest, crate::util::response::Error> {
    conn.run(|c| {
        let found_username = get_by_username(&user.username.to_owned().unwrap(), c);
        let found_email = get_by_email(&user.email.to_owned().unwrap(), c);

        let is_found_by_username = has_found_user(found_username);
        let is_found_by_email = has_found_user(found_email);

        if is_found_by_username {
            Err(Error::error(Some((vec!["username_exists".to_owned()], ErrorType::RequestInvalid)), Status::Conflict))
        } else if is_found_by_email {
            Err(Error::error(Some((vec!["email_exists".to_owned()], ErrorType::RequestInvalid)), Status::Conflict))
        } else {
            Ok(user)
        }
    })
    .await
}

pub async fn insert(conn: &DbConn, user: NewUser) -> Result<usize, crate::util::response::Error> {
    conn.run(|c| {
        diesel::insert_into(users::table).values(user).execute(c).map_err(|e| get_auth_error_response(e))
    }).await
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
        Ok(_) => true,
        Err(_) => false,
    }
}
