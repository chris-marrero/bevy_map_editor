# Mission 2 — Automapping

**Status:** IN PROGRESS
**Date:** 2026-02-28

## What Is Being Built

Rule-based automapping system: `bevy_map_automap` engine crate + editor integration + rule editor UI.

- **Geordi** — engine crate (`bevy_map_automap`): rule parsing, apply logic, Wang-style matching. PR #1 merged.
- **Barclay** — editor integration: `Layer::id`, `automap_config` on `Project`, `AutomapCommand`, layer-delete hook, dialogs. PR #3 pending merge (blocked on rebase — see below).
- **Wesley** — rule editor UI: three-column layout per Troi's spec. PR #2 pending rebase + review.
- **Worf** — test plan written; tests blocked on PR merges.

## Current Blockers

- PR #3 (Barclay) needs rebase onto main to remove cross-branch engine crate commit before merge. Force-push authorization pending (T-16).
- PR #2 (Wesley) blocked on PR #3 merge; Wesley must rebase after.
- T-17 (Wesley): layer combo wiring in `automap_editor.rs` — 5 locations with nil/discarded layer IDs. Blocked on PR #3 merge.
- Worf tests (T-05): blocked on all PRs.

## Protocol Findings (mid-sprint, to be completed at close)

- **Remmick inspection run mid-sprint** — first use of Inspector General persona. Useful: found untracked agent files (BLOCKER), undocumented Wesley stubs (MAJOR), cross-branch contamination claim in retro was inaccurate (MAJOR).
- **Riker spawned mid-sprint** — protocol violation. Riker is a sprint-close agent. Caught and noted; T-15 deferred.
- **Agent knowledge base was untracked in git** — emergency commit to main required. Session commit procedure added to CLAUDE.md (Riker, T-14).
- **Ship log system introduced this sprint** — replaces full retro_log in Riker's startup context.

## Open Debt (new this sprint)

- `no_overlapping_output` tracks center cell only (multi-cell output grids can still overlap)
- Layer combo selection discards user input in 5 locations in `automap_editor.rs` (T-17)
- Layer mapping persistence confirmed in-scope (T-13 done); wiring deferred to T-17
- `apply_automap_config` RNG seed not exposed (deterministic replay impossible)
- `Layer::id` on old project files not stable until first save

## To Complete at Sprint Close

- Merge PR #3 (after rebase)
- Rebase + review PR #2 (Wesley)
- Wesley wires layer combos (T-17)
- Worf writes automapping tests (T-05)
- Fill in final test count and any late findings
