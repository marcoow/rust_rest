use std::env;
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

    let args: Vec<String> = env::args().collect();

    match args.len() {
        0..=1 => println!(r#"❌ Specify command, e.g. """migrate"""#),
        n => match args[1].as_str() {
            "migrate" => {
                if n == 2 {
                    dotenvy::from_filename(".env").unwrap();
                } else if n == 3 && args[2].as_str() == "--test" {
                    dotenvy::from_filename(".env.test").unwrap();
                } else {
                    println!(r#"❌ Invalid command """{:?}"""#, args);
                    return;
                }
                let db_url = env::var("DATABASE_URL").unwrap();

                let (mut client, connection) = tokio_postgres::connect(db_url.as_str(), NoTls)
                    .await
                    .unwrap();

                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("An error occured while connecting to database: {}", e);
                    }
                });

                let report = embedded::migrations::runner()
                    .run_async(&mut client)
                    .await
                    .unwrap();

                let migrations_applied = report.applied_migrations().len();

                match migrations_applied {
                    0 => println!("ℹ️ There were no pendign migrations to apply."),
                    n => println!("✅ Applied {n} migrations."),
                }
            }
            other => println!("❌ Unknown command {:?}!", other),
        },
    }
}
