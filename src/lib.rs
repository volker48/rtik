pub mod cli;
pub mod db;
pub mod ticket;

use rusqlite::Connection;
use ticket::AppError;

pub fn run(cli: cli::Cli, _conn: Connection) -> Result<(), AppError> {
    use cli::Commands;
    match cli.command {
        Commands::Create(_) => todo!("create"),
        Commands::Get { .. } => todo!("get"),
        Commands::Update(_) => todo!("update"),
        Commands::Delete { .. } => todo!("delete"),
        Commands::List(_) => todo!("list"),
    }
}
