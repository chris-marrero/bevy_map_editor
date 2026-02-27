# CLAUDE.md Proposals — From Riker

Maintained by Commander Riker. Picard reviews and applies these to CLAUDE.md.
Each proposal names the source violation, the target section in CLAUDE.md, and the exact text to insert or replace.

---

## Proposal 1 — Escalation Triage Checklist

**Source violations:** V-004 (Lead passed agent-domain escalations to user without filtering), V-002 (Data over-escalated), V-003 (Troi over-escalated)

**Target section in CLAUDE.md:** Replace the existing `### Escalation Handling` section with the following:

---

### Escalation Handling

Tasks assigned to `lead` are escalations. Before surfacing any escalation to the user, apply this triage filter:

**Step 1 — Is this within an agent's existing authority?**

| Type of question | Correct route |
|---|---|
| Technical correctness, architecture, or implementation strategy | Data's domain — return to Data, do not escalate |
| Interaction design, UX micro-decisions (e.g., button layout, reorder mechanism) | Troi's domain — return to Troi, do not escalate |
| Implementation complexity or feasibility concern | Troi routes to Data; Data decides; only escalate if product scope is genuinely unclear |
| Test coverage or testability strategy | Worf/Data domain — do not escalate |

**Step 2 — Does this genuinely require user input?**

Escalate to the user only if:
- It is a product-scope question (does this feature exist at all, what should it do in a way engineers cannot derive)
- It is a preference with no correct answer that only the user can have
- It involves a breaking change or irreversible decision
- Engineers have exhausted their authority and are still blocked

If the answer could be derived from the established architecture, the project's existing patterns, or the user's prior stated intent — it is not a user decision. Engineers must derive it and move on.

**Step 3 — Surface one at a time**

If multiple escalations have reached `lead`, surface them to the user one at a time. Wait for a decision before presenting the next.

You may also create tasks assigned to `lead` yourself if you observe something that requires user awareness.

---

## Proposal 2 — User-Facing Communication Style

**Source violation:** V-005 (Lead used internal identifiers when speaking with the user)

**Target section in CLAUDE.md:** Add a new `### User-Facing Communication Style` subsection under `## Lead Operating Procedures`:

---

### User-Facing Communication Style

Internal tracking identifiers (task IDs like `T-04`, violation numbers like `V-006`, agent names used as technical references) are for agent coordination only. They must not appear in conversation with the user.

When speaking to the user:
- Describe work in natural language: "Wesley completed the rule editor UI" not "T-04 is done"
- Describe process issues as observations: "the team skipped a review step" not "V-004 was triggered"
- Use character names only when the context makes them clear and natural; otherwise use role names

A fresh Lead instance reading only this file will not know what "V-004" means. The user should never be in the position of decoding internal notation.

---

## Proposal 3 — Debt Flagging Convention

**Source violation:** V-006 (Tech debt collection is passive and ad-hoc)

**Target section in CLAUDE.md:** Add a `### Technical Debt Convention` subsection under `## Lead Operating Procedures`:

---

### Technical Debt Convention

The DEBT table in `agents/architecture.md` is the canonical record of known technical debt. It is a living document, not a post-sprint cleanup task.

**Rule:** Any agent who introduces a stub, a placeholder return value, a `TODO` comment, or a deliberate deferment must add a corresponding DEBT table entry at the time the code is written — not at sprint close.

**Enforcement during code review:** Data's GO on any SE implementation requires a debt audit. Before Data signs off, every stub and placeholder in the implementation must be present in the DEBT table. An implementation with undocumented stubs does not pass review.

**Lead's role:** At sprint close, verify the DEBT table reflects the actual state of in-flight work. If entries are missing and stubs are live, create a task for Data to audit and add them before the sprint is marked complete.

---

## Proposal 4 — Parallel SE Coordination (Shared Working Directory)

**Source:** Sprint: Automapping — cross-branch contamination incident (Wesley's changes appeared on Barclay's branch; Picard had to manually reorganize commits)

**Target section in CLAUDE.md:** Add a `### Parallel SE Coordination` subsection under `### Multiple SE Instances`:

---

### Parallel SE Coordination

When multiple SE agents work in parallel, they must not share a working directory without a clear file-ownership protocol. Cross-branch contamination — where one SE's commits appear on another's branch — is a sprint blocker that requires Lead to manually untangle commits.

**Preferred approach: git worktrees**

Each SE working in parallel should operate in a dedicated `git worktree`. Picard assigns the worktree path in the SE's task prompt alongside the branch name. The SE creates the worktree at sprint start:

```
git worktree add .worktrees/<branch-short-name> -b <branch-name>
```

**If worktrees are not used: file-ownership declaration**

Before any parallel SE begins coding, Data must produce a file-ownership table: each SE declares the files they will modify, and no two SEs may list the same file. If their work converges on a shared file (e.g., `lib.rs`, `mod.rs`), they must be sequenced, not parallelized.

Data is responsible for enforcing this — if parallel SEs are assigned overlapping files, Data must catch it at proposal review before coding starts.

**Picard's responsibility:** Include the worktree path or file-ownership scope in the task prompt for every SE assigned to a parallel sprint. Do not leave this to the SEs to negotiate.

---
