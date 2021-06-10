use rocket_sync_db_pools::{database, diesel::PgConnection};

#[database("pg_conn")]
pub struct DbConn(PgConnection);
