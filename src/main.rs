use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use std::env;
use tcurse::ApiClient;

#[derive(Parser)]
#[command(name = "tcurse")]
#[command(about = "CLI tool for interacting with the Recurse Center API")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check in to the hub (creates or updates your visit for today)
    Checkin {
        /// Optional notes to add to your check-in
        #[arg(short, long)]
        notes: Option<String>,
        /// Remove your check-in instead of creating one
        #[arg(short, long)]
        remove: bool,
    },
    /// View who is checked in today
    CheckedIn {
        /// Date to check (defaults to today, format: YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,
    },
}

fn get_token() -> String {
    dotenvy::dotenv().ok();
    env::var("RC_TOKEN").expect("RC_TOKEN must be set (via environment or .env file)")
}

fn get_date_string(date_arg: Option<String>) -> String {
    match date_arg {
        Some(d) => d,
        None => Local::now().format("%Y-%m-%d").to_string(),
    }
}

async fn checkin(client: &ApiClient, notes: Option<String>, remove: bool) -> Result<(), String> {
    let me = client.get_current_user().await?;
    let date = get_date_string(None);

    if remove {
        client.delete_visit(me.id, &date).await?;
        println!("Removed check-in for {}", date);
        return Ok(());
    }

    // Check if already checked in (only block if no new notes to add)
    if let Some(existing) = client.get_visit(me.id, &date).await? {
        if notes.is_none() {
            println!("Already checked in for {}", existing.date);
            if let Some(n) = existing.notes {
                if !n.is_empty() {
                    println!("Notes: {}", n);
                }
            }
            return Ok(());
        }
    }

    let visit = client.create_or_update_visit(me.id, &date, notes.as_deref()).await?;

    println!("Checked in for {}", visit.date);
    if let Some(n) = visit.notes {
        if !n.is_empty() {
            println!("Notes: {}", n);
        }
    }

    Ok(())
}

async fn get_checked_in(client: &ApiClient, date: Option<String>) -> Result<(), String> {
    let date_str = get_date_string(date);

    // Validate date format
    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|_| "Invalid date format. Use YYYY-MM-DD".to_string())?;

    let visits = client.get_visits(&date_str).await?;

    if visits.is_empty() {
        println!("No one is checked in for {}", date_str);
        return Ok(());
    }

    println!("Checked in for {} ({} people):", date_str, visits.len());
    for visit in visits {
        let name = &visit.person.name;
        match &visit.notes {
            Some(n) if !n.is_empty() => println!("  - {} ({})", name, n),
            _ => println!("  - {}", name),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let token = get_token();
    let client = ApiClient::new(token);

    let result = match cli.command {
        Commands::Checkin { notes, remove } => checkin(&client, notes, remove).await,
        Commands::CheckedIn { date } => get_checked_in(&client, date).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
