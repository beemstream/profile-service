use rocket_contrib::{database, databases::diesel::PgConnection};

#[database("pg_conn")]
pub struct DbConn(PgConnection);
