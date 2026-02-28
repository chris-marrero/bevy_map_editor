# Test Engineer's PADD — Lt. Worf

Personal Access Display Device. Long-running personal notes. Read on every spawn.

---

## Current Test Baseline

**34 tests passing.** Breakdown:
- Pre-collision editor sprint: 20 tests
- Collision editor + numeric input: 14 tests (label presence for numeric input panel, drag smoke tests)

**Test location:** `crates/bevy_map_editor/src/ui/tileset_editor.rs`, bottom of file in `#[cfg(test)] mod tests`.

**Test infrastructure:** `egui_kittest` 0.33 with `snapshot` feature (no `wgpu` yet — Phase 3 only).

---

## Current Sprint: Automapping — Blocked

**Active:**
- T-09: Data reviewing PRs #1 (Geordi engine), #3 (Barclay integration). Waiting for GO.
- T-10: Data reviewing PR #2 (Wesley UI). Depends on PR #3 merging first.

**My task (T-05):** Write and run tests for automapping engine + editor integration. Blocked until Data gives GO on both #1 and #3, and PR #3 merges.

**Test plan written** and recorded in `agents/architecture/testing.md`. Ready to execute once unblocked.

---

## Testability Constraints — Live and Documented

**Untestable with egui_kittest:**
- **Drag behavior:** Harness simulates single pointer events, not drag sequences. Framework limitation. Smoke tests (render without panic) are the ceiling. Wesley's drag fix in collision editor was tested this way — 4 smoke tests, all passing. Do not promise assertion tests on drag interactions.

**Phase-gated:**
- **Snapshot tests:** Require `wgpu` feature. Not enabled in Phase 1/2. Manual only, signed off jointly with Troi. Phase 3 scope.

---

## Known Accessibility Patterns (Empirically Verified)

**Label matching:** `harness.get_by_label("Label")` matches against `ui.checkbox()`, `ui.selectable_label()`, and text input labels. Verified with Grid checkbox test.

**Query methods (reference):** `get_by_label()`, `get_by_role()`, `get_by_role_and_label()`, `get_by_value()`, `get_by()`.

**Harness borrow pattern:** Closure captures external state by `&mut` reference. Borrow is released after `harness.run()` returns, so assertions can follow. Verified — no `RefCell` needed for simple cases.

---

## Automapping Test Plan Summary

When unblocked:

1. **Engine tests (bevy_map_automap):** Rule matching, output application. Unit tests, no UI harness.
2. **Editor integration tests (bevy_map_editor):**
   - `automap_config` on Project struct
   - `AutomapCommand` dispatch via `EditorState::pending_action`
   - Automap editor panel UI interactions (per Troi's spec in `agents/automap_ux_spec.md`)
   - Layer deletion hook that clears rules for that layer
3. **Interaction coverage:** All flows in Troi's UX spec must have corresponding tests or documented "untestable" constraints.

**Prerequisites:** Data GO on PR #1 and #3, PR #3 merged to main.

---

## Accessibility Audit Notes

**Widgets flagged for verification:**
- Automap rule editor inputs (when Wesley's code is available)
- Layer selector dropdown (confirm role/label in integration panel)

No unverified assertions in use. All test helpers match empirically verified behavior.

---

## Debt and Watch Items

**In flight:**
- `find_layer_index` stub in `bevy_map_automap/src/apply.rs` — recorded in architecture DEBT table.
- Layer mapping persistence in editor — in-flight, scope TBD with Data.

**Non-blocking but watch:** None currently.

---

## Communication Pointers

- **Troi:** Share test coverage early. Automapping tests will need her spec sign-off.
- **Data:** Escalate to him immediately if PR implementations lack accessibility annotations or are untestable.
- **SEs:** Flag missing labels directly to them. Accessibility is not optional.

---

## Next Actions When Unblocked

1. Data gives GO on PRs #1 and #3 → PR #3 merges.
2. I rebase on updated main (if needed) and begin automapping engine unit tests.
3. Once engine tests pass: begin editor integration tests (automap_config, AutomapCommand dispatch).
4. Once editor tests pass: test automapping rule editor UI per Troi's spec.
5. Share intermediate results with Troi for conformance sign-off.
6. Final run of full test suite (34 baseline + new automapping tests).

**No test is done until it runs and passes. All interaction flows in Troi's spec must be tested or documented as untestable. Partial coverage is not coverage.**

---

## Procedures Reference

**Running tests:**
```bash
cargo test -p bevy_map_editor --lib ui::tileset_editor::tests
```

**Full suite (Phase 1/2):**
```bash
cargo test -p bevy_map_editor --lib
```

**Snapshot tests (Phase 3, manual only):**
```bash
cargo test -p bevy_map_editor --lib --features snapshot,wgpu -- --ignored
```

See `agents/architecture/testing.md` for full procedures and conventions.
