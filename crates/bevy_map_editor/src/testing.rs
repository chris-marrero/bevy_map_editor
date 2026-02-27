//! Test helpers for `bevy_map_editor` UI panels.
//!
//! This module provides state factories, harness builders, interaction free functions,
//! assertion helpers, and AccessKit tree dump utilities for use in `#[cfg(test)]` modules
//! within this crate.
#![allow(dead_code)]
//!
//! # Usage
//!
//! Import the whole module via glob in test modules:
//!
//! ```rust,ignore
//! #[cfg(test)]
//! mod tests {
//!     use crate::testing::*;
//!     // ...
//! }
//! ```
//!
//! # Snapshot Tests (Phase 3)
//!
//! `capture_snapshot` and `render_to_bytes` require the `wgpu` feature on `egui_kittest`.
//! They are gated with `#[cfg(feature = "wgpu")]` and are not active in the current
//! dev-dependency configuration. To use them, add `wgpu` to the `egui_kittest` dev-dep
//! features in `Cargo.toml`. Bless workflow: `UPDATE_SNAPSHOTS=1 cargo test --features wgpu`.
//!
//! # Deferred
//!
//! - `bless_snapshot` — not implemented. Bless workflow is env-var-based (`UPDATE_SNAPSHOTS=1`).
//!
//! # Panel Visibility Helpers
//!
//! `assert_panel_visible` and `assert_panel_not_visible` are implemented and tested.
//! Use `harness_for_panel_visibility` with the panel visibility factory functions to
//! exercise them. Tests live in the `#[cfg(test)]` module at the bottom of this file.

use bevy_egui::egui::accesskit;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;

use crate::commands::{BatchTileCommand, CommandHistory, TileClipboard};
use crate::preferences::EditorPreferences;
use crate::project::Project;
use crate::ui::{
    render_asset_browser, render_inspector, render_menu_bar, render_tileset_editor, render_toolbar,
    render_tree_view, EditorTool, PendingAction, UiState,
};
use crate::{EditorState, EditorViewMode};

// ============================================================================
// Bundle Structs
// ============================================================================

/// Bundle for `render_menu_bar` — wraps all arguments that the function requires as `&mut T`.
///
/// `history` and `clipboard` are `Option<T>` here (not `Option<&T>`) because
/// `Harness::new_state` owns exactly one `State` value. The harness closure borrows
/// from the owned values when forwarding to `render_menu_bar`.
pub struct MenuBarState {
    pub ui_state: UiState,
    pub editor_state: EditorState,
    pub project: Project,
    /// Owned command history. `render_menu_bar` receives `Option<&CommandHistory>`.
    pub history: Option<CommandHistory>,
    /// Owned clipboard. `render_menu_bar` receives `Option<&TileClipboard>`.
    pub clipboard: Option<TileClipboard>,
    pub preferences: EditorPreferences,
}

/// Bundle for `render_tileset_editor` — wraps `EditorState` and `Project`.
///
/// The `cache` parameter (`Option<&TilesetTextureCache>`) is always passed as `None` in tests.
/// This is acceptable because the editor window structure and tab navigation do not require
/// image data; texture rendering is only relevant when a tileset is selected and images are loaded.
pub struct TilesetEditorBundle {
    pub editor_state: EditorState,
    pub project: Project,
}

/// Bundle for panel visibility tests — wraps `UiState`, `EditorState`, and `Project`.
///
/// Used with `harness_for_panel_visibility`. The `UiState` flags
/// (`show_inspector`, `show_tree_view`, `show_asset_browser`) control which panels
/// are rendered. The harness calls each panel render function inside the appropriate
/// `SidePanel`/`TopBottomPanel` wrapper, conditional on those flags.
///
/// Construct via the factory functions:
/// - `panel_visibility_all_visible()` — all three panels visible
/// - `panel_visibility_inspector_hidden()` — Inspector hidden, others visible
/// - `panel_visibility_tree_view_hidden()` — Tree View hidden, others visible
/// - `panel_visibility_asset_browser_visible()` — all panels visible (asset browser on)
/// - `panel_visibility_asset_browser_hidden()` — asset browser off (the default)
pub struct PanelVisibilityBundle {
    pub ui_state: UiState,
    pub editor_state: EditorState,
    pub project: Project,
}

// ============================================================================
// Factory Functions
// ============================================================================

/// Returns `EditorState::default()`.
///
/// `EditorState::default()` has `view_mode = Level`, `show_grid = true`,
/// `current_tool = Select`. This is the baseline precondition for most tests.
pub fn editor_state_default() -> EditorState {
    EditorState::default()
}

/// Returns `EditorState` configured for level view.
///
/// Explicitly documents that the test requires:
/// - `view_mode = EditorViewMode::Level`
/// - `show_grid = true` (default)
/// - Tools enabled (the `ui.disable()` scope does not fire in Level view)
pub fn editor_state_level_view() -> EditorState {
    let mut state = EditorState::default();
    state.view_mode = EditorViewMode::Level;
    state
}

/// Returns `EditorState` configured for world view.
///
/// Use ONLY for tests that verify disabled-state behavior in World view.
/// Do NOT use for tool interaction tests — `ui.disable()` fires in World view,
/// making tool buttons and the Grid checkbox non-interactive.
pub fn editor_state_world_view() -> EditorState {
    let mut state = EditorState::default();
    state.view_mode = EditorViewMode::World;
    state
}

/// Returns `EditorState` with `current_tool = Paint` and `view_mode = Level`.
pub fn editor_state_paint_tool() -> EditorState {
    let mut state = EditorState::default();
    state.current_tool = EditorTool::Paint;
    state.view_mode = EditorViewMode::Level;
    state
}

/// Returns a `MenuBarState` with a minimal valid empty project.
///
/// Suitable for testing menu item clicks that set `pending_action` without
/// requiring any project content (levels, tilesets, etc.).
pub fn menu_bar_state_empty_project() -> MenuBarState {
    MenuBarState {
        ui_state: UiState::default(),
        editor_state: EditorState::default(),
        project: Project::default(),
        history: Some(CommandHistory::default()),
        clipboard: Some(TileClipboard::default()),
        preferences: EditorPreferences::default(),
    }
}

/// Returns a `MenuBarState` where undo is available.
///
/// The `CommandHistory` has one entry on the undo stack so `can_undo() == true`.
/// Use for Edit menu undo/redo enabled-state tests.
pub fn menu_bar_state_with_undo() -> MenuBarState {
    let mut history = CommandHistory::default();
    // Push a no-op BatchTileCommand onto the undo stack to make undo available.
    // Empty changes map = no-op when executed or undone.
    history.push_undo(Box::new(BatchTileCommand::new(
        uuid::Uuid::nil(),
        0,
        std::collections::HashMap::new(),
        "test-undo-entry",
    )));

    MenuBarState {
        ui_state: UiState::default(),
        editor_state: EditorState::default(),
        project: Project::default(),
        history: Some(history),
        clipboard: Some(TileClipboard::default()),
        preferences: EditorPreferences::default(),
    }
}

/// Returns a `PanelVisibilityBundle` with all three panels visible.
///
/// `UiState::default()` has `show_tree_view = true`, `show_inspector = true`,
/// `show_asset_browser = false`. This factory overrides `show_asset_browser` to `true`
/// so all three panels are rendered.
pub fn panel_visibility_all_visible() -> PanelVisibilityBundle {
    let mut ui_state = UiState::default();
    ui_state.show_tree_view = true;
    ui_state.show_inspector = true;
    ui_state.show_asset_browser = true;
    PanelVisibilityBundle {
        ui_state,
        editor_state: EditorState::default(),
        project: Project::default(),
    }
}

/// Returns a `PanelVisibilityBundle` with the Inspector panel hidden.
///
/// `show_inspector = false`, all others visible. Use to verify
/// `assert_panel_not_visible(harness, "Inspector")`.
pub fn panel_visibility_inspector_hidden() -> PanelVisibilityBundle {
    let mut ui_state = UiState::default();
    ui_state.show_inspector = false;
    ui_state.show_tree_view = true;
    ui_state.show_asset_browser = false;
    PanelVisibilityBundle {
        ui_state,
        editor_state: EditorState::default(),
        project: Project::default(),
    }
}

/// Returns a `PanelVisibilityBundle` with the Tree View panel hidden.
///
/// `show_tree_view = false`, all others visible. Use to verify
/// `assert_panel_not_visible(harness, "Tree View")`.
pub fn panel_visibility_tree_view_hidden() -> PanelVisibilityBundle {
    let mut ui_state = UiState::default();
    ui_state.show_tree_view = false;
    ui_state.show_inspector = true;
    ui_state.show_asset_browser = false;
    PanelVisibilityBundle {
        ui_state,
        editor_state: EditorState::default(),
        project: Project::default(),
    }
}

/// Returns a `PanelVisibilityBundle` with the Asset Browser panel visible.
///
/// `show_asset_browser = true`. Use to verify
/// `assert_panel_visible(harness, "Asset Browser")`.
pub fn panel_visibility_asset_browser_visible() -> PanelVisibilityBundle {
    let mut ui_state = UiState::default();
    ui_state.show_tree_view = true;
    ui_state.show_inspector = true;
    ui_state.show_asset_browser = true;
    PanelVisibilityBundle {
        ui_state,
        editor_state: EditorState::default(),
        project: Project::default(),
    }
}

/// Returns a `PanelVisibilityBundle` with the Asset Browser panel hidden.
///
/// `show_asset_browser = false` (the `UiState::default()`). Use to verify
/// `assert_panel_not_visible(harness, "Asset Browser")`.
pub fn panel_visibility_asset_browser_hidden() -> PanelVisibilityBundle {
    let mut ui_state = UiState::default();
    ui_state.show_tree_view = true;
    ui_state.show_inspector = true;
    ui_state.show_asset_browser = false;
    PanelVisibilityBundle {
        ui_state,
        editor_state: EditorState::default(),
        project: Project::default(),
    }
}

// ============================================================================
// Harness Builders
// ============================================================================

/// Returns a `Harness<'static, EditorState>` that renders the toolbar.
///
/// The integration registry is always `None` — integration toolbar buttons
/// are not testable without a running Bevy app.
///
/// # Example
///
/// ```rust,ignore
/// let mut harness = harness_for_toolbar(editor_state_level_view());
/// assert!(harness.state().show_grid);
/// harness.run();
/// harness.get_by_label("Grid").click();
/// harness.run();
/// assert!(!harness.state().show_grid);
/// ```
pub fn harness_for_toolbar(state: EditorState) -> Harness<'static, EditorState> {
    Harness::new_state(
        |ctx, editor_state: &mut EditorState| {
            render_toolbar(ctx, editor_state, None);
        },
        state,
    )
}

/// Returns a `Harness<'static, MenuBarState>` that renders the menu bar.
///
/// The integration registry is always `None`.
pub fn harness_for_menu_bar(state: MenuBarState) -> Harness<'static, MenuBarState> {
    Harness::new_state(
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
        },
        state,
    )
}

/// Returns a `Harness<'static, TilesetEditorBundle>` that renders the tileset editor.
///
/// `cache` is always `None` — texture image rendering is not exercised in unit tests.
pub fn harness_for_tileset_editor(
    state: TilesetEditorBundle,
) -> Harness<'static, TilesetEditorBundle> {
    Harness::new_state(
        |ctx, state: &mut TilesetEditorBundle| {
            render_tileset_editor(ctx, &mut state.editor_state, &mut state.project, None);
        },
        state,
    )
}

/// Returns a `Harness<'static, PanelVisibilityBundle>` that renders the three editor side panels
/// conditional on the `UiState` visibility flags.
///
/// The harness replicates the panel wrapper structure from `ui/mod.rs`:
/// - Tree View: `SidePanel::left("test_tree_view")`, conditional on `show_tree_view`
/// - Inspector: `SidePanel::right("test_inspector")`, conditional on `show_inspector`
/// - Asset Browser: `TopBottomPanel::bottom("test_asset_browser")`, conditional on `show_asset_browser`
///
/// Each panel render function is called inside its wrapper with `None` for the integration
/// registry. The heading nodes produced by `ui.heading(...)` inside each render function
/// are the anchor labels used by `assert_panel_visible` and `assert_panel_not_visible`.
///
/// # Example
///
/// ```rust,ignore
/// let mut harness = harness_for_panel_visibility(panel_visibility_all_visible());
/// harness.run();
/// assert_panel_visible(&harness, "Inspector");
/// assert_panel_visible(&harness, "Tree View");
/// assert_panel_visible(&harness, "Asset Browser");
/// ```
pub fn harness_for_panel_visibility(
    state: PanelVisibilityBundle,
) -> Harness<'static, PanelVisibilityBundle> {
    use bevy_egui::egui;
    Harness::new_state(
        |ctx, bundle: &mut PanelVisibilityBundle| {
            // Tree View — left side panel, conditional on show_tree_view
            if bundle.ui_state.show_tree_view {
                egui::SidePanel::left("test_tree_view").show(ctx, |ui| {
                    render_tree_view(ui, &mut bundle.editor_state, &mut bundle.project, None);
                });
            }

            // Inspector — right side panel, conditional on show_inspector
            if bundle.ui_state.show_inspector {
                egui::SidePanel::right("test_inspector").show(ctx, |ui| {
                    render_inspector(ui, &mut bundle.editor_state, &mut bundle.project, None);
                });
            }

            // Asset Browser — bottom panel, conditional on show_asset_browser
            if bundle.ui_state.show_asset_browser {
                egui::TopBottomPanel::bottom("test_asset_browser").show(ctx, |ui| {
                    render_asset_browser(ui, &mut bundle.ui_state.asset_browser_state);
                });
            }

            // CentralPanel is required when SidePanels are present;
            // without it egui panics. We render an empty one.
            egui::CentralPanel::default().show(ctx, |_ui| {});
        },
        state,
    )
}

// ============================================================================
// Interaction Free Functions
// ============================================================================

/// Click the widget with the given AccessKit label.
///
/// Equivalent to `harness.get_by_label(label).click()` but documents the intent
/// at the test site and keeps the label as a named argument.
///
/// After calling this, run `harness.run()` to process the click.
pub fn click_labeled<State>(harness: &Harness<'_, State>, label: &str) {
    harness.get_by_label(label).click();
}

/// Toggle a checkbox widget with the given AccessKit label (one click).
///
/// This is semantically identical to `click_labeled` but documents that the
/// target is a checkbox being toggled, not an action button being activated.
pub fn toggle_labeled<State>(harness: &Harness<'_, State>, label: &str) {
    harness.get_by_label(label).click();
}

/// Select a selectable label (tool button, tab button) with the given AccessKit label.
///
/// This is semantically identical to `click_labeled` but documents that the
/// target is a selectable item (e.g. `ui.selectable_label`), not an action button.
pub fn select_labeled<State>(harness: &Harness<'_, State>, label: &str) {
    harness.get_by_label(label).click();
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that the given `EditorTool` is the currently active tool.
///
/// Reads from `harness.state().current_tool` — does not use the AccessKit tree.
/// Call after `harness.run()` to observe the post-interaction state.
pub fn assert_tool_active(harness: &Harness<'_, EditorState>, expected: EditorTool) {
    assert_eq!(
        harness.state().current_tool,
        expected,
        "Expected active tool to be {:?}, got {:?}",
        expected,
        harness.state().current_tool,
    );
}

/// Assert that the checkbox with the given AccessKit label is in the expected toggled state.
///
/// Uses `accesskit_node.toggled()` from the AccessKit tree.
/// `expected = true` means the checkbox is checked; `false` means unchecked.
///
/// # Panics
///
/// Panics if the widget is not found, or if its AccessKit node does not expose a `toggled` state
/// (which would indicate it is not a checkbox).
pub fn assert_checkbox_state<State>(harness: &Harness<'_, State>, label: &str, expected: bool) {
    let node = harness.get_by_label(label);
    let toggled = node
        .accesskit_node()
        .toggled()
        .unwrap_or_else(|| panic!("Widget '{label}' does not expose a toggled state (is it a checkbox?)"));
    let checked = toggled == accesskit::Toggled::True;
    assert_eq!(
        checked, expected,
        "Checkbox '{label}': expected checked={expected}, got checked={checked}",
    );
}

/// Assert that the widget with the given AccessKit label is enabled (not disabled).
///
/// Uses `accesskit_node.is_disabled()`. A widget is enabled when `is_disabled() == false`.
pub fn assert_widget_enabled<State>(harness: &Harness<'_, State>, label: &str) {
    let node = harness.get_by_label(label);
    assert!(
        !node.accesskit_node().is_disabled(),
        "Expected widget '{label}' to be enabled, but it is disabled",
    );
}

/// Assert that the widget with the given AccessKit label is disabled.
///
/// Uses `accesskit_node.is_disabled()`. A widget is disabled when `is_disabled() == true`.
pub fn assert_widget_disabled<State>(harness: &Harness<'_, State>, label: &str) {
    let node = harness.get_by_label(label);
    assert!(
        node.accesskit_node().is_disabled(),
        "Expected widget '{label}' to be disabled, but it is enabled",
    );
}

/// Assert that the panel with the given anchor label is visible (its heading node is present).
///
/// Panels are identified by the `ui.heading(...)` call inside them. When the panel is hidden
/// by a boolean guard, no egui call is made and the heading node is absent from the AccessKit
/// tree. When the panel is visible, the node is present.
///
/// Uses `harness.query_by_label(label)` — does NOT panic on a missing node.
///
/// # Panics
///
/// Panics if the heading node is absent (panel is hidden or the anchor label is wrong).
pub fn assert_panel_visible<State>(harness: &Harness<'_, State>, label: &str) {
    assert!(
        harness.query_by_label(label).is_some(),
        "Expected panel anchor '{label}' to be present in the AccessKit tree, but it was not found. Is the panel hidden?",
    );
}

/// Assert that the panel with the given anchor label is not visible (its heading node is absent).
///
/// Panels are identified by the `ui.heading(...)` call inside them. When the panel is hidden
/// by a boolean guard, no egui call is made and the heading node is absent from the AccessKit
/// tree. When the panel is visible, the node is present.
///
/// Uses `harness.query_by_label(label)` — does NOT panic on a missing node.
///
/// # Panics
///
/// Panics if the heading node is present (panel is visible when it should not be).
pub fn assert_panel_not_visible<State>(harness: &Harness<'_, State>, label: &str) {
    assert!(
        harness.query_by_label(label).is_none(),
        "Expected panel anchor '{label}' to be absent from the AccessKit tree, but it was found. Is the panel visible when it should not be?",
    );
}

/// Assert that a pending action value matches the expected action.
///
/// This is a plain function that takes `Option<&PendingAction>` directly, so it works
/// with any state type. The caller is responsible for extracting it:
///
/// ```rust,ignore
/// // For EditorState harness:
/// assert_pending_action(
///     harness.state().pending_action.as_ref(),
///     &PendingAction::Save,
/// );
///
/// // For MenuBarState harness:
/// assert_pending_action(
///     harness.state().editor_state.pending_action.as_ref(),
///     &PendingAction::Undo,
/// );
/// ```
pub fn assert_pending_action(actual: Option<&PendingAction>, expected: &PendingAction) {
    assert_eq!(
        actual,
        Some(expected),
        "Expected pending_action = {:?}, got {:?}",
        expected,
        actual,
    );
}

// ============================================================================
// AccessKit Tree Dump
// ============================================================================

/// Returns the AccessKit tree as an indented string for debugging.
///
/// Each node is formatted as:
/// ```text
/// {Role}[role={Role}, label="{label}", disabled=true, toggled={toggled}]
/// ```
///
/// Optional fields (`label`, `disabled`, `toggled`) are omitted when not present or default.
/// Indentation increases by two spaces per depth level. The root node is at depth 0.
///
/// This function requires no GPU — the AccessKit tree is a pure data structure built
/// from `egui`'s platform output. Call `harness.run()` at least once before calling this.
///
/// # Example
///
/// ```rust,ignore
/// harness.run();
/// println!("{}", accessibility_tree_string(&harness));
/// ```
pub fn accessibility_tree_string<State>(harness: &Harness<'_, State>) -> String {
    let mut buf = String::new();
    fmt_node_recursive(&harness.root(), 0, &mut buf);
    buf
}

/// Print the AccessKit tree to stdout.
///
/// Convenience wrapper around `accessibility_tree_string`. Useful for debugging
/// which labels are available for `get_by_label` targeting.
pub fn dump_accessibility_tree<State>(harness: &Harness<'_, State>) {
    print!("{}", accessibility_tree_string(harness));
}

/// Write the AccessKit tree string to a file.
///
/// The `path` is relative to the process working directory at test time, which is the
/// crate root when running `cargo test`. For example, `"tests/tree_dump.txt"` writes to
/// `crates/bevy_map_editor/tests/tree_dump.txt`.
///
/// # Errors
///
/// Returns `std::io::Error` if the file cannot be written.
pub fn write_accessibility_tree<State>(
    harness: &Harness<'_, State>,
    path: &str,
) -> std::io::Result<()> {
    let tree = accessibility_tree_string(harness);
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, tree)
}

/// Recursive helper for `accessibility_tree_string`.
fn fmt_node_recursive(node: &egui_kittest::Node<'_>, depth: usize, buf: &mut String) {
    let indent = "  ".repeat(depth);
    let ak = node.accesskit_node();
    let role = format!("{:?}", ak.role());
    let label_part = ak
        .label()
        .map(|l| format!(", label={l:?}"))
        .unwrap_or_default();
    let disabled_part = if ak.is_disabled() {
        ", disabled=true"
    } else {
        ""
    };
    let toggled_part = ak
        .toggled()
        .map(|t| format!(", toggled={t:?}"))
        .unwrap_or_default();
    buf.push_str(&format!(
        "{indent}{role}[role={role}{label_part}{disabled_part}{toggled_part}]\n"
    ));

    for child in node.children() {
        fmt_node_recursive(&child, depth + 1, buf);
    }
}

// ============================================================================
// Snapshot Helpers (Phase 3 — not yet active)
//
// `capture_snapshot` and `render_to_bytes` require the `wgpu` feature on `egui_kittest`.
// They are not compiled in the current configuration because:
//   1. `wgpu` is not in the `egui_kittest` dev-dependency features (per Sr SE Decision 6).
//   2. `bevy_map_editor` does not have a `wgpu` feature flag.
//
// To activate Phase 3 snapshot tests:
//   - Add `wgpu` to `egui_kittest` features in `[dev-dependencies]`
//   - Add a `wgpu` feature to `[features]` in this crate's Cargo.toml
//   - Uncomment or re-add these functions gated on that feature
//
// Bless workflow once activated:
//   UPDATE_SNAPSHOTS=1 cargo test --features wgpu
// ============================================================================

// ============================================================================
// Tests — assert_panel_visible / assert_panel_not_visible
//
// These tests verify that the helper functions and the harness builder work
// correctly. They are the authoritative empirical verification that:
//   1. assert_panel_visible passes when the heading node IS present.
//   2. assert_panel_not_visible passes when the heading node IS absent.
//   3. The UiState flags correctly gate panel rendering.
//
// Run with: cargo test -p bevy_map_editor testing::
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::testing::{
        assert_panel_not_visible, assert_panel_visible, harness_for_panel_visibility,
        panel_visibility_all_visible, panel_visibility_asset_browser_hidden,
        panel_visibility_asset_browser_visible, panel_visibility_inspector_hidden,
        panel_visibility_tree_view_hidden,
    };

    // -----------------------------------------------------------------------
    // Inspector visibility
    // -----------------------------------------------------------------------

    /// Precondition: `UiState::default()` has `show_inspector = true`.
    /// When Inspector is visible, `ui.heading("Inspector")` renders and the
    /// "Inspector" label node is present in the AccessKit tree.
    #[test]
    fn inspector_heading_present_when_show_inspector_true() {
        let mut harness = harness_for_panel_visibility(panel_visibility_all_visible());
        // Precondition
        assert!(
            harness.state().ui_state.show_inspector,
            "Precondition failed: show_inspector must be true"
        );
        harness.run();
        assert_panel_visible(&harness, "Inspector");
    }

    /// When `show_inspector = false`, no egui call is made for the inspector panel.
    /// The "Inspector" heading node must be absent from the AccessKit tree.
    #[test]
    fn inspector_heading_absent_when_show_inspector_false() {
        let mut harness = harness_for_panel_visibility(panel_visibility_inspector_hidden());
        // Precondition
        assert!(
            !harness.state().ui_state.show_inspector,
            "Precondition failed: show_inspector must be false"
        );
        harness.run();
        assert_panel_not_visible(&harness, "Inspector");
    }

    // -----------------------------------------------------------------------
    // Tree View visibility
    // -----------------------------------------------------------------------

    /// Precondition: `UiState::default()` has `show_tree_view = true`.
    /// When Tree View is visible, `ui.heading("Tree View")` renders and the
    /// "Tree View" label node is present in the AccessKit tree.
    #[test]
    fn tree_view_heading_present_when_show_tree_view_true() {
        let mut harness = harness_for_panel_visibility(panel_visibility_all_visible());
        // Precondition
        assert!(
            harness.state().ui_state.show_tree_view,
            "Precondition failed: show_tree_view must be true"
        );
        harness.run();
        assert_panel_visible(&harness, "Tree View");
    }

    /// When `show_tree_view = false`, no egui call is made for the tree view panel.
    /// The "Tree View" heading node must be absent from the AccessKit tree.
    #[test]
    fn tree_view_heading_absent_when_show_tree_view_false() {
        let mut harness = harness_for_panel_visibility(panel_visibility_tree_view_hidden());
        // Precondition
        assert!(
            !harness.state().ui_state.show_tree_view,
            "Precondition failed: show_tree_view must be false"
        );
        harness.run();
        assert_panel_not_visible(&harness, "Tree View");
    }

    // -----------------------------------------------------------------------
    // Asset Browser visibility
    // -----------------------------------------------------------------------

    /// `UiState::default()` has `show_asset_browser = false` — it is off by default.
    /// This test verifies that the asset browser heading IS present when the flag is true.
    #[test]
    fn asset_browser_heading_present_when_show_asset_browser_true() {
        let mut harness =
            harness_for_panel_visibility(panel_visibility_asset_browser_visible());
        // Precondition
        assert!(
            harness.state().ui_state.show_asset_browser,
            "Precondition failed: show_asset_browser must be true"
        );
        harness.run();
        assert_panel_visible(&harness, "Asset Browser");
    }

    /// `UiState::default()` has `show_asset_browser = false`.
    /// The "Asset Browser" heading node must be absent from the AccessKit tree.
    #[test]
    fn asset_browser_heading_absent_when_show_asset_browser_false() {
        let mut harness =
            harness_for_panel_visibility(panel_visibility_asset_browser_hidden());
        // Precondition
        assert!(
            !harness.state().ui_state.show_asset_browser,
            "Precondition failed: show_asset_browser must be false"
        );
        harness.run();
        assert_panel_not_visible(&harness, "Asset Browser");
    }

    // -----------------------------------------------------------------------
    // Isolation: hidden panels do not bleed into other panels
    // -----------------------------------------------------------------------

    /// When Inspector is hidden, Tree View must still be visible.
    /// This guards against harness construction errors that accidentally hide all panels.
    #[test]
    fn tree_view_still_visible_when_inspector_hidden() {
        let mut harness = harness_for_panel_visibility(panel_visibility_inspector_hidden());
        harness.run();
        // Inspector must be absent
        assert_panel_not_visible(&harness, "Inspector");
        // Tree View must be present — it is on in panel_visibility_inspector_hidden()
        assert_panel_visible(&harness, "Tree View");
    }

    /// When Tree View is hidden, Inspector must still be visible.
    #[test]
    fn inspector_still_visible_when_tree_view_hidden() {
        let mut harness = harness_for_panel_visibility(panel_visibility_tree_view_hidden());
        harness.run();
        // Tree View must be absent
        assert_panel_not_visible(&harness, "Tree View");
        // Inspector must be present — it is on in panel_visibility_tree_view_hidden()
        assert_panel_visible(&harness, "Inspector");
    }

    // -----------------------------------------------------------------------
    // Diagnostic dump (disabled — leave here for debugging, do not delete)
    // -----------------------------------------------------------------------
    //
    // Uncomment temporarily to inspect the AccessKit tree when a test fails
    // with "widget not found":
    //
    // #[test]
    // fn dump_panel_visibility_tree() {
    //     let mut harness = harness_for_panel_visibility(panel_visibility_all_visible());
    //     harness.run();
    //     dump_accessibility_tree(&harness);
    // }
}
