# Sr Engineer's PADD — Lt. Cmdr. Data

Personal Access Display Device. Long-running important notes. Read on every spawn.

---

## Current Review Queue

Three open PRs in sequence. **Do not review PR #2 until PR #3 merges.**

| PR | Branch | Assignee | Status | Action |
|---|---|---|---|---|
| #1 | `sprint/automapping/geordi-engine` | Geordi | **GO given — MERGED** | Done. New DEBT entry added: `no_overlapping_output` center-cell-only tracking. |
| #3 | `sprint/automapping/barclay-integration` | Barclay | **RETURNED — awaiting T-11 fix** | `find_layer_index` stub not resolved. T-11 assigned. Re-review after Barclay pushes fix. |
| #2 | `sprint/automapping/wesley-ui` | Wesley | Waiting for #3 merge | Will rebase on updated main after #3 merges; then review |

**Merge order is critical.** PR #3 still must merge before PR #2 is reviewed. Do not review PR #3 again until T-11 is marked done by Barclay.

**Cross-branch contamination alert (Remmick audit finding):** `origin/sprint/automapping/barclay-integration` contains Geordi's engine commit (`942ff8a`) as a base commit — same content as `7768402` on Geordi's branch, different hash. If PR #3 is merged as-is, the engine crate lands on main via Barclay's history, not Geordi's. PR #1 will then conflict or create duplicate history. Resolve this before merging PR #3: options are (a) rebase Barclay's branch onto main after squashing/excluding the engine commit, or (b) merge PR #1 first so the engine crate arrives cleanly, then rebase Barclay's branch onto that updated main. Decide and act before issuing GO on PR #3.

---

## Active DEBT Items

From `agents/architecture.md` DEBT table. These are live in current code.

### High Priority (Functional Impact)

- **`find_layer_index` stub in `bevy_map_automap/src/apply.rs`**: Returns permanent `None`. **Blocks:** Every automap rule that targets a named layer silently writes to nowhere; output groups and alternatives that specify `layer_id` are fully ignored at apply time. **Unblock trigger:** When PR #3 merges (`Layer::id` added to core). **Action:** This must be the first fix after #3 merges — delay means silently broken automap in user projects.

### Medium Priority (Architectural)

- **Layer mapping persistence not implemented**: `automap_config` is loaded/saved, but the layer index mappings for output groups are not persisted. Not yet confirmed scope. **Trigger:** Confirm with Picard whether this is in-scope before automapping sprint closes.

### Low Priority (Cosmetic/Optimization)

- **`apply_automap_config` O(rules × width × height) complexity**: Acceptable now; may lag on large levels (256×256+). Monitor and revisit if user reports lag.
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
- **Session state timestamp**: Last updated 2026-02-28. This PADD should be updated whenever review decisions are made or PRs change status.
