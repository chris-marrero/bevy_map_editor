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
| T-09 | data | PENDING | Review PR #1 (Geordi engine: sprint/automapping/geordi-engine) and PR #3 (Barclay integration: sprint/automapping/barclay-integration). Give GO or return with required changes. Merge #3 after GO — this unblocks Wesley's rebase. |
| T-10 | data | PENDING | After PR #3 merges: review PR #2 (Wesley UI: sprint/automapping/wesley-ui) once Wesley rebases on updated main. Give GO or return with required changes. |

### Blocked

| ID | Assigned | Status | Blocked By | Description |
|---|---|---|---|---|
| T-05 | worf | PENDING | T-09, T-10 | Write and run tests for automapping engine + editor integration. Blocked on Data review of PRs #1/#3 and merge of PR #3 (at minimum). |

### Completed

| ID | Assigned | Status | Description |
|---|---|---|---|
| T-01 | troi | DONE | Automap Rule Editor UX spec written. Full spec at agents/automap_ux_spec.md. |
| T-02 | data | DONE | bevy_map_automap architecture and data model designed. |
| T-03 | geordi | DONE | bevy_map_automap engine crate. PR #1 open: https://github.com/chris-marrero/bevy_map_editor/pull/1 |
| T-04 | wesley | DONE | Automap rule editor UI implemented per Troi's spec. PR #2 open: sprint/automapping/wesley-ui. NOTE: PR #2 depends on PR #3 merging first — Wesley must rebase after Barclay's branch merges. |
| T-06 | worf | DONE | Baseline verification: 34 tests, all passing. |
| T-07 | worf | DONE | Automapping test plan written. Recorded in testing.md. |
| T-08 | barclay | DONE | Editor integration: Layer::id, automap_config on Project, AutomapCommand, preferences field, dialogs, layer-delete hook. PR #3 open: sprint/automapping/barclay-integration. cargo check passes. |

---

## Notes

- Final test count (pre-automapping sprint): 34 (20 existing + 14 new). All passing.
- Test file: `crates/bevy_map_editor/src/ui/tileset_editor.rs` — `#[cfg(test)] mod tests` at bottom.
- Drag behavior (Wesley's fix) is untestable with egui_kittest. Documented in testing.md.
- Automapping tests blocked pending Data review + PR merges.
- **PR merge order:** PR #3 (Barclay) must merge before PR #2 (Wesley) can rebase. Data reviews #1 and #3 first.
- `find_layer_index` stub in apply.rs is live; recorded in DEBT table in architecture.md.
- Layer mapping persistence in editor not yet implemented — in-flight debt, confirm scope with Data.
