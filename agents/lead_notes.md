# Lead Notes — Captain Picard

Lightweight scratchpad for things that matter right now. Not protocol (that is CLAUDE.md). Not technical reference (that is architecture.md). Not a task list (that is tasks.md). Not a retrospective archive (that is retro_log.md).

**Read this immediately after CLAUDE.md at every session start.**
**Prune at every session close: remove anything resolved, stale, or captured elsewhere.**

---

## Current Sprint State

**Sprint:** Automapping
**Status:** In progress — waiting on Data to complete PR reviews.

**PRs open:**
- PR #1 — Geordi: `sprint/automapping/geordi-engine` — engine crate. Awaiting Data GO.
- PR #2 — Wesley: `sprint/automapping/wesley-ui` — rule editor UI. Awaiting Data GO. **Must not merge before PR #3.**
- PR #3 — Barclay: `sprint/automapping/barclay-integration` — editor wiring. Awaiting Data GO. **Merge this first — Wesley rebases on it.**

**Merge order constraint:** PR #3 first, then Wesley rebases, then PR #2. Data is responsible for enforcing this order.

**Blocked tasks:**
- T-05 (Worf tests) — blocked on Data reviewing PRs #1 and #3 and PR #3 merging.

**Nothing is merged yet. No GO has been given by Data. Worf has not started automapping tests.**

---

## Watch Items

- **Wesley rebase**: After PR #3 merges, Wesley must rebase `sprint/automapping/wesley-ui` on updated main before PR #2 can be reviewed or merged. Data should not review PR #2 against a stale base.

- **Layer mapping persistence**: Rules can reference layer IDs in the UI but project serialization for layer associations is not implemented. This is in-flight debt — confirm scope with Data before sprint close. Not yet in DEBT table.

- **`find_layer_index` stub**: `apply.rs` returns permanent `None`. In DEBT table. Fix is gated on `Layer::id` landing in `bevy_map_core`. `Layer::id` was added by Barclay (PR #3) — once that merges, Data should confirm the DEBT item is resolvable.

---

## Decisions Made This Session

_(none yet — session is fresh)_

---

## Handle at Next Session Start

- Check whether Data has reviewed any of PRs #1, #2, #3 since last session.
- If PR #3 has merged: confirm Wesley has rebased and is ready for PR #2 review.
- If all PRs merged: unblock T-05 and spawn Worf.
- Check retro_log.md — the automapping sprint entry is still marked IN PROGRESS. Close it when sprint is complete.

---

## Pending Riker Proposals

The following proposals are in `agents/riker_claude_md_proposals.md` and have NOT yet been applied to CLAUDE.md. Picard must apply them or explicitly defer them:

- **Proposal 1** — Escalation Triage Checklist (replaces existing Escalation Handling section)
- **Proposal 2** — User-Facing Communication Style (new subsection)
- **Proposal 3** — Technical Debt Convention (new subsection)
- **Proposal 4** — Parallel SE Coordination (new subsection under Multiple SE Instances)
- **Proposal 5** — Lead Notes system (new subsection — this file's own integration into CLAUDE.md, pending Picard applying it)

---

## Protocol Notes

- **Picard never edits production files directly.** Every code change, no matter how trivial, must be a task assigned to an agent. This was violated in the Collision Editor sprint. Do not repeat it.
- **Data does not spawn agents.** Picard spawns all agents, including SE personas.
- **Review gate is mandatory:** SE done → Data reviews → Troi reviews UX-adjacent items → Data gives GO → Worf runs. Skipping this gate was a sprint violation in the Collision Editor sprint.
