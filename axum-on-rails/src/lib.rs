use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub mod cli;
pub mod test_helpers;

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;
