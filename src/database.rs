use diesel::pg::PgConnection;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use std::env;

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
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
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

