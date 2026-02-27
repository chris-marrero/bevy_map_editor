//! Toolbar UI for tool selection

use crate::ui::dialogs::PendingAction;
use crate::{EditorState, EditorViewMode};
use bevy_egui::egui;
use bevy_map_integration::registry::IntegrationRegistry;
use serde::{Deserialize, Serialize};

/// Available editor tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EditorTool {
    #[default]
    Select,
    Paint,
    Erase,
    Fill,
    Terrain,
    Entity,
}

impl EditorTool {
    /// Returns true if this tool supports Point/Rectangle modes
    pub fn supports_modes(&self) -> bool {
        matches!(
            self,
            EditorTool::Paint | EditorTool::Erase | EditorTool::Terrain
        )
    }
}

/// Tool mode for painting operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ToolMode {
    /// Single tile/point painting (click or drag)
    #[default]
    Point,
    /// Rectangle fill (drag to define area)
    Rectangle,
    /// Line drawing (drag to define line)
    Line,
}

impl ToolMode {
    pub fn label(&self) -> &'static str {
        match self {
            ToolMode::Point => "Point",
            ToolMode::Rectangle => "Rect",
            ToolMode::Line => "Line",
        }
    }
}

/// Render the toolbar
pub fn render_toolbar(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    integration_registry: Option<&IntegrationRegistry>,
) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // View mode toggle
            ui.label("View:");
            if ui
                .selectable_label(editor_state.view_mode == EditorViewMode::Level, "Level")
                .on_hover_text("Edit level (L)")
                .clicked()
            {
                editor_state.view_mode = EditorViewMode::Level;
            }
            if ui
                .selectable_label(editor_state.view_mode == EditorViewMode::World, "World")
                .on_hover_text("World overview (W)")
                .clicked()
            {
                editor_state.view_mode = EditorViewMode::World;
            }

            ui.separator();

            // Tool selection - disabled in World view
            let tools_enabled = editor_state.view_mode == EditorViewMode::Level;
            if !tools_enabled {
                ui.disable();
            }

            // Tool selection - grouped by category
            ui.label("Tools:");

            // Selection tools
            if ui
                .selectable_label(editor_state.current_tool == EditorTool::Select, "Select")
                .clicked()
            {
                editor_state.current_tool = EditorTool::Select;
            }

            ui.separator();

            // Painting tools
            let paint_tools = [
                (EditorTool::Paint, "Paint"),
                (EditorTool::Erase, "Erase"),
                (EditorTool::Fill, "Fill"),
            ];

            for (tool, name) in paint_tools {
                if ui
                    .selectable_label(editor_state.current_tool == tool, name)
                    .clicked()
                {
                    editor_state.current_tool = tool;
                }
            }

            ui.separator();

            // Terrain tool (for autotiling)
            if ui
                .selectable_label(editor_state.current_tool == EditorTool::Terrain, "Terrain")
                .clicked()
            {
                editor_state.current_tool = EditorTool::Terrain;
            }

            ui.separator();

            // Entity tool
            if ui
                .selectable_label(editor_state.current_tool == EditorTool::Entity, "Entity")
                .clicked()
            {
                editor_state.current_tool = EditorTool::Entity;
            }

            ui.separator();

            // Tool mode dropdown (for applicable tools)
            if editor_state.current_tool.supports_modes() {
                ui.label("Mode:");
                egui::ComboBox::from_id_salt("tool_mode")
                    .selected_text(editor_state.tool_mode.label())
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut editor_state.tool_mode,
                            ToolMode::Point,
                            ToolMode::Point.label(),
                        );
                        ui.selectable_value(
                            &mut editor_state.tool_mode,
                            ToolMode::Rectangle,
                            ToolMode::Rectangle.label(),
                        );
                        ui.selectable_value(
                            &mut editor_state.tool_mode,
                            ToolMode::Line,
                            ToolMode::Line.label(),
                        );
                    });

                // Random paint toggle (for Paint tool only)
                if editor_state.current_tool == EditorTool::Paint {
                    ui.toggle_value(&mut editor_state.random_paint, "Random")
                        .on_hover_text(
                            "Paint random tiles from selection (Ctrl+click to add tiles)",
                        );
                }

                // Flip toggles (for Paint tool)
                if editor_state.current_tool == EditorTool::Paint {
                    ui.toggle_value(&mut editor_state.paint_flip_x, "X")
                        .on_hover_text("Flip tile horizontally (X key)");
                    ui.toggle_value(&mut editor_state.paint_flip_y, "Y")
                        .on_hover_text("Flip tile vertically (Y key)");
                }

                ui.separator();
            }

            // Layer selection
            ui.label("Layer:");
            if let Some(layer_idx) = editor_state.selected_layer {
                ui.label(format!("{}", layer_idx));
            } else {
                ui.label("(none)");
            }

            ui.separator();

            // Grid toggle
            ui.checkbox(&mut editor_state.show_grid, "Grid");

            ui.separator();

            // Zoom controls
            if ui.button("-").clicked() {
                editor_state.zoom = (editor_state.zoom / 1.25).max(0.25);
            }
            ui.label(format!("{}%", (editor_state.zoom * 100.0) as i32));
            if ui.button("+").clicked() {
                editor_state.zoom = (editor_state.zoom * 1.25).min(4.0);
            }

            // Tileset Editor button
            ui.separator();
            if ui.button("Tileset Editor").clicked() {
                editor_state.show_tileset_editor = true;
            }

            // Run Game button
            ui.separator();
            if ui
                .button("Run Game")
                .on_hover_text("Run the game (saves project first)")
                .clicked()
            {
                editor_state.pending_action = Some(PendingAction::RunGame);
            }

            // Integration toolbar buttons
            if let Some(registry) = integration_registry {
                let buttons: Vec<_> = registry
                    .ui_contributions()
                    .iter()
                    .filter_map(|ext| {
                        if let bevy_map_integration::editor::EditorExtension::ToolbarButton {
                            name,
                            ..
                        } = ext
                        {
                            Some(name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                if !buttons.is_empty() {
                    ui.separator();
                    for name in &buttons {
                        let _ = ui.button(name.as_str());
                    }
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use crate::testing::{
        assert_checkbox_state, assert_tool_active, editor_state_level_view,
        editor_state_paint_tool, harness_for_toolbar, select_labeled, toggle_labeled,
    };
    use crate::ui::toolbar::EditorTool;

    /// Clicking the "Grid" checkbox toggles show_grid from true to false.
    ///
    /// Uses `harness_for_toolbar` and `toggle_labeled` from `crate::testing`.
    ///
    /// Preconditions (confirmed via `editor_state_level_view` factory):
    /// - `show_grid = true`
    /// - `view_mode = Level` (tools enabled; `ui.disable()` does not fire)
    #[test]
    fn toolbar_grid_checkbox_toggle() {
        let mut harness = harness_for_toolbar(editor_state_level_view());

        // Precondition: show_grid starts true
        assert!(
            harness.state().show_grid,
            "editor_state_level_view().show_grid must be true for this test to be meaningful"
        );

        harness.run();
        toggle_labeled(&harness, "Grid");
        harness.run();

        assert!(
            !harness.state().show_grid,
            "show_grid should be false after toggling the Grid checkbox"
        );
    }

    /// Clicking the "Paint" selectable label sets current_tool to EditorTool::Paint.
    ///
    /// Uses `harness_for_toolbar`, `select_labeled`, and `assert_tool_active` from `crate::testing`.
    ///
    /// Preconditions (confirmed via `editor_state_level_view` factory):
    /// - `current_tool = Select` (default)
    /// - `view_mode = Level` (tools enabled)
    #[test]
    fn toolbar_paint_tool_select() {
        let mut harness = harness_for_toolbar(editor_state_level_view());

        // Precondition: tool starts as Select
        assert_eq!(
            harness.state().current_tool,
            EditorTool::Select,
            "editor_state_level_view().current_tool must be Select for this test to be meaningful"
        );

        harness.run();
        select_labeled(&harness, "Paint");
        harness.run();

        assert_tool_active(&harness, EditorTool::Paint);
    }

    /// `editor_state_paint_tool()` produces an EditorState with Paint as the active tool.
    ///
    /// Verifies that the factory function and `assert_tool_active` work together.
    #[test]
    fn factory_paint_tool_state() {
        let harness = harness_for_toolbar(editor_state_paint_tool());

        // No interaction needed — just verify the factory produced the right initial state.
        assert_tool_active(&harness, EditorTool::Paint);
    }

    /// `assert_checkbox_state` reports the Grid checkbox as checked when show_grid is true.
    ///
    /// Tests the AccessKit tree assertion path without any interaction.
    #[test]
    fn assert_checkbox_state_grid_checked() {
        let mut harness = harness_for_toolbar(editor_state_level_view());
        harness.run();

        // Grid starts checked
        assert_checkbox_state(&harness, "Grid", true);

        toggle_labeled(&harness, "Grid");
        harness.run();

        // Grid is now unchecked
        assert_checkbox_state(&harness, "Grid", false);
    }

    /// Snapshot test: toolbar rendered with `EditorState::default()`.
    ///
    /// Captures a visual baseline of the toolbar in its default state (Select tool, Level view,
    /// Grid checked, no mode widgets visible). Requires the `wgpu` feature on `egui_kittest`.
    ///
    /// To bless the baseline image on first run or after intentional visual changes:
    ///   `UPDATE_SNAPSHOTS=1 cargo test -p bevy_map_editor toolbar_default_snapshot`
    ///
    /// The baseline PNG is stored at:
    ///   `crates/bevy_map_editor/tests/snapshots/toolbar_default_snapshot.png`
    #[test]
    fn toolbar_default_snapshot() {
        use crate::testing::editor_state_default;
        let mut harness = harness_for_toolbar(editor_state_default());
        harness.run();
        harness.snapshot("toolbar_default_snapshot");
    }

    /// Snapshot test: toolbar rendered with `current_tool = EditorTool::Paint`.
    ///
    /// Captures a visual baseline of the toolbar when the Paint tool is active. In this state
    /// the Mode combobox and Random/X/Y flip toggle buttons are visible. This exercises the
    /// conditional widget section that only appears for tools that `supports_modes()`.
    ///
    /// To bless the baseline image on first run or after intentional visual changes:
    ///   `UPDATE_SNAPSHOTS=1 cargo test -p bevy_map_editor toolbar_paint_tool_snapshot`
    ///
    /// The baseline PNG is stored at:
    ///   `crates/bevy_map_editor/tests/snapshots/toolbar_paint_tool_snapshot.png`
    #[test]
    fn toolbar_paint_tool_snapshot() {
        use crate::testing::editor_state_paint_tool;
        let mut harness = harness_for_toolbar(editor_state_paint_tool());
        harness.run();
        harness.snapshot("toolbar_paint_tool_snapshot");
    }
}
