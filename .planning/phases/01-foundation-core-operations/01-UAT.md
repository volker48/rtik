---
status: complete
phase: 01-foundation-core-operations
source: [01-01-SUMMARY.md, 01-02-SUMMARY.md, 01-03-SUMMARY.md]
started: 2026-02-22T16:30:00Z
updated: 2026-02-22T16:45:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Create a ticket
expected: Run `rtik create --name "Buy milk" --desc "Grocery store"` — prints `Created: #1 Buy milk` and exits 0
result: pass

### 2. Get a ticket
expected: Run `rtik get 1` — prints compact paragraph: `#1 Buy milk [todo]` on line 1, `Grocery store` on line 2, `Created: 2026-02-22 | Updated: 2026-02-22` on line 3
result: pass

### 3. Update a ticket status
expected: Run `rtik update 1 --status WIP` — prints `Updated: #1 Buy milk` with status normalized to lowercase (wip), exits 0
result: pass

### 4. List tickets
expected: Run `rtik list` — shows aligned table with ID/STATUS/NAME headers, ticket appears with status `wip`, name truncated at 40 chars if long
result: pass

### 5. List tickets with timestamps
expected: Run `rtik list --timestamps` — same table plus CREATED and UPDATED date columns (date only, not full ISO timestamp)
result: pass

### 6. Delete a ticket
expected: Run `rtik delete 1` — prints `Deleted: #1 Buy milk`, exits 0
result: pass

### 7. Error on missing ticket
expected: Run `rtik get 999` — prints error message to stderr, exits 1
result: pass

### 8. Error on update with no fields
expected: Run `rtik update 1` (no --name/--desc/--status flags) — prints error about required fields to stderr, exits 1
result: pass

### 9. DB persistence between invocations
expected: Create a ticket, then get it in a separate command — data persists. DB stored at `.rtik.db` in current directory (or RTIK_DB env var path)
result: pass

### 10. Broken pipe safety
expected: Run `rtik list | head -1` — outputs the header line cleanly, exits 0, no "Broken pipe" panic or error message
result: pass

## Summary

total: 10
passed: 10
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
