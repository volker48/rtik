use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::PathBuf;
use std::time::Duration;

use crate::ticket::AppError;

static MIGRATIONS: &[M<'static>] = &[
    M::up(
        "CREATE TABLE tickets (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            status      TEXT NOT NULL DEFAULT 'todo'
                        CHECK(status IN ('todo','wip','blocked','done')),
            created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );"
    ),
    M::up(
        "CREATE TABLE tickets_new (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            status      TEXT NOT NULL DEFAULT 'todo'
                        CHECK(status IN ('todo','in-progress','blocked','done')),
            claimed_by  TEXT,
            claimed_at  TEXT,
            block_reason TEXT,
            created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );
        INSERT INTO tickets_new
            SELECT id, name, description,
                   CASE status WHEN 'wip' THEN 'in-progress' ELSE status END,
                   NULL, NULL, NULL,
                   created_at, updated_at
            FROM tickets;
        DROP TABLE tickets;
        ALTER TABLE tickets_new RENAME TO tickets;
        CREATE TABLE ticket_deps (
            ticket_id  INTEGER NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
            depends_on INTEGER NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
            PRIMARY KEY (ticket_id, depends_on),
            CHECK (ticket_id != depends_on)
        );"
    ),
];

pub fn resolve_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("RTIK_DB") {
        return PathBuf::from(path);
    }
    let mut dir = std::env::current_dir().expect("cannot read cwd");
    loop {
        let candidate = dir.join(".rtik.db");
        if candidate.exists() {
            return candidate;
        }
        if !dir.pop() {
            break;
        }
    }
    std::env::current_dir()
        .expect("cannot read cwd")
        .join(".rtik.db")
}

pub fn open_connection(path: &std::path::Path) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Migrations::from_slice(MIGRATIONS)
        .to_latest(&mut conn)
        .expect("migration failed");
    Ok(conn)
}
