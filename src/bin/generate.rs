use clap::{arg, Command};
use std::fs::File;
use std::time::SystemTime;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn cli() -> Command {
    Command::new("db")
        .about("A CLI tool to generate project files.")
        .subcommand_required(true)
        .subcommand(
            Command::new("migration")
                .about("Generate a new migration file")
                .arg(arg!([NAME])),
        )
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("migration", sub_matches)) => {
            let name = sub_matches
                .get_one::<String>("NAME")
                .map(|s| s.as_str())
                .expect(
                "No migration name specified – must specify a name to use for the migration file!",
            );
            generate_migration(name).await;
        }
        _ => unreachable!(),
    }
}

async fn generate_migration(name: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let name = format!("V{}__{}.sql", timestamp.as_secs(), name);
    File::create(format!("./db/migrations/{}", name)).expect("❌ Could not create migration file!");
    println!("✅ Created migration {}.", name);
}
