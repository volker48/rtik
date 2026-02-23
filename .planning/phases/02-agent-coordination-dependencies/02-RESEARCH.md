# Phase 2: Agent Coordination & Dependencies - Research

**Researched:** 2026-02-22
**Domain:** SQLite atomic writes, dependency graph management, state machine enforcement (Rust/rusqlite)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Claim behavior**
- Failed claims print the current owner and claim time: `Error: ticket #5 claimed by agent-2 since 14:23`
- Force-claiming supported via `--force` flag (overrides another agent's claim)
- Force-release supported via `--force` flag (anyone can release, not just owner)
- Successful claim prints minimal confirmation: `Claimed #ID`
- Agent identity for `claimed_by`: Claude's discretion (environment variable or positional arg)
- Claiming a ticket auto-sets its status to `in-progress` (fewer commands for agents)
- Releasing a ticket resets status back to `todo`
- Claiming a ticket with unmet deps: warn but allow (`Warning: 2 dependencies not done`)

**Status state machine**
- Valid statuses: `todo`, `in-progress`, `blocked`, `done`
- `blocked` works both ways:
  - Auto-computed: ticket with unmet deps is shown as blocked
  - Manually settable: agent explicitly blocks with `rtik block #ID <reason>` (reason required)
  - Missing reason on `rtik block` → exit 2 with usage error
- Done is re-openable to `in-progress` (edge case: bug found, needs rework)
- Marking done auto-releases the claim
- Valid transition enforcement: Claude's discretion on exact rules (prevent real coordination bugs)
- Invalid transition error shows valid options: `Error: from done, valid transitions are: in-progress`

**Dependency display**
- In list view (`rtik ls`): Claude's discretion on what to show
- In detail view (`rtik get #ID`): compact dep list `Depends on: #3, #7` plus reverse deps `Required by: #9, #12`
- Dedicated `rtik deps #ID` command for dependency tree view

**Coordination error messages**
- Force operations warn to stderr: `Warning: overriding claim by agent-2`
- Circular dep error shows the cycle: `Error: cycle: #3 → #7 → #3`
- Status transition errors show valid options: `Error: from done, valid transitions are: in-progress`

### Claude's Discretion
- Agent identity mechanism (env var vs positional arg)
- Exact status transition rules beyond the stated constraints
- Dependency count/blocked indicator format in list view (`rtik ls`)
- Depth/format of `rtik deps #ID` tree output

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| STATE-04 | Status transitions are validated (cannot go from done to todo) | State machine table enforced in Rust before DB write; AppError variant for invalid transition |
| COORD-01 | Agent can claim ticket (sets claimed_by field) | Atomic UPDATE WHERE claimed_by IS NULL in IMMEDIATE transaction; conn.changes() to detect conflict |
| COORD-02 | Claimed ticket records claimed_at timestamp | New column via M::up ALTER TABLE; strftime default or Rust-generated timestamp |
| COORD-03 | Agent can release/unclaim ticket (allows reassignment) | UPDATE SET claimed_by=NULL, claimed_at=NULL, status='todo'; --force bypasses owner check |
| COORD-04 | User can add dependency between tickets | New ticket_deps table (ticket_id, depends_on_id) with FK constraints; INSERT with cycle check |
| COORD-05 | User can remove dependency between tickets | DELETE FROM ticket_deps WHERE ticket_id=? AND depends_on_id=? |
| COORD-06 | System detects and rejects circular dependencies | In-process DFS over deps loaded from DB before INSERT; or SQLite recursive CTE trigger |
| TECH-03 | Claim operations use atomic UPDATE to prevent race conditions | IMMEDIATE transaction + UPDATE WHERE claimed_by IS NULL + conn.changes() == 0 means taken |
| TECH-04 | Database transactions use IMMEDIATE mode to prevent write starvation | TransactionBehavior::Immediate on all write transactions; busy_timeout already set |
</phase_requirements>

## Summary

Phase 2 adds three related but distinct concerns: (1) atomic claim/release with a soft-lock protocol, (2) status state machine enforcement, and (3) dependency graph management with cycle prevention. All three build directly on the existing rusqlite + rusqlite_migration + Clap foundation from Phase 1.

The atomic claim pattern is straightforward in SQLite: a single `UPDATE tickets SET claimed_by=?, claimed_at=?, status='in-progress' WHERE id=? AND claimed_by IS NULL` wrapped in an IMMEDIATE transaction is safe across concurrent processes. After the UPDATE, `conn.changes()` tells you whether you won (1 row affected) or lost (0 rows, another agent got there first). No external locking library is needed.

Cycle detection for dependencies is the most algorithmically interesting part. The safe approach for this codebase is an in-process DFS over the dep graph loaded from SQLite immediately before each INSERT. This avoids SQLite trigger complexity while keeping the logic testable and readable. With small ticket counts (tens to low hundreds), loading the full dep graph from DB is trivial.

**Primary recommendation:** Use IMMEDIATE transactions + `conn.changes()` for atomic claiming; in-process DFS for cycle detection; new migration for schema additions. No new crate dependencies needed.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rusqlite | 0.38 (already present) | All DB operations including atomic writes | Already in use; `conn.changes()` and `transaction_with_behavior` cover all Phase 2 needs |
| rusqlite_migration | 2.4 (already present) | Add new columns and tables via sequential migrations | Already in use; `M::up("ALTER TABLE ...")` pattern confirmed |
| clap | 4.5 (already present) | New subcommands: claim, release, block, dep add/remove/show | Already in use; derive API matches existing pattern |
| thiserror | 2 (already present) | New AppError variants for claim conflicts and cycle detection | Already in use |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none new) | — | — | All Phase 2 needs are covered by existing deps |

**No new dependencies are needed.** The Rust standard library (HashMap, Vec, HashSet) is sufficient for DFS cycle detection. petgraph is unnecessary complexity for this scale.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| In-process DFS for cycle detection | SQLite recursive CTE trigger | Trigger approach is hard to test, fragile to maintain, and cycle path reporting is awkward; Rust DFS is cleaner |
| In-process DFS for cycle detection | petgraph crate | petgraph is well-tested but adds a dependency for a 30-line DFS; not justified |
| IMMEDIATE transaction for claims | Application-level mutex | Mutex only works in-process; IMMEDIATE works across processes sharing the same file |
| IMMEDIATE transaction for claims | EXCLUSIVE transaction | EXCLUSIVE blocks all readers; IMMEDIATE only blocks writers, which is sufficient |

**Installation:**
```bash
# No new dependencies — all existing
cargo build
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── cli.rs         # Add: Claim, Release, Block, Dep subcommands
├── db.rs          # Add: migration M2 (columns + ticket_deps table)
├── ticket.rs      # Add: claim_ticket, release_ticket, block_ticket,
│                  #      add_dep, remove_dep, list_deps, status transition
│                  #      validation, cycle detection
├── lib.rs         # Add: new command dispatch arms
└── main.rs        # Unchanged
tests/
└── integration.rs # Add: atomic claim tests, cycle tests, state machine tests
```

### Pattern 1: Atomic Claim via IMMEDIATE Transaction + changes()

**What:** Use `BEGIN IMMEDIATE` to serialize writers, then check `conn.changes()` after the UPDATE to determine success vs. conflict.
**When to use:** Any time a claim operation is needed; the IMMEDIATE mode acquires a reserved lock immediately, preventing any other writer from starting.

```rust
// Source: https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html
//         https://docs.rs/rusqlite/latest/rusqlite/enum.TransactionBehavior.html
pub fn claim_ticket(
    conn: &mut Connection,
    id: i64,
    agent: &str,
    force: bool,
) -> Result<(), AppError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let now = chrono_free_utc_now();

    if force {
        // Fetch current owner for warning, then overwrite unconditionally
        let current_owner: Option<String> = tx
            .query_row(
                "SELECT claimed_by FROM tickets WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(id),
                other => AppError::Db(other),
            })?;
        if let Some(owner) = current_owner {
            eprintln!("Warning: overriding claim by {}", owner);
        }
        tx.execute(
            "UPDATE tickets SET claimed_by=?1, claimed_at=?2, status='in-progress',
             updated_at=?2 WHERE id=?3",
            params![agent, now, id],
        )?;
    } else {
        tx.execute(
            "UPDATE tickets SET claimed_by=?1, claimed_at=?2, status='in-progress',
             updated_at=?2 WHERE id=?3 AND claimed_by IS NULL",
            params![agent, now, id],
        )?;
        if tx.changes() == 0 {
            // Either not found or already claimed — distinguish them
            let row: Option<(i64, Option<String>, Option<String>)> = tx
                .query_row(
                    "SELECT id, claimed_by, claimed_at FROM tickets WHERE id=?1",
                    params![id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional()?;
            match row {
                None => return Err(AppError::NotFound(id)),
                Some((_, Some(owner), claimed_at)) => {
                    return Err(AppError::AlreadyClaimed(id, owner, claimed_at.unwrap_or_default()))
                }
                Some(_) => {} // race: was released between our UPDATE and this read; harmless
            }
        }
    }
    tx.commit()?;
    Ok(())
}
```

### Pattern 2: Schema Migration — New Columns and Table

**What:** Add `claimed_by`, `claimed_at`, `block_reason` to `tickets`, plus new `ticket_deps` table, as migration M2.

```rust
// Source: https://github.com/cljoly/rusqlite_migration/blob/master/examples/simple/src/main.rs
static MIGRATIONS: &[M<'static>] = &[
    M::up(
        "CREATE TABLE tickets (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            status      TEXT NOT NULL DEFAULT 'todo'
                        CHECK(status IN ('todo','in-progress','blocked','done')),
            created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );"
    ),
    M::up(
        "ALTER TABLE tickets ADD COLUMN claimed_by TEXT;
         ALTER TABLE tickets ADD COLUMN claimed_at TEXT;
         ALTER TABLE tickets ADD COLUMN block_reason TEXT;
         CREATE TABLE ticket_deps (
             ticket_id   INTEGER NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
             depends_on  INTEGER NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
             PRIMARY KEY (ticket_id, depends_on),
             CHECK (ticket_id != depends_on)
         );"
    ),
];
```

**CRITICAL:** The existing M1 schema has `CHECK(status IN ('todo','wip','blocked','done'))` but CONTEXT.md specifies `in-progress` not `wip`. This must be resolved in M2. SQLite cannot modify CHECK constraints via ALTER TABLE — the approach is to update M2 to recreate the table via the "12-step ALTER TABLE" pattern or, simpler: update M1 to use `in-progress` from the start (since Phase 1 was `wip`, Phase 2 renames it). **See Open Questions #1.**

### Pattern 3: In-Process DFS Cycle Detection

**What:** Load all dependency edges from DB into a `HashMap<i64, Vec<i64>>`, then run DFS from the proposed new `depends_on` node to see if it can reach `ticket_id`. If yes, there's a cycle.
**When to use:** Before every `INSERT INTO ticket_deps`.

```rust
// No external crate needed — standard Rust collections
use std::collections::{HashMap, HashSet};

pub fn would_create_cycle(
    conn: &Connection,
    ticket_id: i64,
    new_dep: i64,
) -> Result<Option<Vec<i64>>, AppError> {
    // Load all existing edges: ticket_id -> depends_on
    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT ticket_id, depends_on FROM ticket_deps"
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
    for row in rows {
        let (from, to) = row?;
        adj.entry(from).or_default().push(to);
    }
    // Hypothetically add the new edge
    adj.entry(ticket_id).or_default().push(new_dep);

    // DFS from new_dep: if we can reach ticket_id, it's a cycle
    let mut visited: HashSet<i64> = HashSet::new();
    let mut path: Vec<i64> = Vec::new();
    if dfs_finds_target(&adj, new_dep, ticket_id, &mut visited, &mut path) {
        path.insert(0, ticket_id); // complete the cycle representation
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
    if current == target { return true; }
    if !visited.insert(current) { return false; }
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
```

### Pattern 4: Status State Machine Enforcement

**What:** Validate allowed transitions before writing to DB. Return clear error with valid options.

```rust
pub fn validate_transition(from: &str, to: &str) -> Result<(), AppError> {
    let allowed: &[&str] = match from {
        "todo"        => &["in-progress", "blocked"],
        "in-progress" => &["done", "blocked", "todo"],
        "blocked"     => &["in-progress", "todo"],
        "done"        => &["in-progress"],
        _             => &[],
    };
    if allowed.contains(&to) {
        Ok(())
    } else {
        Err(AppError::InvalidTransition(from.to_string(), allowed.to_vec()))
    }
}
```

State machine rationale (Claude's discretion):
- `todo → in-progress`: normal start
- `todo → blocked`: manually blocked before starting (valid scenario)
- `in-progress → done`: normal completion
- `in-progress → blocked`: hit external blocker mid-work
- `in-progress → todo`: abandon without releasing separately (auto-release)
- `blocked → in-progress`: blocker resolved
- `blocked → todo`: reset
- `done → in-progress`: rework (explicitly required by CONTEXT.md)
- **FORBIDDEN:** `done → todo`, `done → blocked` (prevent accidental status reset)

### Anti-Patterns to Avoid

- **DEFERRED transactions for claims:** A DEFERRED transaction reads first, then tries to upgrade to a write lock. The gap between the SELECT and the lock acquisition is a race window. Use IMMEDIATE.
- **SELECT then UPDATE as separate statements:** Classic TOCTOU race. The atomic `UPDATE WHERE claimed_by IS NULL` eliminates this entirely.
- **Recursive CTE triggers for cycle detection:** The trigger approach in SQLite is hard to surface cycle paths and harder to unit test. Keep cycle detection in Rust.
- **petgraph for cycle detection:** Adds a crate dependency for ~30 lines of DFS. The standard library is sufficient.
- **Ignoring conn.changes():** The return value of `execute()` is the affected row count; not checking it means missing the "already claimed" signal.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Transaction isolation | Custom file locks, mutexes | SQLite IMMEDIATE transactions | WAL + IMMEDIATE is process-safe; custom locks won't work across processes |
| Schema versioning | Manual user_version pragma | rusqlite_migration (already present) | Already used; M::up() handles ALTER TABLE and multi-statement batches |
| Timestamp generation | New chrono dependency | `chrono_free_utc_now()` already in ticket.rs | Already exists and works; no new dep needed |

**Key insight:** SQLite's write serialization guarantees that an `UPDATE WHERE claimed_by IS NULL` + `conn.changes()` is all that is needed for atomic claiming — no external locking, no compare-and-swap, no retry loops.

## Common Pitfalls

### Pitfall 1: Status Rename — `wip` vs `in-progress`
**What goes wrong:** CONTEXT.md specifies statuses `todo`, `in-progress`, `blocked`, `done`. The existing Phase 1 schema uses `wip` instead of `in-progress`. The DB CHECK constraint only allows the Phase 1 values.
**Why it happens:** The CONTEXT.md discussion locked in `in-progress` but Phase 1 was already shipped with `wip`.
**How to avoid:** Phase 2 migration must handle the rename. Options:
  1. M2 uses `CREATE TABLE tickets_new ... INSERT INTO tickets_new SELECT ..., CASE status WHEN 'wip' THEN 'in-progress' ELSE status END, ...` (12-step SQLite table rebuild). This is the correct approach since SQLite cannot modify CHECK constraints.
  2. Accept both `wip` and `in-progress` as valid in the CHECK constraint (ambiguous, not recommended).
**Warning signs:** Integration tests that check `status == "wip"` will fail after migration; update them.

### Pitfall 2: IMMEDIATE Transaction Requires `&mut Connection`
**What goes wrong:** `transaction_with_behavior` takes `&mut Connection`, but the current codebase passes `&Connection` to most ticket functions.
**Why it happens:** rusqlite's transaction API requires mutable borrow to prevent nested transactions at compile time.
**How to avoid:** Claim and release functions must accept `&mut Connection`. The `lib.rs` dispatch has mutable access to `conn` and can pass it correctly. Keep read-only functions as `&Connection`.
**Warning signs:** Compiler error "cannot borrow `conn` as mutable because it is also borrowed as immutable."

### Pitfall 3: Cycle Path Reporting
**What goes wrong:** The error message `Error: cycle: #3 → #7 → #3` requires knowing the full cycle path, not just that a cycle exists.
**Why it happens:** Boolean cycle detection doesn't capture the path.
**How to avoid:** The DFS in Pattern 3 above tracks a path vector and returns it on detection. The caller formats it: `path.iter().map(|id| format!("#{}", id)).collect::<Vec<_>>().join(" → ")`.

### Pitfall 4: Agent Identity — Environment Variable
**What goes wrong:** If `claimed_by` is a positional arg, every agent invocation must supply it, making scripts verbose. If using env var, agents that forget to set it produce confusing ownership.
**How to avoid (recommendation):** Use `RTIK_AGENT` environment variable with a fallback to the system hostname. This is consistent with `RTIK_DB` (already in `db.rs`). Fail with a clear error if neither is set. Hostname is auto-discoverable via `std::env::var("HOSTNAME").or_else(|_| hostname::get())` — but `hostname` crate adds a dep. Simpler: use `RTIK_AGENT` env var only, document it, fail fast if unset.

### Pitfall 5: ON DELETE CASCADE for ticket_deps
**What goes wrong:** If a ticket is deleted while it has deps, orphan rows remain in `ticket_deps`, corrupting the dep graph.
**How to avoid:** The `ticket_deps` schema above uses `ON DELETE CASCADE` on both FKs, and `open_connection()` already sets `PRAGMA foreign_keys = ON`, so cascades will fire.
**Warning signs:** Deleting a ticket doesn't clean up its dep entries; `rtik deps` shows phantom tickets.

### Pitfall 6: `rtik block` Command vs Status Field
**What goes wrong:** Conflating the `block` command with the `update --status blocked` path. The `block` command requires a reason; a plain status update to `blocked` via `update` may bypass reason validation.
**How to avoid:** Two paths, two behaviors: (1) `rtik block #ID <reason>` sets `status=blocked` AND `block_reason=<reason>` — reason required, exits 2 if missing; (2) Status transitions to `blocked` via update command also require reason OR check if block_reason is already set. Simplest: only allow `blocked` status via the `block` command, not via `update --status blocked`.

## Code Examples

Verified patterns from official sources:

### IMMEDIATE Transaction
```rust
// Source: https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html
let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
tx.execute("UPDATE tickets SET ... WHERE id=?1 AND claimed_by IS NULL", params![id])?;
if tx.changes() == 0 { /* already claimed */ }
tx.commit()?;
```

### Multi-statement Migration
```rust
// Source: https://github.com/cljoly/rusqlite_migration/blob/master/examples/simple/src/main.rs
M::up(
    "ALTER TABLE tickets ADD COLUMN claimed_by TEXT;
     ALTER TABLE tickets ADD COLUMN claimed_at TEXT;
     CREATE TABLE ticket_deps (...);"
)
```

### Querying Optional Column
```rust
// Source: https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html
let owner: Option<String> = conn.query_row(
    "SELECT claimed_by FROM tickets WHERE id=?1",
    params![id],
    |r| r.get(0),
)?;
```

### Dependency List Query
```rust
// Forward deps for rtik get #ID output
let mut stmt = conn.prepare(
    "SELECT depends_on FROM ticket_deps WHERE ticket_id=?1 ORDER BY depends_on"
)?;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Application-level file locks for SQLite concurrency | IMMEDIATE transactions + WAL mode | SQLite 3.7+ (2010) | No external locking infrastructure needed |
| petgraph for any graph work | Std library DFS for simple DAGs | Always true for small graphs | Saves a crate dependency |
| DEFERRED transactions (SQLite default) | IMMEDIATE for write paths | Best practice established in SQLite docs | Eliminates upgrade-lock races |

**No deprecated approaches are in use in the current codebase.** The existing rusqlite patterns from Phase 1 are correct and extend cleanly.

## Open Questions

1. **Status rename: `wip` → `in-progress`**
   - What we know: Phase 1 shipped with `wip`; CONTEXT.md requires `in-progress`. They are incompatible.
   - What's unclear: Whether any existing data in `.rtik.db` needs migration (yes, if tests or manual use created `wip` tickets).
   - Recommendation: M2 must include a 12-step table rebuild to change the CHECK constraint AND run `UPDATE tickets SET status='in-progress' WHERE status='wip'`. Update all Phase 1 integration tests that assert `status == "wip"`.

2. **Agent identity: env var alone vs env var + hostname fallback**
   - What we know: CONTEXT.md leaves this to Claude's discretion.
   - What's unclear: Whether agents in practice will always set `RTIK_AGENT`.
   - Recommendation: Use `RTIK_AGENT` env var only; fail fast with `Error: RTIK_AGENT not set — set it to identify this agent` if unset. No hostname crate dep.

3. **Dep count display in `rtik ls`**
   - What we know: CONTEXT.md leaves format to Claude's discretion.
   - What's unclear: Whether to show dep count, blocked indicator, or both.
   - Recommendation: Add a `[2 deps]` suffix for tickets with deps, and a `BLOCKED` status indicator if any deps are not `done`. Keep it concise: `   5  blocked   Fix auth [3 deps]`.

## Sources

### Primary (HIGH confidence)
- [rusqlite Transaction docs](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html) — transaction_with_behavior, commit, rollback
- [rusqlite TransactionBehavior](https://docs.rs/rusqlite/latest/rusqlite/enum.TransactionBehavior.html) — IMMEDIATE/DEFERRED/EXCLUSIVE semantics
- [rusqlite Connection docs](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html) — changes(), busy_timeout, execute
- [rusqlite_migration simple example](https://github.com/cljoly/rusqlite_migration/blob/master/examples/simple/src/main.rs) — multi-statement M::up(), ALTER TABLE pattern
- [SQLite Atomic Commit](https://sqlite.org/atomiccommit.html) — atomicity guarantees for single statements

### Secondary (MEDIUM confidence)
- [SQLite forum: cycle detection via recursive CTE trigger](https://sqlite.org/forum/info/5797b67db27b3689) — confirmed trigger-based cycle detection is possible but verbose; validates Rust-based alternative
- [Concurrent writes in SQLite](https://tenthousandmeters.com/blog/sqlite-concurrent-writes-and-database-is-locked-errors/) — WAL + IMMEDIATE transaction interaction verified
- [SQLite WAL mode official docs](https://sqlite.org/wal.html) — single-writer serialization confirmed

### Tertiary (LOW confidence)
- None required — all critical claims verified with primary sources.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — existing deps confirmed sufficient; no new deps needed
- Architecture: HIGH — IMMEDIATE + changes() pattern is the canonical SQLite atomic claim; verified in docs
- Cycle detection: HIGH — DFS algorithm is well-established; Rust stdlib sufficient
- Status rename pitfall: HIGH — directly observed discrepancy in codebase
- Pitfalls: HIGH — derived from official SQLite locking docs and rusqlite API

**Research date:** 2026-02-22
**Valid until:** 2026-08-22 (rusqlite API is stable; SQLite locking model is decades-stable)
