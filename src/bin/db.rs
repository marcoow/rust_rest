use clap::{arg, value_parser, ArgMatches, Command};
use core::str::FromStr;
use std::env;
use std::fs;
use std::fs::File;
use std::time::SystemTime;
use tokio_postgres::{config::Config, Client, NoTls};
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./db/migrations");
}

enum Environment {
    Development,
    Test,
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
            Command::new("reset")
                .about("Reset the database (drop, re-create, migrate)")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
        .subcommand(Command::new("seed").about("Seed the database"))
        .subcommand(
            Command::new("generate")
                .subcommand_required(true)
                .subcommand(
                    Command::new("migration")
                        .about("Generate a new migration file")
                        .arg(arg!([NAME])),
                ),
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
            drop(sub_matches).await;
        }
        Some(("create", sub_matches)) => {
            create(sub_matches).await;
        }
        Some(("migrate", sub_matches)) => {
            migrate(sub_matches).await;
        }
        Some(("reset", sub_matches)) => {
            drop(sub_matches).await;
            create(sub_matches).await;
            migrate(sub_matches).await;
        }
        Some(("seed", _sub_matches)) => {
            seed().await;
        }
        Some(("generate", sub_matches)) => match sub_matches.subcommand() {
            Some(("migration", sub_matches)) => {
                let name = sub_matches
                        .get_one::<String>("NAME")
                        .map(|s| s.as_str())
                        .expect("No migration name specified – must specify a name to use for the migration file!");
                generate_migration(name).await;
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

async fn drop(sub_matches: &ArgMatches) {
    let env = choose_env(sub_matches);

    match env {
        Environment::Development => println!("ℹ️ Dropping development database…"),
        Environment::Test => println!("ℹ️ Dropping test database…"),
    }

    let db_config = get_db_config(&env);
    let db_name = db_config.get_dbname().unwrap();
    let root_db_client = get_root_db_client(&env).await;

    let result = root_db_client
        .execute(&format!("drop database if exists {}", &db_name), &[])
        .await;

    match result {
        Ok(_) => println!("✅ Database {} dropped successfully.", &db_name),
        Err(_) => println!("❌ Dropping database {} failed!", &db_name),
    }
}

async fn create(sub_matches: &ArgMatches) {
    let env = choose_env(sub_matches);

    match env {
        Environment::Development => println!("ℹ️ Creating development database…"),
        Environment::Test => println!("ℹ️ Creating test database…"),
    }

    let db_config = get_db_config(&env);
    let db_name = db_config.get_dbname().unwrap();
    let root_db_client = get_root_db_client(&env).await;

    let result = root_db_client
        .execute(&format!("create database {}", &db_name), &[])
        .await;

    match result {
        Ok(_) => println!("✅ Database {} created successfully.", &db_name),
        Err(_) => println!("❌ Creating database {} failed!", &db_name),
    }
}

async fn migrate(sub_matches: &ArgMatches) {
    let env = choose_env(sub_matches);

    match env {
        Environment::Development => println!("ℹ️ Migrating development database…"),
        Environment::Test => println!("ℹ️ Migrating test database…"),
    }

    let mut client = get_db_client(&env).await;

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

async fn seed() {
    println!("ℹ️ Seeding development database…");

    let mut client = get_db_client(&Environment::Development).await;

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

fn choose_env(sub_matches: &ArgMatches) -> Environment {
    let env = sub_matches
        .get_one::<String>("env")
        .map(|s| s.as_str())
        .unwrap_or("development");

    if env == "test" {
        Environment::Test
    } else {
        Environment::Development
    }
}

fn get_db_config(env: &Environment) -> Config {
    match env {
        Environment::Test => read_dotenv_config(".env.test"),
        Environment::Development => read_dotenv_config(".env"),
    }

    let db_url = env::var("DATABASE_URL").unwrap();
    Config::from_str(&db_url).unwrap()
}

async fn get_db_client(env: &Environment) -> Client {
    let db_config = get_db_config(env);
    let (client, connection) = db_config.connect(NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("An error occured while connecting to database: {}", e);
        }
    });

    client
}

async fn get_root_db_client(env: &Environment) -> Client {
    let db_config = get_db_config(env);
    let mut root_db_config = db_config.clone();
    root_db_config.dbname("postgres");
    let (client, connection) = root_db_config.connect(NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("An error occured while connecting to database: {}", e);
        }
    });

    client
}

async fn generate_migration(name: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let name = format!("V{}__{}.sql", timestamp.as_secs(), name);
    File::create(format!("./db/migrations/{}", name)).expect("❌ Could not create migration file!");
    println!("✅ Created migration {}.", name);
}
