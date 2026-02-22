# Phase 1: Foundation & Core Operations - Research

**Researched:** 2026-02-22
**Domain:** Rust CLI binary with SQLite persistence (Clap + rusqlite + rusqlite_migration)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Phase Boundary:** Build the SQLite-backed data layer and CLI binary (Rust + Clap + rusqlite) with full
CRUD for tickets. Agents can create, view, update, delete, and list tickets with persistent storage.
No claiming, no search, no export — this phase is purely the foundation.

**Command syntax:**
- Named flags for all fields: `rtik create --name "Fix bug" --desc "Details here"`
- Short aliases: `-n` for `--name`, `-d` for `--desc`
- `rtik get <id>` displays a compact plain-text paragraph format:
  ```
  #1 Buy milk [todo]
  Go to grocery store
  Created: 2026-02-22 | Updated: 2026-02-22
  ```
- `rtik delete <id>` outputs a confirmation on success: `Deleted: #1 Buy milk`

**List output format:**
- Aligned table with column headers: `ID`, `STATUS`, `NAME`
- Default columns: ID, status, name only (minimal)
- Optional CLI flags to show timestamps (created/updated) — exact flags at Claude's discretion
- Default sort: by ID ascending
- Long names truncated at ~40 chars with ellipsis

**Database location:**
- Project-local: `.rtik.db` in the project directory
- Walks up parent directories to find `.rtik.db` (git-like behavior)
- Override via `RTIK_DB` environment variable
- If no `.rtik.db` found anywhere and no env var: auto-create `.rtik.db` in current working directory (zero setup required)

**Update command design:**
- Named flags per field: `rtik update <id> [--name "..."] [--desc "..."] [--status <status>]`
- At least one flag required — no-arg update is an error: "Error: at least one field required (--name, --desc, --status)"
- Success output: short confirmation line, e.g. `Updated: #1 Buy milk`
- Status values are case-insensitive on input (`WIP`, `wip`, `Wip` all accepted), normalized to lowercase on save

### Claude's Discretion

- Flag names for optional timestamp columns in `rtik list` (e.g. `--show-dates`, `--verbose`, `--created`)
- Exact truncation logic for names (character count, ellipsis style)
- Error message formatting and stderr vs stdout routing
- Schema migration implementation approach (within WAL mode + SQLite constraint)
- Exact Clap builder patterns and subcommand structure

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CRUD-01 | User can create ticket with name and description via CLI | Clap derive: `create` subcommand with `--name`/`-n` and `--desc`/`-d` flags; rusqlite `execute()` INSERT |
| CRUD-02 | User can view ticket details by ID | Clap: `get <id>` positional arg; rusqlite `query_row()` + struct mapping |
| CRUD-03 | User can update ticket fields (name, description, status) | Clap: `update <id>` with optional flags; dynamic SQL UPDATE; validate at least one flag set |
| CRUD-04 | User can delete ticket by ID | Clap: `delete <id>` positional arg; rusqlite `execute()` DELETE; return row count to detect not-found |
| CRUD-05 | User can list all tickets | Clap: `list` subcommand with optional timestamp flag; rusqlite `prepare()` + `query_map()` |
| STATE-01 | Ticket has status field with values: todo, WIP, blocked, done | TEXT column in schema; validated enum in Rust; normalized to lowercase on save |
| STATE-02 | Ticket automatically tracks created_at timestamp on creation | SQLite DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) or application-side chrono |
| STATE-03 | Ticket automatically updates updated_at timestamp on modification | Application-side: bind current timestamp in UPDATE statement |
| STATE-05 | Ticket has unique auto-incrementing ID | SQLite INTEGER PRIMARY KEY autoincrement; `last_insert_rowid()` to retrieve |
| TECH-01 | All data persists in SQLite database | rusqlite 0.38.0 with `Connection::open()` |
| TECH-02 | SQLite uses WAL mode for concurrent reads during writes | `pragma_update(None, "journal_mode", "WAL")` + `pragma_update(None, "synchronous", "NORMAL")` |
| TECH-05 | CLI compiles to single binary with zero runtime dependencies | rusqlite `bundled` feature statically links libsqlite3; `cargo build --release` |
| TECH-06 | Database schema supports migrations for future changes | rusqlite_migration 2.4.1: `Migrations::new(vec![M::up(...)])` + `to_latest(&mut conn)` |
| CLI-04 | CLI provides helpful error messages with context | All errors to stderr via `eprintln!`; include what failed and what ID/name was involved |
| CLI-05 | CLI exits with standard codes (0=success, 1=error, 2=usage) | Clap auto-exits with 2 for usage errors; `std::process::exit(1)` for runtime errors |
| CLI-06 | CLI handles broken pipe gracefully (piping to head/grep) | `sigpipe` crate 0.1.3: call `sigpipe::reset()` at top of `main()`; or manual SIGPIPE reset |
</phase_requirements>

## Summary

Phase 1 builds the entire foundation: a single Rust binary (`rtik`) that persists ticket data in a
local SQLite database and exposes CRUD operations via a structured CLI. The technology choices are
locked: Clap 4 (derive API), rusqlite 0.38.0 (with the `bundled` feature for zero runtime deps), and
rusqlite_migration 2.4.1 for forward-compatible schema management. WAL mode with `synchronous=NORMAL`
is the required SQLite configuration.

The most interesting implementation details are: (1) the directory-walking database discovery (git-like
`RTIK_DB` env var or walk parents for `.rtik.db`), (2) the broken-pipe fix for CLI-06 which Rust
doesn't handle by default, and (3) building dynamic UPDATE SQL when only some fields are provided.
Status validation (case-insensitive input, lowercase normalization) and the "at least one flag" guard
on update are small but concrete behaviors that must be tested.

**Primary recommendation:** Use `src/main.rs` + `src/lib.rs` split — main.rs handles CLI parsing and
process exit, lib.rs contains database and domain logic. This makes integration-testing possible
without spawning a subprocess.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.5.60 | CLI argument parsing | De-facto standard Rust CLI library; derive API is zero-boilerplate |
| rusqlite | 0.38.0 | SQLite FFI bindings | Only ergonomic synchronous SQLite crate for Rust; inspired by rust-postgres |
| rusqlite_migration | 2.4.1 | Schema versioning | Uses `user_version` pragma (lightweight); embeds SQL in binary; no CLI needed |
| sigpipe | 0.1.3 | SIGPIPE reset | One-call fix for Rust's broken-pipe panic problem; required for CLI-06 |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| thiserror | latest | Error type derivation | Deriving `std::error::Error` on domain error enums cleanly |

**Installation (Cargo.toml):**
```toml
[package]
name = "rtik"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "rtik"
path = "src/main.rs"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
rusqlite = { version = "0.38", features = ["bundled"] }
rusqlite_migration = "2.4"
sigpipe = "0.1"
thiserror = "2"
```

Note: `features = ["bundled"]` statically links libsqlite3 into the binary — this is what delivers
TECH-05 (zero runtime dependencies). Without it, the target system must have libsqlite3 installed.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rusqlite (sync) | sqlx (async) | sqlx adds tokio runtime; overkill for a single-process CLI; async adds complexity |
| rusqlite_migration | refinery | refinery requires separate migration files; rusqlite_migration embeds SQL in binary |
| sigpipe crate | manual unsafe SIGPIPE reset | Manual reset works but sigpipe is well-tested and readable |
| thiserror | anyhow | anyhow is easier but loses structured error types needed for clean exit codes |

## Architecture Patterns

### Recommended Project Structure

```
src/
├── main.rs          # CLI entry point: parse args, call lib, set exit code
├── lib.rs           # Public API: db module, ticket module, error types
├── db.rs            # Database connection, WAL setup, migration, path discovery
├── ticket.rs        # Ticket struct, CRUD functions, status validation
└── cli.rs           # Clap structs: Cli, Commands, CreateArgs, UpdateArgs, etc.
tests/
└── integration.rs   # End-to-end tests using in-memory or temp-file DB
```

The `main.rs` + `lib.rs` split is the idiomatic Rust pattern. `main.rs` only: calls `sigpipe::reset()`,
parses the CLI, calls lib functions, and maps errors to exit codes. All logic in `lib.rs` modules.

### Pattern 1: Clap Derive with Subcommands and Positional IDs

**What:** Use `#[derive(Parser)]` on the top-level struct and `#[derive(Subcommand)]` on the commands
enum. Subcommands with a required ID use a plain `id: i64` field (positional). Optional update fields
use `Option<String>`.

**When to use:** All CLI parsing in this phase.

```rust
// Source: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
use clap::{Parser, Subcommand, Args};

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
    /// Update ticket fields
    Update(UpdateArgs),
    /// Delete a ticket
    Delete { id: i64 },
    /// List all tickets
    List(ListArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(short = 'n', long)]
    pub name: String,
    #[arg(short = 'd', long)]
    pub desc: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    pub id: i64,
    #[arg(short = 'n', long)]
    pub name: Option<String>,
    #[arg(short = 'd', long)]
    pub desc: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Show created/updated timestamps
    #[arg(long)]
    pub timestamps: bool,
}
```

Clap automatically exits with code 2 and prints to stderr when required args are missing. This
satisfies CLI-05 for usage errors without extra code.

### Pattern 2: Database Connection Setup (WAL + Migrations)

**What:** Open connection, configure WAL mode and synchronous=NORMAL, run migrations.

**When to use:** Called once at startup, before any CRUD.

```rust
// Source: https://docs.rs/rusqlite/latest/rusqlite/ and rusqlite_migration docs
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

const MIGRATIONS: &[M] = &[
    M::up(
        "CREATE TABLE IF NOT EXISTS tickets (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            status      TEXT NOT NULL DEFAULT 'todo',
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );"
    ),
];

pub fn open_db(path: &std::path::Path) -> rusqlite::Result<Connection> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    let migrations = Migrations::from_slice(MIGRATIONS);
    migrations.to_latest(&mut conn)
              .expect("migration failed");
    Ok(conn)
}
```

rusqlite_migration uses SQLite's `user_version` pragma to track applied migrations — no extra table
needed. `from_slice` works on a static slice of `M` values, which is the easiest embed pattern.

### Pattern 3: Directory-Walking DB Discovery

**What:** Walk parent dirs from `cwd` looking for `.rtik.db`; fall back to creating in `cwd`.

**When to use:** Called before `open_db`, resolves the path to pass in.

```rust
pub fn resolve_db_path() -> std::path::PathBuf {
    // 1. Env var override
    if let Ok(path) = std::env::var("RTIK_DB") {
        return std::path::PathBuf::from(path);
    }
    // 2. Walk up from cwd
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
    // 3. Auto-create in cwd
    std::env::current_dir()
        .expect("cannot read cwd")
        .join(".rtik.db")
}
```

### Pattern 4: CRUD via rusqlite

**What:** Standard insert, query_row, query_map, execute patterns.

```rust
// Source: https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html

// INSERT — returns last inserted id
pub fn create_ticket(conn: &Connection, name: &str, desc: &str) -> rusqlite::Result<i64> {
    let now = chrono_or_manual_utc_string(); // see pitfalls
    conn.execute(
        "INSERT INTO tickets (name, description, status, created_at, updated_at)
         VALUES (?1, ?2, 'todo', ?3, ?3)",
        rusqlite::params![name, desc, now],
    )?;
    Ok(conn.last_insert_rowid())
}

// SELECT one — query_row returns Err(QueryReturnedNoRows) on not-found
pub fn get_ticket(conn: &Connection, id: i64) -> rusqlite::Result<Ticket> {
    conn.query_row(
        "SELECT id, name, description, status, created_at, updated_at
         FROM tickets WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Ticket {
                id:          row.get(0)?,
                name:        row.get(1)?,
                description: row.get(2)?,
                status:      row.get(3)?,
                created_at:  row.get(4)?,
                updated_at:  row.get(5)?,
            })
        },
    )
}

// SELECT all — query_map collects into Vec
pub fn list_tickets(conn: &Connection) -> rusqlite::Result<Vec<Ticket>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, status, created_at, updated_at
         FROM tickets ORDER BY id ASC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Ticket {
            id:          row.get(0)?,
            name:        row.get(1)?,
            description: row.get(2)?,
            status:      row.get(3)?,
            created_at:  row.get(4)?,
            updated_at:  row.get(5)?,
        })
    })?;
    rows.collect()
}

// DELETE — check rows_affected to detect not-found
pub fn delete_ticket(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let affected = conn.execute("DELETE FROM tickets WHERE id = ?1", rusqlite::params![id])?;
    Ok(affected > 0)
}
```

### Pattern 5: Dynamic UPDATE for partial field updates

**What:** Build the SET clause at runtime based on which flags were provided.

**When to use:** `update` subcommand where all three fields are optional but at least one required.

```rust
pub fn update_ticket(
    conn: &Connection,
    id: i64,
    name: Option<&str>,
    desc: Option<&str>,
    status: Option<&str>,
) -> rusqlite::Result<bool> {
    let now = utc_now_string();
    let mut sets: Vec<&str> = Vec::new();
    // Build conditionally — see pitfall about SQL injection via column names
    // This pattern is safe because column names are hard-coded strings, not user input
    if name.is_some()   { sets.push("name = ?2"); }
    if desc.is_some()   { sets.push("description = ?3"); }
    if status.is_some() { sets.push("status = ?4"); }
    // sets is guaranteed non-empty by caller validation
    let sql = format!(
        "UPDATE tickets SET {}, updated_at = ?5 WHERE id = ?1",
        sets.join(", ")
    );
    // Note: params must line up with placeholders — use params_from_iter or manual match
    // ...bind name/desc/status in positions 2/3/4, now in position 5
    Ok(conn.execute(&sql, ...)? > 0)
}
```

Note: Positional parameter binding gets complicated with optional fields. Use `rusqlite::params![]`
with `Option<&str>` bindings and adjust SQL to only reference present values.

### Pattern 6: Broken Pipe Fix (CLI-06)

```rust
// Source: https://docs.rs/sigpipe/latest/sigpipe/
fn main() {
    sigpipe::reset(); // must be first line — resets SIGPIPE before any I/O
    // ... rest of main
}
```

Without this, `rtik list | head -1` causes a panic with "failed printing to stdout: Broken pipe".

### Anti-Patterns to Avoid

- **Storing timestamps in SQLite with DEFAULT CURRENT_TIMESTAMP:** Returns local time, not UTC. Use
  application-side `utc_now_string()` or `strftime('%Y-%m-%dT%H:%M:%SZ', 'now')` in SQL.
- **Using `rusqlite::Error::QueryReturnedNoRows` as "not found":** This is correct — but must be
  matched explicitly; don't unwrap `query_row` results without checking for this variant.
- **Calling `process::exit()` inside library functions:** Library code should return errors; only
  `main.rs` calls `process::exit()`.
- **Forgetting to check rows_affected on DELETE/UPDATE:** Zero rows means the ID doesn't exist — must
  return an appropriate error, not silent success.
- **Building the binary without `bundled` feature:** Without it, the binary depends on the system's
  `libsqlite3.so`; violates TECH-05.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Argument parsing | Custom argv parser | clap 4 derive | Handles help, validation, short/long flags, exit codes automatically |
| Schema versioning | `IF NOT EXISTS` on every boot | rusqlite_migration | Handles ordering, idempotency, `user_version` tracking |
| Broken pipe handling | `if err.kind() == BrokenPipe { exit(0) }` everywhere | `sigpipe::reset()` | Rust ignores SIGPIPE at the OS level — must be reset before any I/O |
| SQLite static linking | Bundling libsqlite3 manually | rusqlite `bundled` feature | Cargo feature handles compile-time bundling correctly |

**Key insight:** The two most dangerous hand-roll traps are schema migrations (easy to get ordering
wrong, hard to recover) and broken-pipe (appears to work until piped to `head`, then panics in prod).

## Common Pitfalls

### Pitfall 1: Dynamic UPDATE SQL with positional parameters

**What goes wrong:** Building a dynamic SET clause like `"name = ?2, status = ?4"` and then binding
parameters linearly causes position mismatches when some fields are None.

**Why it happens:** rusqlite positional params (`?1`, `?2`) don't skip None values — you have to
either remap positions or use named params.

**How to avoid:** Two approaches:
1. Always bind all three values (name/desc/status) and only include SET entries for non-None ones —
   the skipped positional params don't need to appear in SQL, they just need to be bound at the right
   slot.
2. Use named parameters `(:name, :desc, :status)` with `named_params![]` macro and only include
   non-None fields in SET.

**Warning signs:** "wrong number of parameters" or "invalid column index" errors at runtime.

### Pitfall 2: Timestamp format inconsistency

**What goes wrong:** SQLite's `CURRENT_TIMESTAMP` returns `YYYY-MM-DD HH:MM:SS` (no T, no Z). If
you later format application-side timestamps as ISO 8601, the data is inconsistent.

**Why it happens:** SQLite CURRENT_TIMESTAMP uses SQL standard format, not ISO 8601.

**How to avoid:** Always generate timestamps application-side with a consistent format:
```rust
fn utc_now_string() -> String {
    // Using std only (no chrono dep needed for Phase 1):
    // SystemTime → Duration since epoch → format manually
    // OR use strftime('%Y-%m-%dT%H:%M:%SZ', 'now') in SQL
}
```
The SQL approach `strftime('%Y-%m-%dT%H:%M:%SZ', 'now')` is simplest and avoids a chrono dependency.

### Pitfall 3: Missing rows_affected check on DELETE/UPDATE

**What goes wrong:** `conn.execute("DELETE ... WHERE id = ?1", ...)` returns `Ok(0)` for a missing
ID. If you don't check, you silently "succeed" and print "Deleted: #999" for a ticket that doesn't
exist.

**How to avoid:** Always match on the returned row count:
```rust
match conn.execute(...)? {
    0 => Err(AppError::NotFound(id)),
    _ => Ok(()),
}
```

### Pitfall 4: WAL mode pragma not persisting across connections

**What goes wrong:** Journal mode is a connection-level setting that persists to the file. However,
if you open the DB before setting WAL mode on the first run, the DB file is created in the default
journal mode.

**How to avoid:** Always set pragmas immediately after `Connection::open()`, before any other
operations. The migration setup already enforces this order.

### Pitfall 5: Status normalization forgotten on input

**What goes wrong:** User passes `WIP` and it gets stored as `WIP`; later comparisons against `wip`
fail.

**How to avoid:** Normalize to lowercase immediately upon CLI arg parsing, before any DB call:
```rust
let status_normalized = status.to_lowercase();
// validate: must be one of "todo", "wip", "blocked", "done"
```

### Pitfall 6: Clap exit codes vs application exit codes conflated

**What goes wrong:** Clap exits with 2 for usage errors automatically. Application errors (ticket not
found, DB error) should exit with 1. If you call `process::exit(2)` for app errors, it misleads
scripts.

**How to avoid:** Let Clap handle its own exit. In `main.rs`, after parsing, only call
`process::exit(1)` for runtime errors. Never call `process::exit(2)` manually.

## Code Examples

Verified patterns from official sources:

### Connection Open + WAL + Migration (startup sequence)

```rust
// Source: https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html
//         https://docs.rs/rusqlite_migration/latest/rusqlite_migration/struct.Migrations.html
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

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
];

pub fn open_connection(path: &std::path::Path) -> anyhow::Result<Connection> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Migrations::from_slice(MIGRATIONS).to_latest(&mut conn)?;
    Ok(conn)
}
```

### rusqlite INSERT + last_insert_rowid

```rust
// Source: https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html
conn.execute(
    "INSERT INTO tickets (name, description) VALUES (?1, ?2)",
    rusqlite::params![name, description],
)?;
let id = conn.last_insert_rowid();
```

### rusqlite query_row (single result)

```rust
// Returns Err(rusqlite::Error::QueryReturnedNoRows) if not found
let ticket: Ticket = conn.query_row(
    "SELECT id, name, description, status, created_at, updated_at
     FROM tickets WHERE id = ?1",
    rusqlite::params![id],
    |row| Ok(Ticket {
        id:          row.get(0)?,
        name:        row.get(1)?,
        description: row.get(2)?,
        status:      row.get(3)?,
        created_at:  row.get(4)?,
        updated_at:  row.get(5)?,
    }),
)?;
```

### rusqlite query_map (list)

```rust
// Source: https://docs.rs/rusqlite/latest/rusqlite/struct.Statement.html
let mut stmt = conn.prepare(
    "SELECT id, name, description, status, created_at, updated_at
     FROM tickets ORDER BY id ASC"
)?;
let tickets: Vec<Ticket> = stmt.query_map([], |row| {
    Ok(Ticket { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?,
                status: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)? })
})?.collect::<rusqlite::Result<_>>()?;
```

### main.rs entry point structure

```rust
// Source: https://docs.rs/sigpipe/latest/sigpipe/
// Source: https://rust-cli.github.io/book/in-depth/exit-code.html
use clap::Parser;

fn main() {
    sigpipe::reset(); // CLI-06: must be first

    let cli = rtik::cli::Cli::parse(); // exits with 2 on usage error (CLI-05)
    let db_path = rtik::db::resolve_db_path();
    let conn = match rtik::db::open_connection(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: cannot open database: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = rtik::run(cli, conn) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

### Clap status validation with error

```rust
// Source: clap derive docs + Rust CLI book
fn parse_status(raw: &str) -> Result<String, String> {
    let normalized = raw.to_lowercase();
    match normalized.as_str() {
        "todo" | "wip" | "blocked" | "done" => Ok(normalized),
        _ => Err(format!(
            "invalid status '{}': must be one of todo, wip, blocked, done",
            raw
        )),
    }
}
// Use as: #[arg(long, value_parser = parse_status)]
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| clap builder API | clap derive API | clap 3→4 | Derive is idiomatic; builder still works but verbose |
| sqlite3 system lib | rusqlite `bundled` feature | rusqlite ~0.25 | Single binary; no system dependency |
| Manual migration SQL in code | rusqlite_migration | ~2021 | Ordered, versioned, embedded in binary |
| Custom SIGPIPE check | `sigpipe::reset()` | ~2020 | One line vs scattered error checks |

**Deprecated/outdated:**
- `rusqlite::NO_PARAMS`: replaced with `[]` (empty slice) or `rusqlite::params![]` macro
- `chrono` for timestamp generation in Phase 1: unnecessary — SQLite's `strftime` handles UTC timestamps

## Open Questions

1. **Timestamp storage format: SQL DEFAULT vs application-side binding**
   - What we know: SQLite `strftime('%Y-%m-%dT%H:%M:%SZ','now')` generates ISO 8601 UTC correctly
   - What's unclear: Whether `updated_at` should be a SQL DEFAULT or always bound from Rust code
   - Recommendation: Use SQL DEFAULT for `created_at`; always bind `updated_at` explicitly in UPDATE
     statements (SQL DEFAULT only fires on INSERT)

2. **Table alignment in `rtik list` output**
   - What we know: User wants aligned columns (ID, STATUS, NAME)
   - What's unclear: Whether to use a crate (comfy-table, tabled) or manual `format!("{:<width}")` padding
   - Recommendation: Use `format!("{:<5} {:<10} {}", id, status, name)` with manual padding — keeps zero
     extra dependencies; Phase 3 can upgrade if needed

3. **TECH-02 in REQUIREMENTS.md lists TECH-03 (atomic claim) as Phase 2 scope**
   - What we know: WAL mode (TECH-02) is Phase 1; atomic claim operations (TECH-03) are Phase 2
   - What's unclear: N/A — clear boundary
   - Recommendation: Enable WAL mode in Phase 1 schema setup; don't implement claim logic yet

## Sources

### Primary (HIGH confidence)
- https://docs.rs/rusqlite/latest/rusqlite/ — version 0.38.0, Connection API, pragma_update
- https://docs.rs/rusqlite_migration/latest/rusqlite_migration/struct.Migrations.html — Migrations API, M struct, from_slice
- https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html — derive subcommands, Args, positional args
- https://docs.rs/clap/latest/clap/ — version 4.5.60 confirmed
- https://docs.rs/sigpipe/latest/sigpipe/ — version 0.1.3, sigpipe::reset()
- https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html — rusqlite INSERT/query_map patterns

### Secondary (MEDIUM confidence)
- https://rust-cli.github.io/book/in-depth/exit-code.html — exit code conventions (verified with clap docs)
- https://sqlite.org/wal.html — WAL mode synchronous=NORMAL recommendation (official SQLite docs)
- https://cj.rs/blog/sqlite-pragma-cheatsheet-for-performance-and-consistency/ — pragma setup pattern
- https://github.com/rust-lang/rust/issues/46016 — SIGPIPE issue context

### Tertiary (LOW confidence)
- WebSearch results on broken pipe solutions — multiple crates exist; sigpipe chosen for simplicity

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified from docs.rs
- Architecture: HIGH — idiomatic patterns verified from official Rust and Clap docs
- Pitfalls: MEDIUM — dynamic UPDATE SQL and SIGPIPE are verified; timestamp pitfalls from official SQLite docs
- Code examples: HIGH — patterns from official rusqlite/clap documentation

**Research date:** 2026-02-22
**Valid until:** 2026-08-22 (stable ecosystem; rusqlite and clap are mature, slow-moving crates)
