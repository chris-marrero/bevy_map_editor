# Granted Permissions

Maintained by the Lead. Updated when the user grants a new permission.
Agents must check this file before asking for a permission that may already exist.

## Format

Each entry records: what is permitted, the scope/conditions, and when it was granted.

---

## Active Permissions

| # | Permission | Scope / Conditions | Granted |
|---|---|---|---|
| 1 | Run `cargo test -p bevy_map_editor` | Any time; to verify tests pass | Session 1 |
| 2 | Run `cargo build --features dynamic_linking` | Any time; for incremental builds | Session 1 (implied by CLAUDE.md) |
| 3 | Write to `agents/` directory | architecture.md, testing.md, permissions.md, retro_log.md — agent domain docs only | Session 1 |
| 4 | Create new source files under `crates/bevy_map_editor/src/` | When approved by Sr SE as part of a sprint task | Session 1 |
| 5 | Add `[dev-dependencies]` to `crates/bevy_map_editor/Cargo.toml` | When approved by Sr SE; no `wgpu` without explicit user approval | Session 1 |
| 6 | Modify `#[cfg(test)]` blocks in existing source files | Toolbar tests, any panel under test — approved per task | Session 1 |

| 7 | Add `wgpu` feature to `egui_kittest` dev-dep | One-time grant for Phase 3 snapshot tests | Session 2 (Phase 3 sprint directive) |
| 8 | Write to `.claude/agents/*.md` context files | Riker only, for context file and quarters system management | Session 3 (user directive: Riker manages all context files) |
| 9 | `git push origin main` | Riker only, at sprint close, after all protocol updates committed | Session 4 (user directive in Riker sprint-close task) |

## Pending / Not Yet Granted

- Pushing to remote / creating PRs — not granted for non-Riker agents; must ask each time
- Modifying production code outside of test infrastructure — requires task assignment + Sr SE approval
