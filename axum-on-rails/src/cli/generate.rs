use clap::{arg, Command};
use std::fs::File;
use std::time::SystemTime;

fn commands() -> Command {
    Command::new("db")
        .about("A CLI tool to generate project files.")
        .subcommand_required(true)
        .subcommand(
            Command::new("migration")
                .about("Generate a new migration file")
                .arg(arg!([NAME]).required(true))
        )
}

pub async fn cli() {
    let matches = commands().get_matches();

    match matches.subcommand() {
        Some(("migration", sub_matches)) => {
            let name = sub_matches
                .get_one::<String>("NAME")
                .map(|s| s.as_str()).unwrap();
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
