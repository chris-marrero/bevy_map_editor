# Mission 1 — Collision Editor Bug Fix + Numeric Input

**Status:** CLOSED
**Date:** 2026-02-27

## What Shipped

- Drag bug fix (Wesley)
- Numeric input panel (Barclay)
- 34 tests passing (+14 new)

## Protocol Findings

**Three violations, same root cause:** Picard acting as implementer rather than coordinator.

1. Picard edited a production file directly (trailing whitespace fix) — bypassed SE assignment
2. Review gate skipped — SE output went to Worf without Data review
3. Retro not written until user prompted it

**Structural response:** Protocol rewritten — all agents simultaneous at sprint start, Picard never touches files, SE→Data→Troi→Worf sequencing made explicit.

**PR workflow introduced this sprint.** SEs now work on feature branches; Data reviews and merges.

## Patterns to Watch

- Canvas drag path is structurally untestable (no AccessKit node on raw painter regions). Second sprint with uncoverable drag logic. Pattern: avoid interaction logic on unlabeled canvas regions.
- "Rect" button label vs. `CollisionDrawMode::Rectangle` inconsistency noted. Routed to Troi for future UX polish pass.

## Open Debt Carried Forward

- `CollisionDragOperation` wildcard arm silently discards unimplemented ops
- `0.01` drag threshold magic constant
- `format!("{:?}", one_way)` debug format in ComboBox
