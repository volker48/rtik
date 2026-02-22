use rusqlite::Connection;
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

pub fn create_ticket(conn: &Connection, name: &str, desc: &str) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO tickets (name, description) VALUES (?1, ?2)",
        rusqlite::params![name, desc],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_ticket(conn: &Connection, id: i64) -> Result<Ticket, AppError> {
    conn.query_row(
        "SELECT id, name, description, status, created_at, updated_at
         FROM tickets WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Ticket {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(id),
        other => AppError::Db(other),
    })
}

pub fn list_tickets(conn: &Connection) -> Result<Vec<Ticket>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, status, created_at, updated_at
         FROM tickets ORDER BY id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Ticket {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            status: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(AppError::Db)
}

pub fn delete_ticket(conn: &Connection, id: i64) -> Result<(), AppError> {
    match conn.execute("DELETE FROM tickets WHERE id = ?1", rusqlite::params![id])? {
        0 => Err(AppError::NotFound(id)),
        _ => Ok(()),
    }
}

pub fn update_ticket(
    conn: &Connection,
    id: i64,
    name: Option<&str>,
    desc: Option<&str>,
    status: Option<&str>,
) -> Result<String, AppError> {
    if name.is_none() && desc.is_none() && status.is_none() {
        return Err(AppError::NoUpdateFields);
    }
    let now = chrono_free_utc_now();
    // Normalize status outside any if-let so the String lives long enough for params.
    let normalized_status: Option<String> = status.map(|s| s.to_lowercase());

    let mut sets: Vec<&str> = Vec::new();
    let mut params: Vec<(&str, &dyn rusqlite::types::ToSql)> = Vec::new();

    if name.is_some() {
        sets.push("name = :name");
        params.push((":name", name.as_ref().unwrap()));
    }
    if desc.is_some() {
        sets.push("description = :desc");
        params.push((":desc", desc.as_ref().unwrap()));
    }
    if let Some(ref ns) = normalized_status {
        sets.push("status = :status");
        params.push((":status", ns));
    }
    sets.push("updated_at = :now");
    params.push((":now", &now));
    params.push((":id", &id));

    let sql = format!("UPDATE tickets SET {} WHERE id = :id", sets.join(", "));
    let affected = conn.execute(&sql, params.as_slice())?;
    match affected {
        0 => Err(AppError::NotFound(id)),
        _ => {
            let t = get_ticket(conn, id)?;
            Ok(t.name)
        }
    }
}

fn chrono_free_utc_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time before epoch")
        .as_secs();
    let (y, mo, d, h, mi, s) = seconds_to_datetime(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, mi, s)
}

fn seconds_to_datetime(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let mins = secs / 60;
    let mi = mins % 60;
    let hours = mins / 60;
    let h = hours % 24;
    let days = hours / 24;
    let (y, mo, d) = days_to_ymd(days);
    (y, mo, d, h, mi, s)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year: u64 = 1970;
    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month: u64 = 1;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}
