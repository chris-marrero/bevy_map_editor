# Task List

Maintained by the Lead. SEs read this; they do not write to it.

---

## Sprint: Collision Editor Bug Fix + Numeric Input — Test Pass

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
| T-10 | data | RETURNED — awaiting T-19, T-20 fixes | PR #2 reviewed. Two blocking issues found. GO withheld. See T-19 and T-20. T-10 re-opens for final review after Wesley's fixes. |
| T-15 | riker | DEFERRED (next sprint close) | Update Riker startup sequence: (1) read last `agents/ship_log/mission N - *.md` entry and `agents/incident_log.md` before making changes; (2) add PADD read to startup. Update CLAUDE.md to: document ship_log system, document incident_log, add Guinan to sprint close sequence (runs before Riker), add instruction for all agents to append to incident_log on failure, add instruction for Picard to log user corrections. Forward this session's user message about Guinan/incident_log as context. |
| T-19 | wesley | PENDING | Fix context menu for rule set delete in `automap_editor.rs`. Currently attached to zero-size `ui.horizontal(|_ui| {}).response` — unreachable by user. Attach `.context_menu()` to the `selectable_label` response instead (capture it from `ui.selectable_label(selected, &name)`). Branch: `sprint/automapping/wesley-ui`. |
| T-20 | wesley | PENDING | Fix double-label rendering on ComboBoxes in `automap_editor.rs`. Pattern `ui.label("X") + ComboBox::from_label("X")` renders the label twice. Affected combos: Edge Handling, Apply Mode, Brush Type (input), Brush (output), Tile selectors (both). Choose one pattern: either `ComboBox::from_label("X")` alone (label renders to the right per egui convention), or `ui.label("X") + ComboBox::from_id_salt(id)`. Do NOT use both together. Branch: `sprint/automapping/wesley-ui`. |

### Blocked

| ID | Assigned | Status | Blocked By | Description |
|---|---|---|---|---|
| T-05 | worf | PENDING | T-10 | Write and run tests for automapping engine + editor integration. Blocked on Data GO on PR #2. |

### Completed

| ID | Assigned | Status | Description |
|---|---|---|---|
| T-01 | troi | DONE | Automap Rule Editor UX spec written. Full spec at agents/automap_ux_spec.md. |
| T-02 | data | DONE | bevy_map_automap architecture and data model designed. |
| T-03 | geordi | DONE | bevy_map_automap engine crate. PR #1 merged. |
| T-04 | wesley | DONE | Automap rule editor UI implemented per Troi's spec. |
| T-06 | worf | DONE | Baseline verification: 34 tests, all passing. |
| T-07 | worf | DONE | Automapping test plan written. Recorded in testing.md. |
| T-08 | barclay | DONE | Editor integration: Layer::id, automap_config on Project, AutomapCommand, preferences field, dialogs, layer-delete hook. PR #3 merged. |
| T-09 | data | DONE | PR #1 (Geordi): GO, merged. PR #3 (Barclay): rebased, GO, merged to main (4aa06c4). |
| T-11 | barclay | DONE | `find_layer_index` implemented (ada4a38), included in PR #3 merge. |
| T-12 | data | DONE | Five DEBT entries added for Wesley's automap_editor.rs stubs. |
| T-13 | data | DONE | Layer mapping persistence confirmed in-scope. DEBT entry added. T-17 assigned to Wesley. |
| T-14 | riker | DONE | Session commit procedure added to CLAUDE.md. remmick.md ratified. Pushed to main. |
| T-16 | lead | DONE | Rebase + merge authorized. PR #3 merged to main (4aa06c4). |
| T-17 | wesley | DONE | Layer combo wiring complete in automap_editor.rs — all 5 locations fixed. cargo check passes. Included in PR #2. |
| T-18 | wesley | DONE | sprint/automapping/wesley-ui rebased onto main. Pushed. PR #2 open: https://github.com/chris-marrero/bevy_map_editor/pull/2 |

---

## Notes

- Final test count (pre-automapping sprint): 34 (20 existing + 14 new). All passing.
- Automapping tests (T-05) blocked on Data GO on PR #2.
- agent files: commit to main after every session per session commit procedure (CLAUDE.md).
- ship_log/ created: mission 1 (collision editor, closed), mission 2 (automapping, in progress).
