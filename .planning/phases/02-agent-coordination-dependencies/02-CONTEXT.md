# Phase 2: Agent Coordination & Dependencies - Context

**Gathered:** 2026-02-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Multiple agents coordinate work through atomic ticket claiming and dependency tracking. Delivers:
atomic claim/release operations, status state machine enforcement, and dependency graph management —
all via CLI. Search/filtering and export are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Claim behavior
- Failed claims print the current owner and claim time: `Error: ticket #5 claimed by agent-2 since 14:23`
- Force-claiming supported via `--force` flag (overrides another agent's claim)
- Force-release supported via `--force` flag (anyone can release, not just owner)
- Successful claim prints minimal confirmation: `Claimed #ID`
- Agent identity for `claimed_by`: Claude's discretion (environment variable or positional arg)
- Claiming a ticket auto-sets its status to `in-progress` (fewer commands for agents)
- Releasing a ticket resets status back to `todo`
- Claiming a ticket with unmet deps: warn but allow (`Warning: 2 dependencies not done`)

### Status state machine
- Valid statuses: `todo`, `in-progress`, `blocked`, `done`
- `blocked` works both ways:
  - Auto-computed: ticket with unmet deps is shown as blocked
  - Manually settable: agent explicitly blocks with `rtik block #ID <reason>` (reason required)
  - Missing reason on `rtik block` → exit 2 with usage error
- Done is re-openable to `in-progress` (edge case: bug found, needs rework)
- Marking done auto-releases the claim
- Valid transition enforcement: Claude's discretion on exact rules (prevent real coordination bugs)
- Invalid transition error shows valid options: `Error: from done, valid transitions are: in-progress`

### Dependency display
- In list view (`rtik ls`): Claude's discretion on what to show
- In detail view (`rtik get #ID`): compact dep list `Depends on: #3, #7` plus reverse deps `Required by: #9, #12`
- Dedicated `rtik deps #ID` command for dependency tree view

### Coordination error messages
- Force operations warn to stderr: `Warning: overriding claim by agent-2`
- Circular dep error shows the cycle: `Error: cycle: #3 → #7 → #3`
- Status transition errors show valid options: `Error: from done, valid transitions are: in-progress`

### Claude's Discretion
- Agent identity mechanism (env var vs positional arg)
- Exact status transition rules beyond the stated constraints
- Dependency count/blocked indicator format in list view (`rtik ls`)
- Depth/format of `rtik deps #ID` tree output

</decisions>

<specifics>
## Specific Ideas

- `blocked` status has dual meaning: auto (unmet deps) and manual (agent-set with reason)
- Release is a full undo: claim cleared + status reset to `todo`
- Done is not terminal: re-openable to `in-progress` for rework scenarios

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-agent-coordination-dependencies*
*Context gathered: 2026-02-22*
