use rusqlite::{Connection, TransactionBehavior};
use std::collections::{HashMap, HashSet};
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
    #[error("invalid status '{0}': must be one of todo, in-progress, blocked, done")]
    InvalidStatus(String),
    #[error("ticket #{0} already claimed by {1} since {2}")]
    AlreadyClaimed(i64, String, String),
    #[error("from {from}, valid transitions are: {valid}")]
    InvalidTransition { from: String, valid: String },
    #[error("block reason is required")]
    BlockReasonRequired,
    #[error("ticket #{0} not claimed by you ({1})")]
    NotOwner(i64, String),
    #[error("ticket #{0} is not currently claimed")]
    NotClaimed(i64),
    #[error("RTIK_AGENT not set — set it to identify this agent")]
    AgentNotSet,
    #[error("cycle detected: {0}")]
    CyclicDependency(String),
    #[error("dependency from #{0} to #{1} not found")]
    DepNotFound(i64, i64),
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
}

pub fn validate_transition(from: &str, to: &str) -> Result<(), AppError> {
    let allowed: &[&str] = match from {
        "todo" => &["in-progress", "blocked"],
        "in-progress" => &["done", "blocked", "todo"],
        "blocked" => &["in-progress", "todo"],
        "done" => &["in-progress"],
        _ => &[],
    };
    if allowed.contains(&to) {
        Ok(())
    } else {
        Err(AppError::InvalidTransition {
            from: from.to_string(),
            valid: allowed.join(", "),
        })
    }
}

pub fn claim_ticket(conn: &mut Connection, id: i64, agent: &str, force: bool) -> Result<(), AppError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

    // Check unmet deps and warn.
    let unmet_count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM ticket_deps td
         JOIN tickets dep ON dep.id = td.depends_on
         WHERE td.ticket_id = ?1 AND dep.status != 'done'",
        rusqlite::params![id],
        |row| row.get(0),
    )?;
    if unmet_count > 0 {
        eprintln!("Warning: {} dependencies not done", unmet_count);
    }

    let now = chrono_free_utc_now();
    if force {
        let existing: Option<String> = tx
            .query_row(
                "SELECT claimed_by FROM tickets WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(id),
                other => AppError::Db(other),
            })?;
        if let Some(ref owner) = existing {
            if owner != agent {
                eprintln!("Warning: overriding claim by {}", owner);
            }
        }
        tx.execute(
            "UPDATE tickets SET claimed_by = ?1, claimed_at = ?2, status = 'in-progress', updated_at = ?2 WHERE id = ?3",
            rusqlite::params![agent, now, id],
        )?;
    } else {
        let affected = tx.execute(
            "UPDATE tickets SET claimed_by = ?1, claimed_at = ?2, status = 'in-progress', updated_at = ?2 WHERE id = ?3 AND claimed_by IS NULL",
            rusqlite::params![agent, now, id],
        )?;
        if affected == 0 {
            let result: rusqlite::Result<(Option<String>, Option<String>)> = tx.query_row(
                "SELECT claimed_by, claimed_at FROM tickets WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            );
            match result {
                Err(rusqlite::Error::QueryReturnedNoRows) => return Err(AppError::NotFound(id)),
                Err(e) => return Err(AppError::Db(e)),
                Ok((Some(owner), claimed_at)) => {
                    let at = claimed_at.unwrap_or_default();
                    return Err(AppError::AlreadyClaimed(id, owner, at));
                }
                Ok((None, _)) => {
                    // Ticket exists but unclaimed — shouldn't happen, but handle gracefully.
                    return Err(AppError::NotFound(id));
                }
            }
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn release_ticket(conn: &mut Connection, id: i64, agent: &str, force: bool) -> Result<(), AppError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

    let result: rusqlite::Result<Option<String>> = tx.query_row(
        "SELECT claimed_by FROM tickets WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    );
    let claimed_by = match result {
        Err(rusqlite::Error::QueryReturnedNoRows) => return Err(AppError::NotFound(id)),
        Err(e) => return Err(AppError::Db(e)),
        Ok(val) => val,
    };

    if force {
        if let Some(ref owner) = claimed_by {
            if owner != agent {
                eprintln!("Warning: overriding claim by {}", owner);
            }
        }
    } else {
        match &claimed_by {
            None => return Err(AppError::NotClaimed(id)),
            Some(owner) if owner != agent => return Err(AppError::NotOwner(id, agent.to_string())),
            _ => {}
        }
    }

    let now = chrono_free_utc_now();
    tx.execute(
        "UPDATE tickets SET claimed_by = NULL, claimed_at = NULL, status = 'todo', updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, id],
    )?;

    tx.commit()?;
    Ok(())
}

pub fn block_ticket(conn: &Connection, id: i64, reason: &str) -> Result<String, AppError> {
    let (current_status, name): (String, String) = conn
        .query_row(
            "SELECT status, name FROM tickets WHERE id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(id),
            other => AppError::Db(other),
        })?;

    validate_transition(&current_status, "blocked")?;

    let now = chrono_free_utc_now();
    conn.execute(
        "UPDATE tickets SET status = 'blocked', block_reason = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![reason, now, id],
    )?;

    Ok(name)
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

pub struct ListFilter {
    pub status: Option<String>,
    /// None = no filter, Some(true) = claimed only, Some(false) = unclaimed only
    pub claimed: Option<bool>,
    pub claimer: Option<String>,
    /// Each term must appear in name OR description (AND-composed across terms, case-insensitive)
    pub search: Vec<String>,
}

pub fn list_tickets_filtered(conn: &Connection, filter: &ListFilter) -> Result<Vec<Ticket>, AppError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref s) = filter.status {
        conditions.push("status = ?".to_string());
        params.push(Box::new(s.clone()));
    }
    match filter.claimed {
        Some(true) => conditions.push("claimed_by IS NOT NULL".to_string()),
        Some(false) => conditions.push("claimed_by IS NULL".to_string()),
        None => {}
    }
    if let Some(ref c) = filter.claimer {
        conditions.push("claimed_by = ?".to_string());
        params.push(Box::new(c.clone()));
    }
    for term in &filter.search {
        conditions.push("(LOWER(name) LIKE ? OR LOWER(description) LIKE ?)".to_string());
        let pattern = format!("%{}%", term.to_lowercase());
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT id, name, description, status, created_at, updated_at FROM tickets {} ORDER BY id ASC",
        where_clause
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
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

pub fn list_tickets(conn: &Connection) -> Result<Vec<Ticket>, AppError> {
    list_tickets_filtered(conn, &ListFilter { status: None, claimed: None, claimer: None, search: vec![] })
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

    // Validate transition before building SET clause.
    if let Some(ref ns) = normalized_status {
        let current_status: String = conn
            .query_row(
                "SELECT status FROM tickets WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(id),
                other => AppError::Db(other),
            })?;
        validate_transition(&current_status, ns)?;
    }

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
        if ns == "done" {
            sets.push("claimed_by = :claimnil");
            sets.push("claimed_at = :claimnil_at");
            params.push((":claimnil", &rusqlite::types::Null));
            params.push((":claimnil_at", &rusqlite::types::Null));
        }
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

pub struct DepInfo {
    pub forward: Vec<i64>,
    pub reverse: Vec<i64>,
}

pub fn would_create_cycle(
    conn: &Connection,
    ticket_id: i64,
    new_dep: i64,
) -> Result<Option<Vec<i64>>, AppError> {
    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut stmt = conn.prepare("SELECT ticket_id, depends_on FROM ticket_deps")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
    for row in rows {
        let (from, to) = row?;
        adj.entry(from).or_default().push(to);
    }
    adj.entry(ticket_id).or_default().push(new_dep);

    let mut visited: HashSet<i64> = HashSet::new();
    let mut path: Vec<i64> = Vec::new();
    if dfs_finds_target(&adj, new_dep, ticket_id, &mut visited, &mut path) {
        path.insert(0, ticket_id);
        return Ok(Some(path));
    }
    Ok(None)
}

fn dfs_finds_target(
    adj: &HashMap<i64, Vec<i64>>,
    current: i64,
    target: i64,
    visited: &mut HashSet<i64>,
    path: &mut Vec<i64>,
) -> bool {
    if current == target {
        return true;
    }
    if !visited.insert(current) {
        return false;
    }
    path.push(current);
    if let Some(neighbors) = adj.get(&current) {
        for &next in neighbors {
            if dfs_finds_target(adj, next, target, visited, path) {
                return true;
            }
        }
    }
    path.pop();
    false
}

pub fn add_dep(conn: &Connection, ticket_id: i64, depends_on: i64) -> Result<(), AppError> {
    get_ticket(conn, ticket_id)?;
    get_ticket(conn, depends_on)?;
    if ticket_id == depends_on {
        return Err(AppError::CyclicDependency(format!(
            "#{} → #{}",
            ticket_id, depends_on
        )));
    }
    if let Some(path) = would_create_cycle(conn, ticket_id, depends_on)? {
        let cycle_str = path
            .iter()
            .map(|id| format!("#{}", id))
            .collect::<Vec<_>>()
            .join(" → ");
        return Err(AppError::CyclicDependency(cycle_str));
    }
    conn.execute(
        "INSERT INTO ticket_deps (ticket_id, depends_on) VALUES (?1, ?2)",
        rusqlite::params![ticket_id, depends_on],
    )?;
    Ok(())
}

pub fn remove_dep(conn: &Connection, ticket_id: i64, depends_on: i64) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM ticket_deps WHERE ticket_id=?1 AND depends_on=?2",
        rusqlite::params![ticket_id, depends_on],
    )?;
    if conn.changes() == 0 {
        return Err(AppError::DepNotFound(ticket_id, depends_on));
    }
    Ok(())
}

pub fn list_deps(conn: &Connection, ticket_id: i64) -> Result<DepInfo, AppError> {
    let mut stmt = conn.prepare(
        "SELECT depends_on FROM ticket_deps WHERE ticket_id=?1 ORDER BY depends_on",
    )?;
    let forward: Vec<i64> = stmt
        .query_map(rusqlite::params![ticket_id], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut stmt = conn
        .prepare("SELECT ticket_id FROM ticket_deps WHERE depends_on=?1 ORDER BY ticket_id")?;
    let reverse: Vec<i64> = stmt
        .query_map(rusqlite::params![ticket_id], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(DepInfo { forward, reverse })
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
