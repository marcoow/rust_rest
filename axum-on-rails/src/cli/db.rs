use crate::cli::env::{parse_env, Environment};
use crate::cli::ui::{log, log_per_env, LogType};
use clap::{arg, value_parser, Command};
use std::env;
use std::fs;
use tracing::error;
use url::Url;
use sqlx::{ConnectOptions, PgPool, Connection, Executor, migrate::Migrator};
use sqlx::postgres::{PgConnectOptions, PgConnection, PgPoolOptions};
use std::path::Path;

fn commands() -> Command {
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
            Command::new("reset")
                .about("Reset the database (drop, re-create, migrate)")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
        .subcommand(Command::new("seed").about("Seed the database"))
}

pub async fn cli() {
    let matches = commands().get_matches();

    match matches.subcommand() {
        Some(("drop", sub_matches)) => {
            let env = parse_env(&sub_matches);
            drop(&env).await;
        }
        Some(("create", sub_matches)) => {
            let env = parse_env(&sub_matches);
            create(&env).await;
        }
        Some(("migrate", sub_matches)) => {
            let env = parse_env(&sub_matches);
            migrate(&env).await;
        }
        Some(("reset", sub_matches)) => {
            let env = parse_env(&sub_matches);
            drop(&env).await;
            create(&env).await;
            migrate(&env).await;
        }
        Some(("seed", _sub_matches)) => {
            seed().await;
        }
        _ => unreachable!(),
    }
}

fn read_dotenv_config(file: &str) {
    dotenvy::from_filename(file).ok();
}

async fn drop(env: &Environment) {
    log_per_env(
        env,
        LogType::Info,
        "Dropping development database…",
        "Dropping test database…",
    );

    let db_config = get_db_config(&env);
    let db_name = db_config.get_database().unwrap();
    let mut root_connection = get_root_db_client(&env).await;

    let query = format!("DROP DATABASE IF EXISTS {}", db_name);
    let result = root_connection.execute(query.as_str()).await;

    match result {
        Ok(_) => log(
            LogType::Success,
            format!("Database {} dropped successfully.", &db_name).as_str(),
        ),
        Err(_) => log(
            LogType::Error,
            format!("Dropping database {} failed!", &db_name).as_str(),
        ),
    }
}

async fn create(env: &Environment) {
    log_per_env(
        env,
        LogType::Info,
        "Creating development database…",
        "Creating test database…",
    );

    let db_config = get_db_config(&env);
    let db_name = db_config.get_database().unwrap();
    let mut root_connection = get_root_db_client(&env).await;

    let query = format!("CREATE DATABASE {}", db_name);
    let result = root_connection.execute(query.as_str()).await;

    match result {
        Ok(_) => log(
            LogType::Success,
            format!("Database {} created successfully.", &db_name).as_str(),
        ),
        Err(_) => log(
            LogType::Error,
            format!("Creating database {} failed!", &db_name).as_str(),
        ),
    }
}

async fn migrate(env: &Environment) {
    log_per_env(
        env,
        LogType::Info,
        "Migrating development database…",
        "Migrating test database…",
    );

    let db_config = get_db_config(env);
    let mut sqlx_pool = PgPoolOptions::new()
    .connect_with(db_config)
    .await
    .expect("Could not connect to database!");

    let migrator = Migrator::new(Path::new("../db/migrations")).await.unwrap();    
    migrator.migrate(&sqlx_pool).await.unwrap();

    match migrator.migrations. {
        0 => log(LogType::Info, "There were no pending migrations to apply."),
        n => log(
            LogType::Success,
            format!("Applied {} migrations.", n).as_str(),
        ),
    }
}

async fn seed() {
    log(LogType::Info, "Seeding development database…");

    let mut connection = get_db_client(&Environment::Development).await;

    let statements = fs::read_to_string("./db/seeds.sql")
        .expect("Could not read seeds – make sure db/seeds.sql exists!");

    let mut transaction = connection.begin().await.unwrap();
    let result = transaction.execute(statements.as_str()).await;

    match result {
        Ok(_) => {
            transaction
                .commit()
                .await
                .map_err(|_| log(LogType::Error, "Seeding database failed!"));
            log(LogType::Info, "Seeded database.");
        }
        Err(_) => log(LogType::Error, "Seeding database failed!"),
    }
}

fn get_db_config(env: &Environment) -> PgConnectOptions {
    match env {
        Environment::Test => read_dotenv_config(".env.test"),
        Environment::Development => read_dotenv_config(".env"),
    }

    let db_url = Url::parse(env::var("DATABASE_URL").expect("No DATABASE_URL set – cannot run tests!").as_str()).expect("Invalid DATABASE_URL!");
    ConnectOptions::from_url(&db_url).expect("Invalid DATABASE_URL!")
}

async fn get_db_client(env: &Environment) -> PgConnection {
    let db_config = get_db_config(env);
    let connection: PgConnection = Connection::connect_with(&db_config).await.unwrap();

    connection
}

async fn get_root_db_client(env: &Environment) -> PgConnection {
    let db_config = get_db_config(env);
    let root_db_config = db_config.clone().database("postgres");
    let connection: PgConnection = Connection::connect_with(&root_db_config).await.unwrap();
    
    connection
}
