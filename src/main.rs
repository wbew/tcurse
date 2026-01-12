use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tcurse")]
#[command(about = "CLI tool for interacting with the Recurse Center API")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List profiles
    Profiles,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Profiles => {
            println!("Listing profiles...");
        }
    }
}
