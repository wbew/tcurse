use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::env;

const API_BASE: &str = "https://www.recurse.com/api/v1";

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

#[derive(Debug, Deserialize)]
struct Profile {
    id: i64,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct HubVisit {
    date: String,
    #[serde(default)]
    notes: Option<String>,
    person: VisitPerson,
}

#[derive(Debug, Deserialize, Serialize)]
struct VisitPerson {
    id: i64,
    name: String,
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

async fn get_current_user(client: &reqwest::Client, token: &str) -> Result<Profile, String> {
    let response = client
        .get(format!("{}/profiles/me", API_BASE))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    response
        .json::<Profile>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

async fn get_my_visit(client: &reqwest::Client, token: &str, person_id: i64, date: &str) -> Result<Option<HubVisit>, String> {
    let response = client
        .get(format!("{}/hub_visits/{}/{}", API_BASE, person_id, date))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let visit = response
        .json::<HubVisit>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(Some(visit))
}

async fn checkin(client: &reqwest::Client, token: &str, notes: Option<String>, remove: bool) -> Result<(), String> {
    let me = get_current_user(client, token).await?;
    let date = get_date_string(None);

    // Handle removal
    if remove {
        let response = client
            .delete(format!("{}/hub_visits/{}/{}", API_BASE, me.id, date))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        println!("Removed check-in for {}", date);
        return Ok(());
    }

    // Check if already checked in (only block if no new notes to add)
    if let Some(existing) = get_my_visit(client, token, me.id, &date).await? {
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

    let mut request = client
        .patch(format!("{}/hub_visits/{}/{}", API_BASE, me.id, date))
        .bearer_auth(token);

    if let Some(n) = notes {
        request = request.json(&serde_json::json!({ "notes": n }));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let visit: HubVisit = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    println!("Checked in for {}", visit.date);
    if let Some(n) = visit.notes {
        if !n.is_empty() {
            println!("Notes: {}", n);
        }
    }

    Ok(())
}

async fn get_checked_in(client: &reqwest::Client, token: &str, date: Option<String>) -> Result<(), String> {
    let date_str = get_date_string(date);

    // Validate date format
    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|_| "Invalid date format. Use YYYY-MM-DD".to_string())?;

    let response = client
        .get(format!("{}/hub_visits", API_BASE))
        .query(&[("date", &date_str)])
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let visits: Vec<HubVisit> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

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
    let client = reqwest::Client::new();

    let result = match cli.command {
        Commands::Checkin { notes, remove } => checkin(&client, &token, notes, remove).await,
        Commands::CheckedIn { date } => get_checked_in(&client, &token, date).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
