use crate::models::user::{NewUser, NewUserRequest, User};
use crate::schema::users;
use crate::{
    database::DbConn,
    routes::users_util::get_auth_error_response,
    util::response::{Error, ErrorType},
};
use rocket::http::Status;
use rocket_sync_db_pools::diesel::{self, prelude::*};

pub fn get_by_username(username: &str, conn: &PgConnection) -> QueryResult<i32> {
    users::table
        .select(users::id)
        .filter(users::username.eq(&username))
        .get_result::<i32>(conn)
}

pub fn get_by_email(email: &str, conn: &PgConnection) -> QueryResult<i32> {
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

        let is_found_by_username = found_username.is_ok();
        let is_found_by_email = found_email.is_ok();

        let mut errors: Vec<String> = vec![];

        if is_found_by_username {
            errors.push("username_exists".to_owned());
        }

        if is_found_by_email {
            errors.push("email_exists".to_owned());
        }

        if !errors.is_empty() {
            return Err(Error::error(
                Some((errors, ErrorType::RequestInvalid)),
                Status::Conflict,
            ));
        }

        Ok(user)
    })
    .await
}

pub async fn insert(conn: &DbConn, user: NewUser) -> Result<User, crate::util::response::Error> {
    conn.run(|c| {
        diesel::insert_into(users::table)
            .values(user)
            .get_result::<User>(c)
            .map_err(get_auth_error_response)
    })
    .await
}

pub async fn find(conn: &DbConn, identifier: String) -> Result<User, Status> {
    conn.run(move |c| {
        users::table
            .filter(users::email.eq(&identifier))
            .or_filter(users::username.eq(&identifier))
            .get_result::<User>(c)
            .map_err(|_| Status::NotFound)
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
