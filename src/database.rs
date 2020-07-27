use diesel::pg::PgConnection;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use crate::util::globals::DATABASE_URL;

// Type aliases to simplify a bit the types
type PostgresPool = Pool<ConnectionManager<PgConnection>>;
pub type PostgresPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

// Using lazy static to have a global reference to my connection pool
// However, I feel that for testing/mocking this won't be great.
lazy_static! {
    static ref POOL: PostgresPool =  init_pool();
}

fn init_pool() -> PostgresPool {
    // I chose configuration via an environment variable
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.") // Unrecoverable failure!
}

/// Get a connection from a static pool of connections
pub fn get_pooled_connection() -> PostgresPooledConnection {
    let pool = POOL.clone();
    let database_connection = pool.get().expect("Failed to get pooled connection"); // Not sure when a panic is triggered here
    database_connection
}

