# rtik

A lightweight CLI ticketing system for LLM agents. Persists task state across context resets and coordinates multi-agent workflows using a local SQLite database.

## Why

LLM agents lose track of work when their context window resets. rtik gives agents a persistent, queryable task store that survives session restarts and prevents concurrent agents from stepping on each other through atomic ticket claiming.

## Features

- **Persistent state** -- SQLite-backed tickets survive agent crashes and context resets
- **Atomic claiming** -- `IMMEDIATE` transactions prevent race conditions between agents
- **Dependency tracking** -- Declare inter-ticket dependencies with automatic cycle detection
- **Status state machine** -- Validated transitions: `todo` → `in-progress` → `done`, with `blocked` support
- **Token-efficient export** -- Plain text format minimizes context window usage; JSON available for structured consumption
- **Zero runtime dependencies** -- Single static binary (2.9 MB)

## Install

Requires Rust 1.75+.

```bash
cargo build --release
# Binary at target/release/rtik
```

## Quick start

```bash
export RTIK_AGENT="my-agent"

# Create tickets
rtik create -n "Implement auth" -d "Add JWT-based authentication"
rtik create -n "Write tests" -d "Integration tests for auth"

# Add a dependency (#2 depends on #1)
rtik dep add 2 1

# Claim and work
rtik claim 1
rtik update 1 --status done    # auto-clears claim

# List remaining work
rtik list --unclaimed

# Export for agent context
rtik export              # compact plain text
rtik export --json       # structured JSON
```

## Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `create` | `new` | Create a ticket (`-n NAME [-d DESC]`) |
| `get` | | Show ticket details |
| `update` | `up` | Update fields (`--name`, `--desc`, `--status`) |
| `delete` | `rm` | Delete a ticket |
| `list` | `ls` | List tickets with filters |
| `claim` | | Claim a ticket (sets status to `in-progress`) |
| `release` | `rel` | Release a claimed ticket |
| `block` | | Block a ticket with a reason |
| `dep` | | Manage dependencies (`dep add ID DEP` / `dep remove ID DEP`) |
| `deps` | | Show dependency tree |
| `export` | `dump` | Export tickets in plain text or JSON |

### Filters (for `list` and `export`)

```
--status {todo|in-progress|blocked|done}
--claimed / --unclaimed
--claimer AGENT
--search TERM          # substring match, repeatable (AND logic)
--timestamps           # show created/updated dates (list only)
```

## Database resolution

rtik looks for its database in this order:

1. `RTIK_DB` environment variable
2. Walk parent directories for `.rtik.db`
3. `.rtik.db` in the current directory

## Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `RTIK_AGENT` | For claim/release | Agent identifier |
| `RTIK_DB` | No | Override database path |

## Status transitions

```
todo ──→ in-progress ──→ done
 │            │  ↑          │
 └──→ blocked ←──┘     (reopen)
       │     ↑              │
       └─────┘──────────────┘
```

Setting status to `done` automatically clears the claim.

## Development

```bash
cargo test              # all tests
cargo clippy            # lint
cargo fmt --check       # format check
```
