# Pitfalls Research

**Domain:** CLI Ticketing/Task Management for LLM Agents
**Researched:** 2026-02-22
**Confidence:** MEDIUM

## Critical Pitfalls

### Pitfall 1: SQLite Concurrent Write Starvation

**What goes wrong:**
Multiple agents attempting to claim or update tickets simultaneously can lead to unfair lock contention, where some operations are starved indefinitely. SQLite uses database-level locks that can counter-intuitively starve unlucky processes when many concurrent writers compete for access.

**Why it happens:**
SQLite does not schedule concurrent transactions fairly—if multiple transactions are waiting on the same database, any one of them can be granted access next. By default, SQLite uses DEFERRED transactions that start as read transactions and try to upgrade to write transactions, creating upgrade races.

**How to avoid:**
- Enable Write-Ahead Logging (WAL) mode immediately (allows readers during writes)
- Use IMMEDIATE write transactions instead of DEFERRED (prevents upgrade races)
- Set busy timeout to several seconds minimum (default is too low)
- Keep transactions extremely short (claim + update should be <100ms)
- Consider single-threaded write queue pattern if multiple processes

**Warning signs:**
- "Database is locked" errors appearing intermittently
- Agents reporting claim operations failing with timeouts
- Increasing operation latency as more agents run in parallel
- Transaction retry storms in logs

**Phase to address:**
Phase 1 (Core CRUD) - Database configuration must be correct from day one

**Sources:**
- [SQLite Concurrency Best Practices](https://www.sqliteforum.com/p/handling-concurrency-in-sqlite-best)
- [Abusing SQLite to Handle Concurrency](https://blog.skypilot.co/abusing-sqlite-to-handle-concurrency/)
- [Four Ways to Handle SQLite Concurrency](https://medium.com/@gwendal.roue/four-different-ways-to-handle-sqlite-concurrency-db3bcc74d00e)

---

### Pitfall 2: Race Condition on Ticket Claiming

**What goes wrong:**
Two agents simultaneously query for available tickets, both see the same unclaimed ticket, both attempt to claim it, resulting in double-claiming or claim failures. In the worst case, both agents start work on the same task.

**Why it happens:**
Classic check-then-act race condition. The claim operation is logically two steps: (1) query for available tickets, (2) update claim status. Between steps 1 and 2, another agent can insert a claim.

**How to avoid:**
- Use single atomic UPDATE with WHERE conditions: `UPDATE tickets SET claimed_by = ? WHERE id = ? AND claimed_by IS NULL`
- Check affected row count to verify claim succeeded
- Never rely on SELECT...UPDATE pattern—combine into single statement
- Implement optimistic locking with version counter if needed
- Consider advisory locks for claim duration

**Warning signs:**
- Agents report working on duplicate tasks
- Claim operations sometimes silently fail (update returns 0 rows but no error shown)
- Tickets show evidence of being worked on by multiple agents
- Inconsistent claimed_by values

**Phase to address:**
Phase 2 (Claim Mechanism) - Critical to implement atomic claiming correctly

**Sources:**
- [Race Conditions in Distributed Systems](https://medium.com/hippo-engineering-blog/race-conditions-in-a-distributed-system-ea6823ee2548)
- [Handling Race Conditions with Pessimistic Locking](https://newsletter.scalablethread.com/p/how-distributed-systems-avoid-race)

---

### Pitfall 3: Lost Context Window Recovery

**What goes wrong:**
Agent context resets mid-task but can't reconstruct where they were in the work. The ticket description doesn't contain enough detail to resume, forcing the agent to restart from scratch or abandon the work entirely.

**Why it happens:**
Ticket descriptions are written once at creation time but tasks evolve. Agents discover subtasks, blockers, or nuances during execution that never get persisted back to the ticket. The ticket becomes a "starting point" rather than a "current state" record.

**How to avoid:**
- Support ticket comments/updates as append-only log (not just status changes)
- Encourage agents to write progress notes before risky operations
- Implement "checkpoint" pattern: agent writes "Starting X" before X, "Completed X" after
- Export format must include full history, not just current snapshot
- Consider separate "work log" field for agent notes vs. original description

**Warning signs:**
- Agents frequently ask "what was I doing?" after context reset
- Tickets stuck in WIP status with no activity
- Agents restart completed work because they can't tell what's done
- High abandonment rate on partially-completed tickets

**Phase to address:**
Phase 3 (Status Tracking) - Add history/logging mechanisms early
Phase 5 (Export Format) - Include full context in exports

**Sources:**
- [Context Window Management for LLM Agents](https://www.getmaxim.ai/articles/context-window-management-strategies-for-long-context-ai-agents-and-chatbots/)
- [Effective Context Engineering for AI Agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
- [The Context Window Problem](https://factory.ai/news/context-window-problem)

---

### Pitfall 4: Circular Dependency Deadlock

**What goes wrong:**
Ticket A blocks on B, B blocks on C, C blocks on A. All three tickets are marked "blocked" and no agent can make progress. The dependency graph contains cycles that prevent any forward movement.

**Why it happens:**
Dependencies are added incrementally as agents discover relationships. Without cycle detection, A can depend on B before anyone realizes B (or its transitive dependencies) already depends on A. Informational-only dependencies make this worse—agents add them freely without validation.

**How to avoid:**
- Implement cycle detection on dependency insertion (reject if creates cycle)
- Use depth-first search with path tracking during dependency add
- Provide clear error message: "Adding dependency A→B would create cycle: B→C→A"
- Alternatively: allow cycles but warn/flag them in list view
- Make dependency visualization available (show graph structure)

**Warning signs:**
- Multiple tickets perpetually stuck in "blocked" status
- Agents report "can't find any available work" when tickets exist
- Dependency chains that form loops when traced
- Tickets that reference each other in blocking relationships

**Phase to address:**
Phase 4 (Dependency Tracking) - Must implement cycle detection at insertion time

**Sources:**
- [Task Dependency Resolution and Scheduling](https://deepwiki.com/jdx/mise/5.2-task-execution-and-dependency-resolution)
- [Circular Dependency Detection](https://github.com/emosheeep/circular-dependency-scanner)

---

### Pitfall 5: Broken Pipe Silent Failure (Rust println! Issue)

**What goes wrong:**
Agent pipes CLI output to another tool (e.g., `rtik ls | head -5`) but the CLI continues trying to write after the consumer hangs up. In Rust, `println!` ignores EPIPE by default, causing the program to panic or hang instead of exiting gracefully.

**Why it happens:**
Rust's `println!` macro doesn't check for broken pipe errors. When stdout is closed by downstream process, continued writes panic. This is specific to Rust CLI applications and trips up many developers.

**How to avoid:**
- Never use `println!` for output—use explicit stdout handle with error handling
- Implement proper BrokenPipe error handling in main loop
- Use crates designed for CLI output (`clap` + explicit write handling)
- Return early on EPIPE instead of panicking
- Test with pipe consumers: `rtik ls | head -1` should exit cleanly

**Warning signs:**
- CLI panics when output is piped to `head`, `grep --max-count`, etc.
- "thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'"
- CLI hangs when downstream consumer terminates early
- Works fine with direct terminal output but fails in scripts

**Phase to address:**
Phase 1 (Core CRUD) - Fix during initial CLI implementation

**Sources:**
- [Rust CLI Error Handling](https://rust-cli.github.io/book/tutorial/errors.html)
- [Error Handling in Rust CLI Apps](https://github.com/rust-cli/team/issues/12)
- [Effective Error Handling in Rust CLI Apps](https://technorely.com/insights/effective-error-handling-in-rust-cli-apps-best-practices-examples-and-advanced-techniques)

---

### Pitfall 6: Silent Agent Failures (No Ownership Timeout)

**What goes wrong:**
Agent claims ticket, crashes/terminates unexpectedly, ticket remains "claimed" forever. No other agent can work on it because claimed_by is set, but the original agent is gone. Work silently stalls.

**Why it happens:**
Soft claiming provides flexibility but no accountability. Unlike hard distributed locks with TTL (time-to-live), ticket claims persist indefinitely. Dead agents don't release their claims.

**How to avoid:**
- Add claimed_at timestamp field
- List command shows stale claims (e.g., claimed >1 hour ago highlighted)
- Provide explicit "release" or "reset-claim" command for recovery
- Agents periodically heartbeat/refresh their claims
- Alternatively: add --force flag to override existing claims
- Consider "claim expiry" policy (auto-release after N hours)

**Warning signs:**
- Tickets claimed by agents that no longer exist
- Growing backlog of "WIP" tickets with old claim timestamps
- Agents report "no work available" but manual inspection shows WIP tickets
- Claim timestamps hours/days in the past

**Phase to address:**
Phase 2 (Claim Mechanism) - Add claimed_at timestamp from the start
Phase 6 (Search/Filter) - Show stale claims in list view

**Sources:**
- [Agent Coordination Is a Distributed Systems Problem](https://blog.kleisli.io/post/agent-coordination-distributed-systems)
- [Agent Systems Fail Quietly](https://bnjam.dev/posts/agent-orchestration/agent-systems-fail-quietly.html)

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip WAL mode setup | Simpler initial config | Concurrent access failures, poor multi-agent performance | Never—cost to fix later is low |
| Allow ticket description edits | Flexible ticket refinement | History lost, agents can't see what changed, audit trail broken | Never—use comment/update log instead |
| String-based status enum | Easy to add new statuses | Typos create invalid states, no type safety, filtering breaks | Never—use Rust enum from day one |
| Skip dependency validation | Faster dependency insertion | Circular dependencies brick the system | Never—validation is 20 lines of code |
| Store absolute file paths | Works on single machine | Breaks when database copied/shared, not portable | Acceptable for single-user/single-machine guarantee |
| Omit indexes on foreign keys | Faster writes initially | Slow queries as ticket count grows, dependency lookups crawl | Never—indexes are essential |
| Use unwrap() for DB operations | Faster development | Panics instead of graceful errors, no context in failures | Only in initial prototyping, must remove before Phase 1 complete |

## Integration Gotchas

Common mistakes when agents interact with the CLI.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Parsing plain text export | Regex on formatted output | Export structured JSON option for programmatic use, human-readable for manual |
| Multiple claim attempts | Retry loop without backoff | Exponential backoff, max 3 retries, fail gracefully if all fail |
| Assuming claim succeeded | Not checking exit code | Always check exit code, parse stderr for errors |
| Reading all tickets into context | Export full database | Filter query to relevant subset (--status todo --limit 10) |
| Concurrent database access | Multiple processes write simultaneously | Single writer pattern or WAL mode + proper busy timeout |
| Hardcoded database path | CLI assumes specific file location | Support --db-path flag + DB_PATH env var + XDG config standard |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| No indexes on claimed_by, status | List queries slow down | Add indexes during schema creation | >1000 tickets |
| Loading all tickets for filtering | Memory usage grows unbounded | Implement SQL-level filtering (WHERE clauses) | >10K tickets |
| N+1 dependency queries | Listing tickets with deps makes N queries | JOIN or single query with aggregation | >100 tickets with deps |
| Full table scan for search | Search slows linearly with size | Full-text search index (FTS5) for description field | >5K tickets |
| Unbounded export | Export all tickets to plain text | Add --limit flag, default to last 100 | >1000 tickets |
| No connection pooling | Each CLI invocation opens new connection | Acceptable for CLI (short-lived), but avoid in daemon mode | N/A for CLI |

## Security Mistakes

Domain-specific security issues beyond general application security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| World-readable database file | Ticket data exposed to all users | Set file permissions 0600 on DB creation |
| No input validation on ticket content | SQL injection via description field | Use parameterized queries, validate length limits |
| Executable code in ticket descriptions | Command injection if agents eval content | Never execute ticket content, treat as pure data |
| Storing secrets in ticket text | Credentials leaked in exports | Detect and reject common secret patterns (API keys, tokens) |
| No transaction rollback on error | Partial updates leave DB in inconsistent state | Wrap multi-statement operations in transactions |
| Shared database without access control | Agent A can modify Agent B's claimed tickets | Add validation: only owner can update claimed ticket |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Verbose default output | Agents waste tokens on excessive detail | Minimal output by default, --verbose flag for details |
| No visual distinction for ticket state | Hard to scan available work | Color-code status (green=todo, yellow=WIP, red=blocked), use symbols |
| Generic error messages | "Database error" doesn't help debug | Include context: "Failed to claim ticket #42: already claimed by agent-X" |
| Timestamps in absolute time | Confusing across timezones | Show relative time ("2 hours ago") in list view |
| No dry-run mode | Agents can't preview operations | Add --dry-run flag for destructive operations |
| Exit code always 0 | Scripts can't detect failures | Use standard exit codes: 0=success, 1=error, 2=usage error |
| JSON output mixed with logs | Can't parse JSON when debugging enabled | JSON to stdout, logs to stderr, never mix |
| Emojis without --plain flag | Breaks grep, parsing, non-UTF8 terminals | Plain ASCII by default, emojis opt-in with --fancy |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Claim mechanism:** Often missing atomic update check—verify only one agent can claim via row count
- [ ] **Status transitions:** Often missing validation—verify illegal transitions rejected (done→todo blocked)
- [ ] **Dependency tracking:** Often missing cycle detection—verify circular deps are rejected
- [ ] **Export format:** Often missing parseable option—verify both human-readable and JSON export
- [ ] **Search:** Often missing case-insensitive search—verify "FEAT" matches "feature implementation"
- [ ] **Timestamps:** Often missing timezone handling—verify UTC storage, local display
- [ ] **Exit codes:** Often missing non-zero on errors—verify failed operations exit 1
- [ ] **Concurrency:** Often missing WAL mode—verify multiple readers during writes
- [ ] **Error messages:** Often missing error context—verify errors include what failed and why
- [ ] **Idempotency:** Often missing duplicate protection—verify repeated claims of same ticket are safe

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Circular dependencies created | LOW | Add cycle detection validation, then manually break cycle with dependency delete |
| Database corruption from concurrent writes | HIGH | Restore from backup (if exists), otherwise manual data recovery; requires WAL mode fix |
| Stale claims blocking work | LOW | Add claimed_at timestamp column, reset claims older than threshold, release command |
| Lost context from missing history | MEDIUM | Add comments/updates table, can't recover past history but prevents future loss |
| Broken pipe panics in production | LOW | Fix error handling in code, redeploy, no data migration needed |
| No indexes causing slow queries | LOW | Add indexes with CREATE INDEX, no downtime, backward compatible |
| Double-claiming from race condition | MEDIUM | Fix atomic update logic, manually reassign duplicate work, add claim verification |
| Missing dependency validation | LOW | Add validation, audit existing deps for cycles, break any found |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| SQLite concurrent write starvation | Phase 1 (Core CRUD) | Multi-agent stress test: 10 parallel claim operations succeed |
| Race condition on ticket claiming | Phase 2 (Claim Mechanism) | Integration test: concurrent claims of same ticket, only one succeeds |
| Lost context window recovery | Phase 3 (Status Tracking) + Phase 5 (Export) | Agent can resume WIP ticket from export with full context |
| Circular dependency deadlock | Phase 4 (Dependency Tracking) | Unit test: circular dep rejected with clear error message |
| Broken pipe silent failure | Phase 1 (Core CRUD) | Test: `rtik ls \| head -1` exits 0, no panic |
| Silent agent failures | Phase 2 (Claim Mechanism) | Stale claim shown in list view, recoverable with release command |
| No indexes on query columns | Phase 1 (Core CRUD) | Query plan shows index usage for status, claimed_by filters |
| Unbounded ticket export | Phase 5 (Export Format) | Export defaults to last 100, --limit flag works |
| Inadequate error messages | Phase 1-6 (All phases) | Every error includes context: what, why, how to fix |
| Missing transaction rollback | Phase 1 (Core CRUD) | Simulate error mid-operation, verify DB remains consistent |

## Sources

**SQLite & Concurrency:**
- [SQLite Concurrency Best Practices](https://www.sqliteforum.com/p/handling-concurrency-in-sqlite-best)
- [Abusing SQLite to Handle Concurrency](https://blog.skypilot.co/abusing-sqlite-to-handle-concurrency/)
- [File Locking and Concurrency in SQLite](https://sqlite.org/lockingv3.html)
- [Four Ways to Handle SQLite Concurrency](https://medium.com/@gwendal.roue/four-different-ways-to-handle-sqlite-concurrency-db3bcc74d00e)
- [Optimizing SQLite for Multi-User Apps](https://www.sqliteforum.com/p/optimizing-sqlite-for-multi-user)

**Agent Coordination & Distributed Systems:**
- [Agent Coordination Is a Distributed Systems Problem](https://blog.kleisli.io/post/agent-coordination-distributed-systems)
- [Multi-Agent Coordination Strategies](https://galileo.ai/blog/multi-agent-coordination-strategies)
- [Agent Systems Fail Quietly](https://bnjam.dev/posts/agent-orchestration/agent-systems-fail-quietly.html)
- [Race Conditions in Distributed Systems](https://medium.com/hippo-engineering-blog/race-conditions-in-a-distributed-system-ea6823ee2548)
- [Handling Race Conditions with Pessimistic Locking](https://newsletter.scalablethread.com/p/how-distributed-systems-avoid-race)

**LLM Agent Context Management:**
- [Effective Context Engineering for AI Agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
- [Context Window Management for LLM Agents](https://www.getmaxim.ai/articles/context-window-management-strategies-for-long-context-ai-agents-and-chatbots/)
- [The Context Window Problem](https://factory.ai/news/context-window-problem)
- [Cutting Through the Noise: Smarter Context Management](https://blog.jetbrains.com/research/2025/12/efficient-context-management/)

**Rust CLI Best Practices:**
- [Rust CLI Error Handling](https://rust-cli.github.io/book/tutorial/errors.html)
- [Error Handling in Rust CLI Apps - GitHub Discussion](https://github.com/rust-cli/team/issues/12)
- [Effective Error Handling in Rust CLI Apps](https://technorely.com/insights/effective-error-handling-in-rust-cli-apps-best-practices-examples-and-advanced-techniques)

**CLI UX & Output Formatting:**
- [Command Line Interface Guidelines](https://clig.dev/)
- [UX Patterns for CLI Tools](https://www.lucasfcosta.com/blog/ux-patterns-cli-tools)
- [CLI Best Practices](https://hackmd.io/@arturtamborski/cli-best-practices)

**Task Dependencies & Circular Detection:**
- [Task Dependency Resolution and Scheduling](https://deepwiki.com/jdx/mise/5.2-task-execution-and-dependency-resolution)
- [Circular Dependency Scanner](https://github.com/emosheeep/circular-dependency-scanner)

**Backup & Data Protection:**
- [Data Loss Prevention Best Practices](https://www.crashplan.com/blog/6-data-loss-prevention-best-practices/)
- [Backup Strategy Best Practices](https://www.techtarget.com/searchdatabackup/feature/The-7-critical-backup-strategy-best-practices-to-keep-data-safe)

---
*Pitfalls research for: CLI Ticketing/Task Management for LLM Agents*
*Researched: 2026-02-22*
