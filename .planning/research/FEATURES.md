# Feature Research

**Domain:** CLI Ticketing/Task Management Systems
**Researched:** 2026-02-22
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Create/Read/Update/Delete tickets | Core CRUD operations are fundamental to any task system | LOW | Standard database operations with CLI interface |
| List/filter tickets | Users need to query and view tickets by various criteria | LOW | Basic query with WHERE clauses, filtering by status/tags |
| Unique ticket IDs | Essential for referencing and tracking specific tickets | LOW | Auto-incrementing IDs or UUIDs |
| Ticket status field | Users must track progress (todo, in progress, done, blocked) | LOW | Enum or string field with predefined values |
| Ticket descriptions | Context about what the ticket is for | LOW | Text field storage |
| Timestamps | When tickets were created and last updated | LOW | Auto-populated datetime fields |
| Search functionality | Find tickets by name or description content | LOW-MEDIUM | Full-text search on text fields |
| Plain text export | CLI tools need readable, parseable output formats | LOW | Format data as text/JSON/CSV |
| Persistent storage | Tasks must survive between sessions | LOW | SQLite or similar file-based database |

### Differentiators (Competitive Advantage)

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Agent-optimized export format | Minimizes token usage when agents load context | LOW | Custom format with only essential fields (ID, name, description, deps) |
| Soft claiming (reassignable) | Agents can crash/abandon work; flexible reassignment prevents deadlocks | MEDIUM | Ownership tracking with explicit transfer mechanism |
| Dependency tracking (informational) | Shows relationships without hard enforcement; agents decide context-specifically | MEDIUM | Graph structure stored in DB, non-blocking validation |
| Git-based sync/undo | Distributed teams can sync without server; built-in version control | HIGH | Requires git integration, conflict resolution |
| Context system (auto-filtering) | Automatically applies filters/tags based on working context to reduce cognitive load | MEDIUM | Stateful context tracking, automatic tag application |
| Markdown notes per ticket | Rich formatting for detailed planning without leaving plain text | LOW-MEDIUM | Store markdown, optionally render or display raw |
| Custom attributes/fields | Users can extend schema for domain-specific needs | MEDIUM | Dynamic schema or JSON field storage |
| Urgency calculation | Automatic priority based on multiple factors (due date, age, dependencies) | MEDIUM | Algorithm weighing various attributes |
| Virtual tags | Computed tags like +BLOCKED, +BLOCKING based on ticket state | MEDIUM | Runtime evaluation during queries |
| Single binary distribution | No runtime dependencies makes deployment trivial | LOW-MEDIUM | Static linking (Rust naturally supports this) |
| Parallel agent coordination | Multiple agents work simultaneously with visibility into claims | MEDIUM | Requires atomic claim operations, clear ownership display |
| Offline-first design | Work fully offline, sync when connected | HIGH | Requires conflict resolution strategy |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Hard dependency enforcement | Seems logical to block tasks until dependencies complete | Agents need flexibility to work out of order based on context; creates artificial bottlenecks | Track dependencies as informational; let agents decide |
| Real-time collaboration/sync | Users want instant updates across devices | Adds massive complexity (websockets, conflict resolution, server infrastructure) for minimal value in agent use cases | Git-based sync or periodic refresh is sufficient |
| Rich GUI/Web interface | Visual interfaces seem more "modern" | CLI tool for agents; GUI adds maintenance burden and distracts from core use case | Stay CLI-focused; export to formats other tools can visualize |
| User authentication/permissions | Multi-user systems "need" security | Single-user local tool; auth adds complexity without value | Assume single user on local machine |
| Recurring tasks | Users want automated task creation | Complex edge cases (scheduling, timezone handling, exceptions); better handled by external schedulers | Document how to use cron/scheduled tasks to create tickets |
| Time tracking built-in | Seems useful for productivity metrics | Feature bloat; many dedicated time tracking tools exist; hard to get right | Allow external time tracking tools to reference ticket IDs |
| Email/notification integration | Users want to be "notified" of updates | Agents poll for work; notifications add complexity without benefit | Export format allows external notification systems to monitor |
| Complex workflow states | More granular statuses seem more powerful | Creates decision paralysis; simple states (todo/WIP/blocked/done) cover 95% of cases | Keep status enum minimal; use tags for additional context |
| JIRA-like feature completeness | Users want "everything JIRA has" | Feature bloat; tool becomes complex and slow; loses "lightweight" advantage | Stay ruthlessly minimal; integrate with full tools when needed |
| Hierarchical tasks (subtasks) | Nested structure seems organized | Adds query complexity; agents can model this with dependencies instead | Use dependencies and tags to model relationships |

## Feature Dependencies

```
[Plain text export]
    └──requires──> [Ticket CRUD]

[Search functionality]
    └──requires──> [Ticket storage]

[Dependency tracking]
    └──requires──> [Unique IDs]

[Virtual tags]
    └──requires──> [Dependency tracking]

[Soft claiming]
    └──requires──> [Unique IDs]
    └──requires──> [Timestamps]

[Context system] ──enhances──> [List/filter tickets]

[Git-based sync]
    └──requires──> [Plain text storage format]
    └──conflicts──> [Real-time sync]

[Urgency calculation]
    └──requires──> [Timestamps]
    └──requires──> [Dependencies]
    └──requires──> [Status field]

[Agent coordination]
    └──requires──> [Soft claiming]
    └──requires──> [List/filter]

[Offline-first] ──conflicts──> [Real-time features]

[Single binary] ──requires──> [Embedded database (SQLite)]
```

### Dependency Notes

- **Plain text export requires Ticket CRUD:** Can't export what doesn't exist
- **Virtual tags require Dependency tracking:** Tags like +BLOCKED computed from dependency graph
- **Git-based sync conflicts with Real-time sync:** Can't have both; different architecture paradigms
- **Offline-first conflicts with Real-time features:** Real-time requires constant connectivity
- **Context system enhances List/filter:** Adds automatic filtering but doesn't block basic filtering
- **Agent coordination requires Soft claiming:** Need ownership visibility for parallel work

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [x] **Ticket CRUD operations** — Can't be a task system without create/read/update/delete
- [x] **Unique ticket IDs** — Required for referencing and dependencies
- [x] **Status field (todo/WIP/blocked/done)** — Core state tracking for agents
- [x] **Description field** — Context about what needs to be done
- [x] **Timestamps (created_at, updated_at)** — Track when work was added/changed
- [x] **Soft claiming mechanism** — Agent coordination requires ownership visibility
- [x] **List/filter tickets** — Agents query for available work
- [x] **Dependency tracking (informational)** — Show relationships without blocking
- [x] **Search by name/description** — Find relevant tickets quickly
- [x] **Plain text export format** — Agent-optimized, token-efficient output
- [x] **SQLite storage** — Persistent, file-based, no server required
- [x] **Short CLI aliases** — Speed-optimized commands (new, ls, claim, etc.)

**Rationale:** These 12 features provide complete agent workflow: create tickets, claim work, track dependencies, update status, search/filter for next task, export for context. Everything else is enhancement.

### Add After Validation (v1.x)

Features to add once core is working and validated with real agent usage.

- [ ] **Virtual tags (+BLOCKED, +BLOCKING, etc.)** — Add when dependency tracking proves useful; triggers: users manually checking dependencies frequently
- [ ] **Context system** — Add when filter patterns become repetitive; triggers: users running same filter combinations repeatedly
- [ ] **Urgency calculation** — Add when prioritization becomes an issue; triggers: users asking "what should I work on next?"
- [ ] **Markdown notes per ticket** — Add when descriptions become unwieldy; triggers: users requesting richer formatting
- [ ] **Custom attributes** — Add when users request domain-specific fields; triggers: multiple users asking for same extension points
- [ ] **Git-based sync** — Add when multi-device usage is validated; triggers: users manually copying DB files between machines

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Offline-first with sync** — Defer until distributed team usage is validated; requires significant conflict resolution work
- [ ] **Advanced reporting/analytics** — Defer until patterns emerge in usage data
- [ ] **Integrations (GitHub, JIRA, etc.)** — Defer until clear integration needs emerge
- [ ] **Plugin system** — Defer until extension patterns are established

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority | MVP |
|---------|------------|---------------------|----------|-----|
| Ticket CRUD | HIGH | LOW | P1 | ✓ |
| Unique IDs | HIGH | LOW | P1 | ✓ |
| Status field | HIGH | LOW | P1 | ✓ |
| Description | HIGH | LOW | P1 | ✓ |
| Timestamps | HIGH | LOW | P1 | ✓ |
| Soft claiming | HIGH | MEDIUM | P1 | ✓ |
| List/filter | HIGH | LOW | P1 | ✓ |
| Dependencies (informational) | HIGH | MEDIUM | P1 | ✓ |
| Search | HIGH | MEDIUM | P1 | ✓ |
| Plain text export | HIGH | LOW | P1 | ✓ |
| SQLite storage | HIGH | LOW | P1 | ✓ |
| Short CLI aliases | HIGH | LOW | P1 | ✓ |
| Virtual tags | MEDIUM | MEDIUM | P2 | |
| Context system | MEDIUM | MEDIUM | P2 | |
| Urgency calculation | MEDIUM | MEDIUM | P2 | |
| Markdown notes | MEDIUM | LOW-MEDIUM | P2 | |
| Custom attributes | MEDIUM | MEDIUM | P2 | |
| Git sync | MEDIUM | HIGH | P2 | |
| Offline-first sync | LOW | HIGH | P3 | |
| Advanced analytics | LOW | MEDIUM | P3 | |
| Integrations | LOW | HIGH | P3 | |
| Plugin system | LOW | HIGH | P3 | |

**Priority key:**
- P1: Must have for launch (MVP)
- P2: Should have, add when usage patterns validate need
- P3: Nice to have, future consideration after PMF

## Competitor Feature Analysis

| Feature | Taskwarrior | Todo.txt | dstask | git-bug | Our Approach |
|---------|-------------|----------|--------|---------|--------------|
| Dependencies | Full support with blocking | No native support | Git-based tracking | Issue links | Informational only; no hard enforcement |
| Tags | Unlimited tags | Context (@) and projects (+) | Tags + projects | Labels | Standard tagging + virtual computed tags |
| Priority | H/M/L + urgency calc | (A-Z) manual | Manual priority | No explicit priority | Status-based + optional urgency |
| Storage | Text files | Single text file | Git repo | Git repo | SQLite (queryable, fast) |
| Sync | Taskserver (complex) | Dropbox/manual | Git push/pull | Git push/pull | Future: git-based; v1: local only |
| Claiming/ownership | No built-in | No built-in | No built-in | No built-in | Core feature: soft claiming for agents |
| Recurrence | Full support | No support | No support | No (manual) | Anti-feature: use external scheduler |
| Sub-tasks | Via dependencies | No support | Via dependencies | No native | Anti-feature: use dependencies + tags |
| CLI speed | Fast but verbose | Extremely fast | Fast | Moderate (git overhead) | Optimized: short aliases, single binary |
| Agent optimization | Not designed for agents | Not designed for agents | Not designed for agents | Not designed for agents | Purpose-built: token-efficient export, soft claiming |

## Sources

### CLI Task Management Tools
- [Slant: Todo.txt vs Taskwarrior comparison](https://www.slant.co/versus/4416/4422/~todo-txt_vs_taskwarrior)
- [LWN: Managing tasks with todo.txt and Taskwarrior](https://lwn.net/Articles/824333/)
- [Taskwarrior best practices documentation](https://taskwarrior.org/docs/best-practices/)
- [Taskwarrior workflow examples](https://taskwarrior.org/docs/workflow/)
- [GitHub: dstask - Git powered terminal-based todo manager](https://github.com/naggie/dstask)

### Ticketing Systems & Features
- [The Digital Project Manager: Best Project Management Ticketing Systems](https://thedigitalprojectmanager.com/tools/best-project-management-ticketing-system/)
- [InvGate: Ticketing System Guide for 2026](https://blog.invgate.com/ticketing-system)
- [Faveo: IT Service Desk Ticketing System Features](https://www.faveohelpdesk.com/it-service-desk-ticketing-system-for-2026/)

### LLM Agent Tools & Architecture
- [AImultiple: LLM Orchestration frameworks](https://research.aimultiple.com/llm-orchestration/)
- [Lasso Security: Top Agentic AI Tools in 2026](https://www.lasso.security/blog/agentic-ai-tools)
- [Medium: Agentic AI in 2026](https://medium.com/@kkamdar/agentic-ai-in-2026-llms-are-no-longer-just-chatbots-theyre-running-the-show-334659be6a2d)
- [Towards Data Science: How Agents Plan Tasks with To-Do Lists](https://towardsdatascience.com/how-agents-plan-tasks-with-to-do-lists/)

### Lightweight CLI Trackers
- [GitHub: issue - Simple distributed command-line issue tracker](https://github.com/marekjm/issue)
- [GitHub: trackdown - Issue Tracking with plain Markdown](https://github.com/mgoellnitz/trackdown)

### Plain Text Formats & Interoperability
- [Todo.txt: Future-proof task tracking](http://todotxt.org/)
- [TasksMD: Open Data Task Management Using Markdown](https://tasks.md/)
- [Plaintext Productivity website](https://plaintext-productivity.net/)

### Feature Bloat & Pitfalls
- [ZenHub: Daily Task Management Optimized](https://www.zenhub.com/blog-posts/daily-task-management-explained-and-optimized)
- [ClickUp: Best Task Management Software](https://clickup.com/blog/task-management-software/)

---
*Feature research for: CLI Ticketing/Task Management Systems*
*Researched: 2026-02-22*
