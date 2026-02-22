# rtik - Agent Ticketing CLI

## What This Is

A lightweight CLI ticketing system designed for LLM agents to persist work state across context resets, coordinate multi-agent workflows, and track task dependencies. Agents can query available work, claim tickets, update status, and export tasks in agent-optimized plain text formats.

## Core Value

Agents never lose track of work when context resets - persistent, queryable task state that survives session restarts and enables multi-agent coordination.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Ticket CRUD operations via CLI (create, read, update, delete)
- [ ] Ticket schema with required fields: unique ID, short name, description, created_at, updated_at, status, dependencies, claimed_by
- [ ] Status field with values: todo, WIP, blocked, done
- [ ] Dependency tracking between tickets (informational - not enforced)
- [ ] Claim mechanism for agent ownership (soft ownership, reassignable)
- [ ] Search functionality by ticket name or description
- [ ] Export tickets to plain text format (ID, name, description, deps)
- [ ] SQLite database for persistent storage
- [ ] Short CLI command aliases optimized for speed (new, ls, claim, etc.)
- [ ] Configurable query filters when listing tickets
- [ ] Single binary distribution (no runtime dependencies)

### Out of Scope

- Web UI or GUI - CLI only, agents don't need visual interfaces
- Hard dependency enforcement - track dependencies but let agents decide whether to respect them
- User authentication - single-user local tool, no multi-user concerns
- Remote sync or cloud storage - local SQLite file only
- Full JIRA-like features - intentionally simplified for agent workflows
- Rich formatting in descriptions - plain text only
- Attachments or file uploads - text-based task tracking only

## Context

**Problem:** LLM agents lose work state when their context window resets or sessions restart. They need a persistent, external memory for tracking tasks, understanding what work is available, and coordinating with other agents working in parallel.

**Workflow:** Pull model - agents query available work based on filters (status, dependencies, claimed status), claim tickets to signal ownership, update status as work progresses, and mark complete when done. Other agents can see what's claimed to avoid duplicate work.

**Export format:** Plain text optimized for token efficiency when agents load context. Minimal format includes: ticket ID, name, description, and dependencies - enough to understand and start work without unnecessary verbosity.

**Agent coordination:** Multiple parallel agents may work simultaneously. Claiming provides visibility into who's working on what, but allows reassignment if needed (soft locks, not hard exclusivity).

## Constraints

- **Tech stack**: Rust + Clap + rusqlite — single binary, fast execution, no runtime dependencies
- **Storage**: SQLite only — local file-based database, no external database server
- **Interface**: CLI only — no web interface, GUI, or API server
- **Simplicity**: Deliberately simpler than JIRA/Linear — closer to todo tracking than full project management

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust for implementation | Single binary, fast, good CLI ecosystem (Clap) | — Pending |
| Soft claiming (reassignable) | Agents may crash or abandon work - need flexibility to reassign | — Pending |
| Dependencies informational only | Agents can decide context-specifically whether deps matter - avoid over-constraining | — Pending |
| Blocked status is manual | Blocked means external issue, separate from dependency state | — Pending |
| Plain text export format | Token-efficient for LLM context, human-readable for debugging | — Pending |

---
*Last updated: 2025-02-22 after initialization*
