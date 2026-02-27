# bevy_map_editor — Sprint Retrospective Log

Maintained by the Lead. Each entry captures process findings from a completed sprint or notable mid-sprint correction. Frame findings as system design questions, not agent failures.

---

## 2026-02-26 — assert_panel_visible sprint (CLOSED)

### Process Violation: Troi escalated directly to user instead of tasking Data

**What happened:** Troi's `assert_panel_visible` spec identified that the Tree View panel heading reads `"Project"` rather than `"Tree View"`. Rather than creating a task for Data and his engineers to investigate, Troi escalated the question directly to the user via her agent output.

**Why this matters:** The communication protocol is: agents escalate to Picard via the task list. Picard surfaces to the user only when engineers cannot resolve it themselves. The heading rename question is well within Data's investigative authority — it requires reading source, checking usages, and making a call. No user input was needed.

**Correction applied:** Picard created Task #3 (investigate Tree View heading) and Task #4 (add Asset Browser heading) and assigned them to Data, bypassing the unnecessary user escalation.

**System design question:** Should Troi's agent prompt be updated to explicitly state that open questions for other agents must be recorded as tasks, not returned as escalations to Picard? The current prompt likely does not distinguish between "question for the user" and "question for another agent." This conflation is the root cause.

**Second process finding — Data spawning agents:**
Data attempted to spawn Geordi directly using a skill call, which failed. The user identified this and directed that Picard spawns all agents at sprint start. CLAUDE.md and `sr-software-engineer.md` were updated to reflect this. The system design question: should agent prompts explicitly prohibit spawning subagents, or is it sufficient to instruct only Picard on the correct protocol? Current approach: instruct both sides (Picard's protocol + Data's prohibition).

---
