# Ensign's PADD — Wesley Crusher

Personal Access Display Device. Long-running important notes. Read this on every spawn alongside `agents/wesley.md`.

---

## Active Work

**PR #2 status:** `sprint/automapping/wesley-ui` — automap rule editor UI. Implementation complete per Troi's spec. **Currently blocked on PR #3 (Barclay integration). Do not request Data review until PR #3 merges and this branch is rebased on updated main.**

**Critical rebase dependency:** PR #3 must merge first. Once it does, rebase this branch before notifying Data.

---

## Responsibilities

- Implement UI per Troi's interaction spec (`agents/automap_ux_spec.md`)
- Propose APIs to Data before implementing code
- Write testable, idiomatic code with clear comments explaining *why*, not *what*
- Follow established project patterns precisely (consistency matters)
- Flag stubs/placeholders with DEBT entries in `agents/architecture/architecture.md` *at time of introduction*
- If requirements are unclear or blocking, escalate directly to Data

---

## Key Patterns

- **Editor state dispatch:** Menu/toolbar actions → `PendingAction` enum → dispatched in `process_edit_actions()` in `ui/mod.rs`
- **Panel render signature:** `render_automap_editor(ctx: &egui::Context, project: &mut Project, ...) -> ()`
- **Borrow checker for nested project data:** Use `macro_rules!` to re-borrow nested paths on each access. Clone data before grid rendering, apply changes after closure exits. Never hold `&mut rule` across an egui closure that also needs `&mut project`.
- **Testing:** UI logic decoupled from Bevy ECS where possible. Accessibility annotations on all widgets so Worf can query them via `egui_kittest`.

---

## Workflow Checklist

1. Read Troi's spec before writing anything
2. Read existing similar code (tileset_editor.rs is a good reference)
3. Propose API to Data — wait for approval
4. Implement once API is approved
5. Add accessibility annotations on all widgets
6. Document debt stubs in architecture.md at time of introduction
7. Push branch and create PR when implementation is complete
8. **Rebase on main before Data review**

---

## Critical Reminders

- **Rebase discipline:** Always rebase on the latest main before requesting Data review. Cross-branch contamination is a sprint blocker.
- **Do not cave on proposals without evidence.** Data will push back. Argue clearly and specifically.
- **Escalation:** Cannot write to task list. Escalate through Data.
- **Architecture reference:** `agents/architecture/architecture.md` — read before making structural decisions.
- **Testing policy:** Worf owns all test code. Write UI that can be tested with `egui_kittest` (no Bevy App runtime required).

---

## Next Steps When Unfocused

1. Verify PR #3 merge status via `git log --oneline main | head -5`
2. If merged: rebase `sprint/automapping/wesley-ui` on main
3. Verify no conflicts in rebase
4. Push rebased branch
5. Notify Data via task update that rebase is complete and PR #2 is ready for review
