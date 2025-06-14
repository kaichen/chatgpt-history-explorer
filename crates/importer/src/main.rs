use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

use importer::{create_database, extract_conversations_from_zip, import_conversations, models::Conversation};

#[derive(Parser)]
#[command(name = "chatgpt-importer")]
#[command(about = "Import ChatGPT conversations from zip file to SQLite database")]
struct Args {
    /// Path to the zip file containing conversations.json
    zip_file: PathBuf,

    /// Output SQLite database file name (optional, defaults to conversations.db)
    #[arg(short, long, default_value = "conversations.db")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Extracting conversations.json from {}", args.zip_file.display());
    let conversations_json = extract_conversations_from_zip(&args.zip_file)?;

    println!("Parsing conversations data...");
    let conversations: Vec<Conversation> = serde_json::from_str(&conversations_json)
        .context("Failed to parse conversations.json")?;

    println!("Creating SQLite database at {}", args.output.display());
    let conn = create_database(&args.output)?;

    println!("Importing {} conversations...", conversations.len());
    import_conversations(&conn, &conversations, &args.zip_file)?;

    println!("Import completed successfully!");
    println!("Database created at: {}", args.output.display());

    Ok(())
}
