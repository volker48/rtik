# Phase 3: Search, Filtering & Export - Context

**Gathered:** 2026-02-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Agents efficiently discover relevant work and export ticket context in token-optimized formats.
Includes: filter/search on `ls`, a separate `export` subcommand, and short CLI aliases.
Creating tickets, claiming, and dependency management are previous phases.

</domain>

<decisions>
## Implementation Decisions

### Filter & search interface
- Filters specified as `--flag` style (e.g. `rtik ls --status open --unclaimed`)
- Multiple filters always compose with AND — no OR mode
- `--search` is a flag on the existing `ls` command, not a separate subcommand
- `--status` accepts whatever status values are already stored in the DB (no new enum defined here)

### Search behavior
- Case-insensitive always
- `--search` searches name AND description together (no separate flags)
- Multiple `--search` values supported and AND-matched (both terms must appear somewhere)
- `--search` is fully composable with status/claimed/claimer filters — all conditions AND together

### Export scope & format
- Separate `export` subcommand (not a flag on `ls`)
- `export` accepts the same filter flags as `ls` — same filtering, different output
- Plain text format is one-line summary per ticket: `T-1 [open] Fix auth bug deps:T-2,T-3`
  - Fields: ID, status, name, dep list (IDs) — claimer omitted from plain text
- JSON format available via `--json` flag on the `export` command

### Alias coverage
- All commands get compiled-in aliases (no user config)
- Style: Unix-style abbreviations
- Alias set: `new`, `ls`, `get`, `up`, `rm`, `claim`, `rel`, `dep`, `deps`, `dump` (for export)
- Aliases are not configurable — baked into the binary

### Claude's Discretion
- Exact one-line format punctuation/spacing (e.g. brackets vs parens for status)
- How `--unclaimed` vs `--claimed` flags are surfaced (one flag with value, or two booleans)
- How `--claimer <name>` interacts with `--claimed` / `--unclaimed`
- JSON schema shape for export

</decisions>

<specifics>
## Specific Ideas

- The tool is agent-first — token efficiency is the primary driver for the plain text format
- `dump` as the export alias has a "give me everything" feel that fits the intent

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 03-search-filtering-export*
*Context gathered: 2026-02-23*
