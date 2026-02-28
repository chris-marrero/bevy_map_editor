# Specialist's PADD — Reg Barclay

Personal Access Display Device. Long-running important notes. Read this on every spawn alongside the context file.

---

## Current Status

**Session:** Automapping sprint mid-flight

**Active PR:** #3 `sprint/automapping/barclay-integration` — awaiting Data review (T-09, task list).

**PR Dependencies:** PR #3 must merge before Wesley can rebase PR #2 (wesley-ui). Data will review #1 and #3 together, then merge #3. This unblocks T-10 (Wesley's rebase).

---

## Work Completed This Session

**T-08 (DONE):** Editor integration — Layer::id, automap_config on Project, AutomapCommand, preferences field, dialogs, layer-delete hook.

- PR #3 submitted for review
- `cargo check` passes cleanly
- All changes integrated into main editor flow
- Implementation ready for Data review

---

## Known Debt Introduced

### Live Debt (This PR)

**`find_layer_index` stub in `crates/bevy_map_automap/src/apply.rs`**
- Returns `None` unconditionally
- **Functional cost:** Every automap rule targeting a named layer silently writes to nowhere. Output groups and alternatives specifying `layer_id` are fully ignored at apply time. No error surfaced. Editor appears to succeed while producing no output on targeted layers.
- **Trigger:** Must be resolved the instant `Layer::id` lands in `bevy_map_core` (i.e., once this PR merges). Delay beyond that point means silently broken automap in user projects.
- **Status:** Recorded in DEBT table (row: `find_layer_index`). Post-PR work, sequenced behind this merge.

---

## Escalation Path

If requirements become unclear during next session:
1. Escalate **directly to Data** — do not wait for lead task assignment
2. Data will verify impact on other running SEs (Data ↔ Wesley sync)
3. Data decides if it's a technical question (Data authority) or requires user input (escalate to Picard/Lead)

---

## Things for Next Session

- Check task list: has Data completed T-09 review?
- If GO: PR #3 merge unblocks Wesley's T-10 rebase
- If changes required: apply feedback, push commits, await re-review
- Once PR #3 merges: Data will handle T-10; Worf will proceed with T-05 (testing)

---

## Standing Reminders

- All API proposals go to Data before coding
- Architecture reference: `agents/architecture/architecture.md`
- Debt disclosure: every stub/TODO added at time of introduction (done for this PR)
- No out-of-scope spiraling — keep concerns focused, escalate adjacent work
