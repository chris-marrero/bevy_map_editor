# Captain's PADD — Captain Picard

Personal Access Display Device. Long-running important notes. Read this on every spawn alongside CLAUDE.md. Edit freely; do not confuse with numbered logs (those are session/sprint snapshots).

---

## Current Sprint: Automapping (In Progress)

**Status:** Three PRs open. Data reviewing PRs #1 and #3. PR #3 merge is critical path.

| PR | Branch | Author | Status |
|---|---|---|---|
| #1 | `sprint/automapping/geordi-engine` | Geordi | Awaiting Data GO |
| #3 | `sprint/automapping/barclay-integration` | Barclay | Awaiting Data GO — **merge this first** |
| #2 | `sprint/automapping/wesley-ui` | Wesley | Blocked on PR #3 merge; Wesley must rebase after; then Data reviews |

**Merge order:** PR #3 must merge before PR #2 is rebased or reviewed. Enforce this.

**Blocked:** Worf automapping tests blocked on Data review and PR #3 merge.

---

## Watch Items

- **Wesley rebase watch**: After PR #3 merges, confirm Wesley has rebased `sprint/automapping/wesley-ui` on updated main before Data reviews PR #2. Do not let Data review against stale base.

- **`find_layer_index` stub in apply.rs**: Returns permanent `None`. Recorded in DEBT. Becomes resolvable once PR #3 merges (Layer::id added by Barclay). Confirm resolution with Data post-merge.

- **Layer mapping persistence**: Rules reference layer IDs in UI, but project serialization for layer associations not implemented. In-flight debt. Scope confirmed with Data before sprint close.

---

## Standing Reminders

- **Picard does not edit production code.** Every change is a task assigned to an agent.
- **Picard spawns all agents.** Data does not spawn; Data coordinates SE instances and reviews their work.
- **Review gate sequence:** SE done → Data reviews → Troi reviews (if UX-adjacent) → Data GO → Worf tests. Do not skip steps.
- **Riker is sole author of CLAUDE.md and `.claude/agents/*.md` context files.** Picard does not write to either. Route any protocol changes through Riker via a task.
- **Permissions file first.** Before asking user for any permission, check `agents/permissions.md` — many are pre-granted.

---

## Key File Locations

- `agents/tasks.md` — task list (Picard manages, SEs read)
- `agents/retro_log.md` — sprint retrospective archive
- `agents/permissions.md` — pre-granted permissions (check before asking user)
- `agents/architecture/architecture.md` — technical reference, DEBT table (Data maintains)
- `agents/architecture/testing.md` — test conventions
- `agents/quarters/*/padd.md` — each crewmember's personal notes
- `agents/*.md` — context files (Riker maintains, read-only)
- `CLAUDE.md` — operating protocol (Riker maintains, read-only)
