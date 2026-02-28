# Sr Engineer's PADD — Lt. Cmdr. Data

Personal Access Display Device. Long-running important notes. Read on every spawn.

---

## Current Review Queue

Three open PRs in sequence. **Do not review PR #2 until PR #3 merges.**

| PR | Branch | Assignee | Status | Action |
|---|---|---|---|---|
| #1 | `sprint/automapping/geordi-engine` | Geordi | **GO given — MERGED** | Done. New DEBT entry added: `no_overlapping_output` center-cell-only tracking. |
| #3 | `sprint/automapping/barclay-integration` | Barclay | **GO ON CONTENT — awaiting rebase+merge** | T-11 fix (ada4a38) reviewed and correct. Cross-branch contamination issue analyzed (see below). Requires rebase onto main + force-push + GitHub PR creation + merge. T-16 created for Picard to obtain authorization. |
| #2 | `sprint/automapping/wesley-ui` | Wesley | Waiting for #3 merge | Will rebase on updated main after #3 merges; then review. T-12 DEBT entries now in place — GO can be issued on PR #2 content once Wesley rebases. Note: T-17 (layer combo wiring) must be included in PR #2 or a follow-on PR. |

**Merge order is critical.** PR #3 still must merge before PR #2.

**Cross-branch contamination — ANALYZED (2026-02-28):**
- Merge base of `sprint/automapping/barclay-integration` and `origin/main` is `f16e549` (pre-engine-crate)
- `942ff8a` (Barclay's copy of engine crate) is in Barclay's branch history; `7768402` (Geordi's copy) is already in main via `1c803e5`
- The two engine commits have IDENTICAL content (diff is empty — confirmed)
- A `--merge` PR merge would bring `942ff8a` into main's history via Barclay's lineage — confusing double-history
- **Resolution: rebase Barclay's branch onto current main.** Git's patch-id mechanism will skip `942ff8a` (identical to `7768402` already in main). Result: 6 Barclay-only commits cleanly on top of current main
- Requires: push authorization (T-16, assigned to lead)
- Note: No GitHub PR currently exists for Barclay's branch — user may need to `gh pr create` first

---

## Active DEBT Items

From `agents/architecture/architecture.md` DEBT table. These are live in current code.

### High Priority (Functional Impact)

- **[T-12/T-13] Layer combo wiring stubs in `automap_editor.rs`** (Wesley, 5 locations):
  - Lines 1199, 1217: `let _ = id;` — user combo selections silently discarded
  - Lines 1239, 1254, 1274: `Uuid::nil()` — new rules/output-alts always target no real layer
  - **In-scope for this sprint** (T-13 determination, 2026-02-28). T-17 created for Wesley.
  - All five instances documented in DEBT table.

### Medium Priority (Architectural)

- **`apply_automap_config` O(rules × width × height) complexity**: Acceptable now; may lag on large levels (256×256+). Monitor and revisit if user reports lag.

### Low Priority (Cosmetic/Optimization)

- **Magic constant `0.01` in `drag_stopped()`**: Collision drag-commit threshold. Extract to named const if ever tuned.
- **`format!("{:?}", one_way)` in CollisionProperties**: Exposes Rust debug format (`Top` instead of "Top (Pass from below)"). Cosmetic; address during next UX polish pass.
- **Canvas drag untestable with `egui_kittest`**: `handle_collision_canvas_input` canvas region has no AccessKit node. Testability decision deferred; documented in `agents/architecture/testing.md`.

---

## Architectural Authority

### SE Escalation Path (direct to Data)

When an SE says "this requirement is unclear or changed — we need to rethink":

1. **SE escalates to Data directly** (does not create a task for Picard first).
2. **Data verifies impact** on other SEs currently running (may ask them).
3. **Data decides** if it is a technical question (fixable by revert + correction) or a user intent question (must go to Picard).
4. **If Data is confident**: Data may **authorize a git revert** of the affected work. Notify Picard immediately via task assigned to `lead`.
5. **If Data is uncertain**: Escalate to Picard via task. Picard surfaces to user if needed.

**Key invariant:** Data makes technical calls; Picard makes product/user-intent calls.

### File Locations Under Data's Authority

- `agents/architecture/architecture.md` — primary reference; updated whenever a pattern is introduced, API approved, or DEBT changed (note: path is `agents/architecture/architecture.md`, not the old `agents/architecture.md`)
- `agents/architecture/sprint_log.md` — sprint-specific decisions and context; append when capturing *why* something was done
- `agents/architecture/testing.md` — testing procedures, patterns, snapshot inventory
- `agents/permissions.md` — permission log (maintained by Lead, but Data reads before asking agents to proceed)
- `agents/quarters/data/padd.md` — this file

---

## Review Checklist (Mandatory Before GO)

Before approving any SE implementation:

1. **Debt audit:** Every stub, `TODO`, `FIXME`, placeholder return value must be in the DEBT table (with trigger and cost).
2. **Edge cases:** What boundary conditions does this not handle? Is that acceptable?
3. **Pattern consistency:** Does this follow established architectural patterns? If not, is there a reasoned argument?
4. **Naming precision:** Are types, functions, variables unambiguous?
5. **Architectural fit:** Does this belong in the layer/module it is in, or should it move?

An implementation with undocumented stubs does not pass review.

---

## Key Context Maintained Across Sessions

- **Crates:** `bevy_map_core` (types), `bevy_map_autotile` (wang), `bevy_map_automap` (rules), `bevy_map_editor` (UI)
- **Testing:** `egui_kittest` rig with `snapshot` + `wgpu` features; 34 tests baseline, all passing
- **Borrow checker pattern:** Use `macro_rules!` to re-borrow nested project data across egui closures; never hold `&mut rule` across closure
- **Core patterns:** `EditorState::pending_action` → `process_edit_actions()` dispatch; `CommandHistory` for undo/redo
- **Panel render signature:** All accept `&egui::Context` + relevant state, return `()` or result struct — do not deviate
- **Build:** `cargo build --features dynamic_linking` for fast iteration

---

## Watch Items

- **Automapping sprint tests**: Blocked on PR #3 merge. Worf has test plan (in `agents/architecture/testing.md`); ready to write tests immediately after Barclay's integration work is approved and merged.
- **T-16 (lead)**: Authorization to rebase + force-push Barclay's branch. No push permission currently granted. Until this is resolved, PR #3 cannot merge.
- **T-17 (Wesley)**: Layer combo wiring — blocked on PR #3 merge. Must be in PR #2 scope or separate PR. Data must review Wesley's wiring proposal before coding.
- **Session state timestamp**: Last updated 2026-02-28. This PADD should be updated whenever review decisions are made or PRs change status.
