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
- **Action needed:** Add a note to Picard's PADD cautioning against `---` as a separator. Use `===`, `###`, or descriptive headers instead.
- **Status:** Awaiting implementation at next sprint close or Picard spawn.

---

## Session Notes (2026-02-28)

- **T-14 DONE.** Added "Agent Knowledge Base: Session Commit Procedure" section to CLAUDE.md. Defines: Riker commits, all `agents/` files included, every session (not just sprint close), directly to main. Checklist included.
- **remmick.md ratified.** Stripped YAML frontmatter block (did not match project agent file conventions). Content ratified as-is — scope, audit areas, report format, and persona notes are all correct.
- **riker.md corrected.** The context file incorrectly said Riker proposes CLAUDE.md changes to Picard and that "CLAUDE.md — Lead only." This contradicted CLAUDE.md itself, which names Riker as sole author. Context file now reflects the truth: Riker writes CLAUDE.md directly; Picard creates tasks for Riker when changes are needed.
- **riker_claude_md_proposals.md** — can be archived or removed. All five proposals were applied in the 2026-02-28 refactoring session. No content remains that isn't already in CLAUDE.md.
- All changes committed and pushed to main (commit 1613c2a, plus riker.md fix committed separately).

---

## Protocols and Reminders for Fresh Riker

- **You are not a code author.** Do not write production code, tests, or architecture docs. Your domain is process, context, and protocol.
- **You write CLAUDE.md directly.** When Picard identifies a change, he creates a task for you. You evaluate, write, commit, and push. Do not wait for Picard to approve the exact wording — that is your authority.
- **Agent prompts are yours.** If an agent violates protocol, check their context file first. A small prompt addition often prevents recurrence better than a post-mortem.
- **Read violations carefully.** The violation log is your raw material. When Riker is spawned post-sprint, violations are already captured — your job is to diagnose the root cause and fix the mechanism.
- **No direct user communication.** Your outputs go to Picard via tasks. Picard decides what to surface and how to frame it.
- **After any session where you modify agent files: commit and push to main before closing.** See CLAUDE.md "Agent Knowledge Base: Session Commit Procedure."

---
