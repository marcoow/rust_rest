use clap::{arg, value_parser, Command};
use core::str::FromStr;
use std::env;
use std::fs;
use tokio_postgres::{config::Config, NoTls};
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
            Command::new("drop")
                .about("Drop the database")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
        .subcommand(
            Command::new("create")
                .about("Create the database")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
        .subcommand(
            Command::new("migrate")
                .about("Migrate the database")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
        .subcommand(
            Command::new("seed")
                .about("Seed the database")
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
        Some(("drop", sub_matches)) => {
            let env = sub_matches
                .get_one::<String>("env")
                .map(|s| s.as_str())
                .unwrap_or("development");

            if env == "test" {
                println!("ℹ️ Dropping test database…");
                read_dotenv_config(".env.test");
            } else {
                println!("ℹ️ Dropping development database…");
                read_dotenv_config(".env");
            }

            let db_url = env::var("DATABASE_URL").unwrap();
            let db_config = Config::from_str(&db_url).unwrap();
            let db_name = db_config.get_dbname().unwrap();
            let mut root_db_config = db_config.clone();
            root_db_config.dbname("postgres");

            let (client, connection) = root_db_config.connect(NoTls).await.unwrap();

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    error!("An error occured while connecting to database: {}", e);
                }
            });

            let result = client
                .execute(&format!("drop database if exists {}", &db_name), &[])
                .await;

            match result {
                Ok(_) => println!("✅ Database {} dropped successfully.", &db_name),
                Err(_) => println!("❌ Dropping database {} failed!", &db_name),
            }
        }
        Some(("create", sub_matches)) => {
            let env = sub_matches
                .get_one::<String>("env")
                .map(|s| s.as_str())
                .unwrap_or("development");

            if env == "test" {
                println!("ℹ️ Creating test database…");
                read_dotenv_config(".env.test");
            } else {
                println!("ℹ️ Creating development database…");
                read_dotenv_config(".env");
            }

            let db_url = env::var("DATABASE_URL").unwrap();
            let db_config = Config::from_str(&db_url).unwrap();
            let db_name = db_config.get_dbname().unwrap();
            let mut root_db_config = db_config.clone();
            root_db_config.dbname("postgres");

            let (client, connection) = root_db_config.connect(NoTls).await.unwrap();

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    error!("An error occured while connecting to database: {}", e);
                }
            });

            let result = client
                .execute(&format!("create database {}", &db_name), &[])
                .await;

            match result {
                Ok(_) => println!("✅ Database {} created successfully.", &db_name),
                Err(_) => println!("❌ Creating database {} failed!", &db_name),
            }
        }
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
        Some(("seed", sub_matches)) => {
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

            let statements = fs::read_to_string("./db/seeds.sql")
                .expect("Could not read seeds – make sure db/seeds.sql exists!");

            let transaction = client.transaction().await.unwrap();
            let result = transaction.execute(statements.as_str(), &[]).await;
            match result {
                Ok(_) => {
                    let _ = transaction
                        .commit()
                        .await
                        .map_err(|_| println!("❌ Seeding database failed."));
                    println!("✅ Seeded database.");
                }
                Err(_) => println!("❌ Seeding database failed."),
            }
        }
        _ => unreachable!(),
    }
}
