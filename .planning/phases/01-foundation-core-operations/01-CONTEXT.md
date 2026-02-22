# Phase 1: Foundation & Core Operations - Context

**Gathered:** 2026-02-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the SQLite-backed data layer and CLI binary (Rust + Clap + rusqlite) with full CRUD for tickets.
Agents can create, view, update, delete, and list tickets with persistent storage.
No claiming, no search, no export — this phase is purely the foundation.

</domain>

<decisions>
## Implementation Decisions

### Command syntax

- Named flags for all fields: `rtik create --name "Fix bug" --desc "Details here"`
- Short aliases: `-n` for `--name`, `-d` for `--desc`
- `rtik get <id>` displays a compact plain-text paragraph format:
  ```
  #1 Buy milk [todo]
  Go to grocery store
  Created: 2026-02-22 | Updated: 2026-02-22
  ```
- `rtik delete <id>` outputs a confirmation on success: `Deleted: #1 Buy milk`

### List output format

- Aligned table with column headers: `ID`, `STATUS`, `NAME`
- Default columns: ID, status, name only (minimal)
- Optional CLI flags to show timestamps (created/updated) — exact flags at Claude's discretion
- Default sort: by ID ascending
- Long names truncated at ~40 chars with ellipsis

### Database location

- Project-local: `.rtik.db` in the project directory
- Walks up parent directories to find `.rtik.db` (git-like behavior)
- Override via `RTIK_DB` environment variable
- If no `.rtik.db` found anywhere and no env var: auto-create `.rtik.db` in current working directory (zero setup required)

### Update command design

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

</decisions>

<specifics>
## Specific Ideas

- The `rtik get` output format should be compact and readable in both human terminal and agent context — not verbose key-value blocks
- The list view is primarily for scanning; detail view (`get`) is for reading the full ticket
- Zero-setup experience: no `rtik init` required, first command auto-creates the DB

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-foundation-core-operations*
*Context gathered: 2026-02-22*
