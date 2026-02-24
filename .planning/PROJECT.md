# rtik - Agent Ticketing CLI

## What This Is

A lightweight Rust CLI ticketing system for LLM agents to persist work state across context resets, coordinate multi-agent workflows, and track task dependencies. Agents can query available work, claim tickets atomically, update status through a validated state machine, and export tasks in agent-optimized plain text or JSON formats.

## Core Value

Agents never lose track of work when context resets — persistent, queryable task state that survives session restarts and enables multi-agent coordination.

## Requirements

### Validated

- ✓ Ticket CRUD operations via CLI (create, read, update, delete) — v1.0
- ✓ Ticket schema with required fields: unique ID, short name, description, created_at, updated_at, status, dependencies, claimed_by — v1.0
- ✓ Status field with values: todo, in-progress, blocked, done — v1.0 (wip rejected in favor of in-progress)
- ✓ Dependency tracking between tickets with cycle detection — v1.0
- ✓ Claim mechanism for agent ownership (atomic IMMEDIATE transactions) — v1.0
- ✓ Search functionality by ticket name or description (substring match) — v1.0
- ✓ Export tickets to plain text format (ID, name, status, deps) and JSON — v1.0
- ✓ SQLite database for persistent storage with WAL mode — v1.0
- ✓ Short CLI command aliases optimized for speed (new, ls, dump, get, up, rm, rel, dep, deps) — v1.0
- ✓ Configurable query filters when listing tickets (status, claimed, claimer, search) — v1.0
- ✓ Single binary distribution (no runtime dependencies) — v1.0

### Active

(None — planning v1.1 requirements next)

### Out of Scope

- Web UI or GUI — CLI only, agents don't need visual interfaces
- Hard dependency enforcement — track dependencies but let agents decide whether to respect them
- User authentication — single-user local tool, no multi-user concerns
- Remote sync or cloud storage — local SQLite file only
- Full JIRA-like features — intentionally simplified for agent workflows
- Rich formatting in descriptions — plain text only
- Attachments or file uploads — text-based task tracking only
- Virtual tags (+BLOCKED, +BLOCKING) — useful but complex; deferred to v1.1
- Context system for auto-filtering — useful for multi-project workflows; deferred
- Work log / append-only history — adds complexity; deferred

## Context

**Shipped:** v1.0 MVP with 1,619 LOC Rust, 51 files, built over 2 days.
**Tech stack:** Rust + Clap 4.5 + rusqlite + serde/serde_json. Single static binary.
**Test coverage:** 33 integration tests across 2 test files (13 Phase 1 + 20 Phase 2 + 15 Phase 3).

**Design decisions confirmed:**
- Directory-walking DB path resolution (env var → parent walk → cwd fallback) works well in practice
- Plain text export intentionally omits description for token efficiency; JSON export includes all fields
- `done` transition auto-clears claim — agents don't need to explicitly release before finishing
- `--force` flag on release allows overriding ownership check when needed

**Known gaps (tech debt from audit):**
- `block_reason` stored in DB but not shown in `get` output — display gap only
- `done_clears_claim` test can't assert `claimed_by=NULL` because `Ticket` struct doesn't expose it
- `release_ticket` resets to `todo` rather than pre-claim status — design decision, could surprise users
- SUMMARY.md files missing `requirements-completed` frontmatter field

## Constraints

- **Tech stack**: Rust + Clap + rusqlite — single binary, fast execution, no runtime dependencies
- **Storage**: SQLite only — local file-based database, no external database server
- **Interface**: CLI only — no web interface, GUI, or API server
- **Simplicity**: Deliberately simpler than JIRA/Linear — closer to todo tracking than full project management

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust for implementation | Single binary, fast, good CLI ecosystem (Clap) | ✓ Good — zero-dep binary works well |
| Soft claiming (reassignable) | Agents may crash or abandon work — need flexibility | ✓ Good — `--force` release covers edge cases |
| Dependencies informational only | Agents can decide context-specifically whether to respect deps | ✓ Good — unmet dep warnings on claim without hard block |
| Blocked status is manual | Blocked means external issue, separate from dependency state | ✓ Good — clean separation of concerns |
| Plain text export format | Token-efficient for LLM context, human-readable for debugging | ✓ Good — description omitted by design for token efficiency |
| WAL mode + IMMEDIATE transactions | Concurrent reads during writes, prevent write starvation | ✓ Good — atomic claiming verified by dual-connection test |
| Directory-walking DB resolution | Agents run from any subdirectory and find the right DB | ✓ Good — works well in practice |
| `done` auto-clears claim | Reduces agent boilerplate (no explicit release before done) | ✓ Good — simplifies agent workflows |

---
*Last updated: 2026-02-24 after v1.0 milestone*
