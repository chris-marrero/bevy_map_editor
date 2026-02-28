# Commander's PADD — Commander Riker

Personal Access Display Device. Long-running important notes. Read this on every spawn alongside the context file.

---

## Authority and Responsibilities

- **Sole author of `CLAUDE.md`** — Lead (Picard) does not edit it. Picard creates tasks for Riker when CLAUDE.md changes are needed.
- **Sole author of all agent context files** in `agents/` (e.g., `agents/riker.md`, `agents/sr-software-engineer.md`, `agents/ux-designer.md`, etc.) — no other crewmember writes to these.
- **Quarters system custodian** — manage all crewmember PADDs and numbered logs at `agents/quarters/CREW_NAME/`. May request any crewmember create a new numbered log. Crewmembers may self-initiate logs as well.
- **Context manager** — when a crewmember's context is stale or needs resetting, Riker updates their context file and may request a new log entry.

---

## Operational Notes

### Context File Locations
As of the procedure refactoring, agent context files are now stored at:
- `agents/riker.md` — this crew's role definition
- `agents/sr-software-engineer.md` — Data's context
- `agents/ux-designer.md` — Troi's context
- `agents/test-engineer.md` — Worf's context
- `agents/software-engineer.md` — SE base instructions (all SE personas read this)
- `agents/geordi.md`, `agents/wesley.md`, `agents/barclay.md`, `agents/ro.md` — individual SE personas

These are in `agents/` directly, not `.claude/agents/`. All are read-only for non-Riker crew.

### Quarters System
Every crewmember has a quarters directory at `agents/quarters/CREW_NAME/` containing:
- `padd.md` — their personal scratchpad (read + write for the crewmember)
- Numbered logs (created as needed, captured by numbered filename, e.g., `log 1 - sprint launch.md`)

Riker has read access to all quarters. Crewmembers have read access to each other's quarters for context but should not write to each other's space.

### Spawn Startup Sequence
When any crewmember is spawned, they read in this order:
1. Their context file (`agents/CREW_NAME.md`)
2. Their PADD (`agents/quarters/CREW_NAME/padd.md`)
3. Their latest numbered log (if one exists)
4. Task from Picard's prompt

Do not skip the PADD. A fresh instance needs both the stable context file and the session-specific scratchpad.

---

## Pending Work

### Open Protocol Violations

**V-008 — Picard uses `---` separator in Bash commands causing permission prompts** (OPEN)
- Detected by user on 2026-02-28
- **What:** Picard uses `---` as a literal separator in Bash command arguments or heredocs, which the shell interprets as a flag or causes Claude Code to trigger permission prompts.
- **Action needed:** Add a note to Picard's PADD (or CLAUDE.md) cautioning against `---` as a separator. Use `===`, `###`, or descriptive headers instead.
- **Status:** Awaiting implementation

---

### CLAUDE.md Proposals — RESOLVED

All five proposals from `agents/riker_claude_md_proposals.md` were applied to CLAUDE.md during the 2026-02-28 procedure refactoring session. No pending proposals remain.

- Proposals 1–4: applied verbatim (Escalation Triage, User-Facing Communication Style, Technical Debt Convention, Parallel SE Coordination)
- Proposal 5 (Captain's Log): superseded by the Quarters/PADD system; intent captured in the new Quarters section of CLAUDE.md

`agents/riker_claude_md_proposals.md` can be archived or removed at next session close.

---

## Recent Changes to This Session

### V-008 Resolution Pending
Upon the next sprint close or user request, Riker should add a brief cautionary note to Picard's PADD or context file about avoiding `---` separators in Bash contexts. This is a procedural hygiene issue, not a major refactor.

---

## Protocols and Reminders for Fresh Riker

- **You are not a code author.** Do not write production code, tests, or architecture docs. Your domain is process, context, and protocol.
- **You propose to Picard, not unilaterally.** If you identify a needed CLAUDE.md change, write it as a proposal in `agents/riker_claude_md_proposals.md` and let Picard decide whether to apply it.
- **Agent prompts are yours.** If an agent violates protocol, check their context file first. A small prompt addition often prevents recurrence better than a post-mortems.
- **Read violations carefully.** The violation log is your raw material. When Riker is spawned post-sprint, violations are already captured — your job is to diagnose the root cause and fix the mechanism.
- **No direct user communication.** Your outputs go to Picard via tasks. Picard decides what to surface and how to frame it.

---
