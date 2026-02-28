# bevy_map_editor — Sprint Architecture Log

This file is an archive of sprint-scoped architecture decisions, design assessments, code
reviews, and SE assignment notes that have been resolved or superseded. Content is moved
here from `agents/architecture.md` when it is no longer needed for day-to-day orientation.

Entries are preserved verbatim. The canonical living reference is `agents/architecture.md`.

---

## UI Testing Rig — Architecture Assessment

**Status: Assessment complete. Implementation not yet started.**

### Problem Statement

The editor has no test infrastructure. `bevy_map_editor/Cargo.toml` has no `[dev-dependencies]` and no `#[cfg(test)]` modules. The goal is a minimal viable rig that allows a Test Engineer to write the first passing test against a real UI panel.

### Version Compatibility

- `bevy_egui` 0.39 resolves egui to **0.33.3**
- `egui_kittest` **0.33.3** is the correct matching version (versioned in sync with egui)
- `egui_kittest` 0.33.3 depends on `egui ^0.33.3` — this is a clean match
- Snapshot tests require the optional `snapshot` + `wgpu` features of `egui_kittest`; these add `egui-wgpu` and `wgpu ^27` as optional deps

### The Fundamental Problem: Bevy Coupling

The primary obstacle to testing is that essentially every render function takes either `&mut EditorState` (a Bevy `Resource`) or references to other Bevy-managed types. The full application boots a Bevy `App` with a windowed renderer, which cannot run headlessly in a unit test context without significant infrastructure.

However, the majority of UI panels accept plain Rust structs as inputs. The Bevy coupling comes only from:
1. The `Resource` derive on `EditorState`, `UiState`, etc. — which is just a marker trait, the struct is otherwise a plain Rust struct
2. `bevy_egui::egui` — which is just a re-export of `egui`; tests can depend on `egui` directly
3. Bevy `Handle<Image>` inside texture caches — which are only relevant to asset-loading systems, not to the UI render functions themselves

This means: **the UI panel functions are already testable with `egui_kittest` without running a Bevy App**, because their arguments are plain data structs. The Bevy `Resource` derive does not prevent instantiation with `EditorState::default()`.

### Where Tests Should Live

**Recommendation: tests live in `bevy_map_editor` directly, in `crates/bevy_map_editor/src/ui/` as `#[cfg(test)]` modules within each file, plus an integration test in `crates/bevy_map_editor/tests/`.**

Tradeoffs evaluated:

| Option | Pros | Cons |
|---|---|---|
| `#[cfg(test)]` in-crate | No new crate, direct access to private types, fast iteration | Tests compiled as part of the main crate build |
| New `bevy_map_editor_tests` crate | Full isolation, no build impact on main crate | Requires a new crate, only has access to public API surface, more setup, harder to justify for internal panel logic |
| `crates/bevy_map_editor/tests/` integration tests | Rust's standard integration test location, accesses only public API, clean separation | Same public-only limitation as a separate crate |

The internal panel functions (e.g., `render_toolbar`) are `pub` at the module level but not `pub` from outside the crate. A `bevy_map_editor_tests` crate would only see what `bevy_map_editor` re-exports. This would force everything test-relevant to be made `pub(crate)` or re-exported, which degrades the public API surface for no gain.

**Decision: start with `#[cfg(test)]` modules inside the panel files** (e.g., a `tests` module at the bottom of `toolbar.rs`). Graduate to `crates/bevy_map_editor/tests/` integration tests only for workflows that span multiple panels.

Do NOT create a `bevy_map_editor_tests` crate. It adds workspace overhead and provides no meaningful benefit over in-crate tests for a single-binary editor.

### Required Dev-Dependencies

Add to `crates/bevy_map_editor/Cargo.toml`:

```toml
[dev-dependencies]
egui_kittest = { version = "0.33", features = ["snapshot"] }
# wgpu feature only if snapshot rendering is needed:
# egui_kittest = { version = "0.33", features = ["snapshot", "wgpu"] }
```

The `egui` crate itself does not need to be a direct dev-dependency — `egui_kittest` re-exports it, and `bevy_egui` already provides `egui` as a dep. However, to avoid version confusion, it is cleaner to import `egui` from `bevy_egui::egui` in test code, consistent with how production code does it.

Note: `wgpu` feature for snapshot tests adds a heavy compile dependency. Start without it. Snapshot tests can be added in a second phase.

### Minimal Viable Test

The smallest test that exercises real UI code:

```rust
// In crates/bevy_map_editor/src/ui/toolbar.rs, at bottom:
#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    #[test]
    fn toolbar_select_tool_click() {
        let mut editor_state = crate::EditorState::default();
        let mut harness = Harness::new(|ctx| {
            render_toolbar(ctx, &mut editor_state, None);
        });
        harness.run();
        // Simulate clicking "Paint" button via AccessKit node label
        harness.get_by_label("Paint").click();
        harness.run();
        assert_eq!(editor_state.current_tool, EditorTool::Paint);
    }
}
```

This tests:
- The harness can render a real egui panel
- AccessKit node labels match button text (confidence in label strings)
- Clicking a selectable label mutates `EditorState`

This does not require Bevy, a window, a GPU, or any asset loading. It is pure egui + data struct mutation.

### What Is NOT Testable With This Rig

- Any code path that depends on Bevy's `AssetServer`, `Handle<Image>`, or `EguiContexts` (the texture cache loading systems)
- World-view painting / rendering (requires a full Bevy app with tilemap rendering)
- File dialogs (platform-native, no egui widget to target)

These require a different approach (e.g., integration tests that boot a headless Bevy app with `MinimalPlugins`). That is out of scope for the MVP rig.

### Phase Plan

**Phase 1 (MVP):** Add `egui_kittest` dev-dep, write the first toolbar test, confirm the harness compiles and runs. This is the Test Engineer's entry point.

**Phase 2:** Expand to menu_bar tests (testing `pending_action` is set correctly on menu item clicks), dialog visibility flag tests.

**Phase 3 (optional):** Add `wgpu` feature and snapshot tests for visual regression on stable panels.

### Open Questions for Test Engineer

1. `egui_kittest::Harness::new` takes a closure that is called once per frame. For panels that depend on mutable state shared across the closure boundary, the Test Engineer must use `Harness::new_ui` or capture the state outside the closure. Confirm this pattern works with the macro-based re-borrow pattern used in the automap editor before writing tests for it.
2. The `integration_registry: Option<&IntegrationRegistry>` argument accepted by `render_toolbar` and `render_menu_bar` should always be passed as `None` in tests — confirm this covers all non-integration code paths.
3. `menu_bar.rs` requires `&mut Project` and `&CommandHistory`. `Project` must be constructable with `Project::default()` or equivalent. Verify this before assigning menu bar tests.

---

## Test Helper Module Architecture

**Author:** Sr Software Engineer
**Date:** 2026-02-26
**Status:** Approved. SE may proceed with implementation.

---

### Decision 1 — Module Location

**Decision: Option A variant. `crates/bevy_map_editor/src/testing.rs`, gated as `#[cfg(test)]`, declared in `lib.rs` as `#[cfg(test)] mod testing;`.**

The UX Designer proposed `src/test_helpers.rs`. The name `testing` is marginally cleaner and consistent with Rust convention (`std::testing` does not exist, but `#[cfg(test)]` modules conventionally use short names). The name is immaterial — what matters is the gate and the location.

**Option B (integration tests directory) is rejected.** The integration test directory (`tests/`) can only see the crate's public API surface. The panel functions (`render_toolbar`, `render_menu_bar`, etc.) are `pub` within the crate but are not part of the external public API of `bevy_map_editor`. Forcing them into the public API surface to accommodate a tests/ location would be a real API degradation with no benefit. Integration test files are appropriate only for tests that explicitly need to cross the crate boundary. None of the currently planned tests do.

**The `#[cfg(test)]` gate is mandatory.** This module must not compile into production builds. It imports `egui_kittest`, which is a dev-dependency. Any production compilation of `testing.rs` would be a build failure. The gate also signals clearly to readers that this module has no production role.

**Module declaration in `lib.rs`:**
```rust
#[cfg(test)]
mod testing;
```

**Import in test modules:**
```rust
#[cfg(test)]
mod tests {
    use crate::testing::*;
    // ...
}
```

The UX Designer proposed `use crate::test_helpers::*;` — SE must update this to `use crate::testing::*;` throughout.

---

### Decision 2 — Bundle Structs: What Is Actually Needed, and What Is Not

I verified the `render_menu_bar` signature directly from source (`menu_bar.rs` lines 16-25):

```rust
pub fn render_menu_bar(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    editor_state: &mut EditorState,
    project: &mut Project,
    history: Option<&CommandHistory>,
    clipboard: Option<&TileClipboard>,
    preferences: &EditorPreferences,
    integration_registry: Option<&IntegrationRegistry>,
)
```

The problem is real: `Harness::new_state` owns exactly one `State`. The closure signature is `FnMut(&egui::Context, &mut State)`. With two `&mut` arguments (`ui_state`, `editor_state`, `project`), there is no way to pass them without a wrapper struct.

**`MenuBarState` bundle is required for any menu bar test.**

I also verified all constituent types have `Default` implementations:
- `UiState` — `#[derive(Resource, Default)]` confirmed in `ui/mod.rs`
- `EditorState` — manual `Default` impl, confirmed previously
- `Project` — `impl Default for Project` confirmed at `project/mod.rs` line 157
- `CommandHistory` — `#[derive(Resource, Default)]` confirmed in `commands/command.rs`
- `TileClipboard` — `#[derive(Resource, Default)]` confirmed in `commands/clipboard.rs`
- `EditorPreferences` — `impl Default for EditorPreferences` confirmed in `preferences/mod.rs`

**`MenuBarState` bundle — approved as specified.** The UX Designer's proposed shape is correct. One clarification from the source: `history` and `clipboard` are `Option<&T>` in the render signature. The bundle owns them as `Option<T>` (not `Option<&T>`), and the harness closure borrows from the owned values when calling `render_menu_bar`. The SE must implement the closure as:

```rust
|ctx, state: &mut MenuBarState| {
    render_menu_bar(
        ctx,
        &mut state.ui_state,
        &mut state.editor_state,
        &mut state.project,
        state.history.as_ref(),
        state.clipboard.as_ref(),
        &state.preferences,
        None,
    );
}
```

**`TilesetEditorBundle` — approved as specified.** I have not verified the `render_tileset_editor` signature in depth, but the pattern is identical. The SE must verify the actual signature before implementing the bundle. If the signature takes texture caches that cannot be constructed with `None` or a stub, the SE must escalate back to me before proceeding. Do not invent workarounds silently.

**No other bundles.** The UX Designer's constraint is correct: no speculative bundles. `render_toolbar` takes only `(&Context, &mut EditorState, Option<&IntegrationRegistry>)` — a single mutable argument, no bundle required. The toolbar harness factory passes `EditorState` directly as the `State` type.

---

### Decision 3 — `EditorInteractions` Trait vs. Free Functions

The UX Designer proposed an extension trait called `EditorInteractions` on `Harness`. I examined the actual `Harness` definition and the `Queryable` implementation.

**The trait-on-Harness approach has a lifetime problem.** `Harness<'a, State>` carries a lifetime `'a` (the closure lifetime). Any extension trait method that calls `self.get_by_label(label).click()` must deal with the `Node<'tree>` lifetime returned by `get_by_label` — specifically, `'tree` is tied to `'node`, which is tied to `&'node self`. This is already handled by the existing `Queryable` impl on `Harness`, which uses a two-lifetime bound `where 'node: 'tree`. An extension trait on `Harness` would need the same two-lifetime generic parameters, making the trait definition non-trivial:

```rust
trait EditorInteractions<'tree, 'node>
where
    'node: 'tree,
{
    fn click_labeled(&'node self, label: &'tree str);
    // ...
}
```

This is implementable but the SE must get the lifetimes right or the compiler will reject it. The alternative is free functions, which are simpler and have the same call ergonomics at the test site.

**Decision: implement as free functions, not a trait.**

Rationale:
1. Free functions on `&Harness<'_, State>` avoid the two-lifetime trait parameter complexity.
2. The UX Designer's stated preference for a trait was "reads naturally as a method chain." However, the spec's own summary shows the target usage as `harness.click_labeled("Paint")`. This is indeed a method call, and free functions cannot provide that syntax. However, `harness.get_by_label("Paint").click()` is already one line and completely readable. The additional wrapper provides one line of savings with non-trivial implementation complexity.
3. The interaction helpers are thin wrappers over `get_by_label(...).click()`. Their value is documentation and label-as-argument ergonomics, not any behavioral logic. Free functions named `click_labeled(harness, label)` accomplish the same documentation goal.

**Pushback on the UX spec's trait approach:** The spec conflates two things — the trait methods on `Harness` (`click_labeled`, `toggle_labeled`, `select_labeled`) and the `EditorInteractions` trait as described at the top of section 2, where it is defined on the node type. The spec is internally inconsistent: it defines `EditorInteractions` on the node, then pivots to placing the helpers on `Harness`. I am resolving this inconsistency by choosing free functions on `&Harness` explicitly. The SE must not implement the node-level trait. Free functions only.

**Approved function signatures for Phase 2:**

```rust
// In crates/bevy_map_editor/src/testing.rs

pub fn click_labeled<State>(harness: &Harness<'_, State>, label: &str) { ... }
pub fn toggle_labeled<State>(harness: &Harness<'_, State>, label: &str) { ... }
pub fn select_labeled<State>(harness: &Harness<'_, State>, label: &str) { ... }
```

These are generic over `State` so they work with any harness type. The SE must use `where` bounds if the `Queryable` impl requires them — check the constraint on `Harness: Queryable`.

---

### Decision 4 — AccessKit Tree Dump: Feasibility

I examined `kittest::State`, `kittest::node::NodeT`, `debug_fmt_node`, and the `Queryable` traversal machinery.

**The tree is fully traversable without a GPU.** `kittest::State` wraps `accesskit_consumer::Tree`, which is built from `egui`'s `platform_output.accesskit_update` (a `TreeUpdate`). This is a pure data structure — no rendering required. After `harness.run()`, the AccessKit tree is available via `harness.kittest_state()`, which returns `&kittest::State`. The root node is `harness.kittest_state().root()`, which returns an `AccessKitNode<'_>`.

**`debug_fmt_node` already exists** in `kittest::node` and provides structured debug output. The UX Designer's proposed `dump_accessibility_tree` is a thin wrapper over recursive traversal of `NodeT::children_recursive()`, reading `accesskit_node.role()`, `accesskit_node.label()`, `accesskit_node.is_disabled()`, and `accesskit_node.toggled()` from `kittest::node::debug_fmt_node`.

However, `debug_fmt_node` outputs Rust debug struct format, not the indented text format the UX Designer specified. The SE will need to write a custom recursive formatter. This is straightforward — `children_recursive()` exists and the node exposes all needed fields through `AccessKitNode`. The custom format is not complex to implement.

**One constraint:** `harness.kittest_state()` returns `&kittest::State`, but `kittest::State` is not re-exported from `egui_kittest` as a named type in a way that lets external code call `state.root()` directly without importing `kittest`. The SE must `use egui_kittest::kittest;` in the helper module to access `kittest::State` and its methods. This is fine since `egui_kittest` re-exports `kittest` as a pub module.

**`dump_accessibility_tree` — approved as a free function taking `&Harness<'_, State>`.**

**`write_accessibility_tree` — approved, but the SE must note that the path is relative to the process working directory at test time.** In `cargo test`, that is the crate root. Document this in the function's doc comment.

---

### Decision 5 — Snapshot Tests: File I/O Ownership

I read the full `egui_kittest/src/snapshot.rs` source.

**`egui_kittest` handles all file I/O for snapshots.** The `Harness::snapshot(name)` method calls `harness.render()` (GPU render), then `try_image_snapshot_options`, which does all of:
- Directory creation (`std::fs::create_dir_all`)
- PNG write (`image.save`)
- Diff image generation and write
- Old backup rename
- New file comparison

The SE does not need to implement any file I/O for snapshot tests. The helper functions `capture_snapshot` and `bless_snapshot` proposed by the UX Designer are thin wrappers:

- `capture_snapshot(harness, name)` — calls `harness.snapshot(name)` (panics on mismatch)
- `bless_snapshot(harness, name)` — sets `UPDATE_SNAPSHOTS=force` env var, then calls `harness.snapshot(name)`. Actually this is wrong — `UPDATE_SNAPSHOTS` is read from the environment at test time, not set by a function call. See correction below.

**Correction to the UX Designer's `bless_snapshot` API:** The egui_kittest snapshot library uses the `UPDATE_SNAPSHOTS` environment variable to control update mode, not a function call. There is no API to force-update a single snapshot programmatically from within a test. The bless mechanism in the UX spec — a companion `#[ignore]` test that calls `bless_snapshot` — is the right pattern, but `bless_snapshot` cannot be implemented as described.

**Revised bless approach:** The companion bless test should simply set `UPDATE_SNAPSHOTS=1` when run, using `std::env::set_var`. This is `unsafe` in Rust 2024 (env mutation is `unsafe` due to threading concerns), but since the bless test is `#[ignore]` and run in isolation, it is acceptable. Alternatively, skip the `bless_snapshot` helper entirely and document that the bless workflow is: `UPDATE_SNAPSHOTS=1 cargo test -- --ignored bless_toolbar_grid_unchecked`. This is the egui project's own workflow (per `HOW_TO_UPDATE_SCREENSHOTS` constant in snapshot.rs: `"Run UPDATE_SNAPSHOTS=1 cargo test --all-features to update the snapshots."`).

**Decision: do not implement `bless_snapshot` as a function.** Instead, the bless workflow is env-var-based. The `#[ignore]` companion test pattern is preserved, but the test body simply calls `harness.snapshot(name)` — identical to the snapshot test. With `UPDATE_SNAPSHOTS=1`, `try_image_snapshot_options` overwrites the existing file. The SE must document this in the testing.md conventions section, not invent a `bless_snapshot` wrapper.

**Snapshot storage path:** The UX Designer specified `crates/bevy_map_editor/tests/snapshots/`. The `SnapshotOptions` default path is `tests/snapshots` (relative to process cwd, which is the crate root during `cargo test`). This resolves to `crates/bevy_map_editor/tests/snapshots/`. The default is correct — the SE does not need to override `SnapshotOptions::output_path`. The directory does not need to exist beforehand; `try_image_snapshot_options` creates it.

**Snapshot helpers that ARE approved:**
- `capture_snapshot(harness, name)` — wrapper for `harness.snapshot(name)`. One line, but documents intent in test code.
- `render_to_bytes(harness) -> image::RgbaImage` — wrapper for `harness.render()`. Requires `wgpu` feature gate.

**`bless_snapshot` is NOT approved.** Remove it from the spec. Bless workflow is: `UPDATE_SNAPSHOTS=1 cargo test --features wgpu -- --ignored bless_<name>`. The companion test body calls `harness.snapshot(name)` identically to the main test.

---

### Decision 6 — What the SE Must Not Do

These are explicit prohibitions to prevent scope creep:

1. Do not create bundle structs for panels not listed in the spec (`TilesetEditorBundle` is conditional on signature verification — see Decision 2).
2. Do not implement `bless_snapshot`. Bless is an env-var workflow.
3. Do not put test helpers in a separate crate. The module stays in `crates/bevy_map_editor/src/testing.rs`.
4. Do not make `testing.rs` publicly accessible outside test compilation. The `#[cfg(test)]` gate is not optional.
5. Do not add `wgpu` to `[dev-dependencies]` in this phase. Snapshot rendering tests are Phase 3. Only `egui_kittest = { version = "0.33", features = ["snapshot"] }` is in dev-deps, as already established.
6. Do not implement `assert_panel_visible` without Data answering Troi's Open Question 1 (harness architecture for panel visibility tests). The UX spec (in `testing.md`) now defines the AccessKit semantics fully — anchor labels are `ui.heading(...)` nodes, absence-based detection — but the harness architecture for rendering full side panels in a test context is not yet specified. The assertion helpers themselves are approved; the harness builder is the open question.

---

### Approved Module Structure

```
crates/bevy_map_editor/src/testing.rs   — the module, #[cfg(test)] only
```

Contents of `testing.rs`, organized as:

1. **Bundle structs** — `MenuBarState` (required). `TilesetEditorBundle` conditional on signature verification.
2. **Factory functions** — `editor_state_default()`, `editor_state_level_view()`, `editor_state_world_view()`, `editor_state_paint_tool()`, `menu_bar_state_empty_project()`, `menu_bar_state_with_undo()`.
3. **Harness builders** — `harness_for_toolbar(EditorState) -> Harness<'static, EditorState>`, `harness_for_menu_bar(MenuBarState) -> Harness<'static, MenuBarState>`.
4. **Interaction helpers** — `click_labeled`, `toggle_labeled`, `select_labeled` as free functions generic over `State`.
5. **Assertion helpers** — `assert_tool_active`, `assert_checkbox_state`, `assert_widget_enabled`, `assert_widget_disabled`, `assert_pending_action`. `assert_panel_visible` / `assert_panel_not_visible` are approved by Troi spec (see `testing.md`) but blocked on harness architecture confirmation from Data before Worf implements them.
6. **Tree dump helpers** — `dump_accessibility_tree`, `accessibility_tree_string`, `write_accessibility_tree`.
7. **Snapshot helpers** — `capture_snapshot` (non-wgpu wrapper for `harness.snapshot()`). `render_to_bytes` gated behind `#[cfg(feature = "wgpu")]`. NOT `bless_snapshot`.

Declaration in `lib.rs`:
```rust
#[cfg(test)]
mod testing;
```

---

### Open Items Requiring SE Verification Before Implementation

1. **`render_tileset_editor` signature** — verify the actual arguments and confirm `TilesetEditorBundle` covers them. If texture caches are non-optional, escalate back.
2. **`assert_widget_enabled` / `assert_widget_disabled`** — the SE must confirm that `accesskit_node.is_disabled()` reflects the egui `ui.disable()` scope correctly. This should be verified empirically in the first test that uses these assertions. Do not assert correctness of the AccessKit disabled state without a passing test.
3. **`Harness: Queryable` lifetime bounds** — the SE must check whether the free functions `click_labeled<State>(harness: &Harness<'_, State>, ...)` need additional lifetime bounds to call `get_by_label`. The `Queryable` impl on `Harness` uses `where 'node: 'tree`. In a free function context this is handled by the compiler implicitly for shared references, but verify it compiles before declaring done.

---

## Automapping Sprint — Architecture Notes

**Author:** Lt. Cmdr. Data
**Date:** 2026-02-26
**Status:** Design complete. Three escalations pending user decision before SE begins.

---

### Crate Layout

**`bevy_map_automap` (new crate, no Bevy dependency):**
- `src/types.rs` — all data types
- `src/apply.rs` — `apply_ruleset`, `apply_automap_config`, and all helpers
- Dependencies: `bevy_map_core`, `serde`, `uuid`, `rand`

**`bevy_map_core` (modification):**
- `Layer` struct receives `pub id: Uuid` with `#[serde(default = "Uuid::new_v4")]` — required for stable layer references. See Escalation 1.

**`bevy_map_editor` (modification):**
- `Project` gains `pub automap_config: AutomapConfig` with `#[serde(default)]`
- `AutomapCommand` implemented in `commands/command.rs`
- `PendingAction::ApplyAutomap` wired in `process_edit_actions`
- `validate_automap_config` added to `Project::validate_and_cleanup`

---

### Core Data Types

```rust
// bevy_map_automap/src/types.rs

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutomapConfig {
    pub rule_sets: Vec<RuleSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub id: Uuid,
    pub name: String,
    pub rules: Vec<Rule>,
    pub settings: RuleSetSettings,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetSettings {
    pub edge_handling: EdgeHandling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EdgeHandling {
    #[default]
    Skip,
    TreatAsEmpty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    /// AND logic: all groups must match.
    pub input_groups: Vec<InputConditionGroup>,
    /// Exactly one selected per match (weighted random).
    pub output_alternatives: Vec<OutputAlternative>,
    #[serde(default)]
    pub no_overlapping_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConditionGroup {
    pub layer_id: Uuid,           // references Layer::id
    pub half_width: u32,          // pattern = (2*hw+1) x (2*hh+1)
    pub half_height: u32,
    pub matchers: Vec<CellMatcher>, // row-major, len = (2*hw+1)*(2*hh+1)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellMatcher {
    Ignore,
    Empty,
    NonEmpty,
    Tile(u32),    // tile index; flip bits stripped before compare
    NotTile(u32), // tile index; flip bits stripped before compare
    Other,        // matches any tile not listed as Tile/NotTile anywhere in this rule
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellOutput {
    Ignore,    // leave cell unchanged
    Empty,     // erase cell (write None)
    Tile(u32), // write specific tile (flip bits preserved)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputAlternative {
    pub id: Uuid,
    pub layer_id: Uuid,
    pub half_width: u32,
    pub half_height: u32,
    pub outputs: Vec<CellOutput>,
    #[serde(default = "default_weight")]
    pub weight: u32,  // 0 = never selected
}
fn default_weight() -> u32 { 1 }
```

---

### Layer References: UUID

Layer references use `Uuid`, not string names or indices. `Layer` gains `pub id: Uuid` with `#[serde(default = "Uuid::new_v4")]`. Old project files load cleanly (fresh UUID assigned on deserialization, stable on next save).

**See Escalation 1** — this is a one-way format migration. User decision required.

---

### Application Algorithm

```
apply_automap_config(level, config, rng):
    for each rule_set in config.rule_sets (if not disabled):
        apply_ruleset(level, rule_set, rng)

apply_ruleset(level, rule_set, rng):
    for each rule in rule_set.rules:
        explicit_tiles = union of all Tile(x)/NotTile(x) indices across all input_groups
        written_positions = HashSet  // for NoOverlappingOutput

        resolve all layer_ids → layer indices; skip rule with warning if any ID not found

        for y in 0..level.height:
            for x in 0..level.width:
                if no_overlapping_output && (x,y) in written_positions: continue
                if all_input_groups_match(level, rule, (x,y), edge_handling, explicit_tiles):
                    alt = select_output_alternative(rule.output_alternatives, rng)
                    if alt is Some:
                        apply_output_alternative(level, alt, (x,y))
                        if no_overlapping_output: mark alt's cells as written

matcher_matches(matcher, cell, edge_handling):
    Ignore      → always true
    Empty       → cell is None (including OOB treated-as-empty)
    NonEmpty    → cell is Some(_)
    Tile(id)    → cell is Some(v) && tile_index(v) == id
    NotTile(id) → cell is Some(v) && tile_index(v) != id, OR cell is None
    Other       → cell is Some(v) && tile_index(v) NOT IN explicit_tiles
    OOB + Skip  → any matcher returns false (position skipped entirely)
    OOB + TreatAsEmpty → treated as None (NonEmpty/Tile/Other still fail)

select_output_alternative(alts, rng):
    total = sum of weights; if 0 return None
    weighted random pick
```

**Key edge cases:**
- `NotTile(x)` on empty cell → true (empty is not tile X)
- `Other` on empty cell → false (Other requires a tile present)
- Zero-weight alternatives → never selected; if all are zero, rule fires but no output
- Rule with no Tile/NotTile matchers → `explicit_tiles` empty; `Other` matches every non-empty tile
- Pattern larger than map → all positions skip (Skip mode); may match at edges (TreatAsEmpty mode)

---

### Project Integration

```rust
// crates/bevy_map_editor/src/project/mod.rs
#[serde(default)]
pub automap_config: AutomapConfig,
```

Follows the exact pattern of `autotile_config`. Old files get an empty config. `validate_and_cleanup` gains `validate_automap_config` that removes references to non-existent layer UUIDs. See Escalation 2 for behavior on layer deletion.

---

### AutomapCommand (Undo/Redo)

```rust
pub struct AutomapCommand {
    pub level_id: Uuid,
    /// layer_index → HashMap<(x,y), (old_tile, new_tile)>
    pub layer_changes: HashMap<usize, HashMap<(u32, u32), (Option<u32>, Option<u32>)>>,
    description: String,
}
```

Execute applies `new_tile` values; undo applies `old_tile` values. Both set `render_state.needs_rebuild = true`. Identical pattern to `BatchTileCommand`, extended to multi-layer.

**Snapshot protocol:** Before calling `apply_automap_config`, snapshot all tile layers. After it returns, diff per layer. Construct command from non-empty diffs. Discard if no changes.

**Testability requirement (from Worf):** `execute()` must be pure — no Bevy API calls. The snapshot + diff construction can call Bevy APIs in the system that wraps the command, but `execute()` and `undo()` themselves operate only on `&mut Project`. Worf will escalate immediately if this is violated.

---

### SE Assignment

**Track A — Engine (`bevy_map_automap` crate):**
Persona: **Geordi**. Non-trivial algorithm; creative problem-solving value.
Branch: `sprint/automapping/geordi-engine`

**Track B — Editor integration (`bevy_map_core`, `commands/`, `project/`):**
Persona: **Barclay**. Touches serialized format, serde attributes, orphan cleanup, multi-layer diff. Edge cases favor Barclay's thoroughness.
Branch: `sprint/automapping/barclay-integration`

**Track C — Visual rule editor UI:** Blocked on Troi's spec (T-01). Assigned after T-01 completes.

**Parallel safety:** Geordi only touches `crates/bevy_map_automap/`. Barclay touches `bevy_map_core/src/layer.rs`, `bevy_map_editor/src/project/mod.rs`, `bevy_map_editor/src/commands/command.rs`. No file overlap. Safe to run in parallel. Sequencing constraint: Geordi must publish the crate skeleton (types only) first so Barclay's `cargo check` passes.

---

### DEBT — Automapping

| Debt | Cost if unaddressed | Trigger to fix |
|---|---|---|
| `apply_automap_config` is O(rules × width × height) | Acceptable now; may lag on large levels with many rules | If user reports lag on levels > 256×256 |
| `Layer::id` on old files is not stable until first save | Rule references assigned on load may diverge if file is edited by another tool | When multi-tool workflows are supported |
| Output alternative grid dimensions stored independently | Visually confusing in the UI editor if grids differ in size | Troi UX spec may constrain this |
| `apply_automap_config` takes `impl Rng` | Deterministic replay requires externally supplied seed | If replay/testing of probabilistic rules is needed |

---

### Escalations (User Decision Required)

**[ESCALATE 1: Layer UUID format migration]** Adding `id: Uuid` to `Layer` changes the serialized format of every project file. Old files load cleanly (fresh UUID assigned on deserialization), but the new field is written on next save — one-way migration; not backward-compatible with older editor builds. **Does the user accept this migration?** If not, we must redesign with name-based references plus validation warnings.

**[ESCALATE 2: Orphan reference behavior on layer delete]** When a layer is deleted, should automap rule references to that layer be silently cleaned up (matching existing `autotile_config` pattern), or should the user see a warning listing affected rules before deletion proceeds? **User decision required.**

**[ESCALATE 3: Flip bit matching]** Current design strips flip bits before tile index comparison (matching Tiled behavior). Should the engine support flip-aware matching — e.g., `TileFlipped(id, flip_x, flip_y)` as a `CellMatcher` variant? Designing for it now costs little; adding it later requires a data format change. **Recommendation: add the variant now, even if the UI only exposes it in a future sprint.** User confirmation preferred.

---

## Collision Editor Sprint — Architecture Notes

**Author:** Lt. Cmdr. Data
**Date:** 2026-02-26
**Status:** Assessment complete. Ready for SE assignment.

---

### Source locations

All collision editor code lives in one file:
`crates/bevy_map_editor/src/ui/tileset_editor.rs`

| Concern | Lines |
|---|---|
| State structs (`CollisionEditorState`, `CollisionDrawMode`, `CollisionDragOperation`, `CollisionDragState`) | 480–559 |
| `render_collision_tab` (three-panel layout, tools panel, canvas panel) | 1930–2103 |
| `render_collision_tile_selector` | 2105–2231 |
| `render_collision_canvas` (allocation, texture draw, shape draw, step sequencing) | 2233–2368 |
| `handle_collision_canvas_input` (double-click, click, drag_started, dragged, drag_stopped, context menu) | 2472–2942 |
| `render_collision_properties` (properties panel; shape name, one-way, layer, mask, action buttons) | 2945–3054 |
| Coordinate helpers | 3056–3139 |

Core collision types live in:
`crates/bevy_map_core/src/collision.rs`

`CollisionShape` is an enum: `None`, `Full`, `Rectangle { offset: [f32;2], size: [f32;2] }`, `Circle { offset: [f32;2], radius: f32 }`, `Polygon { points: Vec<[f32;2]> }`. All coordinates are 0.0–1.0 normalized. There is no maximum polygon point count imposed by the type — `Vec<[f32;2]>` is unbounded.

The mutation API on `Tileset` (in `crates/bevy_map_core/src/tileset.rs`):
- `set_tile_collision_shape(tile_index: u32, shape: CollisionShape)` — overwrites shape, preserves other `CollisionData` fields (body type, one-way, layer, mask). This is the correct write path. After calling it, `project.mark_dirty()` must be called.

---

### Bug 1: Drag doesn't add shape — Root Cause Confirmed

**Verdict: the root cause is exactly as described in the sprint brief.** The analysis is complete.

**In egui, `Response::clicked()` fires only when the mouse button is pressed AND released with no appreciable movement.** When the user drags, egui's internal threshold for "is this a drag?" is crossed before release. On the frame the drag threshold is crossed, `drag_started()` becomes true and `clicked()` becomes permanently false for that gesture. There is no frame on which both `clicked()` and `drag_started()` are true for the same drag gesture.

**The defective code is at lines 2552–2562 (Rectangle) and 2563–2572 (Circle), inside the `if response.clicked()` block:**

```rust
// Line 2539 — this block is NEVER entered during a drag gesture
if response.clicked() {
    ...
    match drawing_mode {
        CollisionDrawMode::Rectangle => {
            // drag_state initialized here — but clicked() is false on a drag
            editor_state...collision_editor.drag_state = Some(CollisionDragState {
                operation: CollisionDragOperation::NewRectangle,
                start_pos: normalized,
                current_pos: normalized,
            });
        }
        CollisionDrawMode::Circle => {
            // same problem
            editor_state...collision_editor.drag_state = Some(CollisionDragState {
                operation: CollisionDragOperation::NewCircle { center: normalized },
                ...
            });
        }
        ...
    }
}
```

When the user performs a drag gesture:
1. Frame N: pointer down — no event fires yet (threshold not crossed).
2. Frame N+k: pointer moves past drag threshold — `drag_started()` = true, `clicked()` = false. `drag_state` is never set for Rectangle/Circle because the `if response.clicked()` block is skipped.
3. Frames N+k..M: `response.dragged()` = true. The `if let Some(ref mut drag_state)` block at line 2619 finds `drag_state = None`. Nothing updates.
4. Frame M: pointer released — `drag_stopped()` = true. `drag_state.take()` returns `None`. No shape is committed.

**The fix requires moving Rectangle and Circle drag initialization from `if response.clicked()` to `if response.drag_started()`.**

The `drag_started()` block currently exists at line 2583, but it is guarded by `drawing_mode == CollisionDrawMode::Select`. The fix is:
- Remove the `Select`-only guard on the existing `drag_started()` block, OR
- Add a parallel `drag_started()` branch for `Rectangle` and `Circle` modes.

**Secondary question: can `double_clicked()` and `drag_started()` conflict?**

The double-click handlers are at lines 2488 and 2512, both gated on `drawing_mode == CollisionDrawMode::Polygon` or `Select`. Rectangle and Circle modes have no double-click handler. There is no conflict.

**Secondary question: can `clicked()` and `drag_started()` fire on the same frame?**

No. In egui's pointer state model, a gesture is classified as either a click or a drag, not both. Once `drag_started()` is true for a gesture, `clicked()` will never be true for that same gesture. However, a clean click (press and release without movement) will fire `clicked()` on the release frame and will never fire `drag_started()`. This means the two branches are mutually exclusive per gesture.

**Consequence for the fix:** After the fix, `clicked()` will still fire for Rectangle/Circle on a clean tap (no movement), initializing `drag_state`. Then `drag_stopped()` fires immediately on the same frame (since pointer was released). `drag_stopped()` at line 2630 calls `drag_state.take()` and checks for minimum size (`width > 0.01 && height > 0.01`). Because start_pos == current_pos on a clean tap, `width = 0` and `height = 0`. The shape will correctly not be committed. This is the correct zero-size guard behavior and it remains correct after the fix. No additional guard is needed for the click-no-drag case.

**Edge case: what if the user releases mid-drag before threshold?** Same as clean click — egui treats sub-threshold movement as a click, not a drag. `clicked()` fires, `drag_state` is initialized, then `drag_stopped()` immediately fires and the zero-size guard prevents spurious shape creation. Correct.

**Exact lines that need to change:**

The `drag_started()` handler at line 2583 must be expanded to cover Rectangle and Circle. The existing `Select`-mode vertex-drag logic stays; Rectangle and Circle cases are added. The `clicked()` block at lines 2552–2572 can be left in place — it handles the clean-tap case that initializes a zero-size drag which is immediately discarded. Both branches producing `drag_state` for the same mode is safe because `drag_started()` and `clicked()` are mutually exclusive per gesture.

---

### Feature: Numeric Input Panel — Assessment

#### 1. Placement

`render_collision_properties` is already called at line 2076 from the right tools panel, after the draw-mode buttons and instructions. It renders: shape name label, separator, one-way combobox, layer DragValue, mask DragValue, separator, "Set Full Collision" and "Clear Collision" buttons.

The numeric input fields for shape coordinates belong in `render_collision_properties`, appended after the existing content. There is no separate section needed. The right panel has a default width of 180px and is resizable. The existing content already includes DragValue widgets, which are compact. Adding up to 5 DragValue fields (x, y, w, h for Rectangle; x, y, r for Circle) will not cause scroll issues at default width — DragValues stack vertically at ~20px each, the total panel height for the densest case (Rectangle: 4 fields) is well under 400px.

**Decision: numeric input is an extension of `render_collision_properties`, not a new panel or section. Do not add a new heading or panel.**

#### 2. Does `render_collision_properties` conflict with numeric input?

No conflict. It already reads `collision_data` by cloning from the tileset (`t.get_tile_properties(tile_idx).map(|p| p.collision.clone())`). The numeric input fields follow the same pattern: read current values from the clone, detect change, write back via `tileset.set_tile_collision_shape(tile_idx, updated_shape)` + `project.mark_dirty()`. The existing function structure handles this cleanly — extend it in-place.

One note: the current code uses `format!("{:?}", one_way)` for the ComboBox selected text (line 2982) which exposes the Rust debug format. This is an existing cosmetic issue; it is not in scope and must not be fixed in this sprint.

#### 3. Live update path: is `tileset.set_tile_collision_shape` + `project.mark_dirty()` sufficient?

Yes. `render_collision_canvas` reads collision data from `project.tilesets` on every frame (line 2328–2332). It does not cache shape values in `CollisionEditorState`. Therefore any write to `tileset.set_tile_collision_shape` takes effect on the next frame's render. No additional state invalidation is needed.

**Confirmed write path:**
```rust
if let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id) {
    tileset.set_tile_collision_shape(tile_idx, updated_shape);
    project.mark_dirty();
}
```
This is the exact same pattern already used at lines 2663, 2678, 3043, 3051.

#### 4. Polygon point list: maximum practical count and type limits

`CollisionShape::Polygon { points: Vec<[f32;2]> }` imposes no upper bound. The type is a `Vec` with no capacity limit.

Practical consideration: the UI should list polygon points as a scrollable list within `render_collision_properties`. With `egui::ScrollArea`, any count is renderable. However:
- The existing polygon point management operations (add via click, delete via context menu) are already bounded by UX — a polygon with hundreds of points is impractical to draw by hand.
- The numeric input for polygon points is a list of (x, y) DragValue pairs per vertex, within a `ScrollArea`. No cap is needed at the data model level.
- The SE should wrap the polygon point list in a `ScrollArea` with a reasonable max height (e.g., 200px) to prevent the panel from growing unbounded.

**No limit needs to be imposed at the type level. The SE should use a bounded `ScrollArea` for the polygon point list in the numeric panel.**

#### 5. New state fields in `CollisionEditorState`?

No new fields are required. The project data (`CollisionShape`) is the single source of truth. The DragValue widgets in `render_collision_properties` read from a cloned `CollisionData` each frame and write back on change. This is the same stateless immediate-mode pattern already used for layer and mask. There is no need for intermediate editing state.

**The only scenario that would require new state is a two-phase "confirm/cancel" edit flow** (e.g., the user types into a field and presses Enter to confirm). The requirement as stated is "type exact values" — immediate live update is sufficient and simpler. No confirm/cancel flow is needed unless Troi specifies it.

---

### SE Persona Recommendation

**Bug fix: Wesley Crusher.**

The bug fix is fully specified. The cause is confirmed, the exact lines are identified, the fix is a well-bounded change to `handle_collision_canvas_input` (add Rectangle and Circle cases to the `drag_started()` block, around lines 2583–2612). The edge cases have been enumerated and the zero-size guard handles them correctly without modification. This is a clean, pattern-adherent change with no ambiguity. Wesley is the right persona: defined scope, no creative problem-solving required, fast clean output.

**Numeric input: Barclay.**

The numeric input extension to `render_collision_properties` touches all four `CollisionShape` variants (None and Full have no editable coordinates, Rectangle has 4 fields, Circle has 3 fields, Polygon has a variable-length list). The polygon case in particular requires: a `ScrollArea`, per-vertex DragValue pairs, correct index tracking for writes, and bounds-safe vector mutation. The `CollisionShape` enum arms must be matched exhaustively. Barclay's thoroughness and edge-case focus are appropriate here. The feature is well-specified but the polygon case has enough surface area for subtle bugs (off-by-one on index, unbounded panel growth, incorrect clamping) that Wesley's speed-over-caution approach would be risky.

**Both can work in parallel.** The bug fix is in `handle_collision_canvas_input`. The numeric input is in `render_collision_properties`. These are separate functions with no shared mutable state. File conflict risk: both are in `tileset_editor.rs`. They must not touch overlapping lines. The bug fix modifies lines approximately 2583–2612. The numeric input modifies lines approximately 2973–3054 (end of `render_collision_properties`). No overlap. Parallel execution is safe.

---

## Collision Editor Sprint — Code Review

**Reviewed by: Lt. Cmdr. Data**
**Files reviewed:** `crates/bevy_map_editor/src/ui/tileset_editor.rs` (lines 2472–2706, 2950–3189)
**Verdict: GO for Worf — with three advisory findings recorded below.**

---

### Change 1: Wesley's drag bug fix (`handle_collision_canvas_input`)

#### Summary of change

Rectangle and Circle drag-state initialization was moved from `response.clicked()` to `response.drag_started()`. The Polygon `clicked()` arm remains in the `clicked()` block untouched. The Select mode vertex drag init was moved inside the `drag_started()` match arm.

#### Finding 1.1 — egui event model: `clicked()` and `drag_started()` mutual exclusivity (CONFIRMED CORRECT)

In egui, a `Response` is backed by a single interaction per widget per frame. `clicked()` returns `true` only when a pointer button is pressed and released within the widget without any drag motion being detected. `drag_started()` returns `true` on the first frame the pointer is classified as dragging (which requires exceeding the drag threshold). These two states are mutually exclusive by design in egui's `Response` implementation: a pointer sequence is either a click or a drag, not both. The original bug — initializing drag state inside `clicked()` — meant the drag state was never set on the first drag frame, because `clicked()` requires release, which had not happened. Moving initialization to `drag_started()` is the correct fix.

**Finding: Correct. No issue.**

#### Finding 1.2 — Polygon `clicked()` arm unaffected (CONFIRMED CORRECT)

The Polygon arm remains at lines 2543–2549 inside the `if response.clicked()` block. This is correct: polygon point accumulation is click-based, not drag-based. The change did not touch this arm. The `CollisionDrawMode::Polygon` branch inside the `drag_started()` match at line 2604 is an explicit empty arm (`CollisionDrawMode::Polygon => {}`), which is correct — polygon drawing does not use drag state.

**Finding: Correct. No issue.**

#### Finding 1.3 — Select mode vertex drag init after relocation (CONFIRMED CORRECT)

The Select mode vertex drag initialization is now inside the `drag_started()` match at lines 2579–2603. It reads the polygon vertex list from `project.tilesets` (immutable borrow), performs the hit test, and if a vertex is found, writes `drag_state` to `editor_state`. The borrow is released before the write. No overlap with the mutable path. The vertex index and `original` position are captured correctly at drag start and stored in `CollisionDragOperation::MoveVertex { index, original }`.

**Finding: Correct. No issue.**

#### Finding 1.4 — Zero-size shape guards in `drag_stopped()` still sufficient (CONFIRMED CORRECT)

Rectangle: `if width > 0.01 && height > 0.01` (line 2650). Circle: `if radius > 0.01` (line 2668). These guards correctly discard the "clean tap" edge case — where the user pressed and released without moving enough to produce a measurable shape. The `drag_started()` change does not affect these guards; they operate on final values computed at release, not on the initialization path.

Advisory: the 0.01 threshold is a magic constant with no named binding. This is pre-existing and is not introduced by Wesley's change. Out of scope for this sprint.

**Finding: Sufficient. Advisory only — magic constant.**

#### Finding 1.5 — Wildcard arm in `drag_stopped()` silently discards `MoveShape` and `ResizeRect`/`ResizeCircle` (ADVISORY)

The `drag_stopped()` match at line 2632 handles `NewRectangle`, `NewCircle`, and `MoveVertex` explicitly. The wildcard `_ => {}` at line 2703 silently discards `MoveShape`, `ResizeRect`, and `ResizeCircle`. These three variants exist in the `CollisionDragOperation` enum (lines 507–514) but have no initialization code in `drag_started()` and no commit code in `drag_stopped()`. This is pre-existing — not introduced by Wesley's change. However, it means if any future code path sets one of those operations, the drag will produce no visible result and no error. The wildcard suppresses the compiler's exhaustiveness guarantee.

This is a pre-existing debt item, not a regression. Wesley's change did not introduce it. Recording it here for Worf's awareness: test coverage cannot verify `MoveShape`/`ResizeRect`/`ResizeCircle` commit paths because none exist. Worf should not write tests for these — they are unimplemented operations, not bugs.

**Finding: Advisory. Pre-existing. No action required this sprint.**

#### Change 1 Verdict: SHIP

---

### Change 2: Barclay's numeric input panel (`render_collision_properties`)

#### Summary of change

A `match &collision_data.shape` block inserted after the shape name label. Five arms: `Rectangle` (4 DragValues, dynamic max), `Circle` (3 DragValues, radius unclamped), `Polygon` (ScrollArea with per-vertex rows, delete guard, add point, deferred mutation), `Full` (label only), `None` (label only). Write-back uses `set_tile_collision_shape` + `mark_dirty` behind `any_changed` flags.

#### Finding 2.1 — Borrow checker: clone-mutate-compare pattern (CORRECT)

`collision_data` is cloned from the tileset at lines 2963–2966 before the `match`. The match borrows `&collision_data.shape` (shared reference into the clone). Local `mut` copies are made from the matched fields (`let mut offset = *orig_offset`, etc.). DragValues reference the local copies. The mutable borrow of `project.tilesets` for write-back occurs only inside `if any_changed`, which is after all DragValue calls complete. No shared reference is held across the mutable borrow. The egui closure pattern (`ui.horizontal(|ui| { ... })`) takes a closure but `offset` and `size` are captured by exclusive mutable reference within the closure; egui processes the closure immediately and returns before the next statement. There is no async or deferred execution.

**Finding: Correct. No borrow checker issue.**

#### Finding 2.2 — Exhaustive variant coverage (CORRECT)

`CollisionShape` has five variants: `None`, `Full`, `Rectangle`, `Circle`, `Polygon`. All five are explicitly matched at lines 2977, 3044, 3092, 3180, 3185. No wildcard arm. The compiler will enforce exhaustiveness if a new variant is ever added to `CollisionShape`, which is the correct behavior.

**Finding: Correct. Full coverage.**

#### Finding 2.3 — Rectangle dynamic width/height max computation (CORRECT WITH ADVISORY)

`max_width = (1.0 - offset[0]).max(0.0)` and `max_height = (1.0 - offset[1]).max(0.0)` are computed after the offset DragValues are rendered (lines 3007–3008). The comment at line 2989 acknowledges this ordering explicitly: offset fields rendered first so the max reflects any edit made this frame. This is correct. If the user drags offset X to 0.8 on this frame, `max_width` becomes 0.2, and width is clamped to 0.2 before the width DragValue is rendered.

`size[0] = size[0].min(max_width)` and `size[1] = size[1].min(max_height)` (lines 3009–3010) apply the clamp to the local copy before rendering. This means if offset is dragged rightward past the current width, the width is silently reduced to fit. The user receives no explicit warning that their width value was altered. This is a UX observation, not a correctness bug — the resulting shape is always geometrically valid (right edge never exceeds 1.0), and the DragValue widget will reflect the clamped value on the next frame.

Advisory: there is a one-frame latency on the clamp. On the frame the offset is pushed far right, the `any_changed` flag will be `true` (from the offset edit), the write-back will fire, and the clamped size will be persisted. The displayed width DragValue will show the clamped value. The behavior is consistent and visible to the user. Acceptable.

**Finding: Correct. Advisory on silent clamp.**

#### Finding 2.4 — Circle radius range: `0.0..=f32::MAX` (CORRECT, DELIBERATE)

Radius is not clamped to 1.0 (lines 3074–3078). The comment at line 3068–3069 references this as Data's assessed item. `CollisionShape::Circle` allows radius to extend past the tile boundary — this is a valid physics configuration (e.g., a circular bumper that overlaps adjacent tiles). The lower bound of 0.0 prevents a negative radius. `f32::MAX` as upper bound is acceptable; no practical collision radius will approach it, and DragValue with speed 0.005 will not reach it by accident.

**Finding: Correct and deliberate.**

#### Finding 2.5 — Polygon scroll area and iteration (CORRECT)

The scroll area uses `id_salt("collision_polygon_points")` (line 3108), which is required when multiple scroll areas exist in the same panel. The iteration is `for i in 0..points.len()` over the cloned vector. Delete and add are deferred via `delete_idx: Option<usize>` and `add_point: bool` flags set inside the loop, applied after the scroll area closes. This is the correct pattern for deferred mutation.

**Finding: Correct.**

#### Finding 2.6 — Delete guard: `points.len() > 3` (CORRECT)

The button is disabled (`add_enabled`) when `points.len() <= 3` (line 3133). A second guard at line 3155 checks the same condition before the actual `remove()`. The second guard is described as a defense against external data putting the polygon in a degenerate state. This is correct defensive coding. A polygon with fewer than 3 points is not a valid convex polygon.

One edge: if a tile's saved data contains a Polygon with 2 or 1 points (malformed persistence), the delete button will be permanently disabled and the user cannot reduce it further — but cannot increase it below 3 either until they add a point. The add button is always enabled (no guard), so they can escape this state by adding a point. The escape path exists.

**Finding: Correct. Edge case handled.**

#### Finding 2.7 — Add point appends `[0.5, 0.5]` (ADVISORY)

When "+ Add Point" is clicked, the new point is appended at `[0.5, 0.5]` (line 3161). This is the center of the tile. For a polygon that already covers the interior (e.g., a square from [0,0] to [1,1]), appending a center point will produce a degenerate concave polygon. The user will need to drag the point to a useful position via either the canvas or the numeric input. There is no path to insert at a specific index from the properties panel — only the canvas double-click (Select mode) performs a `find_best_insertion_index` insertion.

This is a UX limitation, not a correctness bug. The resulting data is valid; it may just be inconvenient to position. Out of scope for this sprint.

**Finding: Advisory. Not a correctness issue.**

#### Finding 2.8 — `len_changed` check is redundant but harmless (ADVISORY)

At line 3168, `let len_changed = points.len() != original_len`. This is computed and OR'd with `any_changed` at line 3169. However, `any_changed` is already set to `true` in the delete and add paths (lines 3157, 3162) before `len_changed` would detect anything. The variable `len_changed` can never be `true` while `any_changed` is `false` given the current code structure. It is dead logic as currently written.

This is not a bug — the write-back fires correctly in all cases. It is slightly misleading code: a reader might believe `len_changed` covers a case `any_changed` does not. It does not.

**Finding: Advisory. Dead logic, no behavioral impact.**

#### Finding 2.9 — Write-back fires on every DragValue interaction, not only on confirmed change (CORRECT)

`any_changed` is set by `ui.add(...).changed()`, which is the standard egui pattern for detecting value change. `changed()` returns `true` only when the value actually changed this frame. Write-back occurs once per changed frame, which is correct for immediate-mode live update. No spurious writes.

**Finding: Correct.**

#### Change 2 Verdict: SHIP

---

### Overall Verdict: GO for Worf

Both changes are correct. No blocking issues. Advisory findings are recorded above — none require code changes before testing.

**Worf should target the following test cases based on the findings above:**

1. Rectangle drag: drag starts and `drag_state` is `Some(NewRectangle)` after `drag_started`, `None` before.
2. Circle drag: same structure as rectangle.
3. Clean tap on Rectangle/Circle canvas: `drag_state` remains `None` after press+release without threshold exceeded; no shape committed.
4. Polygon click: point appended to `polygon_points` on click, no drag state set.
5. Select mode vertex drag: `drag_state` is `Some(MoveVertex { index, original })` after `drag_started` when pointer is within 8px of a vertex.
6. Select mode drag on non-vertex: `drag_state` is `None` after `drag_started` when pointer is not near any vertex.
7. Rectangle numeric input: offset clamped 0–1, width+offset[0] never exceeds 1.0, write-back fires on change.
8. Circle numeric input: radius accepts values > 1.0.
9. Polygon numeric input: delete disabled at exactly 3 points, enabled at 4+; add appends [0.5, 0.5].
10. Polygon with 2 points in data (malformed persistence): delete disabled, add enabled — escape path exists.

**Pre-existing debt noted (not blocking this sprint):**
- Wildcard `_ => {}` in `drag_stopped()` silently discards `MoveShape`, `ResizeRect`, `ResizeCircle` operations. These are unimplemented, not bugs. Recorded in DEBT section below.
- `0.01` drag threshold magic constant in `drag_stopped()`.

---

## Automapping Sprint — Post-T-02 Decision Record

**Author:** Lt. Cmdr. Data
**Date:** 2026-02-26
**Status:** All five user decisions resolved. GO issued to all three SE tracks.

This section supplements the "Automapping Sprint — Architecture Notes" section above. That section contains the original design. This section records the five decisions made at T-02 and their precise architectural implications, answers the two Troi technical questions, and issues GO to each SE track.

---

### Decision 1: Layer UUID — Approved

**Decision:** Proceed as designed. `Layer` gains `pub id: Uuid` with `#[serde(default = "Uuid::new_v4")]`. Old files load cleanly; new UUID assigned on deserialization, stable on next save. One-way format migration accepted.

**Architectural implication:** No change to the design already recorded in the Architecture Notes section. Barclay proceeds.

---

### Decision 2: Orphan Reference Behavior on Layer Delete — Warning Dialog with Persistent Preference

**Decision:** When a layer referenced by automap rules is deleted, the editor shows a warning dialog. The dialog includes a "Do not ask me again" checkbox. The user's preference for this checkbox must be persisted.

**Persistence location:** Editor preferences (not project file). The preference is user-global, not project-specific. It follows the existing pattern for editor preferences stored in `EditorState` or a companion struct.

**Architectural implication for Barclay:**

The warning dialog is triggered from `validate_automap_config` — or more precisely, from the layer-delete path in `process_edit_actions`. The sequence is:

1. User triggers layer delete.
2. Before executing the delete, the system checks whether any `InputConditionGroup.layer_id` or `OutputAlternative.layer_id` in `project.automap_config` references the layer being deleted.
3. If references exist AND the user preference is "ask me" (i.e., `suppress_orphan_warning` is `false`): show the warning dialog with the list of affected rule set/rule names and the "Do not ask me again" checkbox.
4. If the user confirms (or if `suppress_orphan_warning` is `true`): proceed with delete. `validate_automap_config` then cleans up the orphaned references.
5. If the user cancels: abort the delete.

**New field required on `EditorState` (or `EditorPreferences` if that struct exists):**

```rust
/// If true, skip the orphan-reference warning when deleting a layer that automap rules reference.
pub suppress_automap_orphan_warning: bool,
```

Default: `false`. When the user checks "Do not ask me again" and confirms, this is set to `true` and persisted.

**Persistence mechanism:** Barclay must determine where editor preferences are persisted in this codebase (likely `EditorState` serialized to a prefs file, or a dedicated prefs struct). Before writing code, Barclay must bring this back to Data with one sentence: "Editor preferences are persisted at [location] using [mechanism]." If no persistence mechanism exists, escalate to Data before adding one.

**Warning dialog content:** Troi owns the exact wording. Barclay implements the dialog plumbing; Troi confirms the copy before Barclay ships. The dialog must list: how many rule sets are affected, how many rules reference the layer. It need not list every rule name — a count is sufficient per Troi's design authority, unless Troi specifies otherwise.

---

### Decision 3: Flip-Aware Matching — TileFlipped CellMatcher Variant

**Decision:** Flipped tiles are NOT equivalent to non-flipped tiles. The engine must support flip-aware matching. The `TileFlipped` variant is added to `CellMatcher` now. The UI does not expose it this sprint. The data model and matching algorithm must support it.

**Representation:** Three fields — `id: u32`, `flip_x: bool`, `flip_y: bool` — stored as named fields, not packed. Rationale: packing into a u32 saves 1 byte per matcher at the cost of non-obvious decode logic and a serde representation that requires custom handling. Named fields are self-describing in the JSON/RON serialized form, readable by external tools, and require no custom serde. The storage cost is 4 + 1 + 1 = 6 bytes per matcher vs. 5 bytes packed — the delta is negligible at any practical rule count.

**Revised `CellMatcher` enum:**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellMatcher {
    Ignore,
    Empty,
    NonEmpty,
    /// Match a specific tile index, regardless of flip state (flip bits stripped before compare).
    Tile(u32),
    /// Match any tile except the specified index, regardless of flip state.
    NotTile(u32),
    /// Match a specific tile index with exact flip state.
    TileFlipped { id: u32, flip_x: bool, flip_y: bool },
    /// Matches any tile whose index is not listed as Tile/NotTile/TileFlipped in this rule.
    Other,
}
```

**Algorithm change for `matcher_matches`:** The existing `Tile(id)` strips flip bits before comparison. `TileFlipped { id, flip_x, flip_y }` must compare both the tile index (after stripping flip bits) AND the actual flip bits stored in the cell value. Geordi must define the flip bit extraction function — it depends on how flip bits are encoded in the tile value `u32`. He must bring the exact bit layout to Data before writing `matcher_matches`.

**`explicit_tiles` set for `Other` matching:** `TileFlipped` variants contribute their `id` to `explicit_tiles`, the same as `Tile(id)`. The `Other` matcher means "any tile whose base index is not in explicit_tiles." Flip state does not affect explicit_tiles membership.

**Serde compatibility:** `TileFlipped` as a named-fields variant serializes as `{"TileFlipped":{"id":N,"flip_x":true,"flip_y":false}}` in JSON. This is the serde default for enum variants with named fields. No custom serde needed.

---

### Decision 4: "Until Stable" Apply Mode — In Scope with Iteration Cap

**Decision:** "Until Stable" is in scope this sprint. Add it to `RuleSetSettings::apply_mode`. The algorithm must detect cycles. A cap on iterations is required.

**`RuleSetSettings` revised:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetSettings {
    pub edge_handling: EdgeHandling,
    pub apply_mode: ApplyMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ApplyMode {
    #[default]
    Once,
    UntilStable,
}
```

**Loop detection approach:** iteration cap. The cap value is **100 iterations**, hard-coded as a named constant. Rationale: a rule set that does not converge in 100 full passes over the map is almost certainly cycling. 100 passes on a 256x256 map with 10 rules is 100 × 256 × 256 × 10 = 65,536,000 cell evaluations — measurable lag, but not a hang. The cap is sufficient to catch all practical cycling configurations while not hiding pathological rule sets entirely. If the cap is reached, the function returns with the current state and logs a warning (to `eprintln!` or Bevy's `warn!` macro if available in `bevy_map_automap` — Geordi decides based on whether the crate takes a Bevy dep, which it does not; use `eprintln!` with a structured prefix).

**Named constant:**

```rust
/// Maximum number of full-pass iterations in UntilStable mode before aborting.
/// A rule set that does not converge within this many passes is assumed to be cycling.
pub const UNTIL_STABLE_MAX_ITERATIONS: u32 = 100;
```

**The cap is NOT user-configurable this sprint.** Adding a "Max iterations" field to `RuleSetSettings` is deferred. The constant is defined at crate level in `bevy_map_automap` so it is findable and changeable without a data format change if the limit proves wrong in practice.

**Algorithm change for `apply_ruleset`:**

```
apply_ruleset(level, rule_set, rng):
    match rule_set.settings.apply_mode:
        Once => apply_ruleset_once(level, rule_set, rng)
        UntilStable =>
            for iteration in 0..UNTIL_STABLE_MAX_ITERATIONS:
                snapshot = snapshot_relevant_layers(level, rule_set)
                apply_ruleset_once(level, rule_set, rng)
                if level_matches_snapshot(level, snapshot, rule_set):
                    return  // stable
            eprintln!("[automap] rule set '{}' did not converge after {} iterations",
                      rule_set.name, UNTIL_STABLE_MAX_ITERATIONS)
            // return with current state; do not panic
```

`snapshot_relevant_layers` snapshots only the layers referenced in the rule set's rules (not all layers), to minimize memory allocation per iteration. Geordi defines the snapshot representation — a `HashMap<Uuid, Vec<Option<u32>>>` (layer_id → flat cell array) is the straightforward choice.

**Stable detection:** The map is stable when no cell in any referenced layer changed during the last pass. Comparing the pre-pass snapshot to the post-pass state is the definition. Note: with probabilistic output alternatives, a rule set may never be stable if the weights allow different outputs on each pass. The cap correctly handles this — it will always terminate. Geordi must note this in a doc comment on `UNTIL_STABLE_MAX_ITERATIONS`.

---

### Decision 5: Rule Reordering — Up/Down Buttons (Final)

**Decision:** Up/Down arrow buttons for both rule sets and rules. This is Troi's final design. Drag-and-drop deferred. No architectural implication beyond what is already specified in Troi's UX spec section 4 and 5. No SE action needed — this was already resolved in ESCALATE-04 (CLOSED in the spec).

---

### Troi's Technical Questions — Answered

#### ESCALATE-07: Data Model Location (for UI SE and Troi)

These answers apply to the UI SE implementing `automap_editor.rs` and to any test code Worf writes.

**1. Where do `RuleSet`, `Rule`, and related types live?**

All types — `AutomapConfig`, `RuleSet`, `RuleSetSettings`, `ApplyMode`, `EdgeHandling`, `Rule`, `InputConditionGroup`, `CellMatcher`, `CellOutput`, `OutputAlternative` — live in:

```
crates/bevy_map_automap/src/types.rs
```

The crate is `bevy_map_automap`. The UI crate (`bevy_map_editor`) adds `bevy_map_automap` as a dependency. The UI SE imports these types directly:

```rust
use bevy_map_automap::{AutomapConfig, RuleSet, Rule, CellMatcher, /* ... */};
```

**2. Does `project.automap_config` exist as a named field on `Project`, and what is its type?**

Yes. `Project` in `crates/bevy_map_editor/src/project/mod.rs` gains:

```rust
#[serde(default)]
pub automap_config: AutomapConfig,
```

`AutomapConfig` is `bevy_map_automap::AutomapConfig`. This is the same pattern as `autotile_config`. The field is public. The UI SE accesses it as `project.automap_config.rule_sets`.

**3. Is `AutomapCommand::execute()` a pure function over `&mut Project`?**

Yes. `execute()` and `undo()` take `&mut Project` and nothing else. They apply tile changes directly to `project.levels[level_index].layers[layer_index].tiles`. No Bevy API calls inside these methods. The wrapping system in `process_edit_actions` handles the snapshot, diff, and push to `CommandHistory`.

**4. What is the `AutomapCommand` constructor signature?**

```rust
impl AutomapCommand {
    pub fn new(
        level_id: Uuid,
        layer_changes: HashMap<usize, HashMap<(u32, u32), (Option<u32>, Option<u32>)>>,
        description: String,
    ) -> Self
}
```

`layer_changes` maps `layer_index` (usize, not UUID) to a map of `(col, row) → (old_tile, new_tile)`. The layer index is resolved at snapshot time in the wrapping system. The UI SE does not construct `AutomapCommand` directly — that is done in `process_edit_actions` after calling `apply_automap_config`.

---

#### ESCALATE-02: Three-Column Fixed-Width Layout in egui (for UI SE)

**Decision:** Option A — `ui.allocate_ui_with_layout` with explicit width, placed inside a `ui.horizontal` block.

**Rationale:** `SidePanel` (Option B) is a window-level concept; using it inside an `egui::Window` produces confusing nested panel behavior and interferes with the window's own resize logic. `Frame` with `min_rect` constraints (Option C) does not enforce width; it only sets a minimum. `allocate_ui_with_layout` is the correct egui primitive for claiming a fixed horizontal extent within a horizontal layout.

**The pattern the UI SE must use:**

```rust
// Inside the automap editor window UI closure:
ui.horizontal(|ui| {
    // Column 1: Rule Sets — fixed 180px
    ui.allocate_ui_with_layout(
        egui::vec2(180.0, ui.available_height()),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            render_rule_set_column(ui, /* ... */);
        },
    );

    ui.separator();

    // Column 2: Rules — fixed 220px
    ui.allocate_ui_with_layout(
        egui::vec2(220.0, ui.available_height()),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            render_rule_column(ui, /* ... */);
        },
    );

    ui.separator();

    // Column 3: Pattern Editor — fills remaining width
    // No allocate_ui_with_layout needed; just render into the remaining ui space.
    render_pattern_editor_column(ui, /* ... */);
});
```

**Key constraint:** `ui.available_height()` inside the `horizontal` block returns the height of the outer container. Each column's height is set to this value so the separators and backgrounds extend to the full panel height. The UI SE must call `ui.available_height()` before the first `allocate_ui_with_layout` and pass the same value to all three columns. If called inside each closure, the available height may change as widgets are added. Capture it once:

```rust
let col_height = ui.available_height();
ui.horizontal(|ui| {
    ui.allocate_ui_with_layout(egui::vec2(180.0, col_height), /* ... */);
    ui.separator();
    ui.allocate_ui_with_layout(egui::vec2(220.0, col_height), /* ... */);
    ui.separator();
    render_pattern_editor_column(ui, /* ... */);
});
```

This pattern is the authoritative approach. The UI SE must not deviate without first bringing a specific technical objection back to Data.

---

### SE Track GO Orders

---

#### Track A — Geordi (Engine: `bevy_map_automap` crate)

**Branch:** `sprint/automapping/geordi-engine`

**GO. Geordi may begin implementation.**

**What Geordi must implement:**

1. **`crates/bevy_map_automap/Cargo.toml`** — crate manifest. Dependencies: `bevy_map_core`, `serde` (with `derive` feature), `uuid` (with `v4` + `serde` features), `rand`. No Bevy dependency. No `bevy` in the dependency tree.

2. **`crates/bevy_map_automap/src/lib.rs`** — crate root. Re-exports all public types from `types.rs` and the `apply_automap_config` function from `apply.rs`.

3. **`crates/bevy_map_automap/src/types.rs`** — all data types as specified in the Architecture Notes section, PLUS the following additions from this decision record:
   - `RuleSetSettings` must include `apply_mode: ApplyMode`
   - `ApplyMode` enum with variants `Once` and `UntilStable`, `Once` as default
   - `CellMatcher::TileFlipped { id: u32, flip_x: bool, flip_y: bool }` variant added
   - `UNTIL_STABLE_MAX_ITERATIONS: u32 = 100` constant at crate root level

4. **`crates/bevy_map_automap/src/apply.rs`** — full algorithm implementation:
   - `apply_automap_config(level: &mut Level, config: &AutomapConfig, rng: &mut impl Rng)`
   - `apply_ruleset(level: &mut Level, rule_set: &RuleSet, rng: &mut impl Rng)`
   - `apply_ruleset_once(level: &mut Level, rule_set: &RuleSet, rng: &mut impl Rng)` — inner helper
   - `matcher_matches(matcher: &CellMatcher, cell: Option<u32>, edge_handling: EdgeHandling, explicit_tiles: &HashSet<u32>) -> bool`
   - `select_output_alternative<R: Rng>(alts: &[OutputAlternative], rng: &mut R) -> Option<&OutputAlternative>`
   - `UntilStable` loop using `UNTIL_STABLE_MAX_ITERATIONS`, snapshot-compare approach, `eprintln!` warning on cap reached
   - Full edge case handling as specified in the Architecture Notes section

**API proposals Geordi must bring back to Data before writing implementation code:**

1. **Flip bit extraction:** The `TileFlipped` matcher requires extracting `flip_x` and `flip_y` from a `u32` cell value. Geordi must identify exactly which bits encode flip state in `bevy_map_core`'s tile representation, and propose the extraction function signature and bit mask values. Do not assume — read the source. Bring: function name, bit positions, and whether this function belongs in `bevy_map_core` or in `apply.rs`.

2. **`Level` API surface:** `apply_automap_config` takes `&mut Level`. Geordi must confirm which methods or fields on `Level` are used for cell read and write (e.g., `level.get_tile(layer_index, col, row) -> Option<u32>` and `level.set_tile(layer_index, col, row, Option<u32>)`). If these methods do not exist and Geordi must access fields directly, he must name the exact field path. Bring the confirmed read/write path before writing `apply_ruleset_once`.

3. **`UntilStable` snapshot representation:** Propose the type for the pre-pass snapshot used in stable detection. Preferred: `HashMap<usize, Vec<Option<u32>>>` (layer_index → flat cell array, row-major). Confirm that layer indices are stable within a single `apply_ruleset` call (i.e., no layer reallocation occurs during the call). Bring this confirmation before implementing the `UntilStable` branch.

---

#### Track B — Barclay (Integration: `bevy_map_core`, `project/`, `commands/`, warning dialog)

**Branch:** `sprint/automapping/barclay-integration`

**GO. Barclay may begin implementation.**

**What Barclay must implement:**

1. **`crates/bevy_map_core/src/layer.rs`** — add `pub id: Uuid` to `Layer` struct with `#[serde(default = "Uuid::new_v4")]`. Confirm that `Uuid::new_v4` is accessible as a bare function path (required by serde's `default` attribute). If not, a wrapper `fn new_layer_id() -> Uuid { Uuid::new_v4() }` is the standard workaround. Add `uuid` to `bevy_map_core`'s `Cargo.toml` dependencies with `v4` and `serde` features.

2. **`crates/bevy_map_editor/src/project/mod.rs`** — add `#[serde(default)] pub automap_config: AutomapConfig` to `Project`. Add `bevy_map_automap` to `bevy_map_editor`'s `Cargo.toml` dependencies. Implement `validate_automap_config(&mut self)` on `Project` (or as a free function called from `validate_and_cleanup`) that removes `InputConditionGroup` and `OutputAlternative` entries whose `layer_id` does not match any `id` in any layer of any level. The removal is silent — no UI side effects from within this function.

3. **`crates/bevy_map_editor/src/commands/command.rs`** — implement `AutomapCommand` as specified in the Architecture Notes section. Implement `Command` trait (`execute`, `undo`, `description`). The `layer_changes` map uses layer index (usize), not UUID — the index is resolved at snapshot time by the caller. Confirm that `BatchTileCommand` exists as a reference pattern before writing.

4. **`crates/bevy_map_editor/src/preferences/mod.rs`** — add `pub suppress_automap_orphan_warning: bool` to `EditorPreferences`, defaulting to `false`. **NOT `EditorState`.** `EditorState` is not serialized and does not persist across sessions. `EditorPreferences` derives `Serialize`/`Deserialize` and has a working `save()` method. This is the correct home for a user-global "do not ask me again" preference. Update `EditorPreferences::default()` accordingly.

   *(Sprint log correction: earlier versions of this entry placed this field on `EditorState`. That was incorrect and has been corrected here. Barclay's Proposal 3 analysis confirmed `EditorPreferences` is the right location.)*

5. **Layer-delete orphan warning hook** — in `ui/mod.rs` at the existing layer delete call site (line 1103, `tree_view_result.delete_layer`), add the following before executing the delete:
   - Check `project.automap_config` for any rule that references the layer being deleted, using `count_automap_orphan_refs(config, layer_id) -> (usize, usize)` (free function, not a method on `AutomapConfig` — see placement note below).
   - If references exist and `preferences.suppress_automap_orphan_warning` is `false`: set `editor_state.pending_action` to the new variant `PendingAction::ConfirmLayerDeleteWithOrphanWarning { layer_id: Uuid, layer_idx: usize, level_id: Uuid, affected_rule_set_count: usize, affected_rule_count: usize }`. This causes the main loop to render the warning dialog on the next frame.
   - The warning dialog is rendered in `render_dialogs` (in `ui/dialogs.rs`). `render_dialogs` must be extended with a `preferences: &mut EditorPreferences` parameter. The dialog shows the counts, a "Do not ask me again" checkbox bound to `preferences.suppress_automap_orphan_warning`, and Confirm/Cancel buttons. On Confirm: extract `layer_id`, `layer_idx`, `level_id` from the variant, execute the delete, call `preferences.save()` if the checkbox was checked, then call `validate_automap_config`. On Cancel: do nothing.
   - If `preferences.suppress_automap_orphan_warning` is `true`: skip the dialog and proceed directly with delete + `validate_automap_config`.

   **`count_automap_orphan_refs` placement:** This function takes `&AutomapConfig` and returns `(rule_set_count, rule_count)`. It must NOT be a method on `AutomapConfig` (that would couple the data type to editor-delete semantics). It must NOT live in `ui/mod.rs` (UI files should not own data-query logic). Place it in the same module as `validate_automap_config`. Barclay must confirm where `validate_automap_config` is implemented and place the count function there.

6. **Persistence of `suppress_automap_orphan_warning`:** Before writing any persistence code, Barclay must bring one sentence to Data: "Editor preferences are persisted at [location] using [mechanism]." If no editor-preference persistence mechanism exists yet, do not invent one — escalate to Data.

**API proposals Barclay must bring back to Data before writing implementation code:**

1. **`PendingAction::ConfirmLayerDeleteWithOrphanWarning` placement:** Barclay must confirm that `PendingAction` in `ui/dialogs.rs` is the correct enum to extend (not a separate dialog queue). Bring the current `PendingAction` variant list and confirm the naming convention matches existing variants before adding.

2. **Layer delete path:** Confirm where layer deletion is currently handled. Is it in `process_edit_actions` responding to an existing `PendingAction` variant, or in direct UI code? Bring the exact call site before adding the orphan-check hook.

3. **Editor preference persistence:** As noted above — confirm the mechanism before touching it.

---

#### Track C — UI SE (Rule Editor Panel: `bevy_map_editor`)

**Recommended persona: Wesley Crusher.**

Rationale: The UX spec is fully written, complete, and detailed. The column layout approach is now decided. The data types are fully specified. The accessibility requirements are enumerated. This is a well-defined spec with a large but clear surface area. Wesley's strength — clean, pattern-adherent, fast output — fits this track. The complexity is in breadth (many widgets, many states) not in depth (no novel algorithmic problems). Barclay's edge-case focus is not needed here because Troi's spec has already pre-enumerated the edge cases (empty states, disabled states, boundary conditions on grid size). The pattern editor grid is the most complex component but is structurally repetitive — exactly Wesley's wheelhouse.

**Branch:** `sprint/automapping/wesley-ui`

**Prerequisite for Wesley:** Barclay's types (`AutomapConfig`, `RuleSet`, `Rule`, etc. via `bevy_map_automap`) must exist as a crate that compiles before Wesley can `cargo check` his UI code. Sequencing constraint: Wesley must not begin writing code that imports `bevy_map_automap` types until Geordi has pushed the types skeleton (even if `apply.rs` is incomplete). Geordi should push a types-only commit on `geordi-engine` as soon as `types.rs` compiles, so Wesley can unblock. Wesley may write the file structure and module skeleton in the interim without the import.

**GO is conditional on Geordi having a compiling `types.rs` pushed. Wesley may begin the non-type-dependent scaffolding immediately.**

**What Wesley must implement:**

1. **`crates/bevy_map_editor/src/ui/automap_editor.rs`** — new file. The complete automap rule editor panel. This is the primary deliverable.

2. **`crates/bevy_map_editor/src/lib.rs`** — add `pub show_automap_editor: bool` to `EditorState`. Default `false`.

3. **`crates/bevy_map_editor/src/ui/dialogs.rs`** — add `RunAutomapRules` variant to `PendingAction`.

4. **`crates/bevy_map_editor/src/ui/mod.rs`** — wire `PendingAction::RunAutomapRules` in `process_edit_actions`: snapshot all layers in the target level, call `apply_automap_config`, diff, construct `AutomapCommand` if non-empty, push to history, set `automap_editor_state.last_run_status`. Call `render_automap_editor` from the main UI render path when `editor_state.show_automap_editor`.

5. **`crates/bevy_map_editor/src/ui/menu_bar.rs`** — add "Automap Rule Editor..." menu item in the Tools menu per the spec (section 14 of `automap_ux_spec.md`).

6. **`crates/bevy_map_editor/src/commands/shortcuts.rs`** — add `Ctrl+Shift+A` shortcut that toggles `editor_state.show_automap_editor`.

7. **State types** — `AutomapEditorState`, `AutomapEditorTab`, `InputBrushType`, `OutputBrushType` as specified in section 15 of `automap_ux_spec.md`. Add `pub automap_editor_state: AutomapEditorState` to `EditorState`.

**Layout implementation in `automap_editor.rs`:**

Use the column layout pattern decided in ESCALATE-02 above:

```rust
let col_height = ui.available_height();
ui.horizontal(|ui| {
    ui.allocate_ui_with_layout(egui::vec2(180.0, col_height), egui::Layout::top_down(egui::Align::LEFT), |ui| {
        render_rule_set_column(ui, /* ... */);
    });
    ui.separator();
    ui.allocate_ui_with_layout(egui::vec2(220.0, col_height), egui::Layout::top_down(egui::Align::LEFT), |ui| {
        render_rule_column(ui, /* ... */);
    });
    ui.separator();
    render_pattern_editor_column(ui, /* ... */);
});
```

Split the render into private helper functions. Suggested decomposition:
- `render_automap_editor(ctx, editor_state, project)` — top-level, called from `ui/mod.rs`
- `render_automap_toolbar(ui, editor_state, project)` — top strip (Run Rules, Auto on Draw, Level selector)
- `render_rule_set_column(ui, editor_state, project)` — column 1
- `render_rule_column(ui, editor_state, project)` — column 2
- `render_pattern_editor_column(ui, editor_state, project)` — column 3
- `render_input_pattern_tab(ui, editor_state, rule)` — inside column 3, Input tab
- `render_output_patterns_tab(ui, editor_state, rule)` — inside column 3, Output tab
- `render_layer_mapping_strip(ui, editor_state, project)` — bottom strip

These are private functions within `automap_editor.rs`. The names are Wesley's to choose if he has a strong reason — but he must propose deviations to Data before implementing.

**Accessibility requirements:** Wesley must follow Troi's spec section 12 exactly. Every interactive widget must have an accessible label. Grid cells must use accessible labels of the form "Input cell row N col M" and "Output cell row N col M alt K". ComboBox widgets must use `ComboBox::from_label(...)`. All icon buttons (`[^]`, `[v]`, `[x]`, `[-]`, `[+]`) must have `.on_hover_text(...)` tooltips as specified in section 12.

**Grid keyboard navigation:** Arrow key navigation within the grid requires tracked focus state. Wesley must add `pub focused_input_cell: Option<(usize, usize)>` and `pub focused_output_cell: Option<(usize, usize, usize)>` (row, col, alt_index) to `AutomapEditorState`. Key press handling is via `ui.input(|i| i.key_pressed(...))` inside the grid loop.

**"Other" brush type:** Omit from the initial implementation per ESCALATE-05. Input brush types: Ignore, Empty, NonEmpty, Tile, NotTile only.

**Grid dimensions:** Odd numbers only (1, 3, 5, 7, 9) per ESCALATE-06. The `[-]` and `[+]` increment buttons must skip even sizes.

**"Until Stable" apply mode:** The ComboBox in RuleSetSettings must include "Until Stable" as an option (Decision 4 confirmed it is in scope). The display string is "Until Stable".

**Auto on Draw:** May be deferred to a follow-up task per ESCALATE-01. Wesley should render the toggle button as specified but may stub the hook if the implementation complexity blocks other parts of the UI. Bring a specific blocker to Data if deferred.

**API proposals Wesley must bring back to Data before writing implementation code:**

1. **`render_automap_editor` call site in `ui/mod.rs`:** Wesley must confirm where in the main UI render the automap editor window is called. Is it analogous to how `render_tileset_editor` is called? Bring the exact call site (function name, surrounding context) before adding the call.

2. **`process_edit_actions` for `RunAutomapRules`:** Wesley must propose the exact sequence of operations for processing `RunAutomapRules`: which `Level` is accessed, how the snapshot is taken, what happens if no target level is selected. Bring this as a pseudocode proposal before writing the implementation.

3. **Borrow checker strategy for `automap_editor.rs`:** The automap editor mutates both `editor_state` (for selection indices, brush state, tab) and `project` (for rule set names, rule contents, grid cells). Wesley must confirm whether he needs the `macro_rules!` re-borrow pattern used elsewhere in the codebase for nested mutations across egui closures, or whether a simpler approach (clone-mutate-write) suffices for the rule/cell edit paths. Bring the proposed borrow strategy before writing the column render functions.

---

### Parallel Safety Confirmation

Three tracks are running simultaneously:

| Track | SE | Files touched |
|---|---|---|
| A — Engine | Geordi | `crates/bevy_map_automap/` (new crate, all files) |
| B — Integration | Barclay | `bevy_map_core/src/layer.rs`, `bevy_map_editor/src/project/mod.rs`, `bevy_map_editor/src/commands/command.rs`, `bevy_map_editor/src/lib.rs`, `bevy_map_editor/src/ui/dialogs.rs` |
| C — UI | Wesley | `bevy_map_editor/src/ui/automap_editor.rs` (new), `bevy_map_editor/src/ui/mod.rs`, `bevy_map_editor/src/ui/menu_bar.rs`, `bevy_map_editor/src/commands/shortcuts.rs`, `bevy_map_editor/src/lib.rs`, `bevy_map_editor/src/ui/dialogs.rs` |

**File conflict: `bevy_map_editor/src/lib.rs`** — Both Barclay and Wesley add fields to `EditorState` here. They must not edit this file simultaneously. Sequencing: Barclay adds `suppress_automap_orphan_warning` first (it is part of the integration track which has no dependency on Wesley's work). Wesley adds `show_automap_editor` and `automap_editor_state` after Barclay's changes are on the integration branch and rebased. Wesley waits for Barclay's `lib.rs` commit before touching `lib.rs` himself.

**File conflict: `bevy_map_editor/src/ui/dialogs.rs`** — Both Barclay and Wesley extend `PendingAction`. Barclay adds `ConfirmLayerDeleteWithOrphanWarning`. Wesley adds `RunAutomapRules`. Same sequencing constraint: Barclay first, Wesley after Barclay's `dialogs.rs` commit is available.

**All other files are disjoint. Geordi's crate is entirely new. No other overlap.**

---

### Session Status

**Date:** 2026-02-26

**Decisions made this session:**
- All five T-02 user decisions incorporated.
- `TileFlipped` CellMatcher variant designed (named fields, three-field representation).
- `UntilStable` apply mode designed: `UNTIL_STABLE_MAX_ITERATIONS = 100`, not configurable this sprint, `eprintln!` warning on cap.
- Orphan warning dialog: `suppress_automap_orphan_warning` field on **`EditorPreferences`** (not `EditorState`; corrected after Barclay's Proposal 3 analysis confirmed `EditorPreferences` is serialized and has `save()`). `EditorState` is not persisted across sessions.
- Column layout: `allocate_ui_with_layout` pattern, `col_height` captured before `horizontal` block.
- ESCALATE-07 data model questions answered for Troi and UI SE.

**GO issued to:** Geordi (engine), Barclay (integration), Wesley (UI — conditional on Geordi types compiling).

**API proposals outstanding (must return to Data before implementation):**
- Geordi: ~~flip bit layout, `Level` read/write API, `UntilStable` snapshot type~~ **Retroactive review complete (2026-02-27). `types.rs` and `apply.rs` approved with one required correction: remove dead `first_alt` binding in `no_overlapping_output` block (apply.rs lines 170–178). See correction details above.**
- Barclay: ~~`PendingAction` placement, layer-delete call site, editor preference persistence mechanism~~ **All three proposals reviewed and resolved by Data (2026-02-27). See decisions above.**
- Wesley: `render_automap_editor` call site, `RunAutomapRules` processing sequence, borrow strategy for column render functions

**Next action:** Each SE reads this document and the relevant spec, submits API proposals to Data, then awaits Data's response before writing implementation code.
- `format!("{:?}", one_way)` in ComboBox exposes Rust debug format (pre-existing, noted in prior session).
