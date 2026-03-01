# Incident Log

Every agent appends here when a failure state occurs. Picard logs user corrections.
Guinan reads this at sprint close to produce the pre-Riker report.
**This file is cleared at the start of each new sprint.** Historical record lives in Guinan's reports.

## Entry Format

```
### [SPRINT NAME] — [DATE] — [TYPE]: [SHORT TITLE]
**Who:** <agent or user>
**What happened:** <factual description>
**Impact:** <what was blocked, broken, or wasted>
**Resolved by:** <how it was fixed, or OPEN>
```

**Types:** PROCESS | TECHNICAL | BLOCKED | USER_CORRECTION

---

## Sprint: Automapping

### Automapping — 2026-02-28 — TECHNICAL: find_layer_index stub not resolved in PR #3
**Who:** Barclay (SE), caught by Data (review)
**What happened:** PR #3 added `Layer::id` to `bevy_map_core` — the exact trigger condition for the `find_layer_index` DEBT entry — but the stub in `apply.rs` was not updated. PR returned.
**Impact:** PR #3 review cycle extended by one round. T-11 created.
**Resolved by:** Barclay fixed in T-11 (ada4a38). Stub resolved.

### Automapping — 2026-02-28 — PROCESS: Riker spawned mid-sprint
**Who:** Picard
**What happened:** User requested Riker update push procedure. Picard spawned Riker immediately without flagging the timing conflict (Riker is a sprint-close agent).
**Impact:** CLAUDE.md edited mid-sprint. Low risk in this case (content was correct), but protocol was violated.
**Resolved by:** User flagged it. Noted. T-15 deferred to sprint close.
**User correction:** "Riker should have only been run at the end of the sprint."

### Automapping — 2026-02-28 — BLOCKED: Agent knowledge base untracked in git
**Who:** Not attributable to one agent — systemic gap
**What happened:** Remmick audit found all agent files (architecture docs, PADDs, quarters, context files) were untracked in git. They existed only in the local working tree.
**Impact:** BLOCKER. A reset or fresh checkout would have lost the entire knowledge base.
**Resolved by:** Emergency commit to main (cdc53a0). Session commit procedure added to CLAUDE.md.

### Automapping — 2026-02-28 — PROCESS: Cross-branch contamination not caught before PR submission
**Who:** Barclay (SE)
**What happened:** Geordi's engine crate commit was present in Barclay's branch history. PR #3 would have brought it into main via Barclay's lineage.
**Impact:** Required manual rebase + force-push before merge. Remmick flagged it; Data analyzed it.
**Resolved by:** Rebased onto main (git skipped duplicate patch-id). Merged cleanly.

### Automapping — 2026-02-28 — TECHNICAL: Wesley's stubs not in DEBT table at PR submission
**Who:** Wesley (SE), caught by Remmick + Data
**What happened:** Five `let _ = id` and `Uuid::nil()` stubs in `automap_editor.rs` were not documented in the DEBT table when PR #2 was first submitted.
**Impact:** PR #2 could not receive Data GO without DEBT entries. T-12 created retroactively.
**Resolved by:** Data added DEBT entries (T-12). Wesley wired stubs (T-17).

### Automapping — 2026-02-28 — PROCESS: gh pr create failed repeatedly (race condition + existing PR)
**Who:** Picard
**What happened:** `gh pr create` failed twice with "sha can't be blank" (race condition after force-push). Then failed with "PR already exists" (PR #2 was open from Wesley's original T-04 work — never checked before attempting creation).
**Impact:** Wasted cycles, user confusion, two error messages surfaced to user.
**Resolved by:** Identified root causes. Going forward: check `gh pr list --head <branch>` before any `gh pr create`.
**User correction:** "Why do you keep having those pull request create failed sha can't be blank error? And the PR already exists."

### Automapping — 2026-02-28 — TECHNICAL: tasks.md reverted by branch merges
**Who:** Picard / git merge
**What happened:** `git merge --no-ff sprint/automapping/barclay-integration` brought in an old version of `tasks.md` from the branch, overwriting current sprint state. Had to be manually restored twice.
**Impact:** Task list out of sync with sprint reality for portions of the session.
**Resolved by:** Manual rewrites of tasks.md. Root cause: agent doc files should not travel on SE feature branches.

### Automapping — 2026-02-28 — PROCESS: Context menu wired to zero-size region (T-19)
**Who:** Wesley (SE), caught by Data (review)
**What happened:** Rule set delete context menu was attached to `ui.horizontal(|_ui| {}).response` — zero allocated area, unreachable by user. Troi spec conformance failure.
**Impact:** Feature non-functional. PR #2 returned.
**Resolved by:** T-19 assigned to Wesley. Fix in progress.

### Automapping — 2026-02-28 — TECHNICAL: Double-label rendering on ComboBoxes (T-20)
**Who:** Wesley (SE), caught by Data (review)
**What happened:** Pattern `ui.label("X") + ComboBox::from_label("X")` used across six combos, rendering label text twice and creating duplicate AccessKit nodes.
**Impact:** Visual duplication. Would break Worf's label-discovery tests. PR #2 returned.
**Resolved by:** T-20 assigned to Wesley. Fix in progress.

---

## User Requests Log

Requests made by the user during this sprint that reflect process or scope decisions.
Guinan should include these in her report.

| Date | Request | Action taken |
|---|---|---|
| 2026-02-28 | Spawn Remmick as experiment | Done. First use of Inspector General persona. |
| 2026-02-28 | Create directives file (first draft) | Draft written. User to finalize and save. |
| 2026-02-28 | "Riker should only run at sprint close" | Noted. T-15 deferred. Retro item. |
| 2026-02-28 | "Does Riker read the retro log?" | Gap identified. T-15 updated to include ship_log in Riker startup. |
| 2026-02-28 | Create ship_log with mission-based files | Done. mission 1 (collision editor), mission 2 (automapping). |
| 2026-02-28 | Commit agent files directly to main | Done. Emergency commit + push. |
| 2026-02-28 | PR create race condition / existing PR question | Root cause explained. Guard check added to process. |
| 2026-02-28 | Create incident_log + Guinan role | This entry. In progress. |
