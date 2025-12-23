//! New Project dialog

use bevy_egui::egui;

use crate::project::Project;
use crate::EditorState;

/// Render the New Project dialog
pub fn render_new_project_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    if !editor_state.show_new_project_dialog {
        return;
    }

    let mut close_dialog = false;

    egui::Window::new("New Project")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(400.0);

            ui.horizontal(|ui| {
                ui.label("Project Name:");
                ui.add_sized(
                    [250.0, 20.0],
                    egui::TextEdit::singleline(&mut editor_state.new_project_name),
                );
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Save Location:");
                let path_display = editor_state
                    .new_project_save_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "(not selected)".to_string());

                ui.add_sized([250.0, 20.0], egui::Label::new(&path_display).truncate());

                #[cfg(feature = "native")]
                if ui.button("Browse...").clicked() {
                    let default_name = if editor_state.new_project_name.is_empty() {
                        "new_project.map.json"
                    } else {
                        ""
                    };
                    let file_name = if editor_state.new_project_name.is_empty() {
                        default_name.to_string()
                    } else {
                        format!("{}.map.json", editor_state.new_project_name)
                    };

                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Map Project", &["map.json"])
                        .set_file_name(&file_name)
                        .save_file()
                    {
                        editor_state.new_project_save_path = Some(path);
                    }
                }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let can_create = !editor_state.new_project_name.is_empty()
                    && editor_state.new_project_save_path.is_some();

                ui.add_enabled_ui(can_create, |ui| {
                    if ui.button("Create").clicked() {
                        // Create fresh project
                        let mut new_project = Project::default();
                        new_project.schema.project.name = editor_state.new_project_name.clone();

                        // Save to selected path
                        if let Some(ref path) = editor_state.new_project_save_path {
                            match new_project.save(path) {
                                Ok(()) => {
                                    *project = new_project;
                                    // Signal to add to recent projects
                                    editor_state.pending_add_recent_project = Some(path.clone());
                                    close_dialog = true;
                                }
                                Err(e) => {
                                    editor_state.error_message =
                                        Some(format!("Failed to save project: {}", e));
                                }
                            }
                        }
                    }
                });

                if ui.button("Cancel").clicked() {
                    close_dialog = true;
                }
            });
        });

    if close_dialog {
        editor_state.show_new_project_dialog = false;
        editor_state.new_project_name = String::new();
        editor_state.new_project_save_path = None;
    }
}
