# Protocol Violations Log

Maintained by Riker (Cmdr. William Riker). Agents append violations during any sprint. Riker reviews and resolves after each sprint closes by updating agent prompts in `.claude/agents/`.

---

## Format

```
### [SPRINT] V-NNN — <short title>
- **Detected by:** <agent>
- **When:** <sprint phase: launch / mid-sprint / close>
- **Policy violated:** <exact quote or reference from CLAUDE.md or agent prompt>
- **What happened:** <description of the actual behavior>
- **Status:** OPEN | RESOLVED
- **Resolution:** <what Riker changed, and in which file>
```

---

## Sprint: Automapping

### V-006 — Tech debt collection mechanism is passive and ad-hoc
- **Detected by:** Lead (user feedback)
- **When:** Mid-sprint, active development
- **Policy violated:** General quality hygiene — the DEBT table in `agents/architecture.md` is the canonical record of known technical debt, but it has no active collection mechanism. Agents only add entries manually when prompted, or when Data explicitly reviews the sprint log. During active development, agents introduce stub code (missing fields, `None`-returning functions, unresolved imports) that constitutes in-flight debt, but these are not flagged to the DEBT table. A sprint can end with untracked debt if no one explicitly runs a cleanup pass.
- **What happened:** Multiple agents left stubs and missing field references during T-04 and T-08 (Wesley's `show_automap_editor`/`automap_editor_state` fields not yet added to `EditorState`; Barclay's `AutomapCommand` not yet re-exported; `handle_run_automap_rules` stub called but not defined). None of these were logged as debt at the time of introduction. Data's debt update was only triggered by a user prompt, not by agent protocol.
- **Resolution needed:** Riker should establish a "debt flagging convention": any agent who introduces a stub, a `TODO`, or a deliberate placeholder must add a corresponding entry to the DEBT table at the time of introduction — not at sprint close. Data should also include a debt audit as a mandatory step in code review before giving GO. Riker should update Data's prompt and SE base prompt to reflect this.
- **Status:** RESOLVED
- **Resolution:** Added a mandatory "Debt Flagging" section to `.claude/agents/software-engineer.md` (SE base — covers all SE personas). SEs must now add a DEBT table entry at the moment they introduce any stub or placeholder. Added a "Debt audit during code review (mandatory)" block to `.claude/agents/sr-software-engineer.md` (Data): code review must verify all stubs are present in the DEBT table before GO is given. CLAUDE.md proposal written to `agents/riker_claude_md_proposals.md` covering the debt convention at protocol level.

### V-005 — Lead used internal identifiers when speaking with the user
- **Detected by:** Lead (user feedback)
- **When:** Mid-sprint, conversation
- **Policy violated:** Communication style — internal tracking identifiers (violation numbers, task IDs, etc.) are for agent coordination, not user-facing communication. The user should hear natural language, not references to internal bookkeeping.
- **What happened:** Lead referred to violations by identifier (e.g., "V-004") in conversation with the user. The user had to correct this.
- **Resolution needed:** Riker should add a communication style note to CLAUDE.md so a fresh Lead instance knows this from startup. CLAUDE.md is the recovery document — if it is not in there, a reset Lead will repeat the mistake.
- **Status:** RESOLVED
- **Resolution:** No agent prompt change applies here — this is a Lead behavior issue. CLAUDE.md proposal written to `agents/riker_claude_md_proposals.md` with a "User-Facing Communication Style" section instructing Lead never to use internal identifiers (task IDs, violation numbers) when speaking to the user. Picard must apply the proposal to CLAUDE.md.

### V-004 — Lead passed agent-domain escalations to the user without filtering
- **Detected by:** Lead (user feedback)
- **When:** Mid-sprint, escalation handling
- **Policy violated:** CLAUDE.md — "Surface to user one at a time." implies Lead must first determine whether an escalation *belongs* at the user level. If the issue is something engineers can actually decide themselves, Lead must redirect it back with a note rather than asking the user. Lead has been surfacing escalations mechanically without assessing whether each one genuinely requires a user decision.
- **What happened:** Lead surfaced the rule reordering approach (Up/Down vs drag-and-drop) to the user. This was a UX micro-decision within Troi's authority. Lead should have recognized it was in Troi's domain and returned it immediately. The same pattern occurred with flip-bit matching (Data's architectural authority) and the "Until Stable" complexity concern (also Data's domain). Multiple escalations reached the user that should have been filtered.
- **Resolution needed:** Riker should update Lead's operating instructions (CLAUDE.md or a Lead-facing reference) to include a triage checklist: before surfacing any escalation to the user, Lead must ask — "Does this require a product/scope/preference decision that only the user can make? Or does it fall within an agent's existing authority?" If the latter, return it to the agent, not the user.
- **Status:** RESOLVED
- **Resolution:** No individual agent prompt addresses this — it is Lead behavior. CLAUDE.md proposal written to `agents/riker_claude_md_proposals.md` with an "Escalation Triage" section containing an explicit decision filter. Lead must apply the triage filter before every user-facing escalation. Also added prompt-level escalation filters to Data (`.claude/agents/sr-software-engineer.md`) and Troi (`.claude/agents/ux-designer.md`) to reduce the upstream volume of incorrectly routed escalations reaching Lead in the first place.

### V-003 — Troi escalated an implementation complexity concern directly to the user
- **Detected by:** Lead (user feedback)
- **When:** Mid-sprint, spec review
- **Policy violated:** Communication protocol — concerns about implementation difficulty are not user decisions. Troi's domain is interaction design. When she identifies something as potentially hard to implement, the correct route is: flag it to Data, who assesses technical feasibility and escalates to the user only if a genuine product-scope decision is required.
- **What happened:** Troi recommended deferring "Until Stable" apply mode because of implementation complexity (cycle detection, infinite loop risk). She escalated this to Lead/user directly. The user confirmed it should have gone to Data first.
- **Status:** RESOLVED
- **Resolution:** Added an "Escalation Filter" section to `.claude/agents/ux-designer.md` (Troi) explicitly routing implementation complexity concerns to Data, not to `lead`. Also clarified that UX micro-decisions (e.g., drag-and-drop vs. Up/Down for reordering) are within Troi's authority and must not be escalated. Troi must not create tasks assigned to `lead` for questions that belong to Data or are within her own design authority.

### V-002 — Data over-escalated an architectural decision to the user
- **Detected by:** Lead (user feedback)
- **When:** Mid-sprint, architecture review
- **Policy violated:** CLAUDE.md — "Never make decisions. Never reinterpret user feature requests." applies to Lead, but the corollary for Data is: Data has technical authority and must exercise it. Escalations to the user are for decisions that genuinely require user input (product scope, breaking changes, preference). Whether flip bits affect tile equivalence is an architectural correctness question with an obvious answer — not a user decision.
- **What happened:** Data flagged flip-bit matching as an escalation to the user. The user pointed out it was obvious that flipped tiles are not equivalent, and the decision should never have reached them.
- **Status:** RESOLVED
- **Resolution:** Added a "Technical Authority — Escalation Filter" section to `.claude/agents/sr-software-engineer.md` (Data) with an explicit three-way triage: correctness/architecture/strategy questions are Data's to decide; tradeoffs derivable from existing constraints are Data's to decide; only genuine product-scope or user-intent questions belong at `lead` level. The section uses the flip-bit example (derive the answer from first principles) vs. a genuine user decision (does this feature exist at all) to make the distinction concrete.

### V-001 — Worf not polled for free-time tasks at sprint launch
- **Detected by:** Lead (self-reported after user prompt)
- **When:** Sprint launch
- **Policy violated:** CLAUDE.md — "Agent free-time rule: When an agent has no assigned tasks, they may propose tasks within the current sprint scope only. [...] Proposed tasks go on the task list and must be approved by the appropriate supervisor before work begins." Also: "Spawn all agents at the start of the sprint, simultaneously."
- **What happened:** Worf was given a blocked task (T-05) but was not spawned or polled at sprint launch. He was only engaged after the user explicitly pointed out the gap. Worf identified useful prep work (baseline verification, test plan skeleton) that could have started immediately.
- **Status:** RESOLVED
- **Resolution:** Added a "Blocked at Sprint Start" section to both `.claude/agents/test-engineer.md` (Worf) and `.claude/agents/software-engineer.md` (SE base, covering all SE personas). Each section instructs the agent that when their primary task is blocked at sprint launch, they must immediately propose preparatory tasks within sprint scope rather than waiting idle — and escalate those proposals to Data (or Lead in Worf's case) without waiting to be polled. The fix targets the agents who are most likely to land in a blocked-at-launch state, rather than relying solely on the Lead to remember to poll them.

### [Proc-Refactor] V-008 — Picard uses `---` separator in Bash commands causing permission prompts
- **Detected by:** User (2026-02-28)
- **When:** Procedure refactoring session
- **Policy violated:** General agent hygiene — commands should not trigger unnecessary user permission prompts
- **What happened:** Picard used `---` as a separator string inside Bash command arguments (specifically in a subagent prompt). The shell interprets leading `---` as a flag-like argument or causes Claude Code to treat the command as requiring confirmation, prompting the user each time.
- **Status:** OPEN
- **Resolution:** Riker to add a note to Picard's context/PADD and to CLAUDE.md: avoid using `---` as a literal separator inside Bash arguments or heredocs. Use a different separator (e.g., `===`, `###`, or a descriptive header) when structuring multi-section content passed to shell commands.
