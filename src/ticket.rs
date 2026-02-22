use thiserror::Error;

#[derive(Debug)]
pub struct Ticket {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("ticket #{0} not found")]
    NotFound(i64),
    #[error("no fields provided — use --name, --desc, or --status")]
    NoUpdateFields,
    #[error("invalid status '{0}': must be one of todo, wip, blocked, done")]
    InvalidStatus(String),
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
}
