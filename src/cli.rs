use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rtik", about = "Ticket tracker for agents", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new ticket
    Create(CreateArgs),
    /// Show ticket details
    Get { id: i64 },
    /// Update ticket fields (at least one required)
    Update(UpdateArgs),
    /// Delete a ticket
    Delete { id: i64 },
    /// List all tickets
    List(ListArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(short = 'n', long, help = "Ticket name")]
    pub name: String,
    #[arg(short = 'd', long, help = "Ticket description")]
    pub desc: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    pub id: i64,
    #[arg(short = 'n', long)]
    pub name: Option<String>,
    #[arg(short = 'd', long)]
    pub desc: Option<String>,
    #[arg(long, value_parser = parse_status)]
    pub status: Option<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Show created/updated timestamps
    #[arg(long)]
    pub timestamps: bool,
}

pub fn parse_status(raw: &str) -> Result<String, String> {
    let normalized = raw.to_lowercase();
    match normalized.as_str() {
        "todo" | "wip" | "blocked" | "done" => Ok(normalized),
        _ => Err(format!(
            "invalid status '{}': must be one of todo, wip, blocked, done",
            raw
        )),
    }
}
