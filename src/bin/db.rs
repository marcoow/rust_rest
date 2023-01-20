use dotenv_codegen::dotenv;
use tokio_postgres::NoTls;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let db_url = dotenv!("DATABASE_URL");
    let (mut client, connection) = tokio_postgres::connect(db_url, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("An error occured while connecting to database: {}", e);
        }
    });

    embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .unwrap();
}
