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
    /// Claim a ticket for this agent
    Claim(ClaimArgs),
    /// Release a claimed ticket
    Release(ReleaseArgs),
    /// Block a ticket with a reason
    Block(BlockArgs),
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

#[derive(Args)]
pub struct ClaimArgs {
    pub id: i64,
    #[arg(long, help = "Override another agent's claim")]
    pub force: bool,
}

#[derive(Args)]
pub struct ReleaseArgs {
    pub id: i64,
    #[arg(long, help = "Release regardless of ownership")]
    pub force: bool,
}

#[derive(Args)]
pub struct BlockArgs {
    pub id: i64,
    pub reason: String,
}

pub fn parse_status(raw: &str) -> Result<String, String> {
    let normalized = raw.to_lowercase();
    match normalized.as_str() {
        "todo" | "in-progress" | "blocked" | "done" => Ok(normalized),
        _ => Err(format!(
            "invalid status '{}': must be one of todo, in-progress, blocked, done",
            raw
        )),
    }
}
