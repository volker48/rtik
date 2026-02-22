# Architecture Research

**Domain:** CLI ticketing/task management system
**Researched:** 2026-02-22
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      CLI Interface Layer                     │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │  List   │  │ Create  │  │ Update  │  │ Claim   │  ...    │
│  │ Command │  │ Command │  │ Command │  │ Command │         │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘         │
│       │            │            │            │               │
├───────┴────────────┴────────────┴────────────┴───────────────┤
│                   Business Logic Layer                       │
│  ┌──────────────────────────────────────────────────┐        │
│  │            Ticket Operations Service             │        │
│  │  - Validation   - Dependency Resolution          │        │
│  │  - Status State - Claim/Ownership Logic          │        │
│  └────────────────────┬─────────────────────────────┘        │
│                       │                                      │
├───────────────────────┴──────────────────────────────────────┤
│                   Data Access Layer                          │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │ Ticket Repository│  │ Query Builder    │                 │
│  │ (CRUD ops)       │  │ (Search/Filter)  │                 │
│  └────────┬─────────┘  └────────┬─────────┘                 │
│           │                     │                            │
├───────────┴─────────────────────┴────────────────────────────┤
│                   Persistence Layer                          │
│  ┌─────────────────────────────────────────────────────┐     │
│  │            SQLite Database (tickets.db)             │     │
│  │  - Schema Migrations  - Connection Management      │     │
│  └─────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| CLI Interface Layer | Parse commands, handle user input, format output | Clap (derive-based) for argument parsing, commands as subcommands |
| Business Logic Layer | Ticket validation, state transitions, dependency resolution | Pure Rust functions, no external dependencies, fully testable |
| Data Access Layer | Abstract database operations, query building, result mapping | Repository pattern with rusqlite, prepared statements for queries |
| Persistence Layer | Schema management, connection pooling, transactions | SQLite via rusqlite, migration system for schema changes |

## Recommended Project Structure

```
rtik/
├── src/
│   ├── main.rs              # Entry point, CLI setup, minimal logic
│   ├── lib.rs               # Public API, module declarations
│   ├── cli/
│   │   ├── mod.rs           # CLI command definitions (Clap)
│   │   ├── commands/        # Individual command implementations
│   │   │   ├── create.rs    # Create ticket command
│   │   │   ├── list.rs      # List tickets command
│   │   │   ├── update.rs    # Update ticket command
│   │   │   ├── claim.rs     # Claim ticket command
│   │   │   └── export.rs    # Export command
│   │   └── output.rs        # Output formatting/rendering
│   ├── domain/
│   │   ├── mod.rs           # Domain module root
│   │   ├── ticket.rs        # Ticket entity, validation, state machine
│   │   ├── operations.rs    # Business logic for ticket operations
│   │   └── errors.rs        # Domain-specific error types
│   ├── data/
│   │   ├── mod.rs           # Data access module root
│   │   ├── repository.rs    # Repository trait & implementation
│   │   ├── models.rs        # Database models (may differ from domain)
│   │   ├── migrations.rs    # Schema migrations
│   │   └── queries.rs       # SQL query builders
│   └── config/
│       ├── mod.rs           # Configuration handling
│       └── database.rs      # DB connection, path resolution
├── tests/
│   ├── integration/         # End-to-end CLI tests
│   └── common/              # Test utilities
├── Cargo.toml
└── README.md
```

### Structure Rationale

- **src/main.rs minimal:** Thin entry point, delegates to lib.rs for testability
- **src/lib.rs as library:** Makes entire CLI testable as a library, enables integration tests
- **cli/ for interface:** Isolates Clap dependencies, command parsing from business logic
- **domain/ for business logic:** Pure Rust, no database/CLI dependencies, fully testable without I/O
- **data/ for persistence:** Database concerns isolated, can swap rusqlite for another storage later
- **config/ for configuration:** DB path resolution, environment handling separate from core logic

## Architectural Patterns

### Pattern 1: Thin Main, Rich Library

**What:** main.rs contains only CLI setup and error handling, all logic in lib.rs
**When to use:** Always for CLI applications that need testing
**Trade-offs:**
- Pro: Business logic fully testable without spawning process
- Pro: Can reuse logic in other contexts (embedded, WASM, etc.)
- Con: Slightly more boilerplate with module declarations

**Example:**
```rust
// src/main.rs
use rtik::cli;
use std::process;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

// src/lib.rs
pub mod cli;
pub mod domain;
pub mod data;
```

### Pattern 2: Repository Pattern for Data Access

**What:** Abstract database operations behind a trait, concrete implementation with rusqlite
**When to use:** When business logic shouldn't know about database details
**Trade-offs:**
- Pro: Business logic testable with mock repositories
- Pro: Easy to swap storage backend (SQLite -> PostgreSQL)
- Con: Extra abstraction layer for simple CRUD

**Example:**
```rust
// src/data/repository.rs
pub trait TicketRepository {
    fn create(&self, ticket: &NewTicket) -> Result<Ticket>;
    fn find_by_id(&self, id: u64) -> Result<Option<Ticket>>;
    fn list(&self, filter: &TicketFilter) -> Result<Vec<Ticket>>;
}

pub struct SqliteTicketRepository {
    conn: Connection,
}

impl TicketRepository for SqliteTicketRepository {
    // Implementation using rusqlite
}
```

### Pattern 3: Command Pattern with Clap Subcommands

**What:** Each CLI command is a separate module with its own struct and execute logic
**When to use:** CLI with multiple distinct operations (create, list, update, etc.)
**Trade-offs:**
- Pro: Easy to add new commands without touching existing code
- Pro: Each command can have its own argument parsing
- Con: More files to navigate for simple CLIs

**Example:**
```rust
// src/cli/commands/create.rs
use clap::Args;

#[derive(Debug, Args)]
pub struct CreateArgs {
    #[arg(short, long)]
    name: String,

    #[arg(short, long)]
    description: String,
}

impl CreateArgs {
    pub fn execute(&self, repo: &dyn TicketRepository) -> Result<()> {
        // Business logic call
    }
}
```

### Pattern 4: Domain-Driven Status State Machine

**What:** Status transitions controlled by domain logic, not database constraints
**When to use:** When state transitions have business rules (e.g., can't go from done -> todo)
**Trade-offs:**
- Pro: Business rules explicit in code, easier to test
- Pro: Can add complex transition logic (notifications, side effects)
- Con: Database can get out of sync if bypassed

**Example:**
```rust
// src/domain/ticket.rs
pub enum Status {
    Todo,
    WIP,
    Blocked,
    Done,
}

impl Ticket {
    pub fn transition_to(&mut self, new_status: Status) -> Result<()> {
        match (&self.status, &new_status) {
            (Status::Done, Status::Todo) => Err("Cannot reopen done ticket"),
            (Status::Blocked, Status::Done) => Err("Cannot complete blocked ticket"),
            _ => {
                self.status = new_status;
                self.updated_at = Utc::now();
                Ok(())
            }
        }
    }
}
```

## Data Flow

### Request Flow

```
[User Command: rtik create -n "Bug fix" -d "Fix login"]
    ↓
[CLI Parser (Clap)] → parses args into CreateArgs
    ↓
[Command Handler] → validates inputs
    ↓
[Business Logic] → creates Ticket entity, validates state
    ↓
[Repository] → converts to DB model, prepares SQL
    ↓
[Database] → INSERT, returns ID
    ↓
[Response] ← format success message
    ↓
[Terminal Output] ← "Created ticket #42: Bug fix"
```

### Query Flow (List/Search)

```
[User Command: rtik list --status todo]
    ↓
[CLI Parser] → parses into ListArgs with filter
    ↓
[Repository] → builds SELECT with WHERE clause
    ↓
[Database] → executes query, returns rows
    ↓
[Repository] → maps rows to Ticket entities
    ↓
[Output Formatter] → renders as table/plain text
    ↓
[Terminal Output] ← formatted ticket list
```

### Key Data Flows

1. **Create Flow:** CLI → Validation → Domain entity creation → Repository → Database → Response
2. **Query Flow:** CLI → Filter building → Repository query → Database → Entity mapping → Formatting
3. **Claim Flow:** CLI → Check availability → Update owner → Repository → Database → Response
4. **Export Flow:** CLI → Query all tickets → Filter → Plain text rendering → File/stdout

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1k tickets | Single SQLite file, no indexes beyond primary key, simple queries |
| 1k-10k tickets | Add indexes on status, created_at, claimed_by for faster filtering |
| 10k-100k tickets | Consider connection pooling (rusqlite doesn't need it, but prepare_cached helps), batch operations for bulk updates |
| 100k+ tickets | Archive old/done tickets, implement pagination for list commands, consider read-only replicas if multiple agents query simultaneously |

### Scaling Priorities

1. **First bottleneck:** List command becomes slow — Add indexes on frequently filtered columns (status, created_at)
2. **Second bottleneck:** Concurrent writes from multiple agents — Use WAL mode in SQLite for better concurrency
3. **Later bottleneck:** Database file size — Implement archive command to move old tickets to separate DB

## Anti-Patterns

### Anti-Pattern 1: Database Logic in CLI Commands

**What people do:** Put SQL queries directly in command handler functions
**Why it's wrong:** Makes commands untestable, couples CLI to database, violates separation of concerns
**Do this instead:** Use repository pattern, command handlers call domain logic which calls repository

### Anti-Pattern 2: Parsing Business Logic in main.rs

**What people do:** Handle ticket creation, validation, state transitions in main.rs
**Why it's wrong:** Cannot test without spawning process, cannot reuse logic elsewhere
**Do this instead:** main.rs only handles CLI setup and error formatting, delegate to lib.rs

### Anti-Pattern 3: Mixing Domain Models and Database Models

**What people do:** Use the same struct for domain entities and database rows, include rusqlite types in domain
**Why it's wrong:** Domain logic becomes coupled to database schema, hard to change storage
**Do this instead:** Separate domain models (src/domain/) from DB models (src/data/models.rs), map between them

### Anti-Pattern 4: Hard-Coding Database Path

**What people do:** `Connection::open("./tickets.db")` scattered throughout codebase
**Why it's wrong:** Can't test with temporary DBs, can't configure location, fails in different environments
**Do this instead:** Config module with environment variable support, dependency injection of connection

### Anti-Pattern 5: Ignoring SQLite Transactions for Multi-Step Operations

**What people do:** Multiple separate UPDATE/INSERT calls for operations that should be atomic
**Why it's wrong:** Partial failures leave database in inconsistent state, race conditions
**Do this instead:** Wrap multi-step operations in explicit transactions using rusqlite's transaction API

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| SQLite Database | Direct rusqlite connection | Single-file database, no server needed, use WAL mode for concurrency |
| File System | Standard library (std::fs) | For export operations, reading config files |
| Environment | std::env | For DB path, agent ID configuration |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI ↔ Domain | Function calls via commands | Commands construct domain entities, call operations |
| Domain ↔ Data | Repository trait | Domain code depends on trait, not concrete implementation |
| Data ↔ SQLite | rusqlite API | Only data layer imports rusqlite, domain never sees it |
| CLI ↔ Output | Formatter functions | Commands pass data, formatters handle rendering (table, JSON, plain text) |

## Build Order Recommendations

### Component Dependencies (Directed Acyclic Graph)

```
main.rs
   ↓
lib.rs
   ↓
├── config (no dependencies)
│
├── domain (depends on: config)
│   └── errors (no dependencies)
│
├── data (depends on: domain, config)
│   └── migrations (depends on: data)
│
└── cli (depends on: domain, data, config)
    └── commands (depends on: cli, domain, data)
```

### Suggested Build Order for Development

1. **Phase 1: Persistence Foundation**
   - Start with `config/database.rs` — DB path resolution, connection management
   - Build `data/migrations.rs` — Schema creation, table setup
   - Implement basic `data/repository.rs` — CRUD operations without business logic

2. **Phase 2: Domain Core**
   - Define `domain/ticket.rs` — Ticket struct, Status enum, basic validation
   - Add `domain/errors.rs` — Domain-specific error types
   - Build `domain/operations.rs` — Business logic for ticket operations

3. **Phase 3: CLI Interface**
   - Setup `cli/mod.rs` — Clap CLI structure, main command definitions
   - Implement `cli/commands/create.rs` — First command to test end-to-end
   - Add `cli/output.rs` — Basic formatting for responses

4. **Phase 4: Feature Expansion**
   - Add remaining commands (list, update, claim, export)
   - Enhance query builder in `data/queries.rs`
   - Add filtering and search capabilities

5. **Phase 5: Polish**
   - Comprehensive error handling
   - Output formatting options (table, JSON, plain text)
   - Shell completion generation

### Why This Order

- **Database first:** Can't test anything without persistence working
- **Domain after data:** Need to save/load before business logic matters
- **CLI last:** Thin layer over domain, easy to add once logic works
- **Vertical slices:** Get one complete command working (create) before adding others
- **Dependencies flow up:** Each layer only depends on layers below, enabling parallel work

## Sources

- [Rust CLI Book - Getting Started](https://rust-cli.github.io/book/index.html)
- [Kevin K's Blog - CLI Structure in Rust](https://kbknapp.dev/cli-structure-01/)
- [Taskwarrior Documentation](https://taskwarrior.org/docs/)
- [Task Tracker CLI - roadmap.sh](https://roadmap.sh/projects/task-tracker)
- [Rusqlite DeepWiki - Architecture Overview](https://deepwiki.com/rusqlite/rusqlite/1-overview)
- [Martin Fowler - Presentation Domain Data Layering](https://martinfowler.com/bliki/PresentationDomainDataLayering.html)
- [Rust Package Layout - Cargo Book](https://doc.rust-lang.org/cargo/guide/project-layout.html)
- [Medium - Rust Modules and Project Structure](https://medium.com/codex/rust-modules-and-project-structure-832404a33e2e)
- [GeeksforGeeks - Business Logic Layer](https://www.geeksforgeeks.org/dbms/business-logic-layer/)
- [GeeksforGeeks - Data Access Layer](https://www.geeksforgeeks.org/dbms/data-access-layer/)
- [Tweag - Introduction to Dependency Graph](https://www.tweag.io/blog/2025-09-04-introduction-to-dependency-graph/)
- [vFunction - Software Dependencies](https://vfunction.com/blog/software-dependencies/)

---
*Architecture research for: CLI ticketing/task management system*
*Researched: 2026-02-22*
