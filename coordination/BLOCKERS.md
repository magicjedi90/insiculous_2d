# Blockers - Insiculous 2D

**Format:** Agents document issues they can't resolve here. Include what you tried.

---

## Active Blockers

(No blockers yet)

<!-- Example entry:
### BLOCKER-001: PhysicsSystem requires mutable World borrow conflict
**Task:** TASK-015 (spatial hash grid)
**Agent:** agent-03
**Date:** 2026-02-07
**Problem:** Can't integrate spatial hash query during PhysicsSystem::update() because both need &mut World
**What I tried:**
1. Splitting the borrow into two phases (query first, then update) - didn't work because query results reference World
2. Using indices instead of references - partial success but Collider lookup still needs &World
3. Collecting entity IDs first, then processing - works but defeats the purpose of spatial optimization
**Suggested fix:** Maybe a separate SpatialIndex struct that doesn't borrow World, populated before physics step
**Status:** OPEN - needs human review or another agent's input
-->

## Resolved Blockers

(None yet)
