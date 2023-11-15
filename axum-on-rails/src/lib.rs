use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub mod cli;
pub mod test;

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;
