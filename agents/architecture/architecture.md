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
| `CollisionDragOperation` wildcard arm in `drag_stopped()` silently discards `MoveShape`, `ResizeRect`, `ResizeCircle` | If any code path ever initializes one of these operations, the drag commits nothing and produces no error; the compiler's exhaustiveness guarantee is suppressed by `_ => {}` | When `MoveShape` / `ResizeRect` / `ResizeCircle` operations are implemented |
| `0.01` drag-commit threshold is a magic constant in `drag_stopped()` | Minor readability issue; not a behavioral problem | If threshold ever needs tuning — extract to a named const |
| `format!("{:?}", one_way)` in CollisionProperties ComboBox exposes Rust debug format | Cosmetic only — user sees `Top` instead of "Top (Pass from below)"; consistent with existing style but not ideal | When the properties panel gets a UX polish pass |
| `find_layer_index` in `crates/bevy_map_automap/src/apply.rs` is a permanent stub returning `None` | **Functional, not cosmetic.** Every automap rule that targets a named layer silently writes to nowhere; output groups and alternatives that specify `layer_id` are fully ignored at apply time. No error is surfaced. The editor appears to run automap successfully while producing no output on targeted layers. | **Trigger condition now met** (PR #3 adds `Layer::id`). T-11 assigned to Barclay. Resolve before PR #3 merges. |
| `no_overlapping_output` overlap guard in `apply_rule` tracks only the center cell `(x, y)`, not all cells written by the output alternative | For output grids larger than 1×1, adjacent scan positions may write to cells written by a previous match, silently defeating the no-overlap invariant | When users report unexpected tile overlap in rules that use multi-cell output grids with `no_overlapping_output: true` |
| `apply_automap_config` is O(rules × width × height) | Acceptable now; may produce noticeable lag on large levels with many rules | If user reports lag on levels larger than 256×256 |
| `Layer::id` on old project files is not stable until the first save | Rule `layer_id` references assigned on load may diverge if the file is subsequently edited by another tool before saving | When multi-tool workflows are supported |
| Output alternative grid dimensions stored independently per alternative | Visually confusing in the automap UI editor if source and output grids differ in cell size; no validation or warning | When Troi produces a UX spec for the automap rule editor |
| `apply_automap_config` takes `impl Rng` but no seed is exposed | Deterministic replay of probabilistic rules is impossible without externally supplying the seed | If replay or regression testing of probabilistic automap rules is required |

---

## Session Status

**Last updated:** 2026-02-27 (Sprint: Automapping, mid-sprint)

**Cumulative completed:**
- Phase 1 UI testing rig: `egui_kittest` dev-dep added, `toolbar_grid_checkbox_toggle` test passing
- Test helper module (`src/testing.rs`) fully implemented and signed off by Worf
- Phase 3 snapshot tests: `wgpu` feature added to `egui_kittest` dev-dep; two snapshot tests written and blessed
- `assert_panel_visible` / `assert_panel_not_visible` implemented; 8 panel visibility tests added
- Collision editor drag bug fixed (Wesley): Rectangle/Circle drag initialization moved from `clicked()` to `drag_started()`
- Collision editor numeric input panel added (Barclay): DragValue fields for all CollisionShape variants in `render_collision_properties`
- **34 tests passing, 0 failures** (+14 this sprint: 10 label-presence tests for numeric panel, 4 smoke tests per draw mode)
- Agent team: Star Trek TNG personas; SE personas in individual `.claude/agents/` files
- `agents/permissions.md` created
- **Sprint protocol updated:** All agents spawn simultaneously; self-assign tasks; Data reviews code before Worf; Troi reviews UX output from Data; Picard never edits production files

**Drag canvas testability note:** `handle_collision_canvas_input` canvas drag behavior cannot be tested with the current `egui_kittest` rig — the canvas is an unlabeled painter region with no AccessKit node. Testing drag-to-draw requires either an accessible wrapper on the canvas response or extraction of drag state logic into a pure function. This is a Data-level architecture decision for a future sprint.

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

**Blocked / deferred:**
- `process_edit_actions` has no test coverage (see DEBT table)

---

## Sprint: Automapping — Session Status

**Last updated:** 2026-02-27

**Open PRs:**

| PR | Branch | Assignee | Status |
|---|---|---|---|
| #1 | `sprint/automapping/geordi-engine` | Geordi | Open — awaiting Data review |
| #2 | `sprint/automapping/wesley-ui` | Wesley | Open — awaiting Barclay's PR #3 merge first (depends on `show_automap_editor` + `automap_editor_state` fields) |
| #3 | `sprint/automapping/barclay-integration` | Barclay | Open — compiles cleanly standalone; awaiting Data review |

**Merge order:** PR #3 (Barclay) must merge before PR #2 (Wesley) can rebase. Data reviews #1 and #3 first; Wesley rebases #2 after #3 merges.

**Build state:** `cargo check -p bevy_map_editor` passes on `sprint/automapping/barclay-integration`.

**Known live stub:** `find_layer_index` in `crates/bevy_map_automap/src/apply.rs` returns `None` permanently. Recorded in DEBT table. Fix is gated on `Layer::id` landing in `bevy_map_core` (Barclay's branch).

**Tests:** T-05 (Worf) is blocked. No automapping tests written yet. Blocked on Data review of PRs #1, #3 and merge of PR #3.

**Layer mapping persistence:** Not yet implemented. The automap rule editor (Wesley) allows rules to reference layer IDs, but the persistence layer for these associations in the editor's project serialization is not wired. This is in-flight debt — not yet in DEBT table; must be added when scope is confirmed with Data.

**Next actions (in order):**
1. Data reviews PR #1 (Geordi engine) and PR #3 (Barclay integration)
2. Data gives GO on #3 → Barclay's PR merges
3. Wesley rebases PR #2 on updated main; Data reviews
4. All three merged → Worf writes T-05 tests
5. `find_layer_index` stub resolved once `Layer::id` is in `bevy_map_core`

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
- `format!("{:?}", one_way)` in ComboBox exposes Rust debug format (pre-existing, noted in prior session).

---
