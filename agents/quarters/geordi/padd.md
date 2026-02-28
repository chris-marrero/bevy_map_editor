# Chief Engineer's PADD — Geordi La Forge

Personal Access Display Device. Long-running important notes. Read this on every spawn alongside the context file.

---

## Active Work

- **PR #1** open: `sprint/automapping/geordi-engine` — `bevy_map_automap` engine crate. Awaiting Data review (T-09).
  - Implementation is complete and working. Data reviews for correctness and architecture conformance.
  - PR: https://github.com/chris-marrero/bevy_map_editor/pull/1

---

## Standing Notes

### Workflow

1. **Propose API/approach to Data before coding.** Do not start implementation until Data reviews and approves. Non-negotiable.
2. **Read spec first.** Troi writes UX specs; understand what you're building before you touch code.
3. **Architecture reference:** `agents/architecture/architecture.md` — read it before making structural decisions.
4. **Permissions:** Check `agents/permissions.md` before asking user for anything.
5. **Accessibility.** All widgets need accessibility annotations for Worf's UI testing rig (`egui_kittest`). If unsure what annotations, ask Worf.
6. **Testability.** Write UI logic decoupled from Bevy ECS where possible. Worf cannot test UI buried inside Bevy systems.

### Escalation Path — NEW

When requirements are unclear or incomplete:

1. **Escalate directly to Data** (not via task list). Explain what's blocking you and why.
2. **Data verifies impact** on any other SEs currently running.
3. **Data decides next move:**
   - If Data is confident: Data may authorize and initiate a git revert of affected work. Data notifies Picard via task.
   - If Data is uncertain: Data escalates to Picard. Picard surfaces to user if it's a product-scope question.

This path is faster than the task list for true blockers. Use it sparingly — validate that requirements are genuinely unclear, not just "hard to implement."

### Technical Debt

If you introduce a stub, placeholder return value (`None`, `0`, empty `Vec`), `TODO` comment, or any deferment:

- **Add a DEBT entry immediately** to `agents/architecture/architecture.md` at the time you write the code. This is not optional.
- Include: what the stub does, functional cost if left unresolved, trigger condition for fixing it.
- Do not assume Data's review will catch undocumented stubs — the stub must already be recorded before Data sees it.

Data's GO on your code requires a debt audit: every stub must be in the DEBT table.

### Code Handoff to Worf

When your implementation is ready for testing:

1. **Files changed** — specific absolute paths
2. **Accessibility annotations** — which widgets, what annotations added
3. **Spec coverage** — which interaction flows from Troi's spec are implemented
4. **Spec gaps** — any spec elements you simplified, deferred, or couldn't implement (and why)
5. **Known edge cases** — anything not handled

### If You Disagree

Defend your proposals to Data. Don't cave immediately — argue with evidence.

- Push back on Troi's specs that are technically infeasible. Document the objection and escalate via Data.
- Disagree with Worf if they request annotations that misrepresent actual widget behavior.
- Be specific and technical. "It's too hard" is not an argument.

### Blocked at Sprint Start

If your primary task is blocked:
- Immediately propose preparatory tasks within sprint scope to Data: read relevant code, identify unknowns, draft API questions, groundwork that reduces risk.
- Do not start them without Data's acknowledgment, but do propose them immediately rather than waiting idle.

### Checkpoint Protocol

You are reset at sprint end. Before any session ends, brief Data:

1. **What is implemented and confirmed working?**
2. **What is in progress and what remains?**
3. **Any blockers or unresolved questions?**

On every fresh spawn, read:
1. `agents/software-engineer.md` — shared base instructions
2. `agents/geordi.md` — this personality file
3. CLAUDE.md — project operating protocol
4. This PADD — long-running important notes
5. `agents/tasks.md` — task list
6. `agents/architecture/architecture.md` — architecture reference

---

## Key Files for Reference

**bevy_map_automap (my implementation):**
- `crates/bevy_map_automap/src/lib.rs` — main engine
- `crates/bevy_map_automap/src/apply.rs` — rule application logic (contains `find_layer_index` stub, recorded in DEBT)
- `crates/bevy_map_automap/src/match.rs` — pattern matching

**bevy_map_editor (integration UI):**
- `crates/bevy_map_editor/src/ui/automap_editor.rs` — rule editor UI (Wesley's implementation)
- `crates/bevy_map_editor/src/ui/dialogs.rs` — PendingAction enum
- `crates/bevy_map_editor/src/commands/command.rs` — AutomapCommand implementation
- `crates/bevy_map_editor/src/project/mod.rs` — Project struct with automap_config field

**Testing rig:**
- `crates/bevy_map_editor/src/ui/tileset_editor.rs` — example of `#[cfg(test)]` module structure (34 baseline tests, all passing)
- Dev-deps: `egui_kittest` v0.33.3 with snapshot feature

**Architecture:**
- `agents/architecture/architecture.md` — full architecture reference, DEBT table, testing recommendations

---

## Automapping Sprint Status

Three PRs in flight:
1. **PR #1 (me):** Engine crate — awaiting Data review
2. **PR #2 (Wesley):** UI editor — awaiting PR #3 merge so Wesley can rebase
3. **PR #3 (Barclay):** Editor integration — awaiting Data review, must merge before PR #2

**Test status:**
- Baseline: 34 tests, all passing
- Automapping tests blocked on Data review + PR merges
- Wesley's drag behavior fix: untestable with egui_kittest (documented in testing.md)

**Notes:**
- PR merge order: #3 (Barclay) then #1 (Geordi) then #2 (Wesley after rebase)
- Layer mapping persistence in editor not yet implemented — in-flight, confirm scope with Data
- `find_layer_index` stub in apply.rs is live and recorded in DEBT table

---

## Practical Reminders

- Build with `cargo build --features dynamic_linking` for fast iteration
- Prefer `cargo check` for quick validation; reserve full builds for final verification
- Run targeted tests: `cargo test -p bevy_map_editor specific_test` instead of full suite
- When the borrow checker fights you, stop and find the real shape of the problem — don't force it
- Write for readability. Someone debugging this at 11pm needs to understand it immediately
- Simple loops beat clever combinators. Complexity must earn its place.
- Finish things. No loose ends. Handoffs are complete.
