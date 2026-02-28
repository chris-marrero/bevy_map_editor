# Task List

Maintained by the Lead. SEs read this; they do not write to it.

---

## Sprint: Collision Editor Bug Fix + Numeric Input — Test Pass

### Active Tasks

_(none)_

### Completed

| ID | Assigned | Status | Description |
|---|---|---|---|
| T-01 | worf | DONE | Run existing 20 tests — all pass. Baseline confirmed. |
| T-02 | worf | DONE | Write label-presence tests for numeric input panel — 10 tests, all passing. |
| T-03 | worf | DONE | Assess drag behavior testability; wrote 4 smoke tests (render without panic), all passing. Drag behavior NOT testable with current rig — see testing.md. |

---

## Sprint: Automapping

### Active Tasks

| ID | Assigned | Status | Description |
|---|---|---|---|
| T-09 | data | IN PROGRESS | PR #1 (Geordi): GO given, **merged**. PR #3 (Barclay): T-11 fix pushed (ada4a38). **Data re-review complete: GO on content.** Cross-branch contamination confirmed: merge base is `f16e549` (pre-engine-crate). Engine crate commit `942ff8a` is in Barclay's history; main already has it via `1c803e5`. Barclay branch must be rebased onto current main (`origin/main`) before merge. Rebase will skip `942ff8a` (identical patch-id to `7768402`). Rebased branch will be clean: 6 Barclay-only commits on top of main. **Requires push permission for force-push after rebase.** See T-16. |
| T-10 | data | PENDING | After PR #3 merges: review PR #2 (Wesley UI: sprint/automapping/wesley-ui) once Wesley rebases on updated main. Give GO or return with required changes. Note: PR #2 review requires T-12 debt entries to be in DEBT table before GO (now done). |
| T-11 | barclay | DONE | Fix `find_layer_index` in `crates/bevy_map_automap/src/apply.rs`. Implemented correctly; `cargo check` passes. Pushed to `sprint/automapping/barclay-integration` as ada4a38. |
| T-12 | data | DONE | Five DEBT table entries added to `agents/architecture/architecture.md` for Wesley's `automap_editor.rs` stubs: two `let _ = id;` combo discards (lines 1199, 1217), `Uuid::nil()` in `make_default_rule()` (line 1239), `make_default_output_alt()` (line 1254), and `ensure_input_group()` (line 1274 — fifth instance found by Data audit, not in original Remmick report). All five are functional voids; all documented. |
| T-13 | data | DONE | Layer mapping persistence scope confirmed: **in-scope for this automapping sprint**. `TODO(barclay-merge)` labels indicate intentional deferment until PR #3 merged, not out-of-scope work. PR #3 is now ready to merge. After merge, Wesley must wire the layer combo selection path. DEBT entry added to architecture.md. T-16 (Wesley wiring task) created as follow-on. |
| T-15 | riker | DEFERRED (next sprint close) | Update Riker's startup sequence: (1) add read of last `agents/ship_log/mission N - *.md` entry before making protocol changes; (2) add PADD read (`agents/quarters/riker/padd.md`) to startup — missing for Riker, present for all other crew. Also update CLAUDE.md to document the ship_log system: Lead writes a mission log entry per sprint at sprint close; Riker reads the last entry on spawn. |
| T-14 | riker | DONE | Update CLAUDE.md to establish standing procedure: agent knowledge base files committed to main after every session where agent files are modified. Section "Agent Knowledge Base: Session Commit Procedure" added to CLAUDE.md. remmick.md ratified (YAML frontmatter stripped to match project conventions). Committed and pushed to main. |
| T-16 | lead | PENDING | **Authorization required.** Data needs push access to rebase and force-push `sprint/automapping/barclay-integration` onto current main (removes cross-branch engine crate contamination), then merge PR #3. Steps: (1) checkout branch, (2) `git rebase origin/main` (git will skip `942ff8a` via patch-id match), (3) `git push --force-with-lease origin sprint/automapping/barclay-integration`, (4) `gh pr merge` (once PR is open on GitHub). Currently only local branches exist — no remote PR is open for PR #3. User must either: (a) authorize agent to push and create PR, or (b) do the rebase+push themselves. |
| T-17 | wesley | PENDING (blocked on T-09/PR #3 merge) | After PR #3 merges and PR #2 rebases: wire the layer combo selection path in `automap_editor.rs`. Four locations to update: input combo `let _ = id;` (line 1199) → store selected `layer_id` in `AutomapEditorState`, then propagate to `InputConditionGroup.layer_id`. Output combo `let _ = id;` (line 1217) → same for `OutputAlternative.layer_id`. `make_default_rule()` (line 1239) and `make_default_output_alt()` (line 1254) should default to first available layer from project if present, not `Uuid::nil()`. `ensure_input_group()` (line 1274) same as `make_default_rule()`. Data must review proposal before coding begins. |

### Blocked

| ID | Assigned | Status | Blocked By | Description |
|---|---|---|---|---|
| T-05 | worf | PENDING | T-09, T-10 | Write and run tests for automapping engine + editor integration. Blocked on Data review of PRs #1/#3 and merge of PR #3 (at minimum). |

### Completed

| ID | Assigned | Status | Description |
|---|---|---|---|
| T-01 | troi | DONE | Automap Rule Editor UX spec written. Full spec at agents/automap_ux_spec.md. |
| T-02 | data | DONE | bevy_map_automap architecture and data model designed. |
| T-03 | geordi | DONE | bevy_map_automap engine crate. PR #1 reviewed by Data — GO given, **merged**. |
| T-04 | wesley | DONE | Automap rule editor UI implemented per Troi's spec. PR #2 open: sprint/automapping/wesley-ui. NOTE: PR #2 depends on PR #3 merging first — Wesley must rebase after Barclay's branch merges. |
| T-06 | worf | DONE | Baseline verification: 34 tests, all passing. |
| T-07 | worf | DONE | Automapping test plan written. Recorded in testing.md. |
| T-08 | barclay | DONE | Editor integration: Layer::id, automap_config on Project, AutomapCommand, preferences field, dialogs, layer-delete hook. PR #3 open: sprint/automapping/barclay-integration. cargo check passes. |

---

## Notes

- Final test count (pre-automapping sprint): 34 (20 existing + 14 new). All passing.
- Test file: `crates/bevy_map_editor/src/ui/tileset_editor.rs` — `#[cfg(test)] mod tests` at bottom.
- Drag behavior (Wesley's fix) is untestable with egui_kittest. Documented in agents/architecture/testing.md.
- Automapping tests blocked pending Data review + PR merges.
- **PR merge order:** PR #3 (Barclay) must merge before PR #2 (Wesley) can rebase. Data reviews #1 and #3 first.
- `find_layer_index` stub in apply.rs **RESOLVED** (T-11, Barclay, ada4a38) — fix is in PR #3 pending merge.
- Layer mapping persistence: in-scope debt confirmed (T-13 DONE). Wesley wiring task created as T-17. Blocked on PR #3 merge.
- **T-16 (lead):** Authorization needed to rebase + force-push Barclay branch and merge PR #3. No GitHub PR currently open for Barclay's branch.
