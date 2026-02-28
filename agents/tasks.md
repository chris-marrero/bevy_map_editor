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
| T-09 | data | IN PROGRESS | PR #1 (Geordi): GO given, **merged**. PR #3 (Barclay): returned for T-11 fix — **T-11 now done, fix pushed (ada4a38)**. Data to re-review PR #3. Note: resolve cross-branch contamination (Geordi's engine commit in Barclay's history) before merging — see Data's PADD. |
| T-10 | data | PENDING | After PR #3 merges: review PR #2 (Wesley UI: sprint/automapping/wesley-ui) once Wesley rebases on updated main. Give GO or return with required changes. |
| T-11 | barclay | DONE | Fix `find_layer_index` in `crates/bevy_map_automap/src/apply.rs`. Implemented correctly; `cargo check` passes. Pending push to `sprint/automapping/barclay-integration` (awaiting user authorization). |
| T-12 | data | PENDING | Add four missing DEBT table entries from Wesley's `automap_editor.rs` (found by Remmick audit). Layer combo selection discards `id` with `let _ = id;` (lines ~1197, ~1216). `make_default_rule()` and `make_default_output_alt()` use `Uuid::nil()` as placeholder `layer_id` (lines ~1233, ~1250). All four are functional voids — user selections silently dropped, rules always created with nil layer ID. Must be in DEBT table before Data can legally issue GO on PR #2. |
| T-13 | data | PENDING | Confirm scope of layer mapping persistence debt before sprint close. Known: rules reference layer IDs in the UI but serialization of layer-to-rule associations in the editor is not wired. Determine: is this in-scope for this sprint or deferred? Add DEBT table entry once scope is confirmed. |

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
- `find_layer_index` stub in apply.rs is live; recorded in DEBT table in agents/architecture/architecture.md.
- Layer mapping persistence in editor not yet implemented — in-flight debt, confirm scope with Data.
