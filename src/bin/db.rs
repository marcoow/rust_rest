use clap::{arg, value_parser, Command};
use std::env;
use tokio_postgres::NoTls;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./db/migrations");
}

fn cli() -> Command {
    Command::new("db")
        .about("A CLI tool to manage the project's database.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("migrate")
                .about("Migrate the database")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
}

fn read_dotenv_config(file: &str) {
    dotenvy::from_filename(file).ok();
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("migrate", sub_matches)) => {
            let env = sub_matches
                .get_one::<String>("env")
                .map(|s| s.as_str())
                .unwrap_or("development");

            if env == "test" {
                println!("ℹ️ Migrating test database…");
                read_dotenv_config(".env.test");
            } else {
                println!("ℹ️ Migrating development database…");
                read_dotenv_config(".env");
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
                0 => println!("ℹ️ There were no pending migrations to apply."),
                n => println!("✅ Applied {n} migrations."),
            }
        }
        _ => unreachable!(),
    }
}
