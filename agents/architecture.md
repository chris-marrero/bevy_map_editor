# bevy_map_editor — Architecture Reference

Maintained by the Sr Software Engineer. Updated whenever a significant pattern is introduced or changed. A fresh agent instance should be able to orient themselves from this document alone.

---

## Stack

| Component | Version |
|---|---|
| Bevy | 0.18 |
| bevy_egui | 0.39.0 |
| egui (resolved) | 0.33.3 |
| egui-async | 0.3.4 |
| Rust edition | 2021 |
| MSRV | 1.76.0 |

---

## Crate Layout

```
crates/
  bevy_map_core/        — shared types (tilesets, levels, entities, schema)
  bevy_map_autotile/    — Wang tile / 47-tile blob autotiling logic
  bevy_map_automap/     — rule-based automapping
  bevy_map_animation/   — animation data types
  bevy_map_dialogue/    — dialogue tree data
  bevy_map_schema/      — runtime data schema
  bevy_map_codegen/     — code generation from schema
  bevy_map_runtime/     — optional runtime crate (feature-gated)
  bevy_map_integration/ — plugin/extension API for third-party game projects
  bevy_map_editor/      — the editor binary and UI crate (this is the primary concern)
```

---

## Editor State Model

### EditorState (Resource)

`EditorState` is the primary mutable resource. It lives in `crates/bevy_map_editor/src/lib.rs` (line 518). It carries:

- Tool selection: `current_tool: EditorTool`, `tool_mode: ToolMode`
- View state: `view_mode: EditorViewMode`, `zoom`, `camera_offset`, `show_grid`, `show_collisions`
- Dialog visibility flags: `show_new_level_dialog`, `show_settings_dialog`, etc.
- **Pending action dispatch**: `pending_action: Option<PendingAction>`

The `pending_action` field is the central dispatch mechanism. Menu items and toolbar buttons do not execute actions directly — they set `pending_action`, which is then processed by `process_edit_actions()` in `ui/mod.rs`.

### PendingAction (enum)

Defined in `crates/bevy_map_editor/src/ui/dialogs.rs`. All menu-triggered operations (New, Open, Save, Undo, Redo, Cut, Copy, Paste, RunGame, ApplyAutomap, etc.) are variants of this enum.

### UiState (Resource)

Defined in `ui/mod.rs`. Tracks panel visibility (`show_tree_view`, `show_inspector`, `show_asset_browser`), panel sizes, and which integration panels are visible.

---

## UI Architecture

### System Schedule

UI rendering runs in `EguiPrimaryContextPass` (the bevy_egui schedule for the primary window context). The render system is `render_ui`.

Non-rendering systems run in `Update`:
- `load_tileset_textures` — polls Bevy's AssetServer and registers loaded images with egui
- `load_spritesheet_textures` — same for spritesheets
- `load_entity_textures` — same for entity icons/sprites
- `process_edit_actions` — dispatches `EditorState::pending_action`

### UI Module Structure

`crates/bevy_map_editor/src/ui/mod.rs` declares all submodules. Each panel is a standalone function:

| File | Entry point | Primary inputs |
|---|---|---|
| `toolbar.rs` | `render_toolbar(ctx, editor_state, integration_registry)` | `&mut EditorState` |
| `menu_bar.rs` | `render_menu_bar(ctx, ui_state, editor_state, project, history, clipboard, preferences, integration_registry)` | `&mut EditorState`, `&mut Project` |
| `tileset_editor.rs` | `render_tileset_editor(...)` | `TilesetEditorState`, `&mut Project` |
| `automap_editor.rs` | `render_automap_editor(...)` | `&mut Project` |
| `inspector.rs` | `render_inspector(...)` | `Selection`, `&mut Project` |
| `tree_view.rs` | `render_tree_view(...)` | `&mut EditorState`, `&mut Project` |
| `asset_browser.rs` | `render_asset_browser(...)` | `&mut AssetBrowserState` |

### Panel Render Signature Pattern

All top-level panel functions accept `&egui::Context` as their first argument and return either `()` or a result struct (e.g., `AnimationEditorResult`, `AssetBrowserResult`, `TreeViewResult`). This is the established pattern — do not deviate.

### Borrow Checker Pattern for Nested Project Data

When egui closures need both `&mut project.some_subsystem[i]` and `&mut project` at the same time (e.g., in the automap editor), use `macro_rules!` to re-borrow the nested path on each access rather than holding a `&mut` reference across closure boundaries. Clone data before rendering a grid, apply changes after the closure exits. Never hold `&mut rule` or `&mut tileset` across an egui closure that also needs `project`.

---

## Command / Undo System

Defined in `crates/bevy_map_editor/src/commands/`.

- `Command` trait: `fn execute(&mut self, project: &mut Project)`, `fn undo(&mut self, project: &mut Project)`, `fn description(&self) -> &str`
- `CommandHistory`: wraps a stack, exposes `can_undo()`, `can_redo()`, `push()`, `undo()`, `redo()`
- Concrete commands: `BatchTileCommand`, `AutomapCommand`, `MoveEntityCommand`

Undo/Redo are dispatched through `PendingAction::Undo` / `PendingAction::Redo` and resolved in `process_edit_actions()`.

---

## Integration / Plugin API

See `crates/bevy_map_integration/`. Plugins drop a `.toml` file into `~/Library/Application Support/bevy_map_editor/plugins/` on macOS. The `IntegrationRegistry` resource tracks all loaded extensions. UI contributions are `EditorExtension` variants: `ToolbarButton`, `Panel`, `MenuItem`. These are rendered in `toolbar.rs` and `menu_bar.rs` by iterating `registry.ui_contributions()`.

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

## DEBT

| Debt | Cost if unaddressed | Trigger to fix |
|---|---|---|
| ~~No test infrastructure~~ | ~~RESOLVED. 20 tests passing: toolbar, panel visibility (8), helpers (various). `assert_panel_visible` / `assert_panel_not_visible` implemented and verified.~~ | — |
| `process_edit_actions` is not tested | The central dispatch function has no coverage; silent breakage of undo/redo/save is possible | When the undo/redo or save path has a regression |
| Texture cache loading coupled to Bevy Asset system | Cannot unit-test texture loading logic without a running Bevy app | If a texture loading bug is introduced that integration testing would catch |

---

## Session Status

**Last updated:** 2026-02-26

**Cumulative completed:**
- Phase 1 UI testing rig: `egui_kittest` dev-dep added, `toolbar_grid_checkbox_toggle` test passing
- Test helper module (`src/testing.rs`) fully implemented and signed off by Worf
- Phase 3 snapshot tests: `wgpu` feature added to `egui_kittest` dev-dep; two snapshot tests written and blessed
- `assert_panel_visible` / `assert_panel_not_visible` implemented; 8 panel visibility tests added
- **20 tests passing, 0 failures**
- Agent team: Star Trek TNG personas; SE personas in individual `.claude/agents/` files
- `agents/permissions.md` created
- **Sprint protocol updated:** Picard spawns all agents including SE personas; Data advises on persona selection only

**Phase 3 snapshot test inventory:**

| Test | Description | Baseline PNG |
|---|---|---|
| `ui::toolbar::tests::toolbar_default_snapshot` | Toolbar with `EditorState::default()` (Select tool, Level view, Grid checked) | `crates/bevy_map_editor/tests/snapshots/toolbar_default_snapshot.png` |
| `ui::toolbar::tests::toolbar_paint_tool_snapshot` | Toolbar with `current_tool = Paint` (Mode combobox + Random/X/Y toggles visible) | `crates/bevy_map_editor/tests/snapshots/toolbar_paint_tool_snapshot.png` |

**Bless workflow:**
```
UPDATE_SNAPSHOTS=1 cargo test -p bevy_map_editor toolbar_default_snapshot
UPDATE_SNAPSHOTS=1 cargo test -p bevy_map_editor toolbar_paint_tool_snapshot
```

**Snapshot storage path:** `crates/bevy_map_editor/tests/snapshots/` (default from `egui_kittest`, relative to crate root during `cargo test`).

**`capture_snapshot` helper note:** `testing.rs` retains its comment-block placeholder for `capture_snapshot`. The snapshot tests call `harness.snapshot(name)` directly, per the sprint directive that no new helpers should be added. If a project-wide default threshold override is needed in future, `capture_snapshot` can be implemented as a thin wrapper over `harness.snapshot_options(name, opts)` at that time.

**Completed this session (2026-02-26, continued):**
- Tree View heading renamed: `ui.heading("Project")` → `ui.heading("Tree View")` in `tree_view.rs` line 107. Decision: `"Project"` was ambiguous against the "Project" top-level menu and project-name status label. No existing tests referenced the old string. Zero test breakage.
- Asset Browser heading added: `ui.heading("Asset Browser")` inserted at `asset_browser.rs` line 328, before the horizontal toolbar row. This was the final prerequisite for `assert_panel_visible`.
- Both changes compile cleanly (`cargo build -p bevy_map_editor --features dynamic_linking`).
- `testing.md` anchor label table updated.

**Final anchor labels (canonical):**
| Panel | Anchor |
|---|---|
| Inspector | `"Inspector"` |
| Tree View | `"Tree View"` |
| Asset Browser | `"Asset Browser"` |

**Troi Open Question 1 — ANSWERED (2026-02-26):**

Yes. Individual panel render functions can be wrapped directly in `SidePanel`/`TopBottomPanel`
wrappers inside a `egui_kittest::Harness` closure. The panel wrappers (`SidePanel::right`,
`SidePanel::left`, `TopBottomPanel::bottom`) are plain egui API calls that take only
`&egui::Context` — which the harness closure provides. There is no Bevy coupling at the
panel wrapper level. The Bevy coupling lives only in `render_ui`'s system parameter list
(`ResMut<EditorState>` etc.), which is not involved in the inner render functions.

The approved harness pattern for panel visibility tests:

```rust
pub fn harness_for_inspector(state: InspectorBundle) -> Harness<'static, InspectorBundle> {
    Harness::new_state(
        |ctx, state: &mut InspectorBundle| {
            if state.ui_state.show_inspector {
                egui::SidePanel::right("inspector").show(ctx, |ui| {
                    render_inspector(ui, &state.selection, &mut state.project);
                });
            }
        },
        state,
    )
}
```

The `if ui_state.show_*` guard must be present in the test harness closure — this is what
produces the absent-vs-present node behavior that `assert_panel_visible` tests. The render
function itself does not need modification. This pattern generalizes to all three panels:
- Inspector: `egui::SidePanel::right("inspector")`
- Tree View: `egui::SidePanel::left("tree_view")`
- Asset Browser: `egui::TopBottomPanel::bottom("asset_browser")`

The panel `id_source` strings (`"inspector"`, `"tree_view"`, `"asset_browser"`) must match
production code to avoid id collisions if tests ever run multiple panels in one harness.

**Task #2 is now fully unblocked. All prerequisites are complete:**
1. Troi spec defines the assertion semantics (absence-based, anchor = heading node).
2. All three panels have heading nodes with stable anchor strings.
3. Data has confirmed the harness architecture.
Worf may proceed with Task #2 implementation.

**Blocked / deferred:**
- `process_edit_actions` has no test coverage (see DEBT table)

**Next action:** Assign Task #2 to Worf. All prerequisites are satisfied.
