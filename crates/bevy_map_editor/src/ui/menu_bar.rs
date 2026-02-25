//! Menu bar UI

use bevy_egui::egui;
use bevy_map_integration::registry::IntegrationRegistry;
use std::path::PathBuf;

use super::UiState;
use crate::commands::{CommandHistory, TileClipboard};
use crate::preferences::EditorPreferences;
use crate::project::Project;
use crate::EditorState;

use super::PendingAction;

/// Render the menu bar
pub fn render_menu_bar(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    editor_state: &mut EditorState,
    project: &mut Project,
    history: Option<&CommandHistory>,
    clipboard: Option<&TileClipboard>,
    preferences: &EditorPreferences,
    integration_registry: Option<&IntegrationRegistry>,
) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New Project...").clicked() {
                    editor_state.pending_action = Some(PendingAction::New);
                    ui.close();
                }
                if ui.button("Open Project...").clicked() {
                    editor_state.pending_action = Some(PendingAction::Open);
                    ui.close();
                }

                // Open Recent submenu
                ui.menu_button("Open Recent", |ui| {
                    if preferences.recent_projects.is_empty() {
                        ui.label("(No recent projects)");
                    } else {
                        for recent in &preferences.recent_projects {
                            if ui.button(&recent.name).clicked() {
                                editor_state.pending_open_recent_project =
                                    Some(PathBuf::from(&recent.path));
                                ui.close();
                            }
                        }
                        ui.separator();
                        if ui.button("Clear Recent Projects").clicked() {
                            editor_state.pending_clear_recent_projects = true;
                            ui.close();
                        }
                    }
                });

                ui.separator();
                if ui.button("Save").clicked() {
                    editor_state.pending_action = Some(PendingAction::Save);
                    ui.close();
                }
                if ui.button("Save As...").clicked() {
                    editor_state.pending_action = Some(PendingAction::SaveAs);
                    ui.close();
                }
                ui.separator();
                if ui.button("Settings...").clicked() {
                    editor_state.show_settings_dialog = true;
                    ui.close();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    editor_state.pending_action = Some(PendingAction::Exit);
                    ui.close();
                }
            });

            // Edit menu
            ui.menu_button("Edit", |ui| {
                let can_undo = history.map_or(false, |h| h.can_undo());
                let can_redo = history.map_or(false, |h| h.can_redo());

                if ui
                    .add_enabled(can_undo, egui::Button::new("Undo"))
                    .clicked()
                {
                    editor_state.pending_action = Some(PendingAction::Undo);
                    ui.close();
                }
                if ui
                    .add_enabled(can_redo, egui::Button::new("Redo"))
                    .clicked()
                {
                    editor_state.pending_action = Some(PendingAction::Redo);
                    ui.close();
                }
                ui.separator();
                if ui.button("Cut").clicked() {
                    editor_state.pending_action = Some(PendingAction::Cut);
                    ui.close();
                }
                if ui.button("Copy").clicked() {
                    editor_state.pending_action = Some(PendingAction::Copy);
                    ui.close();
                }
                let has_clipboard = clipboard.map_or(false, |c| c.has_content());
                if ui
                    .add_enabled(has_clipboard, egui::Button::new("Paste"))
                    .clicked()
                {
                    editor_state.pending_action = Some(PendingAction::Paste);
                    ui.close();
                }
                ui.separator();
                if ui.button("Select All").clicked() {
                    editor_state.pending_action = Some(PendingAction::SelectAll);
                    ui.close();
                }
            });

            // View menu
            ui.menu_button("View", |ui| {
                if ui
                    .checkbox(&mut ui_state.show_tree_view, "Project Tree")
                    .clicked()
                {
                    ui.close();
                }
                if ui
                    .checkbox(&mut ui_state.show_inspector, "Inspector")
                    .clicked()
                {
                    ui.close();
                }
                if ui
                    .checkbox(&mut ui_state.show_asset_browser, "Asset Browser")
                    .clicked()
                {
                    ui.close();
                }
                ui.separator();
                if ui
                    .checkbox(&mut editor_state.show_grid, "Show Grid")
                    .clicked()
                {
                    ui.close();
                }
                if ui
                    .checkbox(&mut editor_state.show_collisions, "Show Collisions")
                    .clicked()
                {
                    ui.close();
                }
                // Snapping submenu (Tiled-style)
                ui.menu_button("Snapping", |ui| {
                    if ui
                        .checkbox(&mut editor_state.snap_to_grid, "Snap to Grid")
                        .clicked()
                    {
                        ui.close();
                    }
                });

                // Integration panels
                if let Some(registry) = integration_registry {
                    let panels: Vec<_> = registry
                        .ui_contributions()
                        .iter()
                        .filter_map(|ext| {
                            if let bevy_map_integration::editor::EditorExtension::Panel {
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

                    if !panels.is_empty() {
                        ui.separator();
                        for name in &panels {
                            let mut visible = ui_state.integration_panels.contains(name);
                            if ui.checkbox(&mut visible, name.as_str()).clicked() {
                                if visible {
                                    ui_state.integration_panels.insert(name.clone());
                                } else {
                                    ui_state.integration_panels.remove(name);
                                }
                                ui.close();
                            }
                        }
                    }
                }
            });

            // Project menu
            ui.menu_button("Project", |ui| {
                if ui.button("New Level...").clicked() {
                    editor_state.show_new_level_dialog = true;
                    ui.close();
                }
                if ui.button("New Tileset...").clicked() {
                    editor_state.show_new_tileset_dialog = true;
                    ui.close();
                }
                ui.separator();
                if ui.button("Game Settings...").clicked() {
                    editor_state.pending_action = Some(PendingAction::OpenGameSettings);
                    ui.close();
                }
                // Show run option if game project is configured
                let can_run = project.game_config.project_path.is_some()
                    && project.game_config.starting_level.is_some();
                ui.add_enabled_ui(can_run, |ui| {
                    if ui.button("Run Game").clicked() {
                        editor_state.pending_action = Some(PendingAction::RunGame);
                        ui.close();
                    }
                });

                ui.separator();

                // Code generation submenu
                ui.menu_button("Code Generation", |ui| {
                    let _codegen_enabled = project.game_config.enable_codegen;
                    let has_project = project.game_config.project_path.is_some();

                    ui.add_enabled_ui(has_project, |ui| {
                        if ui.button("Generate Now").clicked() {
                            editor_state.pending_action = Some(PendingAction::GenerateCode);
                            ui.close();
                        }
                    });

                    if ui.button("Preview Code...").clicked() {
                        editor_state.pending_action = Some(PendingAction::PreviewCode);
                        ui.close();
                    }

                    ui.separator();

                    ui.add_enabled_ui(has_project, |ui| {
                        if ui.button("Open in VS Code").clicked() {
                            editor_state.pending_action = Some(PendingAction::OpenInVSCode);
                            ui.close();
                        }
                    });
                });
            });

            // Tools menu - Specialized editors
            ui.menu_button("Tools", |ui| {
                // Graphics editors
                if ui.button("Tileset Editor...").clicked() {
                    editor_state.show_tileset_editor = true;
                    ui.close();
                }
                if ui.button("Sprite Sheet Editor...").clicked() {
                    editor_state.show_spritesheet_editor = true;
                    ui.close();
                }
                if ui.button("Animation Editor...").clicked() {
                    editor_state.show_animation_editor = true;
                    ui.close();
                }
                ui.separator();
                // Content editor
                if ui.button("Dialogue Editor...").clicked() {
                    editor_state.show_dialogue_editor = true;
                    ui.close();
                }
                ui.separator();
                // Data editor
                if ui.button("Schema Editor...").clicked() {
                    editor_state.show_schema_editor = true;
                    ui.close();
                }
            });

            // Integrations menu (from EditorExtension::MenuItem contributions)
            if let Some(registry) = integration_registry {
                let menu_items: Vec<_> = registry
                    .ui_contributions()
                    .iter()
                    .filter_map(|ext| {
                        if let bevy_map_integration::editor::EditorExtension::MenuItem {
                            path,
                            ..
                        } = ext
                        {
                            Some(path.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                if !menu_items.is_empty() {
                    ui.menu_button("Integrations", |ui| {
                        for path in &menu_items {
                            // Parse "Category/Item" paths into submenus
                            let parts: Vec<&str> = path.splitn(2, '/').collect();
                            if parts.len() == 2 {
                                ui.menu_button(parts[0], |ui| {
                                    if ui.button(parts[1]).clicked() {
                                        ui.close();
                                    }
                                });
                            } else if ui.button(path.as_str()).clicked() {
                                ui.close();
                            }
                        }
                    });
                }
            }

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("About...").clicked() {
                    editor_state.show_about_dialog = true;
                    ui.close();
                }
            });

            // Project status on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let dirty_indicator = if project.is_dirty() { " *" } else { "" };
                ui.label(format!("{}{}", project.name(), dirty_indicator));
            });
        });
    });
}
