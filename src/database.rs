use rocket_contrib::databases::diesel::PgConnection;

#[database("pg_conn")]
pub struct DbConn(PgConnection);
