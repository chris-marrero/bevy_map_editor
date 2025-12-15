//! Terrain-related dialogs

use bevy_egui::egui;
use bevy_map_autotile::{Color, TerrainSet, TerrainSetType};

use crate::project::Project;
use crate::EditorState;

/// Render the new terrain dialog (legacy 47-tile blob)
pub fn render_new_terrain_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    _project: &mut Project,
) {
    if !editor_state.show_new_terrain_dialog {
        return;
    }

    egui::Window::new("New Terrain")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.new_terrain_name);
            });

            ui.horizontal(|ui| {
                ui.label("First Tile:");
                ui.add(egui::DragValue::new(
                    &mut editor_state.new_terrain_first_tile,
                ));
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    // TODO: Create legacy terrain
                    editor_state.show_new_terrain_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    editor_state.show_new_terrain_dialog = false;
                }
            });
        });
}

/// Render the new terrain set dialog (Tiled-compatible)
pub fn render_new_terrain_set_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    if !editor_state.show_new_terrain_set_dialog {
        return;
    }

    let Some(tileset_id) = editor_state.selected_tileset else {
        editor_state.show_new_terrain_set_dialog = false;
        return;
    };

    egui::Window::new("New Terrain Set")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.new_terrain_name);
            });

            ui.horizontal(|ui| {
                ui.label("Type:");
                egui::ComboBox::from_id_salt("terrain_set_type")
                    .selected_text(format!("{:?}", editor_state.new_terrain_set_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut editor_state.new_terrain_set_type,
                            TerrainSetType::Corner,
                            "Corner",
                        );
                        ui.selectable_value(
                            &mut editor_state.new_terrain_set_type,
                            TerrainSetType::Edge,
                            "Edge",
                        );
                        ui.selectable_value(
                            &mut editor_state.new_terrain_set_type,
                            TerrainSetType::Mixed,
                            "Mixed",
                        );
                    });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    let terrain_set = TerrainSet::new(
                        editor_state.new_terrain_name.clone(),
                        tileset_id,
                        editor_state.new_terrain_set_type,
                    );
                    project.autotile_config.terrain_sets.push(terrain_set);
                    project.mark_dirty();
                    editor_state.show_new_terrain_set_dialog = false;
                    editor_state.new_terrain_name = String::new();
                }
                if ui.button("Cancel").clicked() {
                    editor_state.show_new_terrain_set_dialog = false;
                }
            });
        });
}

/// Render the add terrain to set dialog
pub fn render_add_terrain_to_set_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    if !editor_state.show_add_terrain_to_set_dialog {
        return;
    }

    let Some(terrain_set_id) = editor_state.selected_terrain_set else {
        editor_state.show_add_terrain_to_set_dialog = false;
        return;
    };

    egui::Window::new("Add Terrain")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.new_terrain_name);
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                ui.color_edit_button_rgb(&mut editor_state.new_terrain_color);
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    if let Some(terrain_set) = project
                        .autotile_config
                        .terrain_sets
                        .iter_mut()
                        .find(|ts| ts.id == terrain_set_id)
                    {
                        let color = Color::rgb(
                            editor_state.new_terrain_color[0],
                            editor_state.new_terrain_color[1],
                            editor_state.new_terrain_color[2],
                        );
                        terrain_set.add_terrain(editor_state.new_terrain_name.clone(), color);
                        project.mark_dirty();
                    }
                    editor_state.show_add_terrain_to_set_dialog = false;
                    editor_state.new_terrain_name = String::new();
                    editor_state.new_terrain_color = [0.0, 1.0, 0.0];
                }
                if ui.button("Cancel").clicked() {
                    editor_state.show_add_terrain_to_set_dialog = false;
                }
            });
        });
}
