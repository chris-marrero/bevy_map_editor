# Guinan's Sprint Close Report — Automapping Sprint

**Prepared for:** Riker (first reader), Picard (second reader)
**Sprint:** Automapping (Mission 2)
**Sprint dates:** 2026-02-28 – 2026-03-01
**Incidents logged:** 9
**User corrections:** 2 explicit, several implicit

---

## Incident Summary (what went wrong and patterns)

### Pattern 1 — PR hygiene failures (3 incidents)

Three separate PR-submission problems surfaced in one sprint.

**Wesley's stubs not in DEBT table (PR #2).** Five dead stubs shipped to review without DEBT entries. Data had to create the entries retroactively (T-12). The DEBT audit is supposed to happen before GO, not as a PR review loop. This burned a full review cycle and created a sequencing dependency that blocked Worf.

**Barclay's cross-branch contamination (PR #3).** Geordi's engine crate commit was present in Barclay's branch history. This is exactly the cross-branch contamination scenario the parallel SE coordination protocol exists to prevent. It required a manual rebase and force-push before merge. The fix was mechanical, but the fact that Barclay submitted the PR without catching it means the pre-submission checklist (check that your branch contains only your commits) is not being run.

**`find_layer_index` stub not resolved at PR time (PR #3).** The DEBT entry said: resolve when `Layer::id` lands. `Layer::id` landed in PR #3. Barclay still did not resolve the stub before submitting. Data caught it in review. T-11 was created after the fact. Same failure mode as the Wesley stubs: known technical debt with a known trigger condition, not acted on at the right moment.

The pattern across all three: SEs are treating code review as a discovery pass rather than a confirmation pass. Review should confirm what the SE already knows is correct. Instead, Data and Remmick are finding things the SE should have caught before submission.

### Pattern 2 — Destructive git operations on uncommitted work (1 incident, severe)

Worf wrote 11 tests across two files. None were committed. Picard ran `git reset --hard origin/main` to resolve a stash conflict. All test work was lost. Worf had to be re-spawned and re-write from scratch.

This is the most expensive incident of the sprint in raw work-hours. The root cause is a missing invariant: **never run a destructive git operation with uncommitted work present.** This should be explicit in protocol. It is not enough to rely on agents knowing it implicitly.

The secondary cause: Picard attempted the branch switch in the first place without verifying that Worf's work was committed. Picard should not authorize or execute branch switches without confirming the working tree is clean.

### Pattern 3 — Agent knowledge base not in version control (1 incident, was a blocker)

All agent files — architecture docs, testing docs, task lists, ship logs — were untracked in git at the time of the Remmick audit. A fresh checkout or reset would have silently wiped the entire operational knowledge base. This was a catastrophic risk that existed silently for at least one prior sprint.

The fix (emergency commit + session commit procedure added to CLAUDE.md) is correct. But the fact that this survived into a second sprint without being noticed indicates the session commit procedure needs to be actively verified, not just documented.

### Pattern 4 — Mid-sprint spawning of sprint-close agents (1 incident)

Riker was spawned mid-sprint to update the push procedure. Picard did not flag the timing conflict. Riker is defined as a sprint-close agent. The user had to correct this.

The mechanism that allowed this: Picard has no guard preventing mid-sprint Riker spawns. There is no checklist item that says "is this a sprint-close agent?" before spawning. This needs to be explicit in the sprint protocol.

### Pattern 5 — `gh pr create` race condition and existing-PR blindness (1 incident)

`gh pr create` failed twice: once with a race condition after a force-push, once because a PR already existed on the branch. Picard did not check for an existing PR before attempting creation. The user noticed and had to ask about it.

The fix (check `gh pr list --head <branch>` before `gh pr create`) has been documented. Whether it will be followed depends on whether it becomes a habit or a checklist item.

### Pattern 6 — `tasks.md` on SE branches (1 incident)

Merging Barclay's feature branch brought in an old version of `tasks.md`, overwriting current sprint state. Picard had to manually restore it twice.

Root cause: `tasks.md` (and all agent docs) traveled on the SE branch. Agent docs should not be committed on SE branches. They live on main only. This was resolved as a process finding but is not yet codified as a branch hygiene rule.

### Pattern 7 — Worf compile error on missing import (1 incident, minor)

Worf's second instance (after the git reset) wrote tests using `query_by_label` and `get_by_label` without importing `egui_kittest::kittest::Queryable`. Eight compile errors. Caught and fixed before commit.

This is a minor incident compared to the others, but it follows a pattern: agents are not running `cargo check` as a pre-commit step. If Worf had run `cargo check` before finishing, this would not have been an incident at all.

### Pattern 8 — UX conformance failures caught late (2 incidents, both in PR #2)

Two UX failures appeared in Wesley's PR #2 and had to be returned:

- Context menu attached to zero-size region (unreachable by user)
- Double-label rendering on six ComboBoxes

Both are things Troi's spec should have prevented if Wesley had cross-referenced the spec against the implementation before submitting. Instead, Data found them during code review. T-19 and T-20 remain open. PR #2 has not received GO.

---

## User Corrections (what the user had to tell the crew)

**"Riker should have only been run at the end of the sprint."**
Picard spawned Riker mid-sprint. The user caught it. This is a protocol gap that should not require user correction.

**"Why do you keep having those pull request create failed sha can't be blank error? And the PR already exists."**
Two consecutive `gh pr create` failures were surfaced to the user without explanation. The user had to ask what was happening. The pre-submission checklist (verify no existing PR, wait after force-push) should prevent this from reaching the user at all.

**"That is a critical failure. Note it in the logs. You should note ANY critical failure somewhere."**
This was the user's response to the Worf test loss incident. The incident log system was created as a direct result of this correction. The user should not have been the one to establish this logging requirement. Picard should have had a failure-logging reflex before this sprint.

---

## What the Crew Got Right

**Geordi's engine crate shipped clean.** PR #1 merged without incident. No review cycles, no returned work, no DEBT violations. The engine is the largest functional piece of the sprint and it landed correctly.

**Barclay's integration was thorough.** Despite the stub and cross-branch issues, the integration work itself — `Layer::id`, `automap_config`, `AutomapCommand`, layer-delete hook — was complete and architecturally sound. Data gave GO after one round of fixes.

**Troi's UX spec was detailed enough to catch downstream failures.** The fact that T-19 and T-20 were caught as spec violations — rather than shipping as silent bugs — means the spec was good enough to serve as a reference. Troi's work held.

**Remmick proved useful on first deployment.** The Inspector General persona found the untracked agent files (a real blocker), the undocumented Wesley stubs, and corrected the cross-branch contamination claim in the retro (which had been stated inaccurately). Remmick cost one session but returned two blockers and one factual correction.

**Worf's second-instance recovery was fast.** After the test loss, Worf was re-spawned and re-wrote all 11 tests. Final test count reached 40 editor tests + 15 engine tests. The tests pass. Losing the work was costly; recovering from it was handled well.

**The incident log itself.** Starting this log mid-sprint, in response to the Worf test loss, means this sprint has a real failure record. That alone makes the next sprint better.

---

## Recommendations for Riker (process changes to consider)

**1. Add a pre-PR submission checklist for SEs.**
Before any SE creates or updates a PR, they should run through: (a) `cargo check` passes, (b) all stubs have DEBT entries, (c) DEBT entries with known trigger conditions that fired during this work are resolved, (d) branch contains only commits from their own work (verify with `git log main..HEAD`), (e) no agent doc files (`tasks.md`, `architecture.md`, etc.) are in the commit set.

This moves discovery from Data's code review into the SE's pre-submission self-check. Data's review becomes a confirmation pass, not a discovery pass.

**2. Codify: agent doc files do not travel on SE branches.**
`tasks.md`, `architecture.md`, `testing.md`, `retro_log.md`, `incident_log.md`, `ship_log/`, and `lead_notes.md` all belong on main only. SE branches must not contain commits to these files. Add this to the branch hygiene rules in CLAUDE.md.

**3. Codify: never run destructive git operations with uncommitted work present.**
This rule should be explicit in CLAUDE.md and in any agent prompt where git operations appear. The specific sequence to establish: before any `git reset`, `git checkout`, or branch switch — run `git status`. If the working tree is not clean, commit or stash (and verify the stash succeeded) before proceeding. Picard should not authorize a branch switch without verifying the working tree is clean.

**4. Codify: sprint-close agent guard.**
Add to the sprint protocol: Riker and Guinan are sprint-close agents. They may not be spawned during an active sprint. If a user request appears to require Riker mid-sprint, Picard should note it for sprint close and defer.

**5. Add pre-PR-create check to git workflow.**
Before any `gh pr create`: run `gh pr list --head <branch>`. If a PR already exists, update it rather than creating a new one. If a force-push just occurred, wait 5 seconds before creating. Document this as a workflow step, not just a tribal-knowledge correction.

**6. Make the session commit procedure a verified step, not a documentation item.**
The session commit procedure (commit agent files to main at session end) was added to CLAUDE.md after the Remmick audit. But there is no verification that it is being followed. Consider making it a checklist item that Picard runs at session close: `git status agents/` — if anything is untracked or modified, commit before ending the session.

**7. Add a Troi spec cross-reference step to Wesley's workflow.**
Before Wesley submits any UI implementation for review, he should explicitly compare his implementation against Troi's spec at the widget level. The double-label and zero-size-region failures both would have been caught by reading the spec carefully. This is not a Data review item — it is a Wesley pre-submission item.

---

## Sprint Completion Status

**Incomplete. PR #2 (Wesley's rule editor UI) has not received Data GO.**

| Item | Status |
|---|---|
| Engine crate (PR #1) | MERGED |
| Editor integration (PR #3) | MERGED |
| Rule editor UI (PR #2) | OPEN — awaiting T-19 and T-20 fixes |
| T-19: Fix context menu (zero-size region) | PENDING |
| T-20: Fix double-label ComboBoxes | PENDING |
| T-10: Data final review of PR #2 | PENDING (blocked on T-19, T-20) |
| T-15: Riker startup + Guinan protocol updates to CLAUDE.md | DEFERRED to sprint close |
| T-05 tests | DONE (40 editor, 15 engine, all passing) |

The sprint cannot be declared complete until PR #2 receives Data GO, T-19 and T-20 are resolved, and T-15 is closed by Riker. The functional work is sound — the engine and integration are merged and tested. What remains is UI conformance and protocol documentation.

**Open DEBT (not yet resolved):**
- `no_overlapping_output` tracks center cell only
- `apply_automap_config` RNG seed not exposed
- `Layer::id` stability on old project files (pre-first-save)
- Layer mapping persistence wiring (confirmed in-scope, not yet implemented)

These are known and documented. They are not sprint blockers — they are post-sprint work items.
