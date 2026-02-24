# Phase 3: Search, Filtering & Export - Research

**Researched:** 2026-02-23
**Domain:** Clap 4 alias API, dynamic SQL filtering, serde_json export — all in existing Rust/rusqlite codebase
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Filters specified as `--flag` style (e.g. `rtik ls --status open --unclaimed`)
- Multiple filters always compose with AND — no OR mode
- `--search` is a flag on the existing `ls` command, not a separate subcommand
- `--status` accepts whatever status values are already stored in the DB (no new enum defined here)
- Case-insensitive always
- `--search` searches name AND description together (no separate flags)
- Multiple `--search` values supported and AND-matched (both terms must appear somewhere)
- `--search` is fully composable with status/claimed/claimer filters — all conditions AND together
- Separate `export` subcommand (not a flag on `ls`)
- `export` accepts the same filter flags as `ls` — same filtering, different output
- Plain text format is one-line summary per ticket: `T-1 [open] Fix auth bug deps:T-2,T-3`
  - Fields: ID, status, name, dep list (IDs) — claimer omitted from plain text
- JSON format available via `--json` flag on the `export` command
- All commands get compiled-in aliases (no user config)
- Style: Unix-style abbreviations
- Alias set: `new`, `ls`, `get`, `up`, `rm`, `claim`, `rel`, `dep`, `deps`, `dump` (for export)
- Aliases are not configurable — baked into the binary

### Claude's Discretion

- Exact one-line format punctuation/spacing (e.g. brackets vs parens for status)
- How `--unclaimed` vs `--claimed` flags are surfaced (one flag with value, or two booleans)
- How `--claimer <name>` interacts with `--claimed` / `--unclaimed`
- JSON schema shape for export

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| QUERY-01 | User can filter tickets by status | Dynamic SQL WHERE with `?status` param; extend `list_tickets` to accept `ListFilter` struct |
| QUERY-02 | User can filter tickets by claimed status (claimed vs unclaimed) | `claimed_by IS NOT NULL` / `IS NULL` SQL conditions; boolean `--claimed`/`--unclaimed` flags in Clap |
| QUERY-03 | User can filter tickets by claimer (claimed_by value) | Exact match on `claimed_by` column via named param |
| QUERY-04 | User can search tickets by name (substring match) | SQLite `LIKE '%?%'` with lowercased input; `LOWER(name) LIKE ?` pattern |
| QUERY-05 | User can search tickets by description (substring match) | Same LIKE approach; `LOWER(name) LIKE ? OR LOWER(description) LIKE ?` per search term |
| QUERY-06 | User can combine multiple filters in single query | AND-compose all WHERE clauses in dynamic SQL builder; same pattern as existing `update_ticket` |
| EXPORT-01 | User can export tickets to plain text format | New `Export` subcommand variant in `Commands` enum |
| EXPORT-02 | Plain text export includes: ID, name, description, dependencies | Query ticket + deps per ticket; format one-line per ticket |
| EXPORT-03 | Plain text export is token-efficient (minimal verbosity) | One-line format decided by user: `T-{id} [{status}] {name} deps:{d1},{d2}` |
| EXPORT-04 | User can export tickets to JSON format | `serde` + `serde_json` dependencies; `--json` flag on `export` command; `#[derive(Serialize)]` on export struct |
| CLI-01 | CLI provides short alias 'new' for create command | `#[command(alias = "new")]` on `Create` variant |
| CLI-02 | CLI provides short alias 'ls' for list command | `#[command(alias = "ls")]` on `List` variant |
| CLI-03 | CLI provides short alias 'claim' for claim command | Already named `Claim`; need to check if lowercase parsing is automatic or needs alias |
</phase_requirements>

## Summary

Phase 3 adds filtering, search, and export to an existing Rust CLI. The codebase already uses Clap 4.5 (derive API), rusqlite 0.38, and thiserror 2. No new architectural decisions are needed — the patterns are direct extensions of what already exists.

For filtering and search, the approach is to extend `list_tickets` to accept a `ListFilter` struct and build SQL dynamically using a WHERE clause builder, exactly like `update_ticket` already builds a dynamic SET clause. SQLite's LIKE operator is case-insensitive for ASCII by default; lowercasing both sides with `LOWER()` handles the full range. Multiple `--search` terms each add a `(LOWER(name) LIKE ? OR LOWER(description) LIKE ?)` clause, all AND-joined.

For export, a new `Export` subcommand reuses the same `ListFilter` struct and adds a `--json` flag. The plain text format is one line per ticket. JSON export requires adding `serde` and `serde_json` as new dependencies (the only new deps this phase). Clap command aliases use `#[command(alias = "name")]` on enum variants — this is a raw attribute that forwards to the builder API. Multiple aliases require multiple `#[command(alias = "...")]` annotations.

**Primary recommendation:** Build one `ListFilter` struct shared by `List` and `Export`; all filtering logic lives in `ticket::list_tickets_filtered`; aliases are `#[command(alias = "...")]` on each `Commands` variant.

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.5.60 | CLI parsing, alias support | Already in use; derive macros |
| rusqlite | 0.38.0 | Dynamic SQL filtering | Already in use |
| thiserror | 2.0.18 | Error types | Already in use |

### New Dependencies (this phase only)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.x | Derive Serialize for JSON export | De-facto Rust serialization standard |
| serde_json | 1.0.149 | Serialize structs to JSON string/stdout | The only serde JSON implementation |

**Installation:**
```bash
cargo add serde --features derive
cargo add serde_json
```

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| serde_json | hand-built JSON string | Hand-rolling JSON is fragile; special chars in names/descriptions need escaping |
| SQLite LOWER() | application-side lowercase | Application-side is fine too but SQL-side is simpler and keeps filtering in one place |

## Architecture Patterns

### Recommended File Changes
```
src/
├── cli.rs       # Add filter flags to ListArgs; new ExportArgs; aliases on all Commands variants
├── ticket.rs    # Add ListFilter struct; add list_tickets_filtered(); add export formatting fns
└── lib.rs       # Add Export match arm; wire filter args to ListFilter
```

No new files are needed. All changes extend existing modules.

### Pattern 1: Shared Filter Struct

**What:** Define `ListFilter` once; pass it to both `list_tickets_filtered` (for `ls`) and the export query.
**When to use:** Anytime two commands need identical filtering semantics.

```rust
// In ticket.rs — the filter params for list and export
pub struct ListFilter {
    pub status: Option<String>,
    pub claimed: Option<bool>,      // None=no filter, Some(true)=claimed, Some(false)=unclaimed
    pub claimer: Option<String>,
    pub search: Vec<String>,        // AND-joined; each term must appear in name or description
}
```

### Pattern 2: Dynamic WHERE Clause Builder

**What:** Build SQL WHERE conditions and positional params at runtime, matching the existing `update_ticket` pattern.
**When to use:** Whenever filter set is optional and combinable.

```rust
// In ticket.rs — mirrors the update_ticket dynamic SET approach
pub fn list_tickets_filtered(conn: &Connection, filter: &ListFilter) -> Result<Vec<TicketRow>, AppError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref s) = filter.status {
        conditions.push("status = ?".to_string());
        params.push(Box::new(s.clone()));
    }
    if let Some(claimed) = filter.claimed {
        if claimed {
            conditions.push("claimed_by IS NOT NULL".to_string());
        } else {
            conditions.push("claimed_by IS NULL".to_string());
        }
    }
    if let Some(ref claimer) = filter.claimer {
        conditions.push("claimed_by = ?".to_string());
        params.push(Box::new(claimer.clone()));
    }
    for term in &filter.search {
        let pattern = format!("%{}%", term.to_lowercase());
        // One condition per term: must appear in name OR description
        conditions.push("(LOWER(name) LIKE ? OR LOWER(description) LIKE ?)".to_string());
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT id, name, description, status, claimed_by, created_at, updated_at \
         FROM tickets {} ORDER BY id ASC",
        where_clause
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        // map row to struct
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(AppError::Db)
}
```

**Key detail:** rusqlite's `query_map` with a positional params slice `&[&dyn ToSql]` works cleanly for dynamic queries. Named params cannot be used when the param count is dynamic.

### Pattern 3: Clap Command Aliases (Derive API)

**What:** `#[command(alias = "name")]` is a raw attribute that forwards to `Command::alias()` in the builder API.
**When to use:** Any subcommand that needs a short name.

```rust
// In cli.rs — one alias attr per alias; raw attributes support any Command method
#[derive(Subcommand)]
pub enum Commands {
    /// Create a new ticket
    #[command(alias = "new")]
    Create(CreateArgs),

    /// List tickets (with optional filters)
    #[command(alias = "ls")]
    List(ListArgs),

    /// Show ticket details
    #[command(alias = "get")]
    Get { id: i64 },

    /// Update ticket fields
    #[command(alias = "up")]
    Update(UpdateArgs),

    /// Delete a ticket
    #[command(alias = "rm")]
    Delete { id: i64 },

    /// Claim a ticket for this agent
    // "claim" is already the canonical lowercase — Clap lowercases variant names by default.
    // Add explicit alias anyway to be explicit.
    #[command(alias = "claim")]
    Claim(ClaimArgs),

    /// Release a claimed ticket
    #[command(alias = "rel")]
    Release(ReleaseArgs),

    /// Block a ticket with a reason
    Block(BlockArgs),

    /// Manage ticket dependencies (add/remove)
    #[command(alias = "dep")]
    Dep(DepArgs),

    /// Show dependency tree for a ticket
    #[command(alias = "deps")]
    Deps(DepsArgs),

    /// Export tickets to text or JSON
    #[command(alias = "dump")]
    Export(ExportArgs),
}
```

**Important:** Clap lowercases enum variant names automatically when deriving. `Create` becomes `create`, `List` becomes `list`, etc. The alias `"new"` maps to `create`; `"ls"` maps to `list`. Verify that `Claim`'s canonical invocation is `claim` (not `Claim`) — it is, by Clap's lowercasing rules.

### Pattern 4: JSON Export via serde_json

**What:** Add `#[derive(Serialize)]` to an export-specific struct; call `serde_json::to_string_pretty`.
**When to use:** `export --json` flag.

```rust
// In ticket.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct TicketExport {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub status: String,
    pub claimed_by: Option<String>,
    pub dependencies: Vec<i64>,
}
```

```rust
// In lib.rs export handler
let rows: Vec<ticket::TicketExport> = /* query + dep join */;
if args.json {
    println!("{}", serde_json::to_string_pretty(&rows).unwrap());
} else {
    for r in &rows {
        // plain text one-liner
        let deps = if r.dependencies.is_empty() {
            String::new()
        } else {
            format!(" deps:{}", r.dependencies.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(","))
        };
        println!("T-{} [{}] {}{}", r.id, r.status, r.name, deps);
    }
}
```

### Pattern 5: Claimed/Unclaimed Flag Design

**What:** Two boolean flags: `--claimed` and `--unclaimed`. At most one should be set; treat as a tri-state.
**Why:** Simpler Clap definition than `--claimed <bool>`; clearer intent than a single enum value.
**Validation:** In the handler, if both `claimed` and `unclaimed` are true, emit an error.

```rust
// In cli.rs
#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    pub timestamps: bool,
    #[arg(long, help = "Show only claimed tickets")]
    pub claimed: bool,
    #[arg(long, help = "Show only unclaimed tickets")]
    pub unclaimed: bool,
    #[arg(long, help = "Filter by claimer name")]
    pub claimer: Option<String>,
    #[arg(long, help = "Filter by status")]
    pub status: Option<String>,
    #[arg(long = "search", help = "Substring search (name + description), repeatable")]
    pub search: Vec<String>,
}
```

```rust
// In lib.rs, building ListFilter from ListArgs
fn build_filter(args: &ListArgs) -> Result<ListFilter, AppError> {
    if args.claimed && args.unclaimed {
        // emit usage error
    }
    let claimed = if args.claimed { Some(true) } else if args.unclaimed { Some(false) } else { None };
    Ok(ListFilter {
        status: args.status.clone(),
        claimed,
        claimer: args.claimer.clone(),
        search: args.search.clone(),
    })
}
```

### Anti-Patterns to Avoid

- **Using named params for dynamic WHERE:** rusqlite named params require static param names at compile time. Use positional `?` with a `Vec<Box<dyn ToSql>>` for dynamic conditions.
- **Separate `Ticket` struct for export:** Don't add Serialize to the existing `Ticket` struct (it doesn't carry deps); define a separate `TicketExport` struct.
- **LIKE without LOWER():** SQLite LIKE is case-insensitive for ASCII but case-sensitive for Unicode above U+007F. Always use `LOWER(column) LIKE LOWER(pattern)` or just `LOWER(column) LIKE ?` with a pre-lowercased pattern string.
- **`--search` as multi-value vs repeatable:** Clap's `Vec<String>` for `#[arg(long)]` means the flag is repeatable (`--search foo --search bar`), not space-separated (`--search foo bar`). Repeatable is the right choice for AND semantics.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON serialization | String formatting with `format!("{}", ...)` | `serde_json` | Name/description fields can contain `"`, `\`, newlines; manual escaping has edge cases |
| SQL LIKE with escaping | Custom contains logic | SQLite LIKE + `%` wildcards | SQLite handles `%` in patterns; just pre-lowercase the search term |

**Key insight:** The two most tempting hand-roll targets here (JSON and SQL substring search) each have subtle correctness issues (JSON escaping, Unicode case folding) that the library handles.

## Common Pitfalls

### Pitfall 1: rusqlite positional params with dynamic Vec
**What goes wrong:** Trying to pass `&params` where `params: Vec<(&str, &dyn ToSql)>` (named) when the set of conditions is dynamic.
**Why it happens:** The named param API is ergonomic for static queries but can't be used when the number of `:name` placeholders isn't known at compile time.
**How to avoid:** Use `?` placeholders and `params_refs.as_slice()` with a `Vec<Box<dyn ToSql>>`.
**Warning signs:** Compiler complains about lifetime of `&dyn ToSql` when collecting into a Vec.

### Pitfall 2: Clap alias on Claim is redundant but harmless
**What goes wrong:** Adding `#[command(alias = "claim")]` to `Claim` when the lowercased variant name IS already `claim`.
**Why it happens:** Clap's derive lowercases `Claim` → `claim` automatically.
**How to avoid:** Verify canonical names first; skip alias if it matches the lowercased variant name. It's harmless to add (Clap deduplicates), but adds noise.
**Warning signs:** Clap warns about duplicate alias at startup (it may panic in debug builds).

### Pitfall 3: `--search` combined with `--status` skips no-op filter case
**What goes wrong:** With no filters specified, `list_tickets_filtered` rebuilds the same query that `list_tickets` already has — fine, but `list_tickets` is still referenced in tests.
**Why it happens:** Two functions doing the same thing when no filter is set.
**How to avoid:** Replace `list_tickets` call with `list_tickets_filtered` with an empty filter, or keep both and delegate. Prefer keeping `list_tickets_filtered` as the only path — drop `list_tickets` from the public API or make it a wrapper.
**Warning signs:** Diverging behavior between `ls` (no filters) and filtered `ls`.

### Pitfall 4: Export plain text format lacks dep query
**What goes wrong:** The filtering query returns tickets but not their dependencies; a second query per ticket is needed to populate `deps:T-2,T-3`.
**Why it happens:** The existing `list_tickets` doesn't fetch deps (it only fetches dep counts for display).
**How to avoid:** Either join `ticket_deps` in the export query (more complex SQL, one query), or run `list_deps` per ticket (simple, N+1). For the typical scale of this tool, N+1 is acceptable.
**Warning signs:** Export shows `deps:` with empty list when tickets have dependencies.

### Pitfall 5: JSON export double-serializes
**What goes wrong:** Calling `serde_json::to_string` on a `Vec<TicketExport>` and also wrapping it in another struct.
**Why it happens:** Uncertainty about top-level shape.
**How to avoid:** Keep it simple — JSON output is a top-level array `[{...}, {...}]`. That's the minimal shape that works for programmatic parsing (EXPORT-04).

## Code Examples

### Dynamic WHERE builder (positional params)
```rust
// Source: pattern derived from rusqlite docs + existing update_ticket in ticket.rs
let mut conditions: Vec<String> = Vec::new();
let mut raw_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

if let Some(ref s) = filter.status {
    conditions.push("status = ?".to_string());
    raw_params.push(Box::new(s.clone()));
}
for term in &filter.search {
    let pat = format!("%{}%", term.to_lowercase());
    conditions.push("(LOWER(name) LIKE ? OR LOWER(description) LIKE ?)".to_string());
    raw_params.push(Box::new(pat.clone()));
    raw_params.push(Box::new(pat));
}

let where_sql = if conditions.is_empty() {
    String::new()
} else {
    format!("WHERE {}", conditions.join(" AND "))
};
let sql = format!("SELECT ... FROM tickets {} ORDER BY id", where_sql);
let refs: Vec<&dyn rusqlite::types::ToSql> = raw_params.iter().map(|b| b.as_ref()).collect();
let mut stmt = conn.prepare(&sql)?;
let rows = stmt.query_map(refs.as_slice(), |row| { ... })?;
```

### Clap multiple aliases (derive)
```rust
// Source: Clap 4 raw attribute docs — any Command method usable as attr
#[derive(Subcommand)]
pub enum Commands {
    #[command(alias = "new")]
    Create(CreateArgs),
    #[command(alias = "ls")]
    List(ListArgs),
    #[command(alias = "get")]
    Get { id: i64 },
    #[command(alias = "up")]
    Update(UpdateArgs),
    #[command(alias = "rm")]
    Delete { id: i64 },
    Claim(ClaimArgs),              // canonical name already "claim" — no alias needed
    #[command(alias = "rel")]
    Release(ReleaseArgs),
    Block(BlockArgs),
    #[command(alias = "dep")]
    Dep(DepArgs),
    #[command(alias = "deps")]
    Deps(DepsArgs),
    #[command(alias = "dump")]
    Export(ExportArgs),
}
```

### serde_json struct export
```rust
// Source: serde_json 1.0 docs
use serde::Serialize;

#[derive(Serialize)]
pub struct TicketExport {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub status: String,
    pub claimed_by: Option<String>,
    pub dependencies: Vec<i64>,
}

// Output a JSON array to stdout
let json = serde_json::to_string_pretty(&exports).expect("serialize");
println!("{}", json);
```

### Repeatable --search arg in Clap derive
```rust
// Source: Clap 4 derive — Vec<T> makes arg repeatable
#[arg(long = "search", help = "Substring search in name+description (repeatable)")]
pub search: Vec<String>,
// Usage: rtik ls --search foo --search bar  (both must match)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| clap 2/3 `subcommand_alias()` builder | `#[command(alias = "...")]` derive attr | clap 4.0 | Aliases colocated with subcommand definition; no builder boilerplate |
| `serde_json::to_string` | same | stable | N/A — already best approach |

**Deprecated/outdated:**
- `App::alias()` / `SubCommand::with_name().alias()`: clap 2/3 builder API — project uses clap 4 derive; use `#[command(alias = "...")]` instead.

## Open Questions

1. **`--status` accepts raw DB values — no validation**
   - What we know: CONTEXT.md says "accepts whatever status values are already stored in the DB"
   - What's unclear: Should an invalid status value (e.g., `--status xyz`) silently return 0 results, or error?
   - Recommendation: Silent empty results is simpler and matches the decision not to re-define the status enum here. The planner can pick either; silent is safer.

2. **`--claimer` interaction with `--claimed`/`--unclaimed`**
   - What we know: Left to Claude's discretion
   - What's unclear: If `--claimer alice` is given, should `--unclaimed` be an error, or silently override?
   - Recommendation: `--claimer alice` implies claimed; treat as `claimed_by = 'alice'` only, ignore the `--unclaimed` flag if both are set (or error). Erroring is cleaner.

3. **Plain text ID format: `T-1` vs `#1`**
   - What we know: CONTEXT.md specifies `T-1 [open] Fix auth bug deps:T-2,T-3`
   - What's unclear: The existing codebase uses `#1` format everywhere in terminal output
   - Recommendation: Use `T-{id}` for export as specified (token-efficient, no special chars). Keep `#1` for interactive commands (no change).

4. **N+1 dep queries in export**
   - What we know: Export must include dep IDs per ticket
   - What's unclear: Single JOIN query vs per-ticket `list_deps` call
   - Recommendation: Use `list_deps` per ticket. Scale is small (local tool) and the code is already written. The planner can choose JOIN if preferred.

## Sources

### Primary (HIGH confidence)
- Official rusqlite 0.38 API (in-codebase usage verified): positional params, `query_map`, `prepare` patterns
- Clap 4.5 derive docs (docs.rs): raw attribute pattern for `#[command(alias = "...")]`; verified via `https://docs.rs/clap/latest/clap/_derive/index.html`
- SQLite official docs (sqlite.org/lang_expr.html): LIKE is case-insensitive for ASCII by default
- serde_json 1.0 docs (docs.rs/serde_json): `to_string_pretty`, version 1.0.149

### Secondary (MEDIUM confidence)
- WebSearch: clap 4 alias syntax confirmed by multiple sources; syntax `#[command(alias = "name")]` cross-referenced with builder API docs
- WebSearch: SQLite LIKE behavior confirmed by sqlite.org forum and tutorial sources

### Tertiary (LOW confidence)
- Claim variant alias behavior: Clap lowercases variant names — this is widely documented behavior but not re-verified in official docs for this specific version. Low risk; if wrong, add explicit alias.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in project; serde/serde_json are de-facto standards
- Architecture: HIGH — dynamic WHERE pattern directly mirrors existing `update_ticket`; no novel patterns
- Clap alias syntax: MEDIUM — raw attribute syntax confirmed by multiple sources but edge cases (duplicate alias warning) are LOW confidence
- Pitfalls: MEDIUM — derived from code inspection and library behavior; most are patterns the codebase already navigates

**Research date:** 2026-02-23
**Valid until:** 2026-03-23 (stable libraries; 30-day window is conservative)
