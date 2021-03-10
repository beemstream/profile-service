use crate::models::user::{NewRefreshToken, RefreshToken};
use crate::schema::refresh_tokens;
use crate::{database::DbConn, routes::users_util::get_auth_error_response};
use rocket::http::Status;
use rocket_contrib::databases::diesel::{self, prelude::*};

pub async fn insert(
    conn: &DbConn,
    refresh_token: NewRefreshToken,
) -> Result<usize, crate::util::response::Error> {
    conn.run(|c| {
        diesel::insert_into(refresh_tokens::table)
            .values(refresh_token)
            .execute(c)
            .map_err(|e| get_auth_error_response(e))
    })
    .await
}

pub async fn find_by_token(
    conn: &DbConn,
    token: String,
) -> Result<RefreshToken, crate::util::response::Error> {
    conn.run(move |c| {
        refresh_tokens::table
            .filter(refresh_tokens::token.eq(token))
            .first::<RefreshToken>(c)
            .map_err(|e| {
                info!("before mapping error {:?}", e);

                match e {
                    diesel::result::Error::NotFound => {
                        crate::util::response::Error::Error(Status::Unauthorized)
                    }
                    _ => get_auth_error_response(e),
                }
            })
    })
    .await
}

pub async fn delete(conn: &DbConn, id: i32) -> Result<usize, crate::util::response::Error> {
    conn.run(move |c| {
        let token_id = refresh_tokens::table.filter(refresh_tokens::id.eq(id));
        diesel::delete(token_id)
            .execute(c)
            .map_err(|e| get_auth_error_response(e))
    })
    .await
}
