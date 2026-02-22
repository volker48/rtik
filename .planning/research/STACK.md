# Stack Research

**Domain:** CLI Ticketing System
**Researched:** 2026-02-22
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.90+ | Primary language | Single binary distribution, zero runtime deps, excellent CLI ecosystem, memory safety guarantees |
| clap | 4.5.x | CLI argument parsing | Industry standard for Rust CLIs, derive macros for clean code, auto-generated help, subcommand support |
| rusqlite | 0.38.0+ | SQLite database bindings | Lightweight wrapper around SQLite C API, bundled feature for zero external deps, exposes full SQLite feature set |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| anyhow | 2.0+ | Application-level error handling | Main binary - flexible error propagation with context |
| thiserror | 2.0+ | Library-level custom errors | Creating typed errors at module boundaries |
| serde | 1.0.228+ | Serialization framework | Export formats (JSON/plain text ticket data) |
| serde_json | 1.0+ | JSON serialization | Optional JSON export if needed beyond plain text |
| chrono | 0.4+ | Date/time handling | Timestamp formatting for created_at/updated_at fields |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| assert_cmd | 2.0+ | CLI integration testing | Test actual binary execution with output assertions |
| predicates | 3.1+ | Assertion predicates | Works with assert_cmd for readable test assertions |
| cargo-release | Release management | Automates versioning and changelog (optional) |
| cross | Cross-compilation | Docker-based builds for multiple platforms |

### Optional Enhancement Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| directories-next | 2.0+ | Platform-specific config paths | If adding config file support (out of scope for MVP) |
| colored | 2.0+ | Terminal color output | Pretty printing status, warnings (nice-to-have) |
| env_logger | 0.11+ | Logging via environment vars | Development/debugging with RUST_LOG |

## Installation

```toml
# Cargo.toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
rusqlite = { version = "0.38", features = ["bundled"] }
anyhow = "2.0"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| rusqlite | diesel | Use diesel for complex multi-table schemas with compile-time query validation and ORM features. For simple CRUD on single table, rusqlite avoids overhead. |
| rusqlite | sqlx | Use sqlx for async runtime support or PostgreSQL/MySQL. rtik is synchronous CLI with SQLite only. |
| anyhow | eyre | Use eyre if you need error report hooks or custom formatting. anyhow is more widely adopted with simpler API. |
| clap | structopt | structopt is deprecated - use clap 4.x derive API which merged structopt's functionality |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| structopt | Deprecated, merged into clap v3+ | clap 4.x with derive feature |
| diesel for simple CRUD | Adds migration complexity, compile-time overhead for single-table SQLite | rusqlite with bundled feature |
| async runtime (tokio/async-std) | Unnecessary overhead for synchronous CLI tool | Standard sync Rust |
| ORM abstractions | Over-engineering for simple ticket CRUD operations | Direct SQL with rusqlite |
| lazy_static | Replaced by std::sync::LazyLock in Rust 1.80+ | std::sync::LazyLock |

## Stack Patterns by Variant

**If adding JSON export:**
- Add serde_json to dependencies
- Implement Serialize derive on ticket struct
- Use serde_json::to_string_pretty for output

**If adding colored output:**
- Add colored crate
- Conditional coloring based on terminal detection
- Respect NO_COLOR environment variable

**If cross-platform builds needed:**
- Use cross for Docker-based cross-compilation
- Set up GitHub Actions matrix for multiple targets
- Test bundled rusqlite on all platforms

## Critical Design Decisions

### Why Rusqlite over Diesel

For rtik's use case (single tickets table, simple CRUD), rusqlite is superior because:

1. **No migrations needed** - Simple schema can be created with CREATE TABLE IF NOT EXISTS
2. **Bundled SQLite** - Zero external dependencies with bundled feature
3. **Full SQLite access** - Can use SQLite-specific features like FTS if needed later
4. **Simpler** - No schema.rs, no migration files, just SQL strings with type safety via Rust

Diesel would add significant complexity for minimal benefit:
- Migration system overkill for single table
- Compile-time query checking unnecessary for simple CRUD
- schema.rs maintenance burden
- Harder to use SQLite-specific extensions

### Why Anyhow for Main Binary

Application-level error handling benefits from anyhow because:

1. **Context chains** - `.context("Failed to open database")` adds useful info
2. **Type erasure** - Don't need to define error enum for every module
3. **? operator** - Works seamlessly with any std::error::Error
4. **Debug output** - Automatic error chains in output

Use thiserror for library code or public APIs where typed errors matter.

### Why Clap Derive API

The derive macro approach:
- Reduces boilerplate compared to builder API
- Auto-generates help text from doc comments
- Type-safe argument parsing at compile time
- Easier to maintain as commands grow

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| rusqlite 0.38.x | SQLite 3.51.1 | Bundled version via libsqlite3-sys 0.36.0 |
| clap 4.5.x | clap_complete 4.5.x | Use matching versions for shell completions |
| serde 1.0.x | serde_json 1.0.x | Major version alignment required |
| anyhow 2.0.x | thiserror 2.0.x | Both updated to 2.0 in 2025, work together |

## Build Configuration

### Recommended Cargo.toml Settings

```toml
[package]
name = "rtik"
version = "0.1.0"
edition = "2021"
rust-version = "1.90"

[profile.release]
strip = true        # Remove debug symbols
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization, slower compile
opt-level = "z"     # Optimize for size

[profile.dev]
opt-level = 1       # Faster dev builds with basic optimization
```

### Cross-Platform Binary Distribution

Use GitHub Actions with cross for building releases:

```yaml
targets:
  - x86_64-unknown-linux-gnu
  - x86_64-unknown-linux-musl
  - x86_64-apple-darwin
  - aarch64-apple-darwin
  - x86_64-pc-windows-msvc
```

The musl target provides fully static Linux binaries that work on any Linux distribution.

## Sources

**HIGH confidence sources:**
- [clap crates.io](https://crates.io/crates/clap) - Version 4.5.x confirmed
- [rusqlite crates.io](https://crates.io/crates/rusqlite) - Version 0.38.0 released December 2025
- [Rust ORMs in 2026: Diesel vs SQLx vs SeaORM vs Rusqlite](https://aarambhdevhub.medium.com/rust-orms-in-2026-diesel-vs-sqlx-vs-seaorm-vs-rusqlite-which-one-should-you-actually-use-706d0fe912f3) - February 2026 comparison
- [Rust CLI Best Practices](https://rust-cli.github.io/book/tutorial/cli-args.html) - Official Rust CLI Book
- [serde version 1.0.228](https://docs.rs/crate/serde/latest/) - November 2025 release

**MEDIUM confidence sources:**
- [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/cli-parser.html) - Argument parser comparison
- [Rust Error Handling Guide 2025](https://markaicode.com/rust-error-handling-2025-guide/) - anyhow vs thiserror patterns
- [Cross-compilation in Rust](https://fpira.com/blog/2025/01/cross-compilation-in-rust) - January 2025 guide
- [How I test Rust CLI apps with assert_cmd](https://alexwlchan.net/2025/testing-rust-cli-apps-with-assert_cmd/) - 2025 testing patterns
- [directories-next docs](https://docs.rs/directories-next/) - Config path handling

**Version verification:**
- clap: 4.5.54 documented on docs.rs (search results confirm 4.5.x series active)
- rusqlite: 0.38.0 confirmed December 2025 release
- serde: 1.0.228 confirmed November 2025 release
- anyhow/thiserror: 2.0 major versions confirmed in 2025 error handling guides

---
*Stack research for: CLI Ticketing System (rtik)*
*Researched: 2026-02-22*
